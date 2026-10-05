//! beyond10x/ess#260: a served `501` says, in a member a client reads rather than a prefix it
//! matches, whether the command's effect was committed.
//!
//! The served `501` covers two cases. An unmet obligation stopped the command and nothing was
//! written; or the command's effect and events were committed and delivering what it published to
//! a binding failed. Both answer `{"refused": …, "committed": …}`: `false` for the first, `true`
//! for the second, from the generated Rust server and the generated Go server alike. The Rust
//! transport-free entry point says the same thing as a value: [`Refused::committed`], and a
//! distinct `Refused::Undelivered` for the committed case.
//!
//! The model is `adversary_served_pass2`'s: `note-on-open` escalates through an owed escalation the
//! realization answers unmet every time, while the notebook it invokes stays full, so an `Opened`
//! is undeliverable to it — the committed case. `Ping`'s behaviour is realized by hand here and
//! answers an unmet obligation — the uncommitted case.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};
use serde_json::{json, Value};

const MODEL: &str = "format: ess/15
system: ops
version: v1
domain: ops.core
events:
  - name: ops.core.Opened
    fields:
      - {name: id, type: String}
  - name: ops.core.Stamped
    fields:
      - {name: id, type: String}
  - name: ops.core.Noted
    fields:
      - {name: id, type: String}
  - name: ops.core.NoteDropped
    fields:
      - {name: id, type: String}
  - name: ops.core.Tallied
    fields:
      - {name: id, type: String}
  - name: ops.core.Pinged
    fields: []
commands:
  - name: ops.core.Open
    input:
      - {name: id, type: String}
      - {name: count, type: Integer}
    outcomes:
      - name: opened
        when: count > 0
        emits: [ops.core.Opened, ops.core.Stamped]
        payload:
          ops.core.Opened: {id: input.id}
          ops.core.Stamped: {id: input.id}
      - name: refused
        error: ops.core.Bad
  - name: ops.core.Note
    input:
      - {name: id, type: String}
    outcomes:
      - name: noted
        emits: [ops.core.Noted]
        payload:
          ops.core.Noted: {id: input.id}
      - name: busy
        external: the notebook is full
        error: ops.core.Busy
  - name: ops.core.Tally
    input:
      - {name: id, type: String}
    outcomes:
      - name: tallied
        emits: [ops.core.Tallied]
        payload:
          ops.core.Tallied: {id: input.id}
      - name: jammed
        external: the counter is jammed
        error: ops.core.Jammed
  - name: ops.core.Ping
    outcomes:
      - name: pinged
        emits: [ops.core.Pinged]
errors:
  - name: ops.core.Bad
  - name: ops.core.Busy
  - name: ops.core.Jammed
components:
  - component: ops-service
    owns: {domains: [ops.core]}
    accepts: {commands: [ops.core.Open, ops.core.Note, ops.core.Tally, ops.core.Ping]}
    publishes: {events: [ops.core.Opened, ops.core.Stamped, ops.core.Noted, ops.core.NoteDropped, ops.core.Tallied, ops.core.Pinged]}
    reached_by: network
bindings:
  - id: note-on-open
    when:
      event: ops.core.Opened
    invoke:
      command: ops.core.Note
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure:
      escalate:
        emits: ops.core.NoteDropped
  - id: tally-on-open
    when:
      event: ops.core.Opened
    invoke:
      command: ops.core.Tally
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure: drop
";

/// The Rust realization both harness binaries link: the generated behaviours, except `Ping`, whose
/// behaviour answers an unmet obligation.
const REALIZATION: &str = r#"use ops_types::behaviour::{Context, ExternalCommand, Generated};
use ops_types::core;
use ops_types::core::obligations::{NoteBehavior, OpenBehavior, PingBehavior, TallyBehavior};
use ops_types::obligation::UnmetObligation;

#[derive(Clone, Default)]
pub struct Desk;

impl Context for Desk {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        let command = command.name();
        // The notebook stays full.
        matches!((command, outcome), ("ops.core.Note", "busy"))
    }
}

