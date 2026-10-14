//! An external refusal answered before the row (`before_row: true`, ess/25, beyond10x/ess#534),
//! executed against the generated Rust and Go behaviour.
//!
//! The issue's `SignIn` checks a credential before it reads the seat (`bad-credential`), then
//! selects by the seat's held state and creates the seat for an identity no seat carries. Its
//! behaviour is generated, and a harness supplies only the ports the specification leaves to the
//! implementor — where a seat is stored and which external branch the provider takes — recording
//! each port call. The refusal is forced where the conformance suite forces it: on a `Busy` seat,
//! which `reopened-busy` selects, and on an identity no seat carries, which `first` creates. There
//! the generated behaviour must answer `bad-credential` having asked the provider and read no row;
//! a behaviour reading the seat first records a storage read before the provider. Where the
//! provider declines, the held state and existence answer as declared.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/external-before-row.yaml");

/// The four cases each harness runs: the refusal forced on a `Busy` seat and on an unknown
/// identity, and the provider declining on each.
const CASES: [&str; 4] = [
    "busy-refused",
    "unknown-refused",
    "busy-taken",
    "unknown-taken",
];

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(MODEL, "desk"))
}

const RUST_HARNESS: &str = r#"use std::cell::RefCell;
use std::rc::Rc;
use demo_types::behaviour::{Context, ExternalCommand, Generated, SeatStorage};
use demo_types::desk::{self, obligations::SignInBehavior};

struct State {
    held: Option<desk::SeatSnapshot>,
    refuse: bool,
    calls: Vec<&'static str>,
}
#[derive(Clone)]
struct Ports(Rc<RefCell<State>>);
impl SeatStorage for Ports {
    fn get(&self, id: &String) -> Option<desk::SeatSnapshot> {
        let mut s = self.0.borrow_mut();
        s.calls.push("get");
        s.held.clone().filter(|held| held.data.user_id == *id)
    }
    fn put(&mut self, held: desk::SeatSnapshot) {
        let mut s = self.0.borrow_mut();
        s.calls.push("put");
        s.held = Some(held);
    }
    fn delete(&mut self, _: &String) { panic!("sign-in never deletes"); }
    fn list(&self) -> Vec<desk::SeatSnapshot> { self.0.borrow().held.clone().into_iter().collect() }
}
impl Context for Ports {
    fn external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> bool {
        assert_eq!(command.name(), "demo.desk.SignIn");
        assert_eq!(outcome, "bad-credential");
        let mut s = self.0.borrow_mut();
        s.calls.push("external");
        s.refuse
    }
}
fn main() {
    let case = std::env::args().nth(1).expect("a case");
    let busy = desk::SeatSnapshot {
        state: desk::SeatState::Busy,
        data: desk::SeatData { user_id: "u-1".to_owned(), call_ref: "call-1".to_owned() },
    };
    let (user, refuse) = match case.as_str() {
        "busy-refused" => ("u-1", true),
        "unknown-refused" => ("u-unknown", true),
        "busy-taken" => ("u-1", false),
        "unknown-taken" => ("u-unknown", false),
        _ => panic!("unknown case"),
    };
    let ports = Ports(Rc::new(RefCell::new(State { held: Some(busy.clone()), refuse, calls: Vec::new() })));
    let input = desk::SignIn { user_id: user.to_owned(), secret: "s".to_owned() };
    let outcome = match Generated::new(ports.clone()).sign_in(input).expect("a declared outcome") {
        desk::SignInOutcome::BadCredential { .. } => "bad-credential",
        desk::SignInOutcome::ReopenedBusy { .. } => "reopened-busy",
        desk::SignInOutcome::Reopened { .. } => "reopened",
        desk::SignInOutcome::Opened { .. } => "opened",
        desk::SignInOutcome::First { .. } => "first",
    };
    let s = ports.0.borrow();
    println!("{case}\t{outcome}\t{}", s.calls.join(","));
    match case.as_str() {
        "busy-refused" | "unknown-refused" => {
            assert_eq!(outcome, "bad-credential");
            assert_eq!(s.calls, ["external"], "the provider is asked before any row is read");
            assert_eq!(s.held.as_ref(), Some(&busy), "a refusal changes nothing");
        }
        "busy-taken" => {
            assert_eq!(outcome, "reopened-busy");
            assert_eq!(s.calls.first(), Some(&"external"));
            assert_eq!(s.held.as_ref().map(|held| held.state.clone()), Some(desk::SeatState::Open));
        }
        _ => {
            assert_eq!(outcome, "first");
            assert_eq!(s.calls.first(), Some(&"external"));
            assert_eq!(s.held.as_ref().map(|held| held.data.user_id.as_str()), Some("u-unknown"));
        }
    }
}
"#;

