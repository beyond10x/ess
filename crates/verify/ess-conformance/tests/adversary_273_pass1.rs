//! Adversary pass 1 against beyond10x/ess#273 (captured identities in event expectations).
//!
//! What the unit's own tests do not say:
//!
//! * a sweep over every model this repository carries, synthesized and run against the reference
//!   interpreter twice — as synthesized, and with the #273 identity comparisons taken out — so a
//!   comparison that fails a correct implementation shows up as a scenario only the first run fails;
//! * the same sweep reading the acceptance statement directly: every event field the outcome fills
//!   from an input that the scenario sent as a captured instance is compared with that instance;
//! * two instances of one entity type in one synthesized scenario (a ticket and its parent ticket),
//!   with faults that swap them;
//! * an authored scenario with two queues, and faults the unit's fixture does not apply (the other
//!   real queue, an Optional identity published as null, a zeroed identity, two identities swapped),
//!   run by the Rust runner and the emitted Go and TypeScript runtimes, each required to fail with
//!   the same check code;
//! * admission of the literal half of an identity-comparing expectation against its own shape.

mod support_versions;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::{EssIr, ResolvedPayloadValue};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue, SuiteProvenance,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use ess_primitives::node::Node;

const DESK: &str = include_str!("fixtures/captured-identities.yaml");
const TREE: &str = include_str!("fixtures/adversary-273-tree.yaml");
const GO_TARGET: &str = include_str!("fixtures/adversary-273-runtime.go");
const TS_TARGET: &str = include_str!("fixtures/adversary-273-runtime.mjs");

// ---- models ------------------------------------------------------------------------------------

fn assemble(files: Vec<(String, String)>) -> Option<EssIr> {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(&text).ok()?;
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let spec = Specification::assemble(parsed).ok()?;
    compile(&spec, &sources).ok()
}

fn one(text: &str) -> EssIr {
    assemble(vec![("model.yaml".into(), text.into())]).expect("the model compiles")
}

fn directory(base: &Path) -> Option<EssIr> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).ok()? {
            let path = entry.ok()?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    assemble(
        found
            .into_iter()
            .map(|path| {
                (
                    path.strip_prefix(base).unwrap().display().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                )
            })
            .collect(),
    )
}

/// Every model this repository carries as a file: the example systems and every single-file
/// specification fixture of this crate.
fn every_model() -> Vec<(String, EssIr)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut models = Vec::new();
    for example in [
        "billing",
        "gatepass",
        "oracle-fixture",
        "revision-pair/before",
        "revision-pair/after",
    ] {
        let base = root.join("../../../examples").join(example);
        if let Some(ir) = directory(&base) {
            models.push((format!("examples/{example}"), ir));
        }
    }
    let fixtures = root.join("tests/fixtures");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&fixtures)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|it| it == "yaml"))
        .collect();
    files.sort();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        if let Some(ir) = assemble(vec![("model.yaml".into(), text)]) {
            models.push((
                format!("fixtures/{}", path.file_name().unwrap().to_string_lossy()),
                ir,
            ));
        }
    }
    models
}

fn synthesized(ir: &EssIr) -> Option<ConformanceSuite> {
    catch_unwind(AssertUnwindSafe(|| {
        ess_conformance::synthesize::synthesize(ir).suite
    }))
    .ok()
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The check codes of every failed check per scenario.
fn failed_codes<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, BTreeSet<String>> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let codes = result
                .checks
                .iter()
                .filter(|check| check.status != Status::Passed)
                .map(|check| check.code.to_string())
                .collect();
            (result.scenario.to_string(), codes)
        })
        .collect()
}

fn identity_comparisons(suite: &ConformanceSuite) -> usize {
    suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .map(|step| match step {
            ScenarioStep::ExpectEventValues { payload, .. } => payload
                .values()
                .filter(|value| matches!(value, ScenarioValue::Instance { .. }))
                .count(),
            _ => 0,
        })
        .sum()
}

// ---- sweep: the reference interpreter ------------------------------------------------------------

