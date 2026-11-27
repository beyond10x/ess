//! An assignment between an Optional value and a required one, in generated Go
//! (story:go-synthesis-refuses-an-optional-into-required-assignment), and the wrap direction in
//! generated Rust (story:rust-binding-wraps-a-plain-copy-into-an-optional-member, the last section).
//!
//! For each direction a target either emits the conversion or refuses it by name:
//!
//! - an Optional value into a required field that nothing proves present is refused before
//!   synthesis, by validation, as `type_mismatch` at the assignment and naming both sides: a
//!   binding's mapping and an outcome's payload alike (main already does this; these pin it);
//! - the same value where a condition or a guard proves it present synthesizes, and the module
//!   builds;
//! - a required value into an Optional field is emitted as a wrap (`*T` from `T`), and the module
//!   builds.
//!
//! Every Go module here is built with `go build` and vetted with `go vet` where a Go toolchain is
//! on `PATH`, so a capability the plan reports as generated is in a module Go accepts. A test here
//! spawns no cargo (`ess-xtask/tests/no_test_builds.rs`).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use ess_compiler::diagnostic::Diagnostics;
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::ValidationCode;
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const STORY: &str = "story:go-synthesis-refuses-an-optional-into-required-assignment";

/// A message names its channel always and its topic only sometimes; each binding below reads one
/// of them into a command whose input declares the other optionality.
const MODEL: &str = r"format: ess/33
system: relay
version: v1
domain: relay.chat
events:
  - name: relay.chat.MessageSent
    fields:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: topic, type: Optional<String>}
  - name: relay.chat.MessageRecorded
    fields:
      - {name: message_id, type: String}
  - name: relay.chat.TopicFiled
    fields:
      - {name: message_id, type: String}
      - {name: topic, type: String}
commands:
  - name: relay.chat.SendMessage
    input:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: topic, type: Optional<String>}
    outcomes:
      - name: sent
        emits: [relay.chat.MessageSent]
        payload:
          relay.chat.MessageSent: {message_id: input.message_id, channel: input.channel, topic: input.topic}
  - name: relay.chat.RecordMessage
    input:
      - {name: message_id, type: String}
      - {name: channel, type: Optional<String>}
    outcomes:
      - name: recorded
        emits: [relay.chat.MessageRecorded]
        payload:
          relay.chat.MessageRecorded: {message_id: input.message_id}
  - name: relay.chat.FileTopic
    input:
      - {name: message_id, type: String}
      - {name: topic, type: String}
    outcomes:
      - name: filed
        emits: [relay.chat.TopicFiled]
        payload:
          relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}
BINDINGS
components:
  - component: relay-service
    owns: {domains: [relay.chat]}
    accepts: {commands: [relay.chat.SendMessage, relay.chat.RecordMessage, relay.chat.FileTopic]}
    publishes: {events: [relay.chat.MessageSent, relay.chat.MessageRecorded, relay.chat.TopicFiled]}
    reached_by: network
";

/// The required `channel` copied into the Optional input of `RecordMessage`: the wrap.
const WRAP: &str = r"  - id: recorded
    when: {event: relay.chat.MessageSent}
    invoke: {command: relay.chat.RecordMessage}
    mapping:
      message_id: event.message_id
      channel: event.channel
    delivery: at_least_once
    on_failure: retry
";

/// The Optional `topic` copied into the required input of `FileTopic`, where the condition proves
/// it present.
const PROVED: &str = r"  - id: filed
    when:
      event: relay.chat.MessageSent
      where: [defined(event.topic)]
    invoke: {command: relay.chat.FileTopic}
    mapping:
      message_id: event.message_id
      topic: event.topic
    delivery: at_least_once
    on_failure: retry
";

/// The same, with nothing proving it present.
const UNPROVED: &str = r"  - id: filed
    when: {event: relay.chat.MessageSent}
    invoke: {command: relay.chat.FileTopic}
    mapping:
      message_id: event.message_id
      topic: event.topic
    delivery: at_least_once
    on_failure: retry
