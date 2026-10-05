//! An event expectation checks identity fields against captured instances (beyond10x/ess#273).
//!
//! An authored `expect_event` may write `{$instance: name}` for a payload field typed as that
//! instance's identity, and synthesis asserts every identity-typed payload field the arrangement
//! determines: the input identity, the subject's own identity, and a related row's identity. Both
//! are `expect_event_values` steps, which suite/18 introduced, so an implementation that drops such a
//! field, or publishes another identity in its place, fails in the Rust runner, the emitted Go and
//! TypeScript runtimes, and is still replayable in the browser.
//!
//! The model (`fixtures/captured-identities.yaml`) has three identities in one closing event, each
//! determined differently, and an Optional one, so a dropped field is not already caught by the
//! payload's shape.

mod support_versions;

use std::collections::BTreeMap;
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Cause, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue, SuiteProvenance,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/captured-identities.yaml");
const GO_TARGET: &str = include_str!("fixtures/captured-identities-runtime.go");
const TS_TARGET: &str = include_str!("fixtures/captured-identities-runtime.mjs");

/// Every faulty implementation, each of which publishes a wrong identity somewhere a scenario
/// determines it.
const FAULTS: [&str; 4] = ["drop-subject", "swap-input", "swap-related", "swap-filed"];

/// An identity no run minted.
const STRANGER: &str = "00000000-0000-4000-8000-0000000000ff";

const CLOSE: &str = "desk.tickets.CloseTicket/outcome/closed";
const FILE: &str = "desk.tickets.FileTicket/outcome/filed";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// A queue opened and a ticket filed into it, both captured; then the ticket is closed.
const OPENED_AND_FILED: &str = r"type: ess-scenario/1
domain: desk.tickets
scenario: close-a-filed-ticket
summary: Closing a filed ticket names the ticket and its queue.
arrange:
  - instance: q
    entity: desk.tickets.Queue
  - instance: t
    entity: desk.tickets.Ticket
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.tickets.OpenQueue
    input: {label: front}
    outcome: opened
    events:
      - event: desk.tickets.QueueOpened
        payload: {label: front}
    capture: {instance: q, event: desk.tickets.QueueOpened, field: queue_id}
  - at: 2026-01-05T09:00:01Z
    command: desk.tickets.FileTicket
    input: {queue_id: {$instance: q}}
    outcome: filed
    events:
      - event: desk.tickets.TicketFiled
        payload: {queue_id: {$instance: q}}
    capture: {instance: t, event: desk.tickets.TicketFiled, field: ticket_id}
  - at: 2026-01-05T09:00:02Z
    command: desk.tickets.CloseTicket
    input: {ticket_id: {$instance: t}}
    outcome: closed
    events:
      - event: desk.tickets.TicketClosed
        payload: {ticket_id: {$instance: t}, closed: {$instance: t}, queue_id: {$instance: q}}
";

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

fn authored_suite(ir: &EssIr, text: &str) -> ConformanceSuite {
    let result = authoring(ir, text);
    assert!(result.is_complete(), "{:#?}", result.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    suite.scenarios = result.scenarios;
    suite.select_fresh_format_for(ir);
    suite
}

fn synthesized_suite(ir: &EssIr) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(ir).suite
}

