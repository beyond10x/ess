//! Generated network components run without a hand-written store or business implementation.
use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_synth::{synthesize_for, Target};

const MODEL: &str = include_str!("fixtures/served-notes/system.yaml");

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("served-notes.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn the_served_notes_plan_has_no_obligation() {
    let emission = synthesize_for(&model(MODEL), Target::Rust).unwrap();
    assert_eq!(emission.plan.obligations().count(), 0);
}

fn emitted_text(text: &str, target: Target, case: &str) -> PathBuf {
    emitted_ir(&model(text), target, case)
}

fn emitted_ir(ir: &EssIr, target: Target, case: &str) -> PathBuf {
    emitted_layout(ir, target, ess_synth::OutputLayout::Workspace, case)
}

fn emitted_layout(
    ir: &EssIr,
    target: Target,
    layout: ess_synth::OutputLayout,
    case: &str,
) -> PathBuf {
    let synthesis = ess_synth::synthesize_laid_out(ir, target, layout).unwrap();
    let label = match target {
        Target::Rust => "rust",
        Target::Go => "go",
        _ => unreachable!(),
    };
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "served-entry-{label}-{case}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".expected-model-digest"),
        &synthesis.plan.provenance.source_digest,
    )
    .unwrap();
    for (relative, artifact) in synthesis.artifacts {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    root
}

fn binary(target: Target) -> &'static Path {
    static RUST: OnceLock<PathBuf> = OnceLock::new();
    static GO: OnceLock<PathBuf> = OnceLock::new();
    match target {
        Target::Rust => RUST.get_or_init(|| build(MODEL, target, "fixture")),
        Target::Go => GO.get_or_init(|| build(MODEL, target, "fixture")),
        _ => unreachable!(),
    }
}

fn build(text: &str, target: Target, case: &str) -> PathBuf {
    let root = emitted_text(text, target, case);
    build_at(&root, target, "notebook", "notes", &[])
}

fn build_at(root: &Path, target: Target, system: &str, component: &str, extra: &[&str]) -> PathBuf {
    let name = format!("{component}-server");
    let (output, binary) = match target {
        Target::Go => {
            assert!(
                root.join(format!("cmd/{name}/main.go")).is_file(),
                "missing generated Go entry point"
            );
            let formatted = Command::new("gofmt")
                .args(["-l", "."])
                .current_dir(root)
                .output()
                .unwrap();
            assert!(
                formatted.status.success() && formatted.stdout.is_empty(),
                "generated Go needs formatting in {}:\n{}\n{}",
                root.display(),
                String::from_utf8_lossy(&formatted.stdout),
                String::from_utf8_lossy(&formatted.stderr)
            );
            let vetted = Command::new("go")
                .args(["vet", "./..."])
                .current_dir(root)
                .env("GOWORK", "off")
                .env("GOPROXY", "off")
                .env("GOTOOLCHAIN", "local")
                .output()
                .unwrap();
            assert!(
                vetted.status.success(),
                "{}",
                String::from_utf8_lossy(&vetted.stderr)
            );
            let packages = Command::new("go")
                .args(["build", "./..."])
                .current_dir(root)
                .env("GOWORK", "off")
                .env("GOPROXY", "off")
                .output()
                .unwrap();
            assert!(
                packages.status.success(),
                "{}",
                String::from_utf8_lossy(&packages.stderr)
            );
            (
                Command::new("go")
                    .args(["build", "-o", &name, &format!("./cmd/{name}")])
                    .current_dir(root)
                    .env("GOWORK", "off")
                    .env("GOPROXY", "off")
                    .output()
                    .unwrap(),
                root.join(&name),
            )
        }
        Target::Rust => {
            static BUILDS: std::sync::Mutex<()> = std::sync::Mutex::new(());
            let _build = BUILDS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            assert!(
                root.join(format!("crates/{system}-server/src/bin/{name}.rs"))
                    .is_file()
                    || root.join(format!("src/bin/{name}.rs")).is_file(),
                "missing generated Rust entry point"
            );
            let target_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("served-entry-rust-build-{}", std::process::id()));
            clear_generated_packages(root, &target_dir);
            let output = Command::new(std::env::var_os("CARGO").unwrap())
                .args(["build", "--offline", "--bin", &name, "--target-dir"])
                .arg(&target_dir)
                .args(extra)
                .current_dir(root)
                .env_remove("CARGO_TARGET_DIR")
                .env_remove("CARGO_ENCODED_RUSTFLAGS")
                .env_remove("RUSTC_WRAPPER")
                .env("RUSTFLAGS", "-D warnings")
                .output()
                .unwrap();
            let binary = root.join(&name);
            if output.status.success() {
                std::fs::copy(target_dir.join(format!("debug/{name}")), &binary).unwrap();
            }
            (output, binary)
        }
        _ => unreachable!(),
    };
    assert!(
        output.status.success(),
        "{}: {}",
        root.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    binary
}

#[test]
fn the_generated_context_assigns_distinct_uuids() {
    let text = MODEL.replace("of: Integer", "of: Uuid")
        .replacen("    input:\n      - {name: note_id, type: notebook.notes.NoteId}\n      - {name: text, type: String}", "    input:\n      - {name: text, type: String}", 1)
        .replace("notebook.notes.NoteAdded: {note_id: input.note_id, text: input.text}", "notebook.notes.NoteAdded: {note_id: {generated: true}, text: input.text}");
    for target in [Target::Rust, Target::Go] {
        let binary = build(&text, target, "uuid");
        let server = Server::start(&binary, true, None);
        let mut ids = std::collections::BTreeSet::new();
        for _ in 0..8 {
            let (status, answer) = server.json(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                serde_json::json!({"text": "generated"}),
            );
            assert_eq!(status, 202, "{answer}");
            let id = answer["published"][0]["payload"]["note_id"]
                .as_str()
                .unwrap();
            let uuid = uuid::Uuid::parse_str(id).unwrap();
            assert_eq!(uuid.get_version_num(), 4);
            assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
            assert!(ids.insert(id.to_owned()));
        }
        let (_, answer) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(answer["rows"].as_array().unwrap().len(), 8);
    }
}

#[test]
fn the_generated_context_reads_the_system_clock() {
    let text = MODEL.replacen("      - {name: text, type: String}", "      - {name: text, type: String}\n      - {name: created_at, type: Timestamp}", 1)
        .replace("sets: {text: input.text}", "sets: {text: input.text, created_at: {generated: true}}")
        .replace("      - {name: state, type: notebook.notes.Note.State}", "      - {name: state, type: notebook.notes.Note.State}\n      - {name: created_at, type: Timestamp}");
    exercise_clock(&text, "clock");
}