// The owed escalation is not realized yet: it answers unmet, every time.
impl ops_system::obligations::NoteOnOpenEscalation for Desk {
    fn note_on_open_escalation(&self, _failed: &core::Note) -> Result<core::NoteDropped, UnmetObligation> {
        Err(UnmetObligation { capability: "binding-escalation", source: "note-on-open" })
    }
}

/// The generated behaviours, with `Ping` left unmet.
pub struct Partly(pub Generated<Desk>);

impl OpenBehavior for Partly {
    fn open(&mut self, input: core::Open) -> Result<core::OpenOutcome, UnmetObligation> {
        self.0.open(input)
    }
}

impl NoteBehavior for Partly {
    fn note(&mut self, input: core::Note) -> Result<core::NoteOutcome, UnmetObligation> {
        self.0.note(input)
    }
}

impl TallyBehavior for Partly {
    fn tally(&mut self, input: core::Tally) -> Result<core::TallyOutcome, UnmetObligation> {
        self.0.tally(input)
    }
}

impl PingBehavior for Partly {
    fn ping(&mut self, _input: core::Ping) -> Result<core::PingOutcome, UnmetObligation> {
        Err(UnmetObligation { capability: "command_behavior", source: "ops.core.Ping" })
    }
}

pub type System = ops_system::System<Partly, Desk>;

pub fn system() -> System {
    ops_system::System::new(ops_service::OpsService::new(Partly(Generated::new(Desk))), Desk)
}
"#;

/// The served half: one line per request, `label<TAB>status<TAB>body`.
const SERVED: &str = r##"use ops_server::http;
use ops_server::ops_service as surface;

#[path = "../realization.rs"]
mod realization;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();
    let mut system = realization::system();
    for (label, name, body) in [
        ("unmet", "ops.core.Ping", "{}"),
        ("committed", "ops.core.Open", r#"{"id":"e","count":1}"#),
    ] {
        let (_, method, path) = routes
            .iter()
            .find(|(declared, _, _)| declared == name)
            .unwrap_or_else(|| panic!("`{name}` is a route"));
        let request = http::Request {
            method: method.clone(),
            path: path.clone(),
            query: String::new(),
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        };
        let answered = surface::dispatch(&mut system, &request);
        println!("{label}\t{}\t{}", answered.status, answered.body.replace('\t', " "));
    }
}
"##;

/// The typed half: `handle`'s refusal, one line per case, `label<TAB>committed<TAB>variant`.
const TYPED: &str = r##"use ops_server::entry::Refused;
use ops_server::json;
use ops_server::ops_service as surface;

#[path = "../realization.rs"]
mod realization;

fn main() {
    let mut system = realization::system();
    for (label, name, input) in [
        ("unmet", "ops.core.Ping", "{}"),
        ("committed", "ops.core.Open", r#"{"id":"e","count":1}"#),
    ] {
        let refused = surface::handle(&mut system, name, json::parse(input).expect("JSON"))
            .expect_err("both cases are refused");
        let variant = match &refused {
            Refused::Unknown(_) => "unknown",
            Refused::Input(_) => "input",
            Refused::Unmet(_) => "unmet",
            Refused::Undelivered(_) => "undelivered",
        };
        println!("{label}\t{}\t{variant}\t{}", refused.committed(), refused.status());
    }
}
"##;

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn route_table(ir: &EssIr) -> Vec<String> {
    let component = ir.components().values().next().expect("one component");
    let mut table = Vec::new();
    for route in http::routes(ir, component) {
        let name = match route.serves {
            Served::Command(handle) => ir.command(handle).name.to_string(),
            Served::View(handle) => ir.view(handle).name.to_string(),
        };
        table.extend([name, route.method.as_str().to_owned(), route.path]);
    }
    table
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "served-unfinished-committed-{label}-{}",
        std::process::id()
    ))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn cargo(directory: &Path, arguments: &[String]) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Both Rust harness binaries' output, `(served, typed)`, once per test binary.