/// A comparison #273 added must never fail the reference implementation: every scenario the
/// interpreter fails with the comparisons in, it also fails with them taken out.
#[test]
fn sweep_identity_comparisons_never_fail_the_reference_interpreter() {
    let mut compared = 0;
    let mut swept = Vec::new();
    let mut offenders = Vec::new();
    for (name, ir) in every_model() {
        let Some(suite) = synthesized(&ir) else {
            continue;
        };
        let identities = identity_comparisons(&suite);
        if identities == 0 {
            continue;
        }
        let json = suite.to_canonical_json().expect("admitted");
        let without: ConformanceSuite =
            serde_json::from_str(&support_versions::without_captured_identities(&json)).unwrap();
        let run = |suite: &ConformanceSuite| {
            catch_unwind(AssertUnwindSafe(|| {
                statuses(suite, &Interpreted::for_model(ir.clone()))
            }))
            .ok()
        };
        let (Some(with), Some(before)) = (run(&suite), run(&without)) else {
            continue;
        };
        compared += identities;
        swept.push(format!("{name}: {identities}"));
        for (scenario, status) in &with {
            let earlier = before.get(scenario);
            if *status != Status::Passed && earlier == Some(&Status::Passed) {
                offenders.push(format!("{name} {scenario}: {status:?}"));
            }
        }
    }
    eprintln!(
        "swept {} models, {compared} identity comparisons: {swept:#?}",
        swept.len()
    );
    assert!(compared > 0, "the sweep compared nothing");
    assert_eq!(
        offenders.len(),
        0,
        "identity comparisons failing the reference interpreter: {offenders:#?}"
    );
}