/// The values the `expect_event_values` step for `event` compares, in the scenario `id`.
fn event_values(
    suite: &ConformanceSuite,
    id: &str,
    event: &str,
) -> BTreeMap<String, ScenarioValue> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no `{id}`; have {:?}",
                suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        });
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEventValues {
                event: seen,
                payload,
                ..
            } if seen.to_string() == event => Some(payload.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no value expectation of `{event}` in {:#?}", scenario.steps))
}

/// The instance a step's value refers to, or a panic naming what it is instead.
fn instance_of(value: Option<&ScenarioValue>) -> String {
    match value {
        Some(ScenarioValue::Instance { instance }) => instance.to_string(),
        other => panic!("not an instance reference: {other:?}"),
    }
}

// ---- authoring --------------------------------------------------------------------------------

#[test]
fn an_authored_identity_field_compares_with_a_captured_instance() {
    let ir = model();
    let suite = authored_suite(&ir, OPENED_AND_FILED);
    let id = "desk.tickets/authored/close-a-filed-ticket";
    let closed = event_values(&suite, id, "desk.tickets.TicketClosed");
    assert_eq!(instance_of(closed.get("ticket_id")), "t");
    assert_eq!(instance_of(closed.get("closed")), "t");
    assert_eq!(instance_of(closed.get("queue_id")), "q");
    let filed = event_values(&suite, id, "desk.tickets.TicketFiled");
    assert_eq!(instance_of(filed.get("queue_id")), "q");
    // A value-only payload keeps the step it always compiled to.
    let (_, scenario) = suite.scenarios.iter().next().unwrap();
    assert!(scenario.steps.iter().any(|step| matches!(
        step,
        ScenarioStep::ExpectEvent { event, .. } if event.to_string() == "desk.tickets.QueueOpened"
    )));
}

#[test]
fn an_authored_instance_at_an_event_field_of_another_type_is_still_not_comparable() {
    let ir = model();
    for (written, field) in [
        // Not an identity at all.
        (
            "payload: {label: front}",
            "payload: {label: {$instance: q}}",
        ),
        // The other entity's identity.
        (
            "payload: {queue_id: {$instance: q}}",
            "payload: {queue_id: {$instance: t}}",
        ),
    ] {
        let text = OPENED_AND_FILED.replacen(written, field, 1);
        assert_ne!(text, OPENED_AND_FILED);
        let result = authoring(&ir, &text);
        let causes: Vec<&Cause> = result.refusals.iter().map(|it| &it.cause).collect();
        assert!(
            causes.iter().any(|cause| matches!(
                cause,
                Cause::NotComparable { field: named, .. }
                    if field.contains(&format!("{named}: {{$instance"))
            )),
            "{field}: {causes:#?}"
        );
    }
}

#[test]
fn an_authored_suite_with_instance_valued_event_values_needs_suite_18() {
    let ir = model();
    let suite = authored_suite(&ir, OPENED_AND_FILED);
    assert!(suite.provenance.suite_version.major() >= ess_conformance::fixtures::ORDINARY);
    let json = suite.to_canonical_json().expect("admitted");
    AdmittedSuite::from_json(&json).unwrap_or_else(|error| panic!("{error}"));
    let older = support_versions::legacy_json(&json, ess_conformance::fixtures::ORDINARY - 1);
    assert!(AdmittedSuite::from_json(&older).is_err());
    let mut pinned = suite;
    pinned.provenance.suite_version = ess_conformance::scenario::SuiteFormat::parse(&format!(
        "ess-conformance/{}",
        ess_conformance::fixtures::ORDINARY - 1
    ))
    .unwrap();
    pinned.provenance.scenario_initial_state = None;
    assert!(pinned.to_canonical_json().is_err());
}

// ---- synthesis --------------------------------------------------------------------------------

#[test]
fn synthesis_asserts_every_identity_the_arrangement_determines() {
    let ir = model();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let suite = &synthesis.suite;
    let closed = event_values(suite, CLOSE, "desk.tickets.TicketClosed");
    // The input identity and the subject's own are the one ticket the scenario closes.
    let ticket = instance_of(closed.get("ticket_id"));
    assert_eq!(instance_of(closed.get("closed")), ticket);
    // The related row the ticket was filed into is another instance.
    let queue = instance_of(closed.get("queue_id"));
    assert_ne!(queue, ticket);
    // Filing names a queue no relation declares, so the synthesizer sends a literal identity and
    // compares it as one: the step is unchanged.
    let (_, filing) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == FILE)
        .unwrap();
    assert!(filing.steps.iter().any(|step| matches!(
        step,
        ScenarioStep::ExpectEvent { event, payload, .. }
            if event.to_string() == "desk.tickets.TicketFiled" && payload.contains_key("queue_id")
    )));
    assert!(suite.provenance.suite_version.major() >= ess_conformance::fixtures::ORDINARY);
    AdmittedSuite::from_json(&suite.to_canonical_json().expect("admitted"))
        .unwrap_or_else(|error| panic!("{error}"));
}