#[test]
fn the_generated_context_reads_the_clock_through_a_newtype() {
    let text = MODEL
        .replacen("types:\n", "types:\n  - {name: notebook.notes.CreatedAt, kind: newtype, of: Timestamp}\n", 1)
        .replacen("      - {name: text, type: String}", "      - {name: text, type: String}\n      - {name: created_at, type: notebook.notes.CreatedAt}", 1)
        .replace("sets: {text: input.text}", "sets: {text: input.text, created_at: {generated: true}}")
        .replace("      - {name: state, type: notebook.notes.Note.State}", "      - {name: state, type: notebook.notes.Note.State}\n      - {name: created_at, type: notebook.notes.CreatedAt}");
    exercise_clock(&text, "clock-newtype");
}

fn exercise_clock(text: &str, case: &str) {
    for target in [Target::Rust, Target::Go] {
        let binary = build(text, target, case);
        let server = Server::start(&binary, true, None);
        let before = time::OffsetDateTime::now_utc();
        let (status, answer) = server.json(
            "POST",
            "/notes/commands/add-note",
            Some("Writer"),
            serde_json::json!({"note_id": 1, "text": "clock"}),
        );
        assert_eq!(status, 202, "{answer}");
        let (_, answer) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        let observed = time::OffsetDateTime::parse(
            answer["rows"][0]["created_at"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        assert!(observed >= before && observed <= time::OffsetDateTime::now_utc());
    }
}

fn refuses(binary: &Path, names: &[&str]) {
    let output = Command::new(binary)
        .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        output.stdout.is_empty(),
        "refused startup must not announce ready"
    );
    let message = String::from_utf8_lossy(&output.stderr);
    for name in names {
        assert!(message.contains(name), "expected {name}: {message}");
    }
}

#[test]
fn an_entry_point_needing_caller_attributes_refuses_to_start_naming_them() {
    let text = MODEL
        .replace(
            "    may: [",
            "    attributes: [{name: author, type: String}]\n    may: [",
        )
        .replace("sets: {text: input.text}", "sets: {text: {caller: author}}")
        .replace("text: input.text}", "text: {caller: author}}");
    for target in [Target::Rust, Target::Go] {
        refuses(
            &build(&text, target, "caller"),
            &["caller attribute: author"],
        );
    }
}

#[test]
fn unsupported_memory_context_has_no_panic_or_fabricated_answer() {
    let text = MODEL
        .replace(
            "sets: {text: input.text}",
            "sets: {text: {generated: true}}",
        )
        .replace("text: input.text}", "text: {generated: true}}");
    for target in [Target::Rust, Target::Go] {
        let emission = synthesize_for(&model(&text), target).unwrap();
        for (path, artifact) in emission.artifacts {
            assert!(
                !artifact.contents.contains("panic!") && !artifact.contents.contains("panic("),
                "{target:?} {path}: unsupported context must return a typed refusal"
            );
        }
    }
}

fn fallible_context_model() -> String {
    let commands = "  - name: notebook.notes.RemoveNote\n    input: [{name: note_id, type: notebook.notes.NoteId}]\n    outcomes:\n      - name: removed\n        deletes: notebook.notes.Note\n        instance: note_id\n        emits: [notebook.notes.NoteArchived]\n        payload:\n          notebook.notes.NoteArchived: {note_id: input.note_id, author: {caller: author}}\n      - {name: unknown, unknown_instance: true, error: notebook.notes.NoSuchNote}\n  - name: notebook.notes.Probe\n    input: []\n    outcomes:\n      - {name: external, external: an external choice, error: notebook.notes.NoSuchNote}\n      - {name: otherwise, error: notebook.notes.AlreadyArchived}\n";
    MODEL.replace("    may: [", "    attributes: [{name: author, type: String}]\n    may: [")
        .replace("      - {name: text, type: String}\n  - name: notebook.notes.NoteArchived", "      - {name: text, type: String}\n      - {name: stamp, type: String}\n  - name: notebook.notes.NoteArchived")
        .replace("notebook.notes.NoteAdded: {note_id: input.note_id, text: input.text}", "notebook.notes.NoteAdded: {note_id: input.note_id, text: input.text, stamp: {generated: true}}")
        .replace("      - {name: note_id, type: notebook.notes.NoteId}\nerrors:", "      - {name: note_id, type: notebook.notes.NoteId}\n      - {name: author, type: String}\nerrors:")
        .replace("notebook.notes.NoteArchived: {note_id: input.note_id}", "notebook.notes.NoteArchived: {note_id: input.note_id, author: {caller: author}}")
        .replace("views:\n", &format!("{commands}views:\n"))
        .replace("notebook.notes.AddNote, notebook.notes.ArchiveNote]", "notebook.notes.AddNote, notebook.notes.ArchiveNote, notebook.notes.RemoveNote, notebook.notes.Probe]")
}

#[test]
fn fallible_context_propagates_before_effects_and_keeps_legacy_implementations() {
    for target in [Target::Rust, Target::Go] {
        let root = emitted_text(&fallible_context_model(), target, "fallible-context");
        let output = match target {
            Target::Rust => {
                std::fs::create_dir_all(root.join("crates/notebook-server/tests")).unwrap();
                std::fs::write(
                    root.join("crates/notebook-server/tests/context.rs"),
                    include_str!("fixtures/served-notes/context.rs"),
                )
                .unwrap();
                Command::new(std::env::var_os("CARGO").unwrap())
                    .args([
                        "test",
                        "--offline",
                        "-p",
                        "notebook-server",
                        "--test",
                        "context",
                        "--target-dir",
                    ])
                    .arg(root.join("probe-target"))
                    .current_dir(&root)
                    .env_remove("CARGO_TARGET_DIR")
                    .env_remove("CARGO_ENCODED_RUSTFLAGS")
                    .env_remove("RUSTC_WRAPPER")
                    .env("RUSTFLAGS", "-D warnings")
                    .output()
                    .unwrap()
            }
            Target::Go => {
                std::fs::write(root.join("server/context_test.go"), GO_CONTEXT).unwrap();
                Command::new("go")
                    .args(["test", "./server", "-run", "TestContext", "-count=1"])
                    .current_dir(&root)
                    .env("GOWORK", "off")
                    .env("GOPROXY", "off")
                    .output()
                    .unwrap()
            }
            _ => unreachable!(),
        };
        std::fs::write(
            root.join("context-test.log"),
            [output.stdout.as_slice(), output.stderr.as_slice()].concat(),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{target:?}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fallible_context_names_do_not_collide_with_authored_types_or_commands() {
    let text = fallible_context_model()
        .replace("notebook.notes.Probe", "notebook.notes.TryExternal")
        .replace("notebook.notes.NoteId", "notebook.notes.TryContext")
        .replace("types:\n", "types:\n  - {name: notebook.notes.FallibleContext, kind: newtype, of: String}\n  - {name: notebook.notes.UnmetContext, kind: newtype, of: String}\n");
    for target in [Target::Rust, Target::Go] {
        refuses(
            &build(&text, target, "fallible-names"),
            &[
                "assigned value: String",
                "caller attribute: author",
                "external branch answer",
            ],
        );
    }
}

const GO_CONTEXT: &str = r#"package server

import (
	"example.invalid/notebook/types/behaviour"
	"example.invalid/notebook/types/notes"
	"testing"
)

type legacyContext struct{}
func (legacyContext) CallerAuthor() (string, bool) { return "legacy caller", true }
func (legacyContext) GenerateString() string { return "legacy assigned" }
func (legacyContext) External(string, string) bool { return false }

func TestContextCompatibility(t *testing.T) {
	ports := behaviour.Ports{NewMemoryPorts().NoteStorage, legacyContext{}} // Legacy unkeyed construction stays source-compatible.
	generated := behaviour.New(ports)
	if outcome, err := generated.AddNote(notes.AddNote{NoteId: notes.NewNoteId(7), Text: "seven"}); err != nil || outcome == nil { t.Fatal("legacy assigned context", outcome, err) }
	if outcome, err := generated.ArchiveNote(notes.ArchiveNote{NoteId: notes.NewNoteId(7)}); err != nil || outcome == nil { t.Fatal("legacy caller context", outcome, err) }
	if outcome, err := generated.Probe(notes.Probe{}); err != nil || outcome == nil { t.Fatal("legacy external context", outcome, err) }
	generated = behaviour.NewWithContext(ports, &MemoryContext{})
	if outcome, err := generated.AddNote(notes.AddNote{NoteId: notes.NewNoteId(8), Text: "eight"}); outcome != nil || err == nil || err.Source != "assigned value: String" { t.Fatal("fallible context must take precedence", outcome, err) }
}

func TestContextErrorsBeforeEffects(t *testing.T) {
	ports := NewMemoryPorts()
	generated := behaviour.NewWithContext(ports, &MemoryContext{})
	if _, _, err := (&MemoryContext{}).TryCallerAuthor(); err == nil || err.Source != "caller attribute: author" { t.Fatal("caller refusal", err) }
	if _, err := (&MemoryContext{}).TryGenerateString(); err == nil || err.Source != "assigned value: String" { t.Fatal("assignment refusal", err) }
	if _, err := (&MemoryContext{}).TryExternal("command", "outcome"); err == nil || err.Source != "external branch answer" { t.Fatal("external refusal", err) }
	if outcome, err := generated.AddNote(notes.AddNote{NoteId: notes.NewNoteId(7), Text: "seven"}); outcome != nil || err == nil || err.Source != "assigned value: String" { t.Fatal("late creation error", outcome, err) }
	if len(ports.NoteStorage.List()) != 0 { t.Fatal("late event assignment left the created row") }
	ports.NoteStorage.Put(notes.NoteSnapshot{State: notes.NoteStateActive{}, Data: notes.NoteData{NoteId: notes.NewNoteId(7), Text: "seven"}})
	if outcome, err := generated.ArchiveNote(notes.ArchiveNote{NoteId: notes.NewNoteId(7)}); outcome != nil || err == nil || err.Source != "caller attribute: author" { t.Fatal("late move error", outcome, err) }
	row, found := ports.NoteStorage.Get(notes.NewNoteId(7))
	if _, active := row.State.(notes.NoteStateActive); !found || !active { t.Fatal("late event assignment changed state") }
	if outcome, err := generated.RemoveNote(notes.RemoveNote{NoteId: notes.NewNoteId(7)}); outcome != nil || err == nil || err.Source != "caller attribute: author" { t.Fatal("late delete error", outcome, err) }
	if len(ports.NoteStorage.List()) != 1 { t.Fatal("late event assignment deleted the row") }
	if outcome, err := generated.Probe(notes.Probe{}); outcome != nil || err == nil || err.Source != "external branch answer" { t.Fatal("external propagation", outcome, err) }
}
"#;

#[test]
fn other_assigned_values_and_external_answers_remain_named_obligations() {
    let assigned = MODEL
        .replace(
            "sets: {text: input.text}",
            "sets: {text: {generated: true}}",
        )
        .replace("text: input.text}", "text: {generated: true}}");
    let external = MODEL.replace("      - name: added", "      - name: declined\n        external: an external decision\n        error: notebook.notes.NoSuchNote\n      - name: added");
    for target in [Target::Rust, Target::Go] {
        refuses(
            &build(&assigned, target, "assigned"),
            &["assigned value: String"],
        );
        refuses(
            &build(&external, target, "external"),
            &["external branch answer"],
        );
    }
}

#[test]
fn an_entry_point_with_owed_commands_refuses_to_start_naming_them() {
    let ir = gatepass();
    for target in [Target::Rust, Target::Go] {
        let root = emitted_ir(&ir, target, "gatepass");
        refuses(
            &build_at(&root, target, "gatepass", "pass-service", &[]),
            &["gatepass.visit.RegisterVisit"],
        );
    }
}

fn gatepass() -> EssIr {
    let sources = [
        (
            "system.yaml",
            include_str!("../../../../examples/gatepass/system.yaml"),
        ),
        (
            "visit.yaml",
            include_str!("../../../../examples/gatepass/domains/visit.yaml"),
        ),
        (
            "components.yaml",
            include_str!("../../../../examples/gatepass/components.yaml"),
        ),
    ];
    let specification = Specification::assemble(
        sources
            .into_iter()
            .map(|(path, text)| (Source::new(path), RawSpecFile::parse(text).unwrap())),
    )
    .unwrap();
    compile(&specification, &SourceMap::new()).unwrap()
}

#[test]
fn memory_ports_preserve_the_single_obligation_stub() {
    for target in [Target::Rust, Target::Go] {
        let emission = synthesize_for(&gatepass(), target).unwrap();
        let marker = if target == Target::Rust {
            "source: \"gatepass.visit.RegisterVisit\""
        } else {
            "Source: \"gatepass.visit.RegisterVisit\""
        };
        let stubs: usize = emission
            .artifacts
            .iter()
            .filter(|(path, _)| {
                Path::new(path).extension().is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("rs") || extension.eq_ignore_ascii_case("go")
                })
            })
            .map(|(_, artifact)| artifact.contents.matches(marker).count())
            .sum();
        assert_eq!(stubs, 1, "{target:?}: one stub per declared obligation");
    }
}

#[test]
fn the_single_crate_entry_builds_and_serves_with_the_server_feature() {
    let emission =
        ess_synth::synthesize_laid_out(&model(MODEL), Target::Rust, ess_synth::OutputLayout::Crate)
            .unwrap();
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("served-entry-single-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".expected-model-digest"),
        &emission.plan.provenance.source_digest,
    )
    .unwrap();
    for (relative, artifact) in emission.artifacts {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let binary = build_at(
        &root,
        Target::Rust,
        "notebook",
        "notes",
        &["--features", "server"],
    );
    let server = Server::start(&binary, true, None);
    assert_eq!(
        server
            .json(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                serde_json::json!({"note_id": 1, "text": "one crate"})
            )
            .0,
        202
    );
    let output = Command::new(std::env::var_os("CARGO").unwrap())
        .args(["check", "--offline", "--no-default-features"])
        .current_dir(&root)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

const SIBLING: &str = "domain: notebook.other
events:
  - name: notebook.other.Answered
    fields: [{name: answer, type: String}]
commands:
  - name: notebook.other.Answer
    input: [{name: note_id, type: notebook.notes.NoteId}]
    outcomes:
      - name: answered
        emits: [notebook.other.Answered]
        payload:
          notebook.other.Answered: {answer: {generated: true}}
components:
  - component: other
    owns: {domains: [notebook.other]}
    accepts: {commands: [notebook.other.Answer]}
    publishes: {events: [notebook.other.Answered]}
    reached_by: network
";

fn combined(first: &str, second: &str) -> EssIr {
    let specification = Specification::assemble(
        [("notes.yaml", first), ("other.yaml", second)]
            .into_iter()
            .map(|(path, text)| (Source::new(path), RawSpecFile::parse(text).unwrap())),
    )
    .unwrap();
    compile(&specification, &SourceMap::new()).unwrap()
}

#[test]
fn only_reachable_context_needs_prevent_a_component_from_starting() {
    let binding = "bindings:
  - id: answer-on-note
    when: {event: notebook.notes.NoteAdded}
    invoke: {command: notebook.other.Answer}
    mapping: {note_id: event.note_id}
    delivery: at_least_once
    on_failure: drop
";
    let independent = combined(MODEL, SIBLING);
    let linked = combined(&format!("{MODEL}{binding}"), SIBLING);
    for target in [Target::Rust, Target::Go] {
        let root = emitted_ir(&independent, target, "unrelated-context");
        let binary = build_at(&root, target, "notebook", "notes", &[]);
        let server = Server::start(&binary, true, None);
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/add-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": 1, "text": "unrelated"})
                )
                .0,
            202
        );
        refuses(
            &build_at(&root, target, "notebook", "other", &[]),
            &["assigned value: String"],
        );
        let root = emitted_ir(&linked, target, "reachable-context");
        refuses(
            &build_at(&root, target, "notebook", "notes", &[]),
            &["assigned value: String"],
        );
    }
}

#[test]
fn generated_names_do_not_hide_ports_or_runtime_imports() {
    for target in [Target::Rust, Target::Go] {
        let renamed = MODEL.replace("notebook.notes", "notebook.rand")
            .replace("domain: notebook.rand", "domain: notebook.rand\nnaming: {wire: notes}")
            .replace("{name: notebook.rand.NoteId, kind: newtype, of: Integer}", "{name: notebook.rand.NoteId, kind: newtype, of: Integer, naming: {code: StorageKey}}");
        let binary = build(&renamed, target, "renamed");
        let server = Server::start(&binary, true, None);
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/add-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": 3, "text": "renamed"})
                )
                .0,
            202
        );
    }
    for reserved in ["memory", "static-assets", "entry"] {
        let renamed = MODEL.replace("component: notes", &format!("component: {reserved}"));
        let failure = synthesize_for(&model(&renamed), Target::Rust)
            .err()
            .expect("allocated helper module collision must be explicit");
        assert!(
            failure.to_string().contains(reserved)
                || failure.to_string().contains(&reserved.replace('-', "_"))
        );
        let root = emitted_text(&renamed, Target::Go, reserved);
        let binary = build_at(&root, Target::Go, "notebook", reserved, &[]);
        let server = Server::start(&binary, true, None);
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/add-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": 4, "text": "reserved"})
                )
                .0,
            202
        );
    }
}