/// The acceptance statement, read over every synthesized scenario of every model: an event field
/// the outcome fills from an input the scenario sent as a captured instance is compared with that
/// instance ("synthesis asserts the identity fields it can determine").
#[test]
fn sweep_every_input_identity_an_event_copies_is_compared() {
    let mut checked = 0;
    let mut missed = BTreeSet::new();
    for (name, ir) in every_model() {
        let Some(suite) = synthesized(&ir) else {
            continue;
        };
        for (id, scenario) in &suite.scenarios {
            let mut input: Option<BTreeMap<String, ScenarioValue>> = None;
            let mut outcome = None;
            for step in &scenario.steps {
                match step {
                    ScenarioStep::ExecuteCommand { input: sent, .. } => {
                        input = Some(sent.clone());
                        outcome = None;
                    }
                    ScenarioStep::ExpectOutcome { outcome: taken } => {
                        outcome = Some(taken.clone());
                    }
                    ScenarioStep::ExpectEvent { event, .. }
                    | ScenarioStep::ExpectEventValues { event, .. } => {
                        let (Some(sent), Some(taken)) = (&input, &outcome) else {
                            continue;
                        };
                        let Some(command) = ir
                            .commands()
                            .values()
                            .find(|command| command.name.to_string() == taken.command.to_string())
                        else {
                            continue;
                        };
                        let Some(declared) = command
                            .outcomes
                            .iter()
                            .find(|it| it.name.to_string() == taken.outcome.to_string())
                        else {
                            continue;
                        };
                        let Some(payload) = declared
                            .payload
                            .iter()
                            .find(|payload| payload.event.to_string() == event.to_string())
                        else {
                            continue;
                        };
                        let compared: BTreeMap<String, ScenarioValue> = match step {
                            ScenarioStep::ExpectEventValues { payload, .. } => payload.clone(),
                            _ => BTreeMap::new(),
                        };
                        for field in payload.fields.iter().filter(|it| it.conversion.is_none()) {
                            let ResolvedPayloadValue::InputField { field: from, .. } = &field.value
                            else {
                                continue;
                            };
                            let Some(value @ ScenarioValue::Instance { .. }) = sent.get(from)
                            else {
                                continue;
                            };
                            checked += 1;
                            if compared.get(&field.target) != Some(value) {
                                missed.insert(format!(
                                    "{name} {id}: `{event}.{}` <- input `{from}` = {value:?}",
                                    field.target
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    eprintln!("input identities checked: {checked}");
    assert!(checked > 0, "the sweep found no input identity");
    // Binding scenarios (flow, delivery, mapping, on-failure) compare no event payload values yet
    // (story:binding-site-identity-comparisons). Their misses are pinned exactly, so this goes red
    // when that story lands or when the gap grows; every other scenario must compare every copy.
    let (binding, other): (BTreeSet<_>, BTreeSet<_>) = missed
        .into_iter()
        .partition(|miss| miss.contains("/binding/"));
    assert_eq!(
        other.len(),
        0,
        "input identities an event copies and no step compares: {other:#?}"
    );
    assert_eq!(
        binding.len(),
        4,
        "binding scenarios that copy an input identity without comparing it changed; update this pin \
         with story:binding-site-identity-comparisons: {binding:#?}"
    );
}

// ---- two instances of one entity type, synthesized -------------------------------------------

const CLOSE_TREE: &str = "tree.tickets.CloseTicket/outcome/closed";

/// The interpreter with one fault applied to the `TicketClosed` it publishes.
struct Faulty {
    inner: Interpreted,
    fault: &'static str,
    first_queue: std::cell::RefCell<Option<Node>>,
}

impl Faulty {
    fn new(ir: &EssIr, fault: &'static str) -> Self {
        Self {
            inner: Interpreted::for_model(ir.clone()),
            fault,
            first_queue: std::cell::RefCell::new(None),
        }
    }
}

fn swap(payload: &mut BTreeMap<String, Node>, left: &str, right: &str) {
    let a = payload.get(left).cloned().unwrap_or(Node::Null);
    let b = payload.get(right).cloned().unwrap_or(Node::Null);
    payload.insert(left.into(), b);
    payload.insert(right.into(), a);
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.first_queue.borrow_mut() = None;
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
            let payload = &mut event.payload;
            match (self.fault, event.event.to_string().as_str()) {
                (_, "desk.tickets.QueueOpened") => {
                    let mut first = self.first_queue.borrow_mut();
                    if first.is_none() {
                        *first = payload.get("queue_id").cloned();
                    }
                }
                // Tree: the ticket and its parent, two instances of one entity type.
                ("swap-ticket-parent", "tree.tickets.TicketClosed") => {
                    swap(payload, "ticket_id", "parent_again");
                }
                ("swap-own-parent", "tree.tickets.TicketClosed") => {
                    swap(payload, "own", "parent_id");
                }
                ("parent-is-self", "tree.tickets.TicketClosed") => {
                    let own = payload.get("ticket_id").cloned().unwrap();
                    payload.insert("parent_again".into(), own);
                }
                // Desk.
                ("swap-queue", "desk.tickets.TicketClosed") => {
                    let first = self.first_queue.borrow().clone().unwrap();
                    payload.insert("queue_id".into(), first);
                }
                ("null-closed", "desk.tickets.TicketClosed") => {
                    payload.insert("closed".into(), Node::Null);
                }
                ("zero-ticket", "desk.tickets.TicketClosed") => {
                    payload.insert(
                        "ticket_id".into(),
                        Node::Text("00000000-0000-0000-0000-000000000000".into()),
                    );
                }
                ("swap-pair", "desk.tickets.TicketClosed") => {
                    swap(payload, "ticket_id", "queue_id");
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

#[test]
fn a_ticket_and_its_parent_are_compared_as_two_instances_and_a_swap_fails() {
    let ir = one(TREE);
    let suite = synthesized(&ir).expect("synthesizes");
    let (_, close) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == CLOSE_TREE)
        .unwrap_or_else(|| {
            panic!(
                "no {CLOSE_TREE}: {:?}",
                suite.scenarios.keys().collect::<Vec<_>>()
            )
        });
    let compared = close
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEventValues { event, payload, .. }
                if event.to_string() == "tree.tickets.TicketClosed" =>
            {
                Some(payload.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("TicketClosed compares no identity: {:#?}", close.steps));
    let instance = |field: &str| match compared.get(field) {
        Some(ScenarioValue::Instance { instance }) => instance.to_string(),
        other => panic!("`{field}` is not compared with an instance: {other:?} in {compared:#?}"),
    };
    // The acceptance statement's three sources, on one event: the input identity, the subject's
    // own, and the related row's — here a second instance of the same entity type.
    let ticket = instance("ticket_id");
    assert_eq!(instance("own"), ticket);
    let parent = instance("parent_again");
    assert_ne!(parent, ticket);
    assert_eq!(instance("parent_id"), parent);

    let good = statuses(&suite, &Faulty::new(&ir, "good"));
    assert_eq!(good.get(CLOSE_TREE), Some(&Status::Passed), "{good:#?}");
    for fault in ["swap-ticket-parent", "swap-own-parent", "parent-is-self"] {
        let ran = statuses(&suite, &Faulty::new(&ir, fault));
        assert_eq!(
            ran.get(CLOSE_TREE),
            Some(&Status::Failed),
            "{fault}: {ran:#?}"
        );
    }
}

// ---- two queues, authored, every runner ----------------------------------------------------------

const TWO_QUEUES: &str = r"type: ess-scenario/1
domain: desk.tickets
scenario: close-a-ticket-filed-into-the-second-queue
summary: Closing a ticket names its own queue, not the other one.
arrange:
  - instance: first
    entity: desk.tickets.Queue
  - instance: second
    entity: desk.tickets.Queue
  - instance: t
    entity: desk.tickets.Ticket
timeline:
  - at: 2026-01-05T09:00:00Z
    command: desk.tickets.OpenQueue
    input: {label: front}
    outcome: opened
    capture: {instance: first, event: desk.tickets.QueueOpened, field: queue_id}
  - at: 2026-01-05T09:00:01Z
    command: desk.tickets.OpenQueue
    input: {label: back}
    outcome: opened
    capture: {instance: second, event: desk.tickets.QueueOpened, field: queue_id}
  - at: 2026-01-05T09:00:02Z
    command: desk.tickets.FileTicket
    input: {queue_id: {$instance: second}}
    outcome: filed
    capture: {instance: t, event: desk.tickets.TicketFiled, field: ticket_id}
  - at: 2026-01-05T09:00:03Z
    command: desk.tickets.CloseTicket
    input: {ticket_id: {$instance: t}}
    outcome: closed
    events:
      - event: desk.tickets.TicketClosed
        payload: {ticket_id: {$instance: t}, closed: {$instance: t}, queue_id: {$instance: second}}
";

const AUTHORED: &str = "desk.tickets/authored/close-a-ticket-filed-into-the-second-queue";

fn two_queues(ir: &EssIr) -> ConformanceSuite {
    let result = compile_authored(ir, &[Source::new("two.yaml", TWO_QUEUES)]);
    assert!(result.is_complete(), "{:#?}", result.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    suite.scenarios = result.scenarios;
    suite.select_fresh_format_for(ir);
    suite
}

const DESK_FAULTS: [&str; 4] = ["swap-queue", "null-closed", "zero-ticket", "swap-pair"];

#[test]
fn rust_fails_every_desk_fault_with_the_payload_code() {
    let ir = one(DESK);
    let suite = two_queues(&ir);
    let good = statuses(&suite, &Faulty::new(&ir, "good"));
    assert_eq!(good.get(AUTHORED), Some(&Status::Passed), "{good:#?}");
    for fault in DESK_FAULTS {
        let codes = failed_codes(&suite, &Faulty::new(&ir, fault));
        let codes = codes.get(AUTHORED).cloned().unwrap_or_default();
        assert!(
            codes.contains("ESS-CF-PAYLOAD"),
            "{fault}: failed checks {codes:?}"
        );
    }
}

fn write_all(directory: &Path, artifacts: impl IntoIterator<Item = (String, String)>) {
    for (path, contents) in artifacts {
        let path = directory.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

/// Go and TypeScript, against the same faults, must fail the same scenario with the same code the
/// Rust runner reports.
#[test]
fn go_and_typescript_fail_every_desk_fault_with_the_payload_code() {
    let ir = one(DESK);
    let suite = two_queues(&ir);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-273-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write_all(
        &root,
        ess_conformance::go::emit(&suite)
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
        ess_conformance::ts::emit(&suite)
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
        .expect("tsc");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    let mut problems = Vec::new();
    for fault in std::iter::once("good").chain(DESK_FAULTS) {
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
                .expect("toolchain");
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::write(root.join(format!("{tool}-{fault}.log")), &log).unwrap();
            if output.status.success() != (fault == "good") {
                problems.push(format!(
                    "{tool} {fault}: exit {:?}\n{log}",
                    output.status.code()
                ));
            } else if fault != "good" && !log.contains("ESS-CF-PAYLOAD") {
                problems.push(format!(
                    "{tool} {fault}: failed without ESS-CF-PAYLOAD\n{log}"
                ));
            }
        }
    }
    assert_eq!(problems.len(), 0, "{}", problems.join("\n----\n"));
    std::fs::remove_dir_all(root).ok();
}

// ---- admission ---------------------------------------------------------------------------------

/// The literal half of an expectation is held to the step's own shape at admission
/// (`payload_agrees_with_its_shape`, `admission.rs:63`). A literal written beside no identity is an
/// `expect_event` and is checked; the same literal beside a captured identity is an
/// `expect_event_values` (#273) and must still be checked.
#[test]
fn the_literal_half_of_an_identity_comparing_expectation_is_held_to_its_shape() {
    let ir = one(DESK);
    let text = two_queues(&ir).to_canonical_json().unwrap();
    let mut document: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut tampered = 0;
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "expect_event_values" && step["event"] == "desk.tickets.TicketClosed"
            {
                // A Uuid-typed field holding a literal its declared shape does not admit.
                step["payload"]["queue_id"] =
                    serde_json::json!({"kind": "literal", "value": "not a uuid"});
                tampered += 1;
            }
        }
    }
    assert!(tampered > 0);
    let values = serde_json::to_string(&document).unwrap();
    // Control: the same literal in the pre-#273 `expect_event` form is refused.
    let legacy = support_versions::without_captured_identities(&values);
    let refused = AdmittedSuite::from_json(&legacy).expect_err("expect_event literal checked");
    assert!(refused.to_string().contains("shape"), "{refused}");
    assert!(
        AdmittedSuite::from_json(&values).is_err(),
        "a literal the shape does not admit is admitted once the event also compares an identity"
    );
}

// ---- the committed suites ------------------------------------------------------------------------

/// The interpreter publishing one event field wrongly: dropped, null, the nil UUID, or an identity
/// no run minted.
struct Mutated {
    inner: Interpreted,
    event: String,
    field: String,
    mode: &'static str,
}

impl ConformanceTarget for Mutated {
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
            if event.event.to_string() != self.event {
                continue;
            }
            match self.mode {
                "drop" => {
                    event.payload.remove(&self.field);
                }
                "null" => {
                    event.payload.insert(self.field.clone(), Node::Null);
                }
                "zero" => {
                    event.payload.insert(
                        self.field.clone(),
                        Node::Text("00000000-0000-0000-0000-000000000000".into()),
                    );
                }
                _ => {
                    event.payload.insert(
                        self.field.clone(),
                        Node::Text("00000000-0000-4000-8000-0000000000ff".into()),
                    );
                }
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

/// Every identity the regenerated billing and gatepass suites compare is load-bearing: the scenario
/// that compares it passes the reference and fails it with that one field dropped, null, zeroed or
/// replaced by an identity no run minted.
#[test]
fn every_identity_the_committed_suites_compare_fails_when_dropped_nulled_zeroed_or_replaced() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut tried = 0;
    let mut survivors = Vec::new();
    for name in ["billing", "gatepass"] {
        let ir = directory(&root.join("examples").join(name)).expect("the example compiles");
        let text =
            std::fs::read_to_string(root.join("suites/generated").join(name).join("suite.json"))
                .unwrap();
        let suite: ConformanceSuite = serde_json::from_str(&text).unwrap();
        let good = statuses(&suite, &Interpreted::for_model(ir.clone()));
        let mut compared = BTreeSet::new();
        for (id, scenario) in &suite.scenarios {
            for step in &scenario.steps {
                if let ScenarioStep::ExpectEventValues { event, payload, .. } = step {
                    for (field, value) in payload {
                        if matches!(value, ScenarioValue::Instance { .. }) {
                            compared.insert((id.to_string(), event.to_string(), field.clone()));
                        }
                    }
                }
            }
        }
        assert!(!compared.is_empty(), "{name} compares no identity");
        for (id, event, field) in compared {
            // A scenario the reference interpreter cannot answer is not measured here.
            if good.get(&id) != Some(&Status::Passed) {
                eprintln!(
                    "not measured: {name} {id} {event}.{field}: {:?}",
                    good.get(&id)
                );
                continue;
            }
            for mode in ["drop", "null", "zero", "stranger"] {
                tried += 1;
                let target = Mutated {
                    inner: Interpreted::for_model(ir.clone()),
                    event: event.clone(),
                    field: field.clone(),
                    mode,
                };
                let ran = statuses(&suite, &target);
                if ran.get(&id) != Some(&Status::Failed) {
                    survivors.push(format!(
                        "{name} {id} {event}.{field} {mode}: {:?}",
                        ran.get(&id)
                    ));
                }
            }
        }
    }
    eprintln!("mutants tried: {tried}");
    assert_eq!(survivors.len(), 0, "{survivors:#?}");
}