fn rust() -> &'static (String, String) {
    static OUTPUT: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    OUTPUT.get_or_init(|| {
        let ir = ir();
        let synthesis = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace)
            .expect("the model synthesizes");
        let root = scratch("rust");
        write(&synthesis, &root.join("ops"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src/bin")).expect("mkdir");
        let dependencies = ["ops-server", "ops-system", "ops-service", "ops-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../ops/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"served-unfinished-committed\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\nautobins = false\n\n[[bin]]\nname = \"served\"\n\
                 path = \"src/bin/served.rs\"\n\n[[bin]]\nname = \"typed\"\n\
                 path = \"src/bin/typed.rs\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/realization.rs"), REALIZATION).expect("write");
        std::fs::write(harness.join("src/bin/served.rs"), SERVED).expect("write");
        std::fs::write(harness.join("src/bin/typed.rs"), TYPED).expect("write");
        let target = root.join("target");
        let run = |bin: &str, extra: &[String]| {
            let mut arguments: Vec<String> = [
                "run",
                "--offline",
                "--quiet",
                "--bin",
                bin,
                "--target-dir",
                target.to_str().expect("UTF-8"),
                "--",
            ]
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
            arguments.extend_from_slice(extra);
            let ran = cargo(&harness, &arguments);
            (
                ran.status.success(),
                String::from_utf8_lossy(&ran.stdout).into_owned(),
            )
        };
        let served = run("served", &route_table(&ir));
        let typed = run("typed", &[]);
        let _ = std::fs::remove_dir_all(&root);
        (
            if served.0 {
                served.1
            } else {
                String::from("served harness failed to build or run")
            },
            if typed.0 {
                typed.1
            } else {
                String::from("typed harness failed to build or run")
            },
        )
    })
}

/// The Go harness: an in-package test of the generated `server` package, as
/// `adversary_served_pass2_go` drives it. Every line is `step<TAB>label<TAB>status<TAB>body`.
const GO_HARNESS: &str = r#"package server

import (
	"fmt"
	"net/http/httptest"
	"strings"
	"testing"

	"example.invalid/ops/components/opsservice"
	"example.invalid/ops/system"
	"example.invalid/ops/types/core"
	"example.invalid/ops/types/obligation"
)

type desk struct{}

func (d *desk) Open(input core.Open) (core.OpenOutcome, *obligation.UnmetObligation) {
	if input.Count > 0 {
		return core.OpenOutcomeOpened{Opened: core.Opened{Id: input.Id}, Stamped: core.Stamped{Id: input.Id}}, nil
	}
	return core.OpenOutcomeRefused{}, nil
}

// The notebook stays full.
func (d *desk) Note(input core.Note) (core.NoteOutcome, *obligation.UnmetObligation) {
	return core.NoteOutcomeBusy{}, nil
}

func (d *desk) Tally(input core.Tally) (core.TallyOutcome, *obligation.UnmetObligation) {
	return core.TallyOutcomeTallied{Tallied: core.Tallied{Id: input.Id}}, nil
}

// Ping's behaviour is not realized.
func (d *desk) Ping(input core.Ping) (core.PingOutcome, *obligation.UnmetObligation) {
	return nil, &obligation.UnmetObligation{Capability: "command_behavior", Source: "ops.core.Ping"}
}

func TestServedUnfinished(t *testing.T) {
	s := system.NewSystem(opsservice.New(&desk{}), system.Unimplemented{})
	step := func(label, path, body string) {
		request := httptest.NewRequest("POST", path, strings.NewReader(body))
		answer := dispatchOpsService(s, request)
		fmt.Printf("step\t%s\t%d\t%s\n", label, answer.status, strings.ReplaceAll(answer.body, "\n", " "))
	}
	step("unmet", "/core/commands/Ping", "{}")
	step("committed", "/core/commands/Open", `{"id":"e","count":1}`)
}
"#;