#[test]
fn a_specification_without_network_reach_gets_no_memory_or_entry() {
    let text = MODEL.replace("    reached_by: network\n", "");
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&model(&text), target).unwrap();
        assert!(!synthesis
            .artifacts
            .keys()
            .any(|path| path.contains("-server/")
                || path.starts_with("cmd/")
                || path.ends_with("ess_memory.go")));
    }
}

#[test]
fn a_non_network_domain_named_memory_remains_available() {
    let text = MODEL
        .replace("notebook.notes", "notebook.memory")
        .replace("    reached_by: network\n", "");
    for (layout, label, package) in [
        (
            ess_synth::OutputLayout::Workspace,
            "memory-local-workspace",
            "notebook-types",
        ),
        (
            ess_synth::OutputLayout::Crate,
            "memory-local-crate",
            "notebook",
        ),
    ] {
        let root = emitted_layout(&model(&text), Target::Rust, layout, label);
        let output = Command::new(std::env::var_os("CARGO").unwrap())
            .args(["check", "--offline", "--lib", "-p", package])
            .current_dir(root)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("RUSTFLAGS", "-D warnings")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn a_network_domain_named_memory_remains_available() {
    let text = MODEL.replace("notebook.notes", "notebook.memory").replace(
        "domain: notebook.memory",
        "domain: notebook.memory\nnaming: {wire: notes}",
    );
    for (layout, label, features) in [
        (
            ess_synth::OutputLayout::Workspace,
            "memory-network-workspace",
            &[][..],
        ),
        (
            ess_synth::OutputLayout::Crate,
            "memory-network-crate",
            &["--features", "server"][..],
        ),
    ] {
        let root = emitted_layout(&model(&text), Target::Rust, layout, label);
        let binary = build_at(&root, Target::Rust, "notebook", "notes", features);
        let server = Server::start(&binary, true, None);
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/add-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": 42, "text": "domain memory"})
                )
                .0,
            202
        );
        let (status, rows) =
            server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(status, 200);
        assert_eq!(rows["rows"][0]["note_id"], 42);
        assert_eq!(rows["rows"][0]["text"], "domain memory");
    }
}

