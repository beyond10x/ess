//! An assignment between an Optional value and a required one, in generated Go
//! (story:go-synthesis-refuses-an-optional-into-required-assignment).
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