fn go(directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new("go")
        .args(arguments)
        .current_dir(directory)
        .env("GOCACHE", scratch("gocache"))
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap_or_else(|error| panic!("`go` runs: {error}"));
    eprintln!(
        "go {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn go_output() -> &'static String {
    static OUTPUT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    OUTPUT.get_or_init(|| {
        let synthesis = synthesize_laid_out(&ir(), Target::Go, OutputLayout::Workspace)
            .expect("the model synthesizes for Go");
        let directory = scratch("go");
        write(&synthesis, &directory);
        std::fs::write(
            directory.join("server/served_unfinished_test.go"),
            GO_HARNESS,
        )
        .expect("write");
        let ran = go(
            &directory,
            &[
                "test",
                "-count=1",
                "-timeout",
                "60s",
                "-v",
                "-run",
                "TestServedUnfinished",
                "./server/",
            ],
        );
        let _ = std::fs::remove_dir_all(&directory);
        let _ = std::fs::remove_dir_all(scratch("gocache"));
        assert!(ran.status.success(), "the Go harness builds and runs");
        String::from_utf8_lossy(&ran.stdout)
            .lines()
            .filter_map(|line| line.strip_prefix("step\t"))
            .fold(String::new(), |mut out, line| {
                let _ = writeln!(out, "{line}");
                out
            })
    })
}

/// The `(status, body)` a harness printed for `label`.
fn answer(output: &str, label: &str) -> (u16, Value) {
    let line = output
        .lines()
        .find(|line| line.split('\t').next() == Some(label))
        .unwrap_or_else(|| panic!("the harness printed `{label}`:\n{output}"));
    let fields: Vec<&str> = line.splitn(3, '\t').collect();
    (
        fields[1].parse().expect("a status"),
        serde_json::from_str(fields[2]).expect("the body is JSON"),
    )
}

/// The shape both servers answer each case with: `501`, and exactly `refused` and `committed`.
fn assert_unfinished(language: &str, label: &str, status: u16, body: &Value, committed: bool) {
    assert_eq!(status, 501, "{language} `{label}`: {body}");
    let mut members: Vec<&str> = body
        .as_object()
        .unwrap_or_else(|| panic!("{language} `{label}` answers an object: {body}"))
        .keys()
        .map(String::as_str)
        .collect();
    members.sort_unstable();
    assert_eq!(
        members,
        ["committed", "refused"],
        "{language} `{label}` answers exactly `refused` and `committed`: {body}"
    );
    assert_eq!(
        body["committed"],
        json!(committed),
        "{language} `{label}`: {body}"
    );
    assert!(
        body["refused"]
            .as_str()
            .is_some_and(|text| !text.is_empty()),
        "{language} `{label}` still says in words what was left unfinished: {body}"
    );
}

/// An unmet obligation stops the command before anything is written: `committed: false`.
#[test]
fn the_rust_server_answers_an_unmet_obligation_as_not_committed() {
    let (status, body) = answer(&rust().0, "unmet");
    assert_unfinished("Rust", "unmet", status, &body, false);
    assert!(
        body["refused"]
            .as_str()
            .is_some_and(|text| text.starts_with("unmet obligation: command_behavior")),
        "{body}"
    );
}

/// A failed delivery after the effect was committed: `committed: true`, and the words as before.
#[test]
fn the_rust_server_answers_a_failed_delivery_as_committed() {
    let (status, body) = answer(&rust().0, "committed");
    assert_unfinished("Rust", "committed", status, &body, true);
    assert!(
        body["refused"]
            .as_str()
            .is_some_and(|text| text.starts_with("delivering what the command published: ")),
        "{body}"
    );
}

/// The transport-free entry point says the same as a value: the committed case is its own variant
/// and [`Refused::committed`] reads it, and both still answer `501`.
#[test]
fn the_rust_entry_point_refuses_the_committed_case_as_undelivered() {
    let typed = &rust().1;
    assert_eq!(
        typed.as_str(),
        "unmet\tfalse\tunmet\t501\ncommitted\ttrue\tundelivered\t501\n",
        "{typed}"
    );
}

#[test]
fn the_go_server_answers_an_unmet_obligation_as_not_committed() {
    let (status, body) = answer(go_output(), "unmet");
    assert_unfinished("Go", "unmet", status, &body, false);
    assert!(
        body["refused"]
            .as_str()
            .is_some_and(|text| text.starts_with("unmet obligation: command_behavior")),
        "{body}"
    );
}

#[test]
fn the_go_server_answers_a_failed_delivery_as_committed() {
    let (status, body) = answer(go_output(), "committed");
    assert_unfinished("Go", "committed", status, &body, true);
    assert!(
        body["refused"]
            .as_str()
            .is_some_and(|text| text.starts_with("delivering what the command published: ")),
        "{body}"
    );
}