const GO_HARNESS: &str = r#"package main

import (
	"fmt"
	"os"
	"reflect"
	"strings"

	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/desk"
)

type ports struct {
	held   *desk.SeatSnapshot
	refuse bool
	calls  []string
}

func (p *ports) Get(id string) (desk.SeatSnapshot, bool) {
	p.calls = append(p.calls, "get")
	if p.held == nil || p.held.Data.UserId != id {
		return desk.SeatSnapshot{}, false
	}
	return *p.held, true
}
func (p *ports) Put(row desk.SeatSnapshot) { p.calls = append(p.calls, "put"); p.held = &row }
func (p *ports) Delete(_ string)           { panic("sign-in never deletes") }
func (p *ports) List() []desk.SeatSnapshot {
	if p.held == nil {
		return nil
	}
	return []desk.SeatSnapshot{*p.held}
}
func (p *ports) External(command behaviour.ExternalCommand, outcome string) bool {
	if command.Name() != "demo.desk.SignIn" || outcome != "bad-credential" {
		panic("unexpected external branch")
	}
	p.calls = append(p.calls, "external")
	return p.refuse
}
func require(ok bool, message string) {
	if !ok {
		panic(message)
	}
}
func main() {
	vector := os.Args[1]
	busy := desk.SeatSnapshot{State: desk.SeatStateBusy{}, Data: desk.SeatData{UserId: "u-1", CallRef: "call-1"}}
	user, refuse := "u-1", true
	switch vector {
	case "busy-refused":
	case "unknown-refused":
		user = "u-unknown"
	case "busy-taken":
		refuse = false
	case "unknown-taken":
		user, refuse = "u-unknown", false
	default:
		panic("unknown case")
	}
	held := busy
	p := &ports{held: &held, refuse: refuse}
	generated := behaviour.New(behaviour.Ports{SeatStorage: p, Context: p})
	result, err := generated.SignIn(desk.SignIn{UserId: user, Secret: "s"})
	require(err == nil, "a declared outcome")
	outcome := ""
	switch result.(type) {
	case desk.SignInOutcomeBadCredential:
		outcome = "bad-credential"
	case desk.SignInOutcomeReopenedBusy:
		outcome = "reopened-busy"
	case desk.SignInOutcomeReopened:
		outcome = "reopened"
	case desk.SignInOutcomeOpened:
		outcome = "opened"
	case desk.SignInOutcomeFirst:
		outcome = "first"
	default:
		panic("unexpected outcome type")
	}
	fmt.Printf("%s\t%s\t%s\n", vector, outcome, strings.Join(p.calls, ","))
	switch vector {
	case "busy-refused", "unknown-refused":
		require(outcome == "bad-credential", "the refusal answers")
		require(reflect.DeepEqual(p.calls, []string{"external"}), "the provider is asked before any row is read")
		require(reflect.DeepEqual(*p.held, busy), "a refusal changes nothing")
	case "busy-taken":
		require(outcome == "reopened-busy", "the held state answers")
		require(p.calls[0] == "external", "the provider is asked first")
		_, open := p.held.State.(desk.SeatStateOpen)
		require(open, "the seat is reopened")
	default:
		require(outcome == "first", "existence answers")
		require(p.calls[0] == "external", "the provider is asked first")
		require(p.held.Data.UserId == "u-unknown", "the seat is created")
	}
}
"#;

/// A fresh directory for one target's tree and harness, kept on failure for review.
fn fresh_root(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "external-before-row-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a fresh root");
    root
}

