//! Adversary cases for unit GEN2 of wave W2: the Optional wrap in Go bindings, the repaired Go
//! surface file names, and the clap tree several components sharing one binary generate.
//!
//! Go cases run `go build ./...` and `go vet ./...` over the whole emitted module where `go` is
//! on `PATH`. No case spawns cargo (`ess-xtask/tests/no_test_builds.rs`).

// The cases stay as the adversary wrote them; these pedantic lints fire on its fixture builders
// and on one case's `match`, none on what a case asserts.
#![allow(
    clippy::case_sensitive_file_extension_comparisons,
    clippy::format_collect,
    clippy::format_push_string,
    clippy::single_match_else
)]

use std::path::Path;
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_synth::{synthesize_for, CapabilityKind, Synthesis, SynthesisDisposition, Target};

const WRAP: &str = "story:go-synthesis-refuses-an-optional-into-required-assignment";
const SERVED: &str = "story:go-synthesis-repairs-a-served-component-named-server";
const SHARED: &str = "story:components-sharing-a-cli-binary-generate-one-tree-with-every-group";

fn assemble(documents: &[(&str, &str)]) -> Result<Specification, ValidationErrors> {
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("the fixture `{label}` is well formed: {error}"));
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    Specification::assemble(parsed)
}

fn fixture(documents: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in documents {
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
    }
    let specification =
        assemble(documents).unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// `go build` and `go vet` over every artifact, offline; `Err` carries the toolchain's words.
fn go_accepts(label: &str, synthesis: &Synthesis) -> Result<(), String> {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-w2-gen2-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for artifact in synthesis.artifacts.values() {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
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

// ---- story 1: a required value copied into an Optional Go input ------------------------------

const RELAY_SYSTEM: &str = "\
format: ess/33
system: relay
version: v1
domains:
  - relay.chat
  - relay.desk
";

/// `relay.chat` sends a message whose event carries one field of every shape under attack.
const RELAY_CHAT: &str = "\
domain: relay.chat
types:
  - {name: relay.chat.Label, kind: newtype, of: String}
  - name: relay.chat.Place
    kind: struct
    fields:
      - {name: room, type: String}
events:
  - name: relay.chat.MessageSent
    fields:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: topic, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: place, type: relay.chat.Place}
      - {name: label, type: relay.chat.Label}
commands:
  - name: relay.chat.SendMessage
    input:
      - {name: message_id, type: String}
      - {name: channel, type: String}
      - {name: topic, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: place, type: relay.chat.Place}
      - {name: label, type: relay.chat.Label}
    outcomes:
      - name: sent
        emits: [relay.chat.MessageSent]
        payload:
          relay.chat.MessageSent: {message_id: input.message_id, channel: input.channel, topic: input.topic, tags: input.tags, place: input.place, label: input.label}
";

/// `relay.desk` records a message; `INPUTS` are the extra inputs of `RecordMessage`.
const RELAY_DESK: &str = "\
domain: relay.desk
events:
  - name: relay.desk.MessageRecorded
    fields:
      - {name: message_id, type: String}
commands:
  - name: relay.desk.RecordMessage
    input:
      - {name: message_id, type: String}
INPUTS
    outcomes:
      - name: recorded
        emits: [relay.desk.MessageRecorded]
        payload:
          relay.desk.MessageRecorded: {message_id: input.message_id}
";

/// The wiring: one binding from `MessageSent` to `RecordMessage`, `WHEN` its `when:`, `MAPPING`
/// its extra mapping lines.
const RELAY_WIRING: &str = "\
components:
  - component: chat-service
    owns: {domains: [relay.chat]}
    accepts: {commands: [relay.chat.SendMessage]}
    publishes: {events: [relay.chat.MessageSent]}
    reached_by: network
  - component: desk-service
    owns: {domains: [relay.desk]}
    accepts: {commands: [relay.desk.RecordMessage]}
    publishes: {events: [relay.desk.MessageRecorded]}
    reached_by: network
bindings:
  - id: recorded
    when: WHEN
    invoke: {command: relay.desk.RecordMessage}
    mapping:
      message_id: event.message_id
MAPPING
    delivery: at_least_once
    on_failure: retry
";

/// Synthesizes Go for `RecordMessage` with `inputs` (`(name, type)`) each filled by `mapping`
/// (`(name, source)`), the binding conditioned by `when`, and asserts the binding is generated.
fn relay(inputs: &[(&str, &str)], mapping: &[(&str, &str)], when: &str) -> Synthesis {
    let inputs: String = inputs
        .iter()
        .map(|(name, ty)| format!("      - {{name: {name}, type: {ty}}}\n"))
        .collect();
    let mapping: String = mapping
        .iter()
        .map(|(name, source)| format!("      {name}: {source}\n"))
        .collect();
    let desk = RELAY_DESK.replace("INPUTS\n", &inputs);
    let wiring = RELAY_WIRING
        .replace("WHEN", when)
        .replace("MAPPING\n", &mapping);
    let ir = fixture(&[
        ("system.yaml", RELAY_SYSTEM),
        ("domains/chat.yaml", RELAY_CHAT),
        ("domains/desk.yaml", &desk),
        ("wiring.yaml", &wiring),
    ]);
    let synthesis = synthesize_for(&ir, Target::Go)
        .unwrap_or_else(|failure| panic!("{WRAP}: Go emits: {}", failure.to_canonical_json()));
    for kind in [
        CapabilityKind::BindingTransformation,
        CapabilityKind::BindingDelivery,
    ] {
        assert_eq!(
            synthesis.plan.disposition_of(kind, "recorded"),
            Some(&SynthesisDisposition::Generated),
            "{WRAP}: {kind:?} of `recorded` is generated"
        );
    }
    synthesis
}

const UNCONDITIONED: &str = "{event: relay.chat.MessageSent}";

fn builds(label: &str, synthesis: &Synthesis) {
    if go_available() {
        go_accepts(label, synthesis).unwrap_or_else(|log| panic!("{WRAP}: {label}: {log}"));
    }
}

#[test]
fn adv_gen2_a_required_string_into_a_doubly_optional_input_is_wrapped_twice_and_builds() {
    let synthesis = relay(
        &[("value", "Optional<Optional<String>>")],
        &[("value", "event.channel")],
        UNCONDITIONED,
    );
    builds("nested", &synthesis);
}

#[test]
fn adv_gen2_an_optional_field_into_a_doubly_optional_input_is_wrapped_once_and_builds() {
    let synthesis = relay(
        &[("value", "Optional<Optional<String>>")],
        &[("value", "event.topic")],
        UNCONDITIONED,
    );
    builds("optional-into-nested", &synthesis);
}

#[test]
fn adv_gen2_a_required_list_into_an_optional_list_input_is_wrapped_and_builds() {
    let synthesis = relay(
        &[("value", "Optional<List<String>>")],
        &[("value", "event.tags")],
        UNCONDITIONED,
    );
    builds("list", &synthesis);
}

#[test]
fn adv_gen2_a_struct_of_another_domain_into_an_optional_input_is_wrapped_and_builds() {
    let synthesis = relay(
        &[("value", "Optional<relay.chat.Place>")],
        &[("value", "event.place")],
        UNCONDITIONED,
    );
    builds("struct", &synthesis);
}

#[test]
fn adv_gen2_a_newtype_of_another_domain_into_an_optional_input_is_wrapped_and_builds() {
    let synthesis = relay(
        &[("value", "Optional<relay.chat.Label>")],
        &[("value", "event.label")],
        UNCONDITIONED,
    );
    builds("newtype", &synthesis);
}

#[test]
fn adv_gen2_a_wrap_beside_a_proved_member_in_one_conditioned_binding_builds() {
    // The transformation that checks a proved member (`input_expression`) also carries a copy
    // into an Optional input: both paths in one function.
    let synthesis = relay(
        &[("topic", "String"), ("value", "Optional<String>")],
        &[("topic", "event.topic"), ("value", "event.channel")],
        "\n      event: relay.chat.MessageSent\n      where: [defined(event.topic)]",
    );
    builds("conditioned", &synthesis);
}

#[test]
fn adv_gen2_every_wrap_at_once_builds() {
    let synthesis = relay(
        &[
            ("channel", "Optional<String>"),
            ("nested", "Optional<Optional<String>>"),
            ("tags", "Optional<List<String>>"),
            ("place", "Optional<relay.chat.Place>"),
            ("label", "Optional<relay.chat.Label>"),
            ("topic", "Optional<Optional<String>>"),
        ],
        &[
            ("channel", "event.channel"),
            ("nested", "event.channel"),
            ("tags", "event.tags"),
            ("place", "event.place"),
            ("label", "event.label"),
            ("topic", "event.topic"),
        ],
        UNCONDITIONED,
    );
    builds("all", &synthesis);
}

// ---- story 2: a served component named like a server-package file ------------------------------

fn served_domain(domain: &str) -> String {
    "\
domain: demo.D
types:
  - {name: demo.D.ClientId, kind: newtype, of: Uuid}
entities:
  - name: demo.D.Client
    identity: {name: client_id, type: demo.D.ClientId}
    fields:
      - {name: topic, type: String}
    lifecycle: {initial: Connected, states: [Connected], terminal: [Connected], transitions: []}
events:
  - name: demo.D.Handshaken
    fields: [{name: client_id, type: demo.D.ClientId}]
commands:
  - name: demo.D.Handshake
    input: [{name: topic, type: String}]
    outcomes:
      - name: accepted
        creates: demo.D.Client
        instance: client_id
        sets: {topic: input.topic}
        emits: [demo.D.Handshaken]
        payload: {demo.D.Handshaken: {client_id: {generated: true}}}
"
    .replace("demo.D", &format!("demo.{domain}"))
}

/// Go for one served component per name, each owning a domain of its own.
fn served(names: &[&str]) -> Synthesis {
    let domains: Vec<String> = (0..names.len()).map(|index| format!("d{index}")).collect();
    let system = format!(
        "format: ess/33\nsystem: demo\nversion: v1\ndomains:\n{}",
        domains
            .iter()
            .map(|domain| format!("  - demo.{domain}\n"))
            .collect::<String>()
    );
    let mut components = String::from("components:\n");
    for (name, domain) in names.iter().zip(&domains) {
        components.push_str(&format!(
            "  - component: {name}\n    reached_by: network\n    owns: {{domains: [demo.{domain}]}}\n    \
             accepts: {{commands: [demo.{domain}.Handshake]}}\n    publishes: {{events: [demo.{domain}.Handshaken]}}\n"
        ));
    }
    let texts: Vec<(String, String)> = domains
        .iter()
        .map(|domain| (format!("domains/{domain}.yaml"), served_domain(domain)))
        .collect();
    let mut documents: Vec<(&str, &str)> = vec![("system.yaml", &system)];
    for (label, text) in &texts {
        documents.push((label, text));
    }
    documents.push(("components.yaml", &components));
    let ir = fixture(&documents);
    synthesize_for(&ir, Target::Go)
        .unwrap_or_else(|failure| panic!("{SERVED}: Go emits: {}", failure.to_canonical_json()))
}

fn server_files(synthesis: &Synthesis) -> Vec<&str> {
    synthesis
        .artifacts
        .keys()
        .map(String::as_str)
        .filter(|path| path.starts_with("server/") && path.ends_with(".go"))
        .collect()
}

#[test]
fn adv_gen2_a_lone_served_component_named_server_synthesizes_and_builds() {
    // The story's own fixture shape: the only served component is `server`.
    let synthesis = served(&["server"]);
    assert_eq!(
        server_files(&synthesis),
        [
            "server/ess_memory.go",
            "server/ess_static.go",
            "server/server.go",
            "server/servercomponent.go",
            "server/wire.go"
        ],
        "{SERVED}"
    );
    if go_available() {
        go_accepts("lone-server", &synthesis).unwrap_or_else(|log| panic!("{SERVED}: {log}"));
    }
}

#[test]
fn adv_gen2_repaired_stems_that_collide_again_are_repaired_again_and_build() {
    // `server` repairs to `servercomponent`, which `server-component` is already; `wire` repairs
    // to `wirecomponent`, which `wire-component` is already.
    let synthesis = served(&["server", "server-component", "wire", "wire-component"]);
    let files = server_files(&synthesis);
    assert_eq!(
        files.len(),
        7,
        "{SERVED}: four surfaces beside server.go, wire.go and ess_static.go: {files:?}"
    );
    for kept in ["server/server.go", "server/wire.go", "server/ess_static.go"] {
        assert!(files.contains(&kept), "{SERVED}: {kept} kept: {files:?}");
    }
    if go_available() {
        go_accepts("collide-again", &synthesis).unwrap_or_else(|log| panic!("{SERVED}: {log}"));
    }
}

// ---- story 3: one clap tree for components sharing a binary ------------------------------------

const DESK_SYSTEM: &str = "\
format: ess/1
system: desk
version: v1
domains:
  - desk.pass
  - desk.audit
  - desk.note
";

const PASS: &str = include_str!("fixtures/clap-shared-binary/domains/pass.yaml.in");
const AUDIT: &str = include_str!("fixtures/clap-shared-binary/domains/audit.yaml.in");

/// `desk.note`, a copy of `desk.audit` nobody owns, so two components may both accept its command.
fn note() -> String {
    AUDIT
        .replace("desk.audit", "desk.note")
        .replace("wire: record", "wire: jot")
}

fn desk_documents(components: &str) -> Vec<(String, String)> {
    vec![
        ("system.yaml".to_owned(), DESK_SYSTEM.to_owned()),
        ("domains/pass.yaml".to_owned(), PASS.to_owned()),
        ("domains/audit.yaml".to_owned(), AUDIT.to_owned()),
        ("domains/note.yaml".to_owned(), note()),
        ("components.yaml".to_owned(), components.to_owned()),
    ]
}

fn desk_validation(components: &str) -> Result<Specification, ValidationErrors> {
    let documents = desk_documents(components);
    let borrowed: Vec<(&str, &str)> = documents
        .iter()
        .map(|(label, text)| (label.as_str(), text.as_str()))
        .collect();
    assemble(&borrowed)
}

fn desk_clap(components: &str) -> Synthesis {
    let documents = desk_documents(components);
    let borrowed: Vec<(&str, &str)> = documents
        .iter()
        .map(|(label, text)| (label.as_str(), text.as_str()))
        .collect();
    synthesize_for(&fixture(&borrowed), Target::Clap)
        .unwrap_or_else(|failure| panic!("{SHARED}: clap emits: {}", failure.to_canonical_json()))
}

/// Every `::clap::Command::new("<word>")` in the tree, minus the root and `completions`, sorted.
fn tree_words(tree: &str, root: &str) -> Vec<String> {
    let mut words: Vec<String> = tree
        .split("::clap::Command::new(\"")
        .skip(1)
        .filter_map(|rest| rest.split_once('"').map(|(word, _)| word.to_owned()))
        .filter(|word| word != root && word != "completions")
        .collect();
    words.sort();
    words
}

/// Every `Some(("<word>", …))` arm of `main.rs`, at any depth, minus `completions`, sorted.
fn dispatched_words(main: &str) -> Vec<String> {
    let mut words: Vec<String> = main
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("Some((\""))
        .filter_map(|rest| rest.split_once('"').map(|(word, _)| word.to_owned()))
        .filter(|word| word != "completions")
        .collect();
    words.sort();
    words
}

/// `audit-service` places its command at the top level; `desk-service` a group and a top-level
/// view; `note-service` a group of its own. All three share `desk`.
const MIXED: &str = "\
components:
  - component: audit-service
    owns: {domains: [desk.audit]}
    accepts: {commands: [desk.audit.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      commands: [desk.audit.RecordEntry]
  - component: desk-service
    owns: {domains: [desk.pass]}
    accepts: {commands: [desk.pass.RegisterVisit]}
    reached_by: command_line
    cli:
      binary: desk
      views: [desk.pass.ExpectedVisits]
      groups:
        - name: visits
          commands: [desk.pass.RegisterVisit]
  - component: note-service
    owns: {domains: [desk.note]}
    accepts: {commands: [desk.note.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      groups:
        - name: notes
          commands: [desk.note.RecordEntry]
";

#[test]
fn adv_gen2_tree_and_dispatch_agree_over_top_level_words_views_and_groups_of_three_components() {
    let synthesis = desk_clap(MIXED);
    let tree = &synthesis.artifacts["crates/desk-cli/src/tree.rs"].contents;
    let main = &synthesis.artifacts["crates/desk-cli/src/main.rs"].contents;
    assert_eq!(
        tree_words(tree, "desk"),
        dispatched_words(main),
        "{SHARED}: the tree defines what main dispatches:\n{tree}\n{main}"
    );
    assert_eq!(
        dispatched_words(main),
        ["expected", "jot", "notes", "record", "register", "visits"],
        "{SHARED}: every word of every component:\n{main}"
    );
}

#[test]
fn adv_gen2_a_command_top_level_in_one_and_grouped_in_another_sharing_component_is_refused() {
    let components = "\
components:
  - component: audit-service
    owns: {domains: [desk.audit]}
    accepts: {commands: [desk.audit.RecordEntry, desk.note.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      commands: [desk.note.RecordEntry]
      groups:
        - name: audit
          commands: [desk.audit.RecordEntry]
  - component: desk-service
    owns: {domains: [desk.pass]}
    accepts: {commands: [desk.pass.RegisterVisit, desk.note.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      groups:
        - name: visits
          commands: [desk.pass.RegisterVisit, desk.note.RecordEntry]
";
    let Err(errors) = desk_validation(components) else {
        panic!("{SHARED}: one command at two paths of one binary validates")
    };
    let refused: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::DuplicateDeclaration)
        .collect();
    assert_eq!(refused.len(), 1, "{SHARED}: {errors}");
    for named in [
        "`desk.note.RecordEntry`",
        "`audit-service`",
        "`desk-service`",
    ] {
        assert!(
            refused[0].message.contains(named),
            "{SHARED}: names {named}: {}",
            refused[0].message
        );
    }
}

#[test]
fn adv_gen2_a_group_named_like_another_sharing_components_top_level_command_is_refused() {
    // `audit-service` places `record` at the top level of `desk`; `note-service` names a group
    // `record`. One binary, two subcommands named `record`: a command path declared twice.
    let components = "\
components:
  - component: audit-service
    owns: {domains: [desk.audit]}
    accepts: {commands: [desk.audit.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      commands: [desk.audit.RecordEntry]
  - component: note-service
    owns: {domains: [desk.note]}
    accepts: {commands: [desk.note.RecordEntry]}
    reached_by: command_line
    cli:
      binary: desk
      groups:
        - name: record
          commands: [desk.note.RecordEntry]
";
    match desk_validation(components) {
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::DuplicateDeclaration),
            "{SHARED}: refused as a duplicate: {errors}"
        ),
        Ok(_) => {
            let synthesis = desk_clap(components);
            let tree = &synthesis.artifacts["crates/desk-cli/src/tree.rs"].contents;
            panic!(
                "{SHARED}: validation accepts a group and a command both at `desk record`, and \
                 the tree defines `record` {} times:\n{tree}",
                tree.matches("::clap::Command::new(\"record\")").count()
            );
        }
    }
}