// ---- execution: Rust --------------------------------------------------------------------------

/// The interpreter with one fault applied to what it publishes.
struct Faulty {
    inner: Interpreted,
    fault: &'static str,
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = self.inner.execute_command(request)?;
        for event in &mut result.direct_events {
            let stranger = Node::Text(STRANGER.into());
            match (self.fault, event.event.to_string().as_str()) {
                ("drop-subject", "desk.tickets.TicketClosed") => {
                    event.payload.remove("closed");
                }
                ("swap-input", "desk.tickets.TicketClosed") => {
                    event.payload.insert("ticket_id".into(), stranger);
                }
                ("swap-related", "desk.tickets.TicketClosed")
                | ("swap-filed", "desk.tickets.TicketFiled") => {
                    event.payload.insert("queue_id".into(), stranger);
                }
                _ => {}
            }
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
        self.inner.observe_events(request)
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
}

/// Each scenario's status against `fault` ("good" for none).
fn rust_statuses(
    ir: &EssIr,
    suite: &ConformanceSuite,
    fault: &'static str,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Faulty {
        inner: Interpreted::for_model(ir.clone()),
        fault,
    };
    Runner::for_suite(suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn assert_rust_catches_every_fault(ir: &EssIr, suite: &ConformanceSuite, label: &str) {
    let good = rust_statuses(ir, suite, "good");
    assert!(!good.is_empty());
    assert!(
        good.values().all(|status| *status == Status::Passed),
        "{label} good: {good:#?}"
    );
    for fault in FAULTS {
        let statuses = rust_statuses(ir, suite, fault);
        assert!(
            statuses.values().any(|status| *status == Status::Failed),
            "{label} {fault} passed every scenario: {statuses:#?}"
        );
    }
}

#[test]
fn the_rust_runner_fails_every_identity_fault_in_the_authored_suite() {
    let ir = model();
    assert_rust_catches_every_fault(&ir, &authored_suite(&ir, OPENED_AND_FILED), "authored");
}

#[test]
fn the_rust_runner_fails_every_identity_fault_in_the_synthesized_suite() {
    let ir = model();
    let suite = synthesized_suite(&ir);
    assert_rust_catches_every_fault(&ir, &suite, "synthesized");
    // The scenario that closes a ticket is the one each closing fault fails.
    for fault in ["drop-subject", "swap-input", "swap-related"] {
        assert_eq!(
            rust_statuses(&ir, &suite, fault).get(CLOSE),
            Some(&Status::Failed),
            "{fault}"
        );
    }
    assert_eq!(
        rust_statuses(&ir, &suite, "swap-filed").get(FILE),
        Some(&Status::Failed)
    );
}

// ---- execution: Go and TypeScript -------------------------------------------------------------

fn write_all(directory: &std::path::Path, artifacts: impl IntoIterator<Item = (String, String)>) {
    for (path, contents) in artifacts {
        let path = directory.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

/// Runs `suite` in the emitted Go and TypeScript runtimes against the hand-written desk, healthy
/// and with every fault, and requires the healthy one alone to pass.
fn assert_native_runtimes_catch_every_fault(suite: &ConformanceSuite, label: &str) {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "captured-identities-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    write_all(
        &root,
        ess_conformance::go::emit(suite)
            .unwrap_or_else(|error| panic!("{error}"))
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/desk\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(root.join("essconform/desk_test.go"), GO_TARGET).unwrap();
    write_all(
        &root.join("typescript"),
        ess_conformance::ts::emit(suite)
            .unwrap_or_else(|error| panic!("{error}"))
            .into_iter()
            .map(|artifact| (artifact.path, artifact.contents)),
    );
    let ts = root.join("typescript/essconform");
    std::fs::write(ts.join("desk.mjs"), TS_TARGET).unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .expect("tsc is required for the TypeScript runtime");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    for fault in std::iter::once("good").chain(FAULTS) {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "desk.mjs"], &ts),
        ] {
            let output = Command::new(tool)
                .args(args)
                .env("ESS_IDENTITY_FAULT", fault)
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .env("GOFLAGS", "-mod=mod")
                .env_remove("ESS_REPORT_OUT")
                .current_dir(directory)
                .output()
                .expect("the emitted runtime's toolchain is required");
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::write(root.join(format!("{tool}-{fault}.log")), &log).unwrap();
            assert_eq!(
                output.status.success(),
                fault == "good",
                "{label} {tool} {fault}: {log}"
            );
            if fault != "good" {
                assert!(
                    log.contains("TicketClosed") || log.contains("TicketFiled"),
                    "{label} {tool} {fault} failed for another reason: {log}"
                );
            }
        }
    }
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn go_and_typescript_fail_every_identity_fault_in_the_authored_suite() {
    let ir = model();
    assert_native_runtimes_catch_every_fault(&authored_suite(&ir, OPENED_AND_FILED), "authored");
}

#[test]
fn go_and_typescript_fail_every_identity_fault_in_the_synthesized_suite() {
    let ir = model();
    assert_native_runtimes_catch_every_fault(&synthesized_suite(&ir), "synthesized");
}

// ---- browser replay ---------------------------------------------------------------------------

/// The declaration player replays a suite whose event expectations compare captured identities:
/// only an independently provisioned fixture value is beyond it.
#[test]
fn the_declaration_player_replays_instance_valued_event_expectations() {
    let ir = model();
    let suite = authored_suite(&ir, OPENED_AND_FILED);
    let artifacts =
        ess_conformance::web::emit(&ir, &suite).unwrap_or_else(|error| panic!("{error}"));
    let json = &artifacts["suite.json"].contents;
    assert!(json.contains("\"expect_event_values\""));
    assert!(json.contains("\"instance\": \"t\""));
    let synthesized = synthesized_suite(&ir);
    ess_conformance::web::emit(&ir, &synthesized).unwrap_or_else(|error| panic!("{error}"));
}

const REPLAY_HARNESS: &str = r"import {readFileSync} from 'node:fs'
import {admitReplay} from './admission.js'
const admitted = await admitReplay(readFileSync(new URL('./replay.json', import.meta.url), 'utf8'))
const found = []
for (const [id, scenario] of Object.entries(admitted.suite.scenarios))
  for (const step of scenario.steps)
    if (step.step === 'expect_event_values')
      found.push({id, event: step.event, payload: step.payload})
console.log(JSON.stringify(found))
";

/// The coverage replay's own admitter — the code the browser runs before any replay state exists —
/// admits the paired document, and hands the player the instance references as references.
#[test]
fn coverage_replay_admits_instance_valued_event_expectations() {
    use ess_conformance::coverage::{Origins, Scope};
    use ess_conformance::coverage_build::{build, CoverageSource};
    let ir = model();
    let source = CoverageSource::new("close.yaml", OPENED_AND_FILED.to_owned()).unwrap();
    let input = build(&ir, &[source], Scope::System, Origins::GeneratedAndAuthored)
        .unwrap_or_else(|error| panic!("{error}"));
    let artifacts =
        ess_conformance::web::emit_input(&ir, &input).unwrap_or_else(|error| panic!("{error}"));
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("captured-identities-replay-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("admission.js"),
        &artifacts["admission.js"].contents,
    )
    .unwrap();
    std::fs::write(root.join("replay.json"), &artifacts["replay.json"].contents).unwrap();
    std::fs::write(root.join("harness.mjs"), REPLAY_HARNESS).unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("node is required for the browser admitter");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let found: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let found = found.as_array().unwrap();
    let authored = found
        .iter()
        .find(|step| {
            step["id"] == "desk.tickets/authored/close-a-filed-ticket"
                && step["event"] == "desk.tickets.TicketClosed"
        })
        .unwrap_or_else(|| panic!("{found:#?}"));
    assert_eq!(
        authored["payload"]["ticket_id"],
        serde_json::json!({"kind": "instance", "instance": "t"})
    );
    assert!(found
        .iter()
        .any(|step| step["id"] == CLOSE && step["event"] == "desk.tickets.TicketClosed"));
    std::fs::remove_dir_all(root).ok();
}

const SUITE_HARNESS: &str = r"import {readFileSync} from 'node:fs'
import {admitSuite} from './admission.js'
const verdicts = {}
for (const name of ['unchanged', 'uuid', 'not-uuid']) {
  try { await admitSuite(readFileSync(new URL(`./${name}.json`, import.meta.url), 'utf8')); verdicts[name] = 'admitted' }
  catch (error) { verdicts[name] = String(error) }
}
console.log(JSON.stringify(verdicts))
";

/// The browser's admitter holds the literal half of an `expect_event_values` step to the step's own
/// shape, as `admission.rs` does and as both do for `expect_event`; a reference has no value there.
#[test]
fn the_browser_admitter_holds_event_value_literals_to_their_shape() {
    use ess_conformance::coverage::{Origins, Scope};
    use ess_conformance::coverage_build::{build, CoverageSource};
    let ir = model();
    let source = CoverageSource::new("close.yaml", OPENED_AND_FILED.to_owned()).unwrap();
    let input = build(&ir, &[source], Scope::System, Origins::Authored)
        .unwrap_or_else(|error| panic!("{error}"));
    let original = input.selected().original_json().to_owned();
    // The closing event's `ticket_id` written as a literal instead of the captured instance.
    let literal = |value: &str| {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let mut written = 0;
        for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
            for step in scenario["steps"].as_array_mut().unwrap() {
                if step["step"] == "expect_event_values"
                    && step["event"] == "desk.tickets.TicketClosed"
                {
                    step["payload"]["ticket_id"] =
                        serde_json::json!({"kind": "literal", "value": value});
                    written += 1;
                }
            }
        }
        assert_eq!(written, 1);
        serde_json::to_string(&document).unwrap()
    };
    let (uuid, not_uuid) = (
        literal("00000000-0000-4000-8000-000000000001"),
        literal("not a uuid"),
    );
    // The Rust admitter, the reference both must agree with.
    AdmittedSuite::from_json(&uuid).unwrap_or_else(|error| panic!("{error}"));
    assert!(AdmittedSuite::from_json(&not_uuid).is_err());
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "captured-identities-literals-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let artifacts =
        ess_conformance::web::emit_input(&ir, &input).unwrap_or_else(|error| panic!("{error}"));
    std::fs::write(
        root.join("admission.js"),
        &artifacts["admission.js"].contents,
    )
    .unwrap();
    for (name, text) in [
        ("unchanged", &original),
        ("uuid", &uuid),
        ("not-uuid", &not_uuid),
    ] {
        std::fs::write(root.join(format!("{name}.json")), text).unwrap();
    }
    std::fs::write(root.join("harness.mjs"), SUITE_HARNESS).unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("node is required for the browser admitter");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let verdicts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(verdicts["unchanged"], "admitted", "{verdicts}");
    assert_eq!(verdicts["uuid"], "admitted", "{verdicts}");
    assert!(
        verdicts["not-uuid"]
            .as_str()
            .unwrap()
            .contains("payload value the step's own shape does not admit"),
        "{verdicts}"
    );
    std::fs::remove_dir_all(root).ok();
}