";

fn model(bindings: &[&str]) -> String {
    MODEL.replace("BINDINGS\n", &format!("bindings:\n{}", bindings.concat()))
}

fn compiled(text: &str) -> Result<EssIr, Diagnostics> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{STORY}: {error}"));
    let specification = Specification::assemble([(Source::new("relay.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{STORY}: the model validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
}

fn ir(text: &str) -> EssIr {
    compiled(text).unwrap_or_else(|errors| panic!("{STORY}: the model compiles:\n{errors}"))
}

/// The Go artifacts of `ir`, with `binding`'s transformation and delivery generated.
fn go(ir: &EssIr, binding: &str) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(ir, Target::Go)
        .unwrap_or_else(|failure| panic!("{STORY}: Go emits: {}", failure.to_canonical_json()));
    for kind in [
        CapabilityKind::BindingTransformation,
        CapabilityKind::BindingDelivery,
    ] {
        assert_eq!(
            synthesis.plan.disposition_of(kind, binding),
            Some(&SynthesisDisposition::Generated),
            "{STORY}: {kind:?} of `{binding}` is generated"
        );
    }
    let report = synthesis
        .target
        .as_ref()
        .expect("Go writes a target report");
    assert!(
        !report
            .refusals
            .iter()
            .any(|refusal| refusal.capability.source == binding),
        "{STORY}: Go refuses nothing of `{binding}`: {:?}",
        report.refusals
    );
    synthesis
        .artifacts
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Writes `artifacts` under a fresh directory, then `go build ./...` and `go vet ./...` there,
/// offline. `Err` carries the toolchain's words.
fn go_accepts(label: &str, artifacts: &BTreeMap<String, String>) -> Result<(), String> {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "binding-optional-wrap-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, contents) in artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(path, contents).expect("write");
    }
    let mut verdict = Ok(());
    for step in ["build", "vet"] {
        let output = Command::new("go")
            .args([step, "-buildvcs=false", "./..."])
            .current_dir(&directory)
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("CGO_ENABLED", "0")
            .output()
            .expect("the Go tool runs");
        if !output.status.success() {
            verdict = Err(format!(
                "go {step}:\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
            break;
        }
    }
    let _ = std::fs::remove_dir_all(&directory);
    verdict
}

/// `text` is refused before synthesis, by validation, with exactly one `type_mismatch` at
/// `location` whose message names both sides of the assignment.
fn refused_at(text: &str, location: &str, from: &str, into: &str) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{STORY}: {error}"));
    let Err(errors) = Specification::assemble([(Source::new("relay.yaml"), raw)]) else {
        panic!("{STORY}: an Optional value into a required field validates")
    };
    let refusals: Vec<_> = errors
        .as_slice()
        .iter()
        .map(|error| (error.code, error.location.as_str()))
        .collect();
    assert_eq!(
        refusals,
        [(ValidationCode::TypeMismatch, location)],
        "{STORY}: refused once, at the assignment:\n{errors}"
    );
    let message = &errors.as_slice()[0].message;
    assert!(
        message.contains(&format!("`{from}` has type `Optional<String>`"))
            && message.contains(&format!("`{into}` requires `String`")),
        "{STORY}: the refusal names both sides:\n{message}"
    );
}

#[test]
fn a_required_value_copied_into_an_optional_input_is_wrapped_and_builds() {
    let artifacts = go(&ir(&model(&[WRAP])), "recorded");
    let system = &artifacts["system/system.go"];
    assert!(
        !system.contains("channel: event.Channel,") && !system.contains("Channel: event.Channel,"),
        "{STORY}: a `string` is not assigned into the `*string` input:\n{system}"
    );
    if go_available() {
        go_accepts("wrap", &artifacts).unwrap_or_else(|log| panic!("{STORY}: {log}"));
    }
}

#[test]
fn an_optional_value_a_condition_proves_present_fills_a_required_input_and_builds() {
    let artifacts = go(&ir(&model(&[PROVED])), "filed");
    if go_available() {
        go_accepts("proved", &artifacts).unwrap_or_else(|log| panic!("{STORY}: {log}"));
    }
}

#[test]
fn both_directions_in_one_module_build() {
    let ir = ir(&model(&[WRAP, PROVED]));
    go(&ir, "filed");
    let artifacts = go(&ir, "recorded");
    if go_available() {
        go_accepts("both", &artifacts).unwrap_or_else(|log| panic!("{STORY}: {log}"));
    }
}

#[test]
fn an_optional_value_nothing_proves_present_is_refused_for_a_required_input() {
    refused_at(
        &model(&[UNPROVED]),
        "binding.filed.mapping.topic",
        "relay.chat.MessageSent.topic",
        "relay.chat.FileTopic.topic",
    );
}

/// The outcome half: an outcome that emits an event whose required fields are set from Optional
/// inputs no guard proves present.
#[test]
fn an_outcome_filling_a_required_event_field_from_an_optional_input_is_refused() {
    let text = model(&[]).replace(
        "relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}",
        "relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}\n  - name: relay.chat.Announce\n    input:\n      - {name: message_id, type: String}\n      - {name: topic, type: Optional<String>}\n    outcomes:\n      - name: announced\n        emits: [relay.chat.TopicFiled]\n        payload:\n          relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}",
    ).replace("bindings:\n", "").replace(
        "accepts: {commands: [relay.chat.SendMessage,",
        "accepts: {commands: [relay.chat.Announce, relay.chat.SendMessage,",
    );
    refused_at(
        &text,
        "command.relay.chat.Announce.outcomes.announced.payload.relay.chat.TopicFiled.topic",
        "relay.chat.Announce.topic",
        "relay.chat.TopicFiled.topic",
    );
}

/// The outcome half with a guard proving presence: the behaviour stays owed, named by the read,
/// rather than emitted as a dereference, and the module builds.
#[test]
fn an_outcome_whose_guard_proves_the_optional_input_present_synthesizes_and_builds() {
    let text = model(&[]).replace(
        "relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}",
        "relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}\n  - name: relay.chat.Announce\n    input:\n      - {name: message_id, type: String}\n      - {name: topic, type: Optional<String>}\n    outcomes:\n      - name: announced\n        when: defined(topic)\n        emits: [relay.chat.TopicFiled]\n        payload:\n          relay.chat.TopicFiled: {message_id: input.message_id, topic: input.topic}\n      - name: quiet\n        emits: [relay.chat.MessageRecorded]\n        payload:\n          relay.chat.MessageRecorded: {message_id: input.message_id}",
    ).replace("bindings:\n", "").replace(
        "accepts: {commands: [relay.chat.SendMessage,",
        "accepts: {commands: [relay.chat.Announce, relay.chat.SendMessage,",
    );
    let ir = ir(&text);
    let synthesis = synthesize_for(&ir, Target::Go)
        .unwrap_or_else(|failure| panic!("{STORY}: Go emits: {}", failure.to_canonical_json()));
    let disposition = synthesis
        .plan
        .disposition_of(CapabilityKind::CommandBehavior, "relay.chat.Announce");
    assert!(
        matches!(
            disposition,
            Some(SynthesisDisposition::Generated | SynthesisDisposition::Obligation(_))
        ),
        "{STORY}: the guarded behaviour is generated or owed by name, never refused: \
         {disposition:?}"
    );
    if go_available() {
        let artifacts = synthesis
            .artifacts
            .into_iter()
            .map(|(path, artifact)| (path, artifact.contents))
            .collect();
        go_accepts("guarded", &artifacts).unwrap_or_else(|log| panic!("{STORY}: {log}"));
    }
}

/// The payload's own reverse direction: a required input into an Optional event field.
#[test]
fn an_outcome_filling_an_optional_event_field_from_a_required_input_builds() {
    let text = model(&[]).replace(
        "relay.chat.MessageSent: {message_id: input.message_id, channel: input.channel, topic: input.topic}",
        "relay.chat.MessageSent: {message_id: input.message_id, channel: input.channel, topic: input.channel}",
    ).replace("bindings:\n", "");
    let synthesis = synthesize_for(&ir(&text), Target::Go)
        .unwrap_or_else(|failure| panic!("{STORY}: Go emits: {}", failure.to_canonical_json()));
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "relay.chat.SendMessage"),
        Some(&SynthesisDisposition::Generated),
        "{STORY}: the behaviour is generated"
    );
    if go_available() {
        let artifacts = synthesis
            .artifacts
            .into_iter()
            .map(|(path, artifact)| (path, artifact.contents))
            .collect();
        go_accepts("payload-wrap", &artifacts).unwrap_or_else(|log| panic!("{STORY}: {log}"));
    }
}

// The wrap in generated Rust.

const RUST_STORY: &str = "story:rust-binding-wraps-a-plain-copy-into-an-optional-member";

/// An event whose every member is required, and a command whose input holds each of them as
/// `Optional` of the same type: a `String`, a scalar, a newtype identifier and a struct. `editor`
/// is `Optional` of a different newtype over the same `String`.
const RUST_MODEL: &str = r"format: ess/33
system: relay
version: v1
domain: relay.chat
types:
  - {name: relay.chat.AuthorId, kind: newtype, of: String}
  - {name: relay.chat.EditorId, kind: newtype, of: String}
  - name: relay.chat.Doc
    kind: struct
    fields:
      - {name: title, type: String}
events:
  - name: relay.chat.MessageSent
    fields:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: count, type: Integer}
      - {name: author, type: relay.chat.AuthorId}
      - {name: doc, type: relay.chat.Doc}
  - name: relay.chat.MessageRecorded
    fields:
      - {name: message_id, type: String}
