//! A union mixing a unit variant with a payload variant, through every generated lane
//! (ess/22, beyond10x/ess#418, `docs/design/union-unit-variants.md`).
//!
//! `demo.work.Status` is `Open | Complete { outcome }`. On the wire a payload variant is
//! `{"kind": "Complete", "value": {...}}` and the unit variant is the tag alone, `{"kind": "Open"}`.
//! Each test synthesizes fresh code, compiles it with the target's own toolchain and runs it:
//!
//! - the Rust target declares `Open` as a unit enum variant, and its server codecs and served
//!   command round-trip both variants byte for byte and refuse a unit variant carrying a payload
//!   member and a payload variant carrying none;
//! - the Go target does the same through its codecs and served command;
//! - the suite the model synthesizes, plus one authored scenario sending `Open`, passes against
//!   the interpreter, the generated Rust server and the generated Go server, and fails against a
//!   target that writes `Open` with a payload member or confuses `Open` with `Complete`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::report::Status;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, SuiteProvenance};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use ess_synth::{synthesize_for, Synthesis, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

const OPEN: &str = r#"{"kind":"Open"}"#;
const COMPLETE: &str = r#"{"kind":"Complete","value":{"outcome":"shipped"}}"#;
/// A unit variant written with a payload member.
const OPEN_WITH_PAYLOAD: &str = r#"{"kind":"Open","value":{"outcome":"shipped"}}"#;
/// A payload variant written without one.
const COMPLETE_WITHOUT_PAYLOAD: &str = r#"{"kind":"Complete"}"#;

/// The authored scenario that sends the unit variant: synthesis witnesses the first variant only.
const OPEN_SCENARIO: &str = "type: ess-scenario/1
domain: demo.work
scenario: open-is-reported
summary: An open status is reported as the unit variant it is.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.work.ReportStatus
    actor: demo.work.Reporter
    input:
      status: {kind: Open}
    outcome: reported
    events:
      - event: demo.work.StatusReported
        payload: {status: {kind: Open}}
";

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("work.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("work.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "union-unit-variants-{label}-{}",
        std::process::id()
    ))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

fn run(mut command: Command, label: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{label} runs: {error}"));
    eprintln!(
        "{label}: {command:?}\n{}{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        output.status
    );
    output
}

fn cargo(directory: &Path, target: &Path, arguments: &[&str]) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(arguments)
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER");
    run(command, "cargo")
}