#[test]
fn reusable_types_keep_the_default_dependency_graph_empty_and_build_for_wasm() {
    for (layout, label, package) in [
        (
            ess_synth::OutputLayout::Workspace,
            "library-workspace",
            "notebook-types",
        ),
        (ess_synth::OutputLayout::Crate, "library-crate", "notebook"),
    ] {
        let root = emitted_layout(&model(MODEL), Target::Rust, layout, label);
        let tree = Command::new(std::env::var_os("CARGO").unwrap())
            .args(["tree", "--offline", "-p", package, "--edges", "normal"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            tree.status.success(),
            "{}",
            String::from_utf8_lossy(&tree.stderr)
        );
        let tree = String::from_utf8(tree.stdout).unwrap();
        assert_eq!(
            tree.lines().count(),
            1,
            "default library dependencies: {tree}"
        );
        let built = Command::new(std::env::var_os("CARGO").unwrap())
            .args([
                "check",
                "--offline",
                "--lib",
                "-p",
                package,
                "--target",
                "wasm32-unknown-unknown",
            ])
            .current_dir(root)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("RUSTFLAGS", "-D warnings")
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
    }
}

#[test]
fn generated_runtime_dependencies_respect_the_minimum_rust_version() {
    let rustc = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(rustc.status.success());
    let rustc = String::from_utf8(rustc.stdout).unwrap();
    let host = rustc
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .unwrap();
    let major_minor = |version: &str| {
        let mut parts = version.split('.').map(|part| part.parse::<u32>().unwrap());
        (parts.next().unwrap(), parts.next().unwrap())
    };
    let minimum = major_minor(env!("CARGO_PKG_RUST_VERSION"));
    for (layout, label, features) in [
        (
            ess_synth::OutputLayout::Workspace,
            "msrv-workspace",
            &[][..],
        ),
        (
            ess_synth::OutputLayout::Crate,
            "msrv-crate",
            &["--features", "server"][..],
        ),
    ] {
        let root = emitted_layout(&model(MODEL), Target::Rust, layout, label);
        let metadata = Command::new(std::env::var_os("CARGO").unwrap())
            .args([
                "metadata",
                "--offline",
                "--format-version",
                "1",
                "--filter-platform",
                host,
            ])
            .args(features)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            metadata.status.success(),
            "{}",
            String::from_utf8_lossy(&metadata.stderr)
        );
        let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
        let active: std::collections::BTreeSet<_> = metadata["resolve"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node["id"].as_str().unwrap())
            .collect();
        let incompatible: Vec<_> = metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|package| active.contains(package["id"].as_str().unwrap()))
            .filter(|package| {
                package["rust_version"]
                    .as_str()
                    .is_some_and(|version| major_minor(version) > minimum)
            })
            .map(|package| {
                format!(
                    "{} {} requires Rust {}",
                    package["name"], package["version"], package["rust_version"]
                )
            })
            .collect();
        assert!(
            incompatible.is_empty(),
            "generated dependencies exceed Rust {}: {incompatible:?}",
            env!("CARGO_PKG_RUST_VERSION")
        );
    }
}