commands:
  - name: relay.chat.SendMessage
    input:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: count, type: Integer}
      - {name: author, type: relay.chat.AuthorId}
      - {name: doc, type: relay.chat.Doc}
    outcomes:
      - name: sent
        emits: [relay.chat.MessageSent]
        payload:
          relay.chat.MessageSent: {message_id: input.message_id, channel: input.channel, count: input.count, author: input.author, doc: input.doc}
  - name: relay.chat.RecordMessage
    input:
      - {name: message_id, type: String}
      - {name: channel, type: Optional<String>}
      - {name: count, type: Optional<Integer>}
      - {name: author, type: Optional<relay.chat.AuthorId>}
      - {name: doc, type: Optional<relay.chat.Doc>}
      - {name: editor, type: Optional<relay.chat.EditorId>}
    outcomes:
      - name: recorded
        emits: [relay.chat.MessageRecorded]
        payload:
          relay.chat.MessageRecorded: {message_id: input.message_id}
components:
  - component: relay-service
    owns: {domains: [relay.chat]}
    accepts: {commands: [relay.chat.SendMessage, relay.chat.RecordMessage]}
    publishes: {events: [relay.chat.MessageSent, relay.chat.MessageRecorded]}
    reached_by: network
bindings:
  - id: recorded
    when: {event: relay.chat.MessageSent}
    invoke: {command: relay.chat.RecordMessage}
    mapping:
      message_id: event.message_id