fn write(synthesis: &ess_synth::Synthesis, directory: &Path) {
    for (relative, artifact) in &synthesis.artifacts {
        let file = directory.join(relative);
        std::fs::create_dir_all(file.parent().expect("a parent")).expect("mkdir");
        std::fs::write(file, &artifact.contents).expect("write artifact");
    }
}

fn printed(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// `synthesis` generates `SignIn`'s behaviour rather than owing it.
fn assert_generated(synthesis: &ess_synth::Synthesis, target: Target) {
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.SignIn"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
}

/// Runs every case of the built harness `binary`, each a separate process.
fn run_cases(binary: &Path, root: &Path) {
    for case in CASES {
        let output = Command::new(binary)
            .arg(case)
            .output()
            .expect("the harness runs");
        assert!(
            output.status.success(),
            "`{case}` failed; tree kept at {}:\n{}",
            root.display(),
            printed(&output)
        );
    }
}

#[test]
fn before_row_external_runs_alike_on_the_generated_rust_target() {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, Target::Rust).expect("the model synthesizes");
    assert_generated(&synthesis, Target::Rust);
    let root = fresh_root("rust");
    write(&synthesis, &root.join("demo"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"before-row-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
         [dependencies]\ndemo-types = { path = \"../demo/crates/demo-types\" }\n[workspace]\n",
    )
    .expect("manifest");
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).expect("harness");
    let target = root.join("target");
    let built = Command::new(std::env::var_os("CARGO").expect("cargo executable"))
        .args(["build", "--offline", "--quiet", "--target-dir"])
        .arg(&target)
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the harness builds against the generated ports; tree kept at {}:\n{}",
        root.display(),
        printed(&built)
    );
    run_cases(&target.join("debug/before-row-harness"), &root);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn before_row_external_runs_alike_on_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, Target::Go).expect("the model synthesizes");
    assert_generated(&synthesis, Target::Go);
    let root = fresh_root("go");
    write(&synthesis, &root.join("demo"));
    let harness = root.join("harness");
    std::fs::create_dir_all(&harness).expect("mkdir");
    std::fs::write(
        harness.join("go.mod"),
        "module beforerowharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../demo\n",
    )
    .expect("Go manifest");
    std::fs::write(harness.join("main.go"), GO_HARNESS).expect("Go harness");
    let binary = root.join("go-harness");
    let built = Command::new("go")
        .args(["build", "-o"])
        .arg(&binary)
        .arg(".")
        .current_dir(&harness)
        .env("GOFLAGS", "-mod=mod -buildvcs=false")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .output()
        .expect("the Go toolchain runs");
    assert!(
        built.status.success(),
        "the Go harness builds against the generated ports; tree kept at {}:\n{}",
        root.display(),
        printed(&built)
    );
    run_cases(&binary, &root);
    let _ = std::fs::remove_dir_all(&root);
}

/// Without the key the external branch answers at step 6, beside the held-state branches, and no
/// target generates that command: it stays an obligation, so the key is what generates it.
#[test]
fn without_the_key_the_command_stays_an_obligation() {
    let ir = model().with_commands_rewritten(|command| {
        let mut command = command.clone();
        for outcome in &mut command.outcomes {
            outcome.before_row = false;
        }
        command
    });
    for target in [Target::Rust, Target::Go] {
        let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
        assert!(
            matches!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.SignIn"),
                Some(SynthesisDisposition::Obligation(_))
            ),
            "{target:?}:\n{}",
            synthesis.plan.to_markdown()
        );
    }
}

/// The WebAssembly and command-line targets wrap the Rust system: each generates `SignIn` as the
/// Rust target does, rather than owing it or refusing the model.
#[test]
fn before_row_external_generates_on_the_web_and_clap_targets() {
    let ir = model();
    for target in [Target::Web, Target::Clap] {
        let synthesis = ess_synth::synthesize_for(&ir, target)
            .unwrap_or_else(|failure| panic!("{target:?}: {}", failure.to_canonical_json()));
        assert_generated(&synthesis, target);
    }
}