#[test]
fn decimal_identity_preserves_rendering_equality_and_order_over_http() {
    let text = MODEL.replace("of: Integer", "of: Decimal");
    for target in [Target::Go, Target::Rust] {
        let binary = build(&text, target, "numeric-identity");
        let server = Server::start(&binary, true, None);
        for (id, text) in [
            ("1", "old"),
            ("1.0", "one"),
            ("-0", "old zero"),
            ("0.0", "zero"),
            ("100000000000000000000000", "huge"),
            ("100000000000000000000000.0", "huge replacement"),
        ] {
            let response = server.json(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                serde_json::json!({"note_id": id, "text": text}),
            );
            assert_eq!(response.0, 202, "{target:?} {id}: {response:?}");
        }
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        let labels = rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["text"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            ["old zero", "zero", "old", "one", "huge", "huge replacement"],
            "{target:?}"
        );
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/archive-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": "1.0"})
                )
                .0,
            202
        );
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"][2]["state"], "Active");
        assert_eq!(rows["rows"][3]["state"], "Archived");
    }
}

#[test]
fn structural_identities_use_typed_order_and_ignore_map_insertion_order() {
    let text = MODEL.replace("  - {name: notebook.notes.NoteId, kind: newtype, of: Integer}", "  - name: notebook.notes.NoteId\n    kind: struct\n    fields:\n      - {name: sequence, type: Integer}\n      - {name: labels, type: 'Map<String, List<Optional<Integer>>>'}\n      - {name: kind, type: notebook.notes.Kind}\n      - {name: flag, type: Boolean}\n      - {name: bytes, type: Bytes}\n  - {name: notebook.notes.Kind, kind: enum, variants: [Zulu, Alpha]}");
    for target in [Target::Rust, Target::Go] {
        let binary = build(&text, target, "structural-identity");
        let server = Server::start(&binary, true, None);
        // Raw request bodies deliberately preserve opposite map member order.
        for (id, label) in [
            (
                r#"{"sequence":10,"labels":{"z":[null,2],"a":[1]},"kind":"Zulu","flag":false,"bytes":"AQI="}"#,
                "ten",
            ),
            (
                r#"{"sequence":2,"labels":{"z":[null,2],"a":[1]},"kind":"Zulu","flag":false,"bytes":"AQI="}"#,
                "old",
            ),
            (
                r#"{"sequence":2,"labels":{"a":[1],"z":[null,2]},"kind":"Zulu","flag":false,"bytes":"AQI="}"#,
                "two",
            ),
            (
                r#"{"sequence":2,"labels":{"a":[1],"z":[null,2]},"kind":"Zulu","flag":true,"bytes":"AQI="}"#,
                "true",
            ),
            (
                r#"{"sequence":2,"labels":{"a":[1],"z":[null,2]},"kind":"Zulu","flag":false,"bytes":"AA=="}"#,
                "zero byte",
            ),
        ] {
            let response = server.request(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                &format!("{{\"note_id\":{id},\"text\":\"{label}\"}}"),
            );
            assert_eq!(
                response.0,
                202,
                "{target:?}: {}",
                String::from_utf8_lossy(&response.1)
            );
        }
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"].as_array().unwrap().len(), 4);
        assert_eq!(rows["rows"][0]["text"], "zero byte");
        assert_eq!(rows["rows"][1]["text"], "two");
        assert_eq!(rows["rows"][2]["text"], "true");
        assert_eq!(rows["rows"][3]["text"], "ten");
        assert_eq!(server.json("POST", "/notes/commands/archive-note", Some("Writer"), serde_json::json!({"note_id":{"sequence":2,"labels":{"z":[null,2],"a":[1]},"kind":"Zulu","flag":false,"bytes":"AQI="}})).0, 202);
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"][0]["state"], "Active");
        assert_eq!(rows["rows"][1]["state"], "Archived");
    }
}