MAPPING    delivery: at_least_once
    on_failure: retry
";

/// One row per plain source type copied into an `Optional` of that same type: the input it maps.
const WRAP_ROWS: [(&str, &str); 4] = [
    ("String", "channel"),
    ("Integer", "count"),
    ("newtype identifier", "author"),
    ("struct", "doc"),
];

/// `RUST_MODEL` with `recorded` mapping each of `inputs` from the event's member `source`.
fn rust_model(inputs: &[(&str, &str)]) -> String {
    let mut mapping = String::new();
    for (input, source) in inputs {
        let _ = writeln!(mapping, "      {input}: event.{source}");
    }
    RUST_MODEL.replace("MAPPING", &mapping)
}

/// Every Rust artifact of `ir`, joined, with `recorded`'s transformation and delivery generated
/// and nothing of it refused.
fn rust(ir: &EssIr, row: &str) -> String {
    let synthesis = synthesize_for(ir, Target::Rust).unwrap_or_else(|failure| {
        panic!(
            "{RUST_STORY}: {row}: Rust emits: {}",
            failure.to_canonical_json()
        )
    });
    for kind in [
        CapabilityKind::BindingTransformation,
        CapabilityKind::BindingDelivery,
    ] {
        assert_eq!(
            synthesis.plan.disposition_of(kind, "recorded"),
            Some(&SynthesisDisposition::Generated),
            "{RUST_STORY}: {row}: {kind:?} of `recorded` is generated"
        );
    }
    assert_eq!(
        synthesis.plan.counts().refused,
        0,
        "{RUST_STORY}: {row}: nothing is refused"
    );
    assert!(
        synthesis
            .target
            .as_ref()
            .is_none_or(|report| report.refusals.is_empty()),
        "{RUST_STORY}: {row}: Rust refuses nothing: {:?}",
        synthesis.target
    );
    synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn rust_binding_wraps_a_required_copy_into_an_optional_member() {
    for (row, input) in WRAP_ROWS {
        let ir = ir(&rust_model(&[(input, input)]));
        let code = rust(&ir, row);
        let wrapped = format!("        {input}: Some(event.{input}.clone()),\n");
        assert!(
            code.contains(&wrapped),
            "{RUST_STORY}: {row}: `{input}` is passed as `Some(event.{input}.clone())`:\n{code}"
        );
        go(&ir, "recorded");
    }
    let every: Vec<_> = WRAP_ROWS
        .iter()
        .map(|(_, input)| (*input, *input))
        .collect();
    let ir = ir(&rust_model(&every));
    let code = rust(&ir, "every row in one binding");
    for (_, input) in WRAP_ROWS {
        assert!(
            code.contains(&format!("        {input}: Some(event.{input}.clone()),\n")),
            "{RUST_STORY}: every row in one binding: `{input}` is wrapped:\n{code}"
        );
    }
    go(&ir, "recorded");
}

/// The wrap changes presence, not representation: a newtype copied into an `Optional` of another
/// newtype over the same value is refused by validation, as `type_mismatch` at the mapping, and
/// with the crossing declared it is never passed as a wrapped plain copy.
#[test]
fn rust_binding_never_wraps_a_newtype_into_an_optional_of_another_newtype() {
    let text = rust_model(&[("editor", "author")]);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{RUST_STORY}: {error}"));
    let Err(errors) = Specification::assemble([(Source::new("relay.yaml"), raw)]) else {
        panic!("{RUST_STORY}: an `AuthorId` copied into `Optional<EditorId>` validates")
    };
    let refusals: Vec<_> = errors
        .as_slice()
        .iter()
        .map(|error| (error.code, error.location.as_str()))
        .collect();
    assert_eq!(
        refusals,
        [(
            ValidationCode::TypeMismatch,
            "binding.recorded.mapping.editor"
        )],
        "{RUST_STORY}: refused once, at the mapping:\n{errors}"
    );

    let crossed = text.replace(
        "events:\n",
        "conversions:\n  - from: relay.chat.AuthorId\n    to: Optional<relay.chat.EditorId>\n    \
         because: an author edits what they wrote.\nevents:\n",
    );
    let ir = ir(&crossed);
    let code = match synthesize_for(&ir, Target::Rust) {
        Ok(synthesis) => {
            assert_ne!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::BindingTransformation, "recorded"),
                Some(&SynthesisDisposition::Generated),
                "{RUST_STORY}: a declared crossing into an Optional is not a determined copy"
            );
            synthesis
                .artifacts
                .values()
                .map(|artifact| artifact.contents.clone())
                .collect::<Vec<_>>()
                .join("\n")
        }
        Err(failure) => failure.to_canonical_json(),
    };
    assert!(
        !code.contains("editor: Some(event.author.clone())"),
        "{RUST_STORY}: an `AuthorId` is not passed as an `Option<EditorId>`:\n{code}"
    );
}