fn go(directory: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new("go");
    command
        .args(arguments)
        .current_dir(directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOWORK", "off")
        .env("GOTOOLCHAIN", "local");
    if std::env::var_os("GOCACHE").is_none() {
        command.env("GOCACHE", scratch("gocache"));
    }
    run(command, "go")
}

// ---- the Rust target -----------------------------------------------------------------------------

#[test]
fn the_rust_target_declares_open_as_a_unit_variant() {
    let synthesis = synthesize_for(&ir(), Target::Rust).expect("the model synthesizes");
    let declared = synthesis
        .artifacts
        .values()
        .find(|artifact| artifact.contents.contains("pub enum Status {"))
        .unwrap_or_else(|| panic!("a `Status` enum in {:?}", synthesis.artifacts.keys()));
    let body = declared
        .contents
        .split("pub enum Status {")
        .nth(1)
        .and_then(|rest| rest.split("\n}").next())
        .unwrap();
    assert!(body.contains("\n    Open,"), "a unit variant:{body}");
    assert!(
        body.contains("\n    Complete(Completion),"),
        "a payload variant:{body}"
    );
}

/// The native harness over the generated server's codecs and its served command.
const RUST_HARNESS: &str = r##"
#![cfg(test)]
use demo_server::{json, wire, work_service as surface};
use demo_types::work::{Completion, ReportStatus, Status};

const OPEN: &str = r#"@OPEN@"#;
const COMPLETE: &str = r#"@COMPLETE@"#;
const OPEN_WITH_PAYLOAD: &str = r#"@OPEN_WITH_PAYLOAD@"#;
const COMPLETE_WITHOUT_PAYLOAD: &str = r#"@COMPLETE_WITHOUT_PAYLOAD@"#;

fn complete() -> Status {
    Status::Complete(Completion { outcome: "shipped".into() })
}

fn caller() -> demo_types::actor::Caller {
    demo_types::actor::Caller { actor: demo_types::actor::Actor::ALL[0] }
}

fn system() -> demo_system::System<demo_types::behaviour::Generated<()>> {
    demo_system::System::new(work_service::WorkService::new(demo_types::behaviour::Generated::new(())))
}

#[test]
fn both_variants_round_trip_through_the_named_codec() {
    for (value, text) in [(Status::Open, OPEN), (complete(), COMPLETE)] {
        let mut encoded = String::new();
        wire::encode_demo_work_status(&value, &mut encoded);
        assert_eq!(encoded, text);
        assert_eq!(wire::decode_demo_work_status(&json::parse(text).unwrap(), "status").unwrap(), value);
    }
}

#[test]
fn the_two_malformed_shapes_are_refused() {
    let refused = wire::decode_demo_work_status(&json::parse(OPEN_WITH_PAYLOAD).unwrap(), "status")
        .expect_err("a unit variant carries no payload member");
    assert_eq!(refused.at, "status.value", "{refused:?}");
    let refused = wire::decode_demo_work_status(&json::parse(COMPLETE_WITHOUT_PAYLOAD).unwrap(), "status")
        .expect_err("a payload variant carries its payload");
    assert_eq!(refused.at, "status.value", "{refused:?}");
    let refused = wire::decode_demo_work_status(&json::parse(r#"{"kind":"Open","value":null}"#).unwrap(), "status")
        .expect_err("a null payload member is still a member");
    assert_eq!(refused.at, "status.value", "{refused:?}");
}

#[test]
fn the_served_command_reports_both_variants() {
    let mut system = system();
    for text in [OPEN, COMPLETE] {
        let input = format!(r#"{{"status":{text}}}"#);
        let decoded = wire::decode_command_demo_work_report_status(&json::parse(&input).unwrap(), "").unwrap();
        let mut encoded = String::new();
        wire::encode_command_demo_work_report_status(&decoded, &mut encoded);
        assert_eq!(encoded, input);
        let handled = surface::handle(&mut system, Some(&caller()), "demo.work.ReportStatus", json::parse(&input).unwrap()).unwrap();
        let outcome = format!(r#"{{"outcome":"reported","published":[{{"event":"demo.work.StatusReported","payload":{input}}}]}}"#);
        assert_eq!(handled, json::parse(&outcome).unwrap());
    }
    let _: ReportStatus = ReportStatus { status: Status::Open };
    for text in [OPEN_WITH_PAYLOAD, COMPLETE_WITHOUT_PAYLOAD] {
        let input = format!(r#"{{"status":{text}}}"#);
        assert!(
            surface::handle(&mut system, Some(&caller()), "demo.work.ReportStatus", json::parse(&input).unwrap()).is_err(),
            "{input} is refused as input"
        );
    }
}
"##;

fn rust_harness() -> String {
    RUST_HARNESS
        .replace("@OPEN@", OPEN)
        .replace("@COMPLETE@", COMPLETE)
        .replace("@OPEN_WITH_PAYLOAD@", OPEN_WITH_PAYLOAD)
        .replace("@COMPLETE_WITHOUT_PAYLOAD@", COMPLETE_WITHOUT_PAYLOAD)
}

const RUST_DEPENDENCIES: &str = "demo-types = {path = '../generated/crates/demo-types'}
demo-server = {path = '../generated/crates/demo-server'}
demo-system = {path = '../generated/crates/demo-system'}
work-service = {path = '../generated/crates/work-service'}
";

#[test]
fn rust_server_codecs_round_trip_both_variants_and_refuse_the_malformed_shapes() {
    let synthesis = synthesize_for(&ir(), Target::Rust).expect("the model synthesizes");
    let root = scratch("rust-codecs");
    write(&synthesis, &root.join("generated"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[workspace]\n[package]\nname = 'codec-proof'\nversion = '0.0.0'\nedition = \
             '2021'\n[dependencies]\n{RUST_DEPENDENCIES}"
        ),
    )
    .unwrap();
    std::fs::write(harness.join("src/lib.rs"), rust_harness()).unwrap();
    let target = root.join("target");
    let tested = cargo(&harness, &target, &["test", "--offline"]);
    assert!(tested.status.success(), "the Rust codec harness failed");
    assert!(
        String::from_utf8_lossy(&tested.stdout).contains("test result: ok. 3 passed"),
        "all three Rust codec cases ran"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- the Web target ------------------------------------------------------------------------------

#[test]
fn the_web_page_sends_a_unit_variant_as_its_tag_alone() {
    let synthesis = synthesize_for(&ir(), Target::Web).expect("the model synthesizes to Web");
    let page = synthesis
        .artifacts
        .values()
        .find(|artifact| artifact.path.ends_with("index.html"))
        .unwrap_or_else(|| panic!("a page in {:?}", synthesis.artifacts.keys()));
    assert!(
        page.contents
            .contains("held = carried === null\n      ? { node: element(\"span\", { class: \"note\", text: \"carries nothing\" }), read: () => undefined }"),
        "the page reads a unit variant as absent, so the value sent is the tag alone"
    );
    let catalogue: serde_json::Value =
        serde_json::from_str(&synthesis.artifacts["catalog.json"].contents).expect("JSON");
    assert_eq!(
        catalogue["types"]["demo.work.Status"]["variants"]["Open"],
        serde_json::json!({"spelling": null, "type": null}),
        "the catalogue gives a unit variant no type: {catalogue:#}"
    );
}

// ---- a binding's accessor reading through a unit variant -----------------------------------------

/// The bounded-accessor fixture at ess/22, its `gone` variant carrying nothing: the binding's
/// `choice: event.choice.status` reads through the union.
fn accessor_ir() -> EssIr {
    let text = include_str!("fixtures/bounded-accessor.yaml")
        .replace("format: ess/3", "format: ess/22")
        .replace("Optional<Optional<String>>", "Optional<String>")
        .replace("      gone: String\n", "      gone:\n")
        .replace(
            "        emits: [projection.core.Arrived]\n",
            "        emits: [projection.core.Arrived]\n        payload:\n          \
             projection.core.Arrived:\n            data: {generated: true}\n            \
             partial: {generated: true}\n            choice: {generated: true}\n            \
             wrapped: {generated: true}\n",
        )
        .replace(
            "        emits: [projection.core.Done]\n",
            "        emits: [projection.core.Done]\n        payload:\n          \
             projection.core.Done:\n            result: {generated: true}\n",
        );
    let spec = Specification::assemble([(
        Source::new("accessor.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

const RUST_ACCESSOR: &str = r#"
#![cfg(test)]
use projection_types::core::{Arrived, Body, Choice, Wrapped, Wrapper};

#[test]
fn a_unit_variant_projects_as_unavailable() {
    let body = Body { status: "ready".to_owned(), nested: None };
    let mut event = Arrived {
        data: body.clone(),
        partial: None,
        choice: Choice::Gone,
        wrapped: Wrapped(Wrapper { body: body.clone() }),
    };
    assert_eq!(projection_system::project(&event).choice, None);
    event.choice = Choice::Ready(body);
    assert_eq!(projection_system::project(&event).choice.as_deref(), Some("ready"));
}
"#;

const GO_ACCESSOR: &str = r#"package system

import (
	"testing"

	"example.invalid/projection/types/core"
)

func TestUnitVariantProjection(t *testing.T) {
	body := core.Body{Status: "ready"}
	event := core.Arrived{Data: body, Choice: core.ChoiceGone{}, Wrapped: core.NewWrapped(core.Wrapper{Body: body})}
	if input := Project(event); input.Choice != nil {
		t.Fatalf("a unit variant projects as unavailable: %#v", input)
	}
	event.Choice = core.ChoiceReady{Value: body}
	if input := Project(event); input.Choice == nil || *input.Choice != "ready" {
		t.Fatalf("the payload variant projects its member: %#v", input)
	}
	t.Log("go accessor: unit variant unavailable")
}
"#;

#[test]
fn generated_rust_and_go_project_through_a_unit_variant_as_unavailable() {
    let ir = accessor_ir();
    let root = scratch("accessor");
    let synthesis = synthesize_for(&ir, Target::Rust).expect("the model synthesizes");
    write(&synthesis, &root.join("generated"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[workspace]\n[package]\nname = 'accessor-proof'\nversion = '0.0.0'\nedition = \
         '2021'\n[dependencies]\nprojection-types = {path = \
         '../generated/crates/projection-types'}\nprojection-system = {path = \
         '../generated/crates/projection-system'}\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/lib.rs"), RUST_ACCESSOR).unwrap();
    let tested = cargo(&harness, &root.join("target"), &["test", "--offline"]);
    assert!(tested.status.success(), "the Rust accessor harness failed");
    assert!(String::from_utf8_lossy(&tested.stdout).contains("test result: ok. 1 passed"));

    let synthesis = synthesize_for(&ir, Target::Go).expect("Go represents the model");
    let directory = root.join("go");
    write(&synthesis, &directory);
    std::fs::write(
        directory.join("system/zz_unit_accessor_test.go"),
        GO_ACCESSOR,
    )
    .unwrap();
    assert!(go(&directory, &["vet", "./..."]).status.success(), "go vet");
    let tested = go(
        &directory,
        &[
            "test",
            "-count=1",
            "-run",
            "TestUnitVariantProjection",
            "-v",
            "./system/",
        ],
    );
    assert!(tested.status.success(), "the Go accessor harness failed");
    assert!(
        String::from_utf8_lossy(&tested.stdout).contains("go accessor: unit variant unavailable")
    );
    let _ = std::fs::remove_dir_all(&root);
}

// ---- the Go target -------------------------------------------------------------------------------

/// The Go harness, inside the generated server package so it reaches the unexported codecs.
const GO_HARNESS: &str = r#"package server

import (
	"encoding/json"
	"testing"

	"example.invalid/demo/components/workservice"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/work"
)

func parsed(t *testing.T, text string) any {
	t.Helper()
	var value any
	if err := json.Unmarshal([]byte(text), &value); err != nil {
		t.Fatalf("fixture JSON: %v", err)
	}
	return value
}

func TestUnitVariantCodecs(t *testing.T) {
	cases := []struct {
		value work.Status
		text  string
	}{
		{work.StatusOpen{}, `@OPEN@`},
		{work.StatusComplete{Value: work.Completion{Outcome: "shipped"}}, `@COMPLETE@`},
	}
	for _, each := range cases {
		encoded, err := json.Marshal(encodeDemoWorkStatus(each.value))
		if err != nil || string(encoded) != each.text {
			t.Fatalf("%T encodes as %s, %v; want %s", each.value, encoded, err, each.text)
		}
		back, err := decodeDemoWorkStatus(parsed(t, each.text), "status")
		if err != nil || back != each.value {
			t.Fatalf("%s decodes as %#v, %v", each.text, back, err)
		}
	}
	for _, malformed := range []string{`@OPEN_WITH_PAYLOAD@`, `@COMPLETE_WITHOUT_PAYLOAD@`, `{"kind":"Open","value":null}`} {
		if back, err := decodeDemoWorkStatus(parsed(t, malformed), "status"); err == nil {
			t.Fatalf("%s decodes as %#v; it must be refused", malformed, back)
		}
	}
	running := system.NewSystem(workservice.New(behaviour.New(NewMemoryPorts())))
	for _, each := range cases {
		input := `{"status":` + each.text + `}`
		answer := serveDemoWorkReportStatus(running, []byte(input))
		want := `{"outcome":"reported","published":[{"event":"demo.work.StatusReported","payload":` + input + `}]}`
		if answer.status != 202 || answer.body != want {
			t.Fatalf("served %d %s\nwant %s", answer.status, answer.body, want)
		}
	}
	for _, malformed := range []string{`@OPEN_WITH_PAYLOAD@`, `@COMPLETE_WITHOUT_PAYLOAD@`} {
		answer := serveDemoWorkReportStatus(running, []byte(`{"status":`+malformed+`}`))
		if answer.status != 400 {
			t.Fatalf("%s is answered %d %s; it must be refused as input", malformed, answer.status, answer.body)
		}
	}
	t.Log("go codecs: unit and payload variants round-trip, malformed shapes refused")
}
"#;

#[test]
fn go_server_codecs_round_trip_both_variants_and_refuse_the_malformed_shapes() {
    let synthesis = synthesize_for(&ir(), Target::Go).expect("Go represents the model");
    let directory = scratch("go-codecs");
    write(&synthesis, &directory);
    std::fs::write(
        directory.join("server/zz_unit_variant_test.go"),
        GO_HARNESS
            .replace("@OPEN@", OPEN)
            .replace("@COMPLETE@", COMPLETE)
            .replace("@OPEN_WITH_PAYLOAD@", OPEN_WITH_PAYLOAD)
            .replace("@COMPLETE_WITHOUT_PAYLOAD@", COMPLETE_WITHOUT_PAYLOAD),
    )
    .unwrap();
    assert!(go(&directory, &["vet", "./..."]).status.success(), "go vet");
    let tested = go(
        &directory,
        &[
            "test",
            "-count=1",
            "-run",
            "TestUnitVariantCodecs",
            "-v",
            "./server/",
        ],
    );
    assert!(tested.status.success(), "the Go codec harness failed");
    assert!(String::from_utf8_lossy(&tested.stdout)
        .contains("go codecs: unit and payload variants round-trip, malformed shapes refused"));
    let _ = std::fs::remove_dir_all(&directory);
}

// ---- the suite ---------------------------------------------------------------------------------

/// The suite the model synthesizes, with the authored scenario that sends `Open` beside it.
fn suite(ir: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize(ir);
    assert_eq!(
        synthesis.refusals.len(),
        0,
        "the model synthesizes with no refusal: {:?}",
        synthesis.refusals
    );
    let authoring = ess_conformance::authored::compile(
        ir,
        &[ess_conformance::authored::Source::new(
            "open.yaml",
            OPEN_SCENARIO,
        )],
    );
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    for (id, scenario) in synthesis.suite.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite
}

/// Every scenario's status against one target.
fn statuses<T: ConformanceTarget>(target: &T) -> BTreeMap<String, Status> {
    let ir = ir();
    let suite = suite(&ir);
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, target)
        .into_report();
    report
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The scenario that sends `Open`, and the synthesized one that sends `Complete`.
const OPEN_ID: &str = "demo.work/authored/open-is-reported";

fn assert_all_passed(statuses: &BTreeMap<String, Status>, target: &str) {
    assert!(statuses.len() >= 2, "{target}: {statuses:?}");
    assert!(statuses.contains_key(OPEN_ID), "{target}: {statuses:?}");
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert_eq!(failed.len(), 0, "{target}: {failed:?} of {statuses:?}");
}

#[test]
fn the_suite_passes_against_the_interpreter() {
    let statuses = statuses(&ess_conformance::interpret::Interpreted::for_model(ir()));
    assert_all_passed(&statuses, "interpreted");
    let complete = statuses.keys().filter(|id| id.as_str() != OPEN_ID).count();
    assert!(
        complete >= 1,
        "a synthesized scenario sends `Complete`: {statuses:?}"
    );
}

/// How a faulty target spells the status it publishes.
#[derive(Clone, Copy, Debug)]
enum Fault {
    /// `Open` written with a payload member.
    OpenWithPayload,
    /// `Open` published as `Complete`, and `Complete` as `Open`.
    Swapped,
}

/// A target that is right about everything except how it writes a status.
struct Faulty<T> {
    inner: T,
    fault: Fault,
}

impl<T> Faulty<T> {
    fn mangle(&self, event: &mut ObservedEvent) {
        let Some(Node::Map(status)) = event.payload.get_mut("status") else {
            return;
        };
        let kind = status
            .get("kind")
            .and_then(Node::as_text)
            .map(str::to_owned);
        match (self.fault, kind.as_deref()) {
            (Fault::OpenWithPayload, Some("Open")) => {
                status.insert(
                    "value".to_owned(),
                    Node::Map(BTreeMap::from([(
                        "outcome".to_owned(),
                        Node::Text("shipped".to_owned()),
                    )])),
                );
            }
            (Fault::Swapped, Some("Open")) => {
                status.insert("kind".to_owned(), Node::Text("Complete".to_owned()));
                status.insert(
                    "value".to_owned(),
                    Node::Map(BTreeMap::from([(
                        "outcome".to_owned(),
                        Node::Text("shipped".to_owned()),
                    )])),
                );
            }
            (Fault::Swapped, Some("Complete")) => {
                status.insert("kind".to_owned(), Node::Text("Open".to_owned()));
                status.remove("value");
            }
            _ => {}
        }
    }
}

impl<T: ConformanceTarget> ConformanceTarget for Faulty<T> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("faulty-status", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            self.mangle(event);
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let mut events = self.inner.observe_events(request)?;
        for event in &mut events {
            self.mangle(event);
        }
        Ok(events)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

#[test]
fn a_target_that_writes_open_with_a_payload_member_fails_the_open_scenario() {
    let statuses = statuses(&Faulty {
        inner: ess_conformance::interpret::Interpreted::for_model(ir()),
        fault: Fault::OpenWithPayload,
    });
    assert_eq!(statuses.get(OPEN_ID), Some(&Status::Failed), "{statuses:?}");
}

#[test]
fn a_target_that_confuses_open_with_complete_fails_both_ways() {
    let statuses = statuses(&Faulty {
        inner: ess_conformance::interpret::Interpreted::for_model(ir()),
        fault: Fault::Swapped,
    });
    assert_eq!(statuses.get(OPEN_ID), Some(&Status::Failed), "{statuses:?}");
    assert!(
        statuses
            .iter()
            .any(|(id, status)| id != OPEN_ID && *status == Status::Failed),
        "the scenario sending `Complete` fails too: {statuses:?}"
    );
}

// ---- the suite against the generated servers, over a line protocol -------------------------------

/// The Rust harness: the generated HTTP dispatcher behind one JSON object per line each way.
const RUST_SERVED: &str = r#"
use std::io::{BufRead, Write};

use demo_server::{http, json, work_service as served};
use demo_types::actor::{Actor, Caller};

type System = demo_system::System<demo_types::behaviour::Generated<()>>;

fn assemble() -> System {
    demo_system::System::new(work_service::WorkService::new(demo_types::behaviour::Generated::new(())))
}

fn text<'v>(value: &'v json::Value, name: &str) -> Option<&'v str> {
    match value.member(name) {
        Some(json::Value::Text(text)) => Some(text),
        _ => None,
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();
    let mut system = assemble();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line.expect("the test writes lines");
        let request = json::parse(&line).expect("the test writes JSON");
        let answer = match text(&request, "op") {
            Some("reset") => {
                system = assemble();
                "{\"ok\":true}".to_owned()
            }
            Some("command") => {
                let name = text(&request, "command").unwrap_or_default();
                let body = text(&request, "body").unwrap_or_default();
                let caller = text(&request, "actor").and_then(|actor| {
                    Actor::ALL
                        .iter()
                        .find(|declared| declared.name() == actor)
                        .map(|declared| Caller { actor: *declared })
                });
                match routes.iter().find(|(declared, _, _)| declared == name) {
                    Some((_, method, path)) => {
                        let request = http::Request {
                            method: method.clone(),
                            path: path.clone(),
                            query: String::new(),
                            headers: vec![("content-type".to_owned(), http::JSON.to_owned())],
                            body: body.as_bytes().to_vec(),
                        };
                        let answered = served::dispatch(&mut system, caller.as_ref(), &request);
                        format!("{{\"status\":{},\"answer\":{}}}", answered.status, answered.body)
                    }
                    None => "{\"failure\":\"no route\"}".to_owned(),
                }
            }
            _ => "{\"failure\":\"no operation\"}".to_owned(),
        };
        writeln!(stdout, "{answer}").expect("the test reads lines");
        stdout.flush().expect("the test reads lines");
    }
}
"#;

/// The Go harness: the generated HTTP surface over a real socket, behind the same line protocol.
const GO_SERVED: &str = r#"package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"

	"example.invalid/demo/components/workservice"
	"example.invalid/demo/server"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
)

// authenticate trusts `Authorization: Actor <name>`, which only this harness sends.
func authenticate(request *http.Request) *server.Caller {
	name, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Actor ")
	if !ok {
		return nil
	}
	return &server.Caller{Actor: server.Actor(name)}
}

func started(sys *system.System) (int, error) {
	reader, writer, err := os.Pipe()
	if err != nil {
		return 0, err
	}
	protocol := os.Stdout
	defer func() { os.Stdout = protocol }()
	os.Stdout = writer
	go func() {
		_ = server.ServeWorkService(sys, "127.0.0.1:0", authenticate)
	}()
	lines := bufio.NewReader(reader)
	for {
		line, err := lines.ReadString('\n')
		if err != nil {
			return 0, err
		}
		var record struct {
			Event   string `json:"event"`
			Runtime struct {
				Port int `json:"port"`
			} `json:"runtime"`
		}
		if err := json.Unmarshal([]byte(line), &record); err != nil {
			return 0, err
		}
		if record.Event == "system.ready" {
			return record.Runtime.Port, nil
		}
	}
}

func send(port int, route [2]string, actor string, body string) (any, error) {
	request, err := http.NewRequest(route[0], fmt.Sprintf("http://127.0.0.1:%d%s", port, route[1]), strings.NewReader(body))
	if err != nil {
		return nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	if actor != "" {
		request.Header.Set("Authorization", "Actor "+actor)
	}
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, err
	}
	read, err := io.ReadAll(response.Body)
	_ = response.Body.Close()
	if err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(read))
	decoder.UseNumber()
	var answer any
	if err := decoder.Decode(&answer); err != nil {
		return nil, fmt.Errorf("`%s` is not JSON: %v", read, err)
	}
	return map[string]any{"status": response.StatusCode, "answer": answer}, nil
}

func main() {
	routes := map[string][2]string{}
	for index := 1; index+2 < len(os.Args); index += 3 {
		routes[os.Args[index]] = [2]string{os.Args[index+1], os.Args[index+2]}
	}
	port, err := started(system.NewSystem(workservice.New(behaviour.New(behaviour.Ports{}))))
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	out := bufio.NewWriter(os.Stdout)
	lines := bufio.NewScanner(os.Stdin)
	lines.Buffer(make([]byte, 1<<20), 1<<20)
	for lines.Scan() {
		var request map[string]any
		var answer any
		err := json.Unmarshal(lines.Bytes(), &request)
		if err == nil {
			switch op, _ := request["op"].(string); op {
			case "reset":
				answer = map[string]any{"ok": true}
			case "command":
				name, _ := request["command"].(string)
				body, _ := request["body"].(string)
				actor, _ := request["actor"].(string)
				route, ok := routes[name]
				if !ok {
					err = fmt.Errorf("no route for `%s`", name)
				} else {
					answer, err = send(port, route, actor, body)
				}
			default:
				err = fmt.Errorf("no operation `%s`", op)
			}
		}
		if err != nil {
			answer = map[string]any{"failure": err.Error()}
		}
		encoded, _ := json.Marshal(answer)
		_, _ = out.Write(append(encoded, '\n'))
		_ = out.Flush()
	}
}
"#;

/// The Rust harness binary over the generated server.
fn rust_served() -> PathBuf {
    let synthesis = synthesize_for(&ir(), Target::Rust).expect("the model synthesizes");
    let root = scratch("rust-served");
    write(&synthesis, &root.join("generated"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[workspace]\n[package]\nname = 'served-harness'\nversion = '0.0.0'\nedition = \
             '2021'\npublish = false\n[[bin]]\nname = 'harness'\npath = \
             'src/main.rs'\n[dependencies]\n{RUST_DEPENDENCIES}"
        ),
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), RUST_SERVED).unwrap();
    let target = root.join("target");
    assert!(
        cargo(&harness, &target, &["build", "--offline"])
            .status
            .success(),
        "the Rust served harness builds"
    );
    target.join("debug/harness")
}

/// The Go harness binary over the generated server.
fn go_served() -> PathBuf {
    let synthesis = synthesize_for(&ir(), Target::Go).expect("Go represents the model");
    let root = scratch("go-served");
    let _ = std::fs::remove_dir_all(&root);
    let tree = root.join("demo");
    write(&synthesis, &tree);
    let harness = root.join("harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(
        harness.join("go.mod"),
        "module unitharness\n\ngo 1.21\n\nrequire example.invalid/demo v0.0.0\n\nreplace \
         example.invalid/demo => ../demo\n",
    )
    .unwrap();
    std::fs::write(harness.join("main.go"), GO_SERVED).unwrap();
    let binary = root.join("harness-bin");
    assert!(
        go(
            &harness,
            &["build", "-o", binary.to_str().expect("a UTF-8 path"), "."]
        )
        .status
        .success(),
        "the Go served harness builds"
    );
    binary
}

#[test]
fn the_suite_passes_against_the_generated_rust_server() {
    let binary = rust_served();
    let served = Served::start(&binary, &ir());
    assert_all_passed(&statuses(&served), "generated Rust server");
    let faulty = Faulty {
        inner: Served::start(&binary, &ir()),
        fault: Fault::OpenWithPayload,
    };
    assert_eq!(statuses(&faulty).get(OPEN_ID), Some(&Status::Failed));
    drop((served, faulty));
    let _ = std::fs::remove_dir_all(scratch("rust-served"));
}

#[test]
fn the_suite_passes_against_the_generated_go_server() {
    let binary = go_served();
    let served = Served::start(&binary, &ir());
    assert_all_passed(&statuses(&served), "generated Go server");
    let faulty = Faulty {
        inner: Served::start(&binary, &ir()),
        fault: Fault::Swapped,
    };
    assert_eq!(statuses(&faulty).get(OPEN_ID), Some(&Status::Failed));
    drop((served, faulty));
    let _ = std::fs::remove_dir_all(scratch("go-served"));
}

/// One running harness, and the events one scenario published.
struct Served {
    child: RefCell<Child>,
    stdin: RefCell<ChildStdin>,
    stdout: RefCell<BufReader<ChildStdout>>,
    published: RefCell<Vec<ObservedEvent>>,
    sequence: RefCell<u64>,
}

impl Served {
    fn start(binary: &Path, ir: &EssIr) -> Self {
        let mut routes = Vec::new();
        for component in ir.components().values() {
            for route in ess_gen::http::routes(ir, component) {
                let name = match route.serves {
                    ess_gen::http::Served::Command(handle) => ir.command(handle).name.to_string(),
                    ess_gen::http::Served::View(handle) => ir.view(handle).name.to_string(),
                };
                routes.extend([name, route.method.as_str().to_owned(), route.path]);
            }
        }
        let mut child = Command::new(binary)
            .args(routes)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("the harness starts");
        let stdin = child.stdin.take().expect("piped");
        let stdout = BufReader::new(child.stdout.take().expect("piped"));
        Self {
            child: RefCell::new(child),
            stdin: RefCell::new(stdin),
            stdout: RefCell::new(stdout),
            published: RefCell::default(),
            sequence: RefCell::new(0),
        }
    }

    fn ask(&self, request: &serde_json::Value) -> serde_json::Value {
        let mut stdin = self.stdin.borrow_mut();
        writeln!(stdin, "{request}").expect("the harness reads");
        stdin.flush().expect("the harness reads");
        let mut line = String::new();
        self.stdout
            .borrow_mut()
            .read_line(&mut line)
            .expect("the harness answers");
        serde_json::from_str(&line).unwrap_or_else(|error| panic!("`{line}`: {error}"))
    }

    fn tick(&self) -> u64 {
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        *sequence
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.child.borrow_mut().kill();
        let _ = self.child.borrow_mut().wait();
    }
}

fn json_of(node: &Node) -> serde_json::Value {
    match node {
        Node::Null => serde_json::Value::Null,
        Node::Bool(value) => serde_json::Value::Bool(*value),
        Node::Number(number) => {
            serde_json::from_str(&number.to_string()).expect("a number's spelling is JSON")
        }
        Node::Text(text) => serde_json::Value::String(text.clone()),
        Node::Seq(items) => serde_json::Value::Array(items.iter().map(json_of).collect()),
        Node::Map(members) => serde_json::Value::Object(
            members
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        ),
    }
}

fn node_of(value: &serde_json::Value) -> Node {
    serde_json::from_value(value.clone()).expect("the wire writes JSON a node reads")
}

fn failure(observation: &str, answer: &serde_json::Value) -> TargetError {
    TargetError::unavailable(observation, answer.to_string())
}

impl ConformanceTarget for Served {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "union-unit-variants-generated",
            "1",
        ))
    }

    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        let answer = self.ask(&serde_json::json!({"op": "reset"}));
        if answer["ok"] != serde_json::json!(true) {
            return Err(failure("opening a scenario", &answer));
        }
        self.published.borrow_mut().clear();
        *self.sequence.borrow_mut() = 0;
        Ok(())
    }

    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.published.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        use ess_conformance::scenario::{CommandRef, EventRef, OutcomeRef};
        let observation = format!("invoking `{}`", request.command);
        let body = serde_json::Value::Object(
            request
                .input
                .iter()
                .map(|(name, value)| (name.clone(), json_of(value)))
                .collect(),
        );
        let answer = self.ask(&serde_json::json!({
            "op": "command",
            "command": request.command.to_string(),
            "actor": request.actor.as_ref().map(ToString::to_string),
            "body": body.to_string(),
        }));
        let served = &answer["answer"];
        if answer["status"] == 403 {
            return Err(TargetError::not_granted(served["actor"].as_str()));
        }
        let outcome = served["outcome"]
            .as_str()
            .ok_or_else(|| failure(&observation, &answer))?;
        let outcome = OutcomeRef::new(
            CommandRef::new(request.command.name().clone()),
            outcome
                .parse()
                .map_err(|_| failure(&observation, &answer))?,
        );
        let mut direct_events = Vec::new();
        for published in served["published"].as_array().into_iter().flatten() {
            let reference: EventRef = published["event"]
                .as_str()
                .and_then(|name| name.parse().ok())
                .ok_or_else(|| failure(&observation, &answer))?;
            let mut event = ObservedEvent::new(reference);
            if let Some(serde_json::Value::Object(payload)) = published.get("payload") {
                for (field, value) in payload {
                    event = event.with(field.clone(), node_of(value));
                }
            }
            let event = event
                .in_activity(request.correlation.clone())
                .at(self.tick());
            self.published.borrow_mut().push(event.clone());
            direct_events.push(event);
        }
        let consistency = Some(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.tick()))
                .map_err(|why| TargetError::unavailable(observation.clone(), why.to_string()))?,
        );
        Ok(SemanticCommandResult {
            outcome: Some(outcome),
            error: None,
            consistency,
            direct_events,
            response: None,
        })
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            format!("reading `{}`", request.view),
            "the model declares no view",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "forcing an outcome",
            "none is external",
        ))
    }

    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivering",
            "the model declares no binding",
        ))
    }
}