#[test]
fn json_identity_preserves_number_spelling_and_member_order_over_http() {
    let text = MODEL.replace("of: Integer", "of: Json");
    for target in [Target::Rust, Target::Go] {
        let binary = build(&text, target, "json-identity");
        let server = Server::start(&binary, true, None);
        for (id, label) in [
            ("1", "one"),
            ("1.0", "fraction"),
            ("1e0", "exponent"),
            ("-0", "negative zero"),
            ("0", "zero"),
            (r#"{"b":1,"a":2}"#, "ba"),
            (r#"{"a":2,"b":1}"#, "ab"),
            ("1.0", "replaced"),
        ] {
            let response = server.request(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                &format!("{{\"note_id\":{id},\"text\":\"{label}\"}}"),
            );
            assert_eq!(
                response.0,
                202,
                "{target:?}: {}",
                String::from_utf8_lossy(&response.1)
            );
        }
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        let labels = rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["text"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                "negative zero",
                "zero",
                "one",
                "replaced",
                "exponent",
                "ab",
                "ba"
            ],
            "{target:?}"
        );
        assert_eq!(
            server
                .request(
                    "POST",
                    "/notes/commands/archive-note",
                    Some("Writer"),
                    r#"{"note_id":1.0}"#
                )
                .0,
            202
        );
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"][2]["state"], "Active");
        assert_eq!(rows["rows"][3]["state"], "Archived");
        assert_eq!(rows["rows"][4]["state"], "Active");
    }
}

#[test]
fn go_memory_ports_compare_decoded_json_values_without_changing_native_equality() {
    let root = emitted_text(
        &MODEL.replace("of: Integer", "of: Json"),
        Target::Go,
        "json-port",
    );
    std::fs::write(
        root.join("server/memory_keys_test.go"),
        r#"package server

import (
	"example.invalid/notebook/types/notes"
	"example.invalid/notebook/types/primitives"
	"testing"
)

func TestDecodedIdentity(t *testing.T) {
	compact, spaced := primitives.NewJson("[1]"), primitives.NewJson("[ 1 ]")
	explicitNull, zero := primitives.NewJson("null"), (primitives.Json{})
	if compact == spaced || explicitNull == zero {
		t.Fatal("native primitive equality changed")
	}
	store := NewMemoryPorts().NoteStorage
	for _, value := range []primitives.Json{compact, spaced, explicitNull, zero} {
		store.Put(notes.NoteSnapshot{Data: notes.NoteData{NoteId: notes.NewNoteId(value)}})
	}
	if len(store.List()) != 2 {
		t.Fatal("memory keys did not compare decoded model values")
	}
	if _, found := store.Get(notes.NewNoteId(spaced)); !found {
		t.Fatal("spaced identity was not found")
	}
	store.Delete(notes.NewNoteId(compact))
	if len(store.List()) != 1 {
		t.Fatal("equivalent identity did not delete exactly one row")
	}
	for _, text := range []string{"broken", "broken", "other", "[1] trailing", "[1]"} {
		store.Put(notes.NoteSnapshot{Data: notes.NoteData{NoteId: notes.NewNoteId(primitives.NewJson(text))}})
	}
	if len(store.List()) != 5 {
		t.Fatal("invalid raw identities must replace only identical raw text and remain distinct from valid values")
	}
	if row, found := store.Get(notes.NewNoteId(primitives.NewJson("broken"))); !found || row.Data.NoteId.Value().Value() != "broken" {
		t.Fatal("invalid constructor identity must be retrievable without a panic")
	}
	store.Delete(notes.NewNoteId(primitives.NewJson("[1] trailing")))
	if _, found := store.Get(notes.NewNoteId(primitives.NewJson("[1]"))); !found {
		t.Fatal("invalid raw prefix must not delete a valid decoded key")
	}
}
"#,
    )
    .unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "./server",
            "-run",
            "TestDecodedIdentity",
            "-count=1",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_stateless_served_component_needs_no_storage() {
    let text = "format: ess/18\nsystem: notebook\nversion: v1\ndomain: notebook.notes\nevents:\n  - {name: notebook.notes.Ponged, fields: []}\ncommands:\n  - name: notebook.notes.Ping\n    input: []\n    outcomes: [{name: pong, emits: [notebook.notes.Ponged]}]\ncomponents:\n  - component: notes\n    owns: {domains: [notebook.notes]}\n    accepts: {commands: [notebook.notes.Ping]}\n    publishes: {events: [notebook.notes.Ponged]}\n    reached_by: network\n";
    for target in [Target::Rust, Target::Go] {
        let binary = build(text, target, "empty");
        let server = Server::start(&binary, false, None);
        assert_eq!(server.request("GET", "/openapi.json", None, "").0, 200);
    }
}

#[test]
fn generated_stores_replace_and_remove_only_the_addressed_identity() {
    let deletion = "  - name: notebook.notes.RemoveNote\n    naming: {wire: remove-note}\n    input: [{name: note_id, type: notebook.notes.NoteId}]\n    outcomes:\n      - name: removed\n        deletes: notebook.notes.Note\n        instance: note_id\n        emits: [notebook.notes.NoteArchived]\n        payload:\n          notebook.notes.NoteArchived: {note_id: input.note_id}\n      - {name: unknown, unknown_instance: true, error: notebook.notes.NoSuchNote}\n";
    let text = MODEL
        .replace("views:\n", &format!("{deletion}views:\n"))
        .replace(
            "notebook.notes.AddNote, notebook.notes.ArchiveNote]",
            "notebook.notes.AddNote, notebook.notes.ArchiveNote, notebook.notes.RemoveNote]",
        );
    for target in [Target::Rust, Target::Go] {
        let binary = build(&text, target, "delete");
        let server = Server::start(&binary, true, None);
        for (id, text) in [(10, "ten"), (2, "old"), (2, "new")] {
            assert_eq!(
                server
                    .json(
                        "POST",
                        "/notes/commands/add-note",
                        Some("Writer"),
                        serde_json::json!({"note_id": id, "text": text})
                    )
                    .0,
                202
            );
        }
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"][0]["text"], "new");
        assert_eq!(
            server
                .json(
                    "POST",
                    "/notes/commands/remove-note",
                    Some("Writer"),
                    serde_json::json!({"note_id": 2})
                )
                .0,
            202
        );
        let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        assert_eq!(rows["rows"].as_array().unwrap().len(), 1);
        assert_eq!(rows["rows"][0]["note_id"], 10);
    }
}

struct Server {
    child: std::process::Child,
    address: String,
}
impl Server {
    fn start(binary: &Path, callers: bool, static_root: Option<&Path>) -> Self {
        let mut command = Command::new(binary);
        command.args(["--listen", "127.0.0.1:0"]);
        if callers {
            command.args(["--callers", "actor-header"]);
        }
        if let Some(root) = static_root {
            command.arg("--static").arg(root);
        }
        let mut child = command
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let expected_digest =
            std::fs::read_to_string(binary.parent().unwrap().join(".expected-model-digest"))
                .unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else {
                    break;
                };
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value["event"] == "system.starting"
                        && value["model_digest"].as_str() != Some(&expected_digest)
                    {
                        let _ = sender.send(Err(format!(
                            "wrong generated model: expected {expected_digest}, startup {value}"
                        )));
                        break;
                    }
                    if value["event"] == "system.ready" {
                        let _ = sender
                            .send(Ok(value["runtime"]["address"].as_str().unwrap().to_owned()));
                    }
                }
            }
        });
        let address = match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(address)) => address,
            Ok(Err(error)) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{error}");
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("server did not become ready: {error}");
            }
        };
        Self { child, address }
    }
    fn request(&self, method: &str, path: &str, actor: Option<&str>, body: &str) -> (u16, Vec<u8>) {
        let mut stream = std::net::TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let actor = actor.map_or_else(String::new, |name| {
            format!("Authorization: Actor {name}\r\n")
        });
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{actor}Connection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let boundary = bytes
            .windows(4)
            .position(|bytes| bytes == b"\r\n\r\n")
            .unwrap();
        let head = std::str::from_utf8(&bytes[..boundary]).unwrap();
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, bytes[boundary + 4..].to_vec())
    }
    // Each request consumes a disposable literal at the call site.
    #[allow(clippy::needless_pass_by_value)]
    fn json(
        &self,
        method: &str,
        path: &str,
        actor: Option<&str>,
        body: serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let (status, bytes) = self.request(method, path, actor, &body.to_string());
        (
            status,
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&bytes))),
        )
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn serves(target: Target) {
    let binary = binary(target);
    let server = Server::start(binary, true, None);
    let (status, answer) = server.json(
        "POST",
        "/notes/commands/add-note",
        Some("Writer"),
        serde_json::json!({"note_id": 7, "text": "hello"}),
    );
    assert_eq!(status, 202, "{answer}");
    assert_eq!(answer["outcome"], "added");
    let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
    assert_eq!(rows["rows"][0]["text"], "hello");
    assert_eq!(
        server
            .json(
                "POST",
                "/notes/commands/archive-note",
                Some("Writer"),
                serde_json::json!({"note_id": 7})
            )
            .0,
        202
    );
    let (_, rows) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
    assert_eq!(rows["rows"][0]["state"], "Archived");
    assert_eq!(
        server
            .json(
                "POST",
                "/notes/commands/add-note",
                Some("Reader"),
                serde_json::json!({"note_id": 8, "text": "forbidden"})
            )
            .0,
        403
    );
    let denied = Server::start(binary, false, None);
    assert_eq!(
        denied
            .json(
                "POST",
                "/notes/commands/add-note",
                Some("Writer"),
                serde_json::json!({"note_id": 9, "text": "forbidden"})
            )
            .0,
        403
    );
}

#[test]
fn the_go_entry_point_serves_the_fixture_with_no_hand_written_code() {
    serves(Target::Go);
}

#[test]
fn the_rust_entry_point_serves_the_fixture_with_no_hand_written_code() {
    serves(Target::Rust);
}

#[test]
fn the_static_directory_is_served_beside_the_api() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("served-entry-static-{}", std::process::id()));
    let public = root.join("public");
    std::fs::create_dir_all(&public).unwrap();
    std::fs::write(public.join("index.html"), "served notes").unwrap();
    std::fs::write(public.join("module.wasm"), [0, 97, 115, 109, 255]).unwrap();
    for route in ["notes/views/notes", "notes/commands/add-note"] {
        let path = public.join(route);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "static must not replace an API answer").unwrap();
    }
    std::fs::write(root.join("private.txt"), "outside selected root").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("private.txt"), public.join("escape.txt")).unwrap();
    }
    for target in [Target::Rust, Target::Go] {
        let server = Server::start(binary(target), false, Some(&public));
        assert_eq!(
            server.request("GET", "/", None, ""),
            (200, b"served notes".to_vec())
        );
        assert_eq!(
            server.request("GET", "/module.wasm", None, ""),
            (200, vec![0, 97, 115, 109, 255])
        );
        assert_eq!(
            server
                .json("GET", "/notes/views/notes", None, serde_json::Value::Null)
                .1["rows"],
            serde_json::json!([])
        );
        assert_eq!(
            server.request("POST", "/notes/views/notes", None, "{}").0,
            405
        );
        assert_eq!(
            server
                .request("POST", "/notes/commands/add-note", None, "{}")
                .0,
            403
        );
        for path in [
            "/../private.txt",
            "/%2e%2e/private.txt",
            "/escape.txt",
            "/..%5cprivate.txt",
        ] {
            assert_eq!(
                server.request("GET", path, None, "").0,
                404,
                "{target:?}: {path}"
            );
        }
        let invalid = Command::new(binary(target))
            .args(["--listen", "127.0.0.1:0", "--static"])
            .arg(public.join("index.html"))
            .output()
            .unwrap();
        assert!(!invalid.status.success());
        assert!(
            invalid.stdout.is_empty(),
            "invalid static root announced ready"
        );
    }
}

#[test]
fn the_store_lists_in_identity_order() {
    for target in [Target::Rust, Target::Go] {
        let server = Server::start(binary(target), true, None);
        for id in [10, 2, -1, 2] {
            assert_eq!(
                server
                    .json(
                        "POST",
                        "/notes/commands/add-note",
                        Some("Writer"),
                        serde_json::json!({"note_id": id, "text": "held"})
                    )
                    .0,
                202
            );
        }
        let (_, answer) = server.json("GET", "/notes/views/notes", None, serde_json::Value::Null);
        let ids: Vec<_> = answer["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["note_id"].as_i64().unwrap())
            .collect();
        assert_eq!(ids, [-1, 2, 10]);
    }
}

struct HttpTarget {
    binary: PathBuf,
    server: std::cell::RefCell<Server>,
    routes: std::collections::BTreeMap<String, (String, String)>,
    events: std::cell::RefCell<Vec<ess_conformance::ObservedEvent>>,
    sequence: std::cell::Cell<u64>,
}

impl HttpTarget {
    fn new(binary: &Path, ir: &EssIr) -> Self {
        let routes = ir
            .components()
            .values()
            .flat_map(|component| ess_gen::http::routes(ir, component))
            .map(|route| {
                let name = match route.serves {
                    ess_gen::http::Served::Command(command) => command.name().to_string(),
                    ess_gen::http::Served::View(view) => view.name().to_string(),
                };
                (name, (route.method.as_str().to_owned(), route.path))
            })
            .collect();
        Self {
            binary: binary.to_owned(),
            server: std::cell::RefCell::new(Server::start(binary, true, None)),
            routes,
            events: std::cell::RefCell::default(),
            sequence: std::cell::Cell::new(0),
        }
    }
    fn tick(&self) -> u64 {
        let value = self.sequence.get() + 1;
        self.sequence.set(value);
        value
    }
}

fn json_of(node: &ess_primitives::node::Node) -> serde_json::Value {
    use ess_primitives::node::Node;
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(value) => serde_json::from_str(&value.to_string()).unwrap(),
        Node::Text(value) => serde_json::Value::String(value.clone()),
        Node::Seq(values) => serde_json::Value::Array(values.iter().map(json_of).collect()),
        Node::Map(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), json_of(value)))
                .collect(),
        ),
    }
}

impl ess_conformance::ConformanceTarget for HttpTarget {
    fn identity(
        &self,
    ) -> Result<ess_conformance::ImplementationIdentity, ess_conformance::TargetError> {
        Ok(ess_conformance::ImplementationIdentity::new(
            "generated-served-entry",
            "1",
        ))
    }
    fn begin_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        // Fresh process, not a reset route or an injected store implementation.
        *self.server.borrow_mut() = Server::start(&self.binary, true, None);
        self.events.borrow_mut().clear();
        self.sequence.set(0);
        Ok(())
    }
    fn end_scenario(
        &self,
        _: &ess_conformance::ScenarioContext,
    ) -> Result<(), ess_conformance::TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: ess_conformance::SemanticCommandRequest,
    ) -> Result<ess_conformance::SemanticCommandResult, ess_conformance::TargetError> {
        use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
        let (method, path) = &self.routes[&request.command.to_string()];
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(key, value)| (key.clone(), json_of(value)))
                .collect(),
        );
        let actor = request.actor.as_ref().map(ToString::to_string);
        let (status, answer) = self
            .server
            .borrow()
            .json(method, path, actor.as_deref(), body);
        if status == 403 && answer["refused"] == "not granted" {
            return Err(ess_conformance::TargetError::not_granted(
                answer["actor"].as_str(),
            ));
        }
        if status == 501 {
            return Ok(ess_conformance::SemanticCommandResult::undeclared());
        }
        let failure = || {
            ess_conformance::TargetError::unavailable("command HTTP response", answer.to_string())
        };
        let outcome = answer["outcome"]
            .as_str()
            .ok_or_else(failure)?
            .parse()
            .map_err(|_| failure())?;
        let outcome = OutcomeRef::new(CommandRef::new(request.command.name().clone()), outcome);
        let error = if let Some(error) = answer.get("error") {
            let reference: ErrorRef = error
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(failure)?;
            let mut declared = ess_conformance::DeclaredErrorValue::new(reference);
            for (key, value) in answer
                .get("payload")
                .and_then(serde_json::Value::as_object)
                .into_iter()
                .flatten()
            {
                if !value.is_null() {
                    declared =
                        declared.with(key.clone(), serde_json::from_value(value.clone()).unwrap());
                }
            }
            Some(declared)
        } else {
            None
        };
        let mut direct_events = Vec::new();
        for published in answer["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(failure)?;
            let mut event = ess_conformance::ObservedEvent::new(reference);
            for (key, value) in published["payload"].as_object().into_iter().flatten() {
                event = event.with(key.clone(), serde_json::from_value(value.clone()).unwrap());
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.events.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        Ok(ess_conformance::SemanticCommandResult {
            outcome: Some(outcome),
            error,
            consistency: Some(
                ess_primitives::consistency::ConsistencyToken::new(format!("http:{}", self.tick()))
                    .unwrap(),
            ),
            direct_events,
            response: None,
        })
    }
    fn query_view(
        &self,
        request: ess_conformance::SemanticViewRequest,
    ) -> Result<ess_conformance::SemanticViewResult, ess_conformance::TargetError> {
        let (method, path) = &self.routes[&request.view.to_string()];
        let (status, answer) =
            self.server
                .borrow()
                .json(method, path, None, serde_json::Value::Null);
        let Some(rows) = answer["rows"].as_array().filter(|_| status == 200) else {
            return Err(ess_conformance::TargetError::unavailable(
                "view HTTP response",
                answer.to_string(),
            ));
        };
        Ok(ess_conformance::SemanticViewResult::of(rows.iter().map(
            |row| {
                row.as_object()
                    .unwrap()
                    .iter()
                    .map(|(key, value)| {
                        (key.clone(), serde_json::from_value(value.clone()).unwrap())
                    })
                    .collect::<ess_conformance::target::ViewRow>()
            },
        )))
    }
    fn observe_events(
        &self,
        request: ess_conformance::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::ObservedEvent>, ess_conformance::TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(
        &self,
        _: ess_conformance::ExternalOutcomeControl,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "external control",
            "fixture declares no external outcome",
        ))
    }
    fn redeliver_event(
        &self,
        _: ess_conformance::RedeliveryRequest,
    ) -> Result<(), ess_conformance::TargetError> {
        Err(ess_conformance::TargetError::unsupported(
            "redelivery",
            "fixture declares no binding",
        ))
    }
}

#[test]
fn the_fixture_suite_passes_against_the_generated_go_and_rust_servers() {
    let ir = model(MODEL);
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    for target in [Target::Rust, Target::Go] {
        let adapter = HttpTarget::new(binary(target), &ir);
        let report = ess_conformance::Runner::for_suite(&suite)
            .run_admitted(&admitted, &adapter)
            .into_report();
        assert!(!report.scenarios.is_empty());
        eprintln!(
            "{target:?}: {} HTTP conformance scenarios",
            report.scenarios.len()
        );
        for scenario in &report.scenarios {
            assert_eq!(
                scenario.status,
                ess_conformance::report::Status::Passed,
                "{target:?}: {scenario:#?}"
            );
        }
    }
}

fn clear_generated_packages(root: &Path, target_dir: &Path) {
    // These independently generated workspaces intentionally reuse package names.
    // Invalidate their artifacts so Cargo cannot mistake a different model's fresh
    // binary for this one; keep third-party dependencies in the shared scratch cache.
    let metadata = Command::new(std::env::var_os("CARGO").unwrap())
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    let mut clean = Command::new(std::env::var_os("CARGO").unwrap());
    clean
        .args(["clean", "--offline", "--target-dir"])
        .arg(target_dir)
        .current_dir(root);
    for package in metadata["packages"].as_array().unwrap() {
        clean
            .arg("--package")
            .arg(package["name"].as_str().unwrap());
    }
    let cleaned = clean.output().unwrap();
    assert!(
        cleaned.status.success(),
        "{}",
        String::from_utf8_lossy(&cleaned.stderr)
    );
}
