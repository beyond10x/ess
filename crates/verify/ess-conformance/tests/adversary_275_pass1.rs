//! Adversary pass 1 for beyond10x/ess#275: the caller-swapped run draws fresh identity inputs.
//!
//! The swapped run's identities are redrawn by **key name** (`caller.rs` `redraw`): a value is
//! replaced only where it sits under the input name, the event field the created instance is
//! observed from, or the entity's identity field. A model that carries the same identity under any
//! other name — an event field that echoes it, a stored field (and the view column) that copies it
//! — keeps the first run's literal in those places, so the swapped run sends a fresh identity and
//! still expects the old one back. An implementation that does exactly what the model says fails
//! it.
//!
//! The interpreter does not run `existing_instance:` branches, so the reference target is a
//! hand-written fixture, as in `tests/caller_fresh_identity.rs`.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::{
    report::Status, synthesize::synthesize, AdmittedSuite, ConformanceScenario, ConformanceSuite,
    Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// `caller_fresh_identity.rs`'s model, with the identity echoed in a second event field (`echo`)
/// and copied into a stored field (`origin`) the view shows, plus a command that updates a record
/// by an input of another name (`target`).
const ECHOED: &str = "format: ess/16
system: demo
version: v1
summary: A caller-supplied identity echoed under other names.
domain: demo.records
types:
  - {name: demo.records.RecordId, kind: newtype, of: Uuid}
  - {name: demo.records.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.records.Record
    identity: {name: record_id, type: demo.records.RecordId}
    fields:
      - {name: account_id, type: demo.records.AccountId}
      - {name: origin, type: demo.records.RecordId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.records.Clerk
    attributes:
      - {name: account_id, type: demo.records.AccountId}
    may: [demo.records.RecordEntry, demo.records.Amend]
commands:
  - name: demo.records.RecordEntry
    input:
      - {name: record_id, type: demo.records.RecordId}
      - {name: text, type: String}
    outcomes:
      - name: recorded
        creates: demo.records.Record
        instance: record_id
        sets: {account_id: {caller: account_id}, origin: input.record_id, text: input.text}
        emits: [demo.records.Recorded]
        payload:
          demo.records.Recorded: {record_id: input.record_id, echo: input.record_id, account_id: {caller: account_id}, text: input.text}
      - {name: already-recorded, existing_instance: true, error: demo.records.AlreadyRecorded}
  - name: demo.records.Amend
    input:
      - {name: target, type: demo.records.RecordId}
      - {name: text, type: String}
    outcomes:
      - name: amended
        updates: demo.records.Record
        instance: target
        sets: {text: input.text}
        emits: [demo.records.Amended]
        payload:
          demo.records.Amended: {record_id: input.target, text: input.text}
errors:
  - name: demo.records.AlreadyRecorded
    summary: A record with this id already exists.
events:
  - name: demo.records.Recorded
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: echo, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: text, type: String}
  - name: demo.records.Amended
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: text, type: String}
views:
  - name: demo.records.RecordDetails
    source: demo.records.Record
    consistency: read_your_writes
    fields:
      - {name: record_id, type: demo.records.RecordId}
      - {name: account_id, type: demo.records.AccountId}
      - {name: origin, type: demo.records.RecordId}
      - {name: text, type: String}
";

const RECORDED_SCENARIO: &str = "demo.records.RecordEntry/outcome/recorded";
const AMENDED_SCENARIO: &str = "demo.records.Amend/outcome/amended";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("records.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&ir(ECHOED));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// The scenario's two runs: the steps before the first `RecordEntry` sent by a caller other than
/// the first step's, and the steps from it on.
fn halves(scenario: &ConformanceScenario) -> (&[ScenarioStep], &[ScenarioStep]) {
    let caller_of = |step: &ScenarioStep| match step {
        ScenarioStep::ExecuteCommand {
            command, caller, ..
        } if command.to_string() == "demo.records.RecordEntry" => Some(caller.clone()),
        _ => None,
    };
    let first = caller_of(&scenario.steps[0]).expect("the scenario opens with RecordEntry");
    let at = scenario
        .steps
        .iter()
        .position(|step| caller_of(step).is_some_and(|caller| caller != first))
        .unwrap_or_else(|| panic!("no swapped run: {:#?}", scenario.steps));
    scenario.steps.split_at(at)
}

/// Every literal `record_id` a run sends to `RecordEntry`, serialized.
fn sent_identities(steps: &[ScenarioStep]) -> BTreeSet<String> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.records.RecordEntry" =>
            {
                match input.get("record_id") {
                    Some(ScenarioValue::Literal { value }) => {
                        Some(serde_json::to_string(value).unwrap())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

/// Every string anywhere in `steps`, serialized as JSON strings.
fn strings(steps: &[ScenarioStep]) -> BTreeSet<String> {
    fn walk(value: &serde_json::Value, found: &mut BTreeSet<String>) {
        match value {
            serde_json::Value::String(_) => {
                found.insert(value.to_string());
            }
            serde_json::Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
            serde_json::Value::Object(map) => map.values().for_each(|item| walk(item, found)),
            _ => {}
        }
    }
    let mut found = BTreeSet::new();
    walk(&serde_json::to_value(steps).unwrap(), &mut found);
    found
}

/// The swapped run sends an identity of its own; every place the model echoes that identity must
/// then hold the swapped one, so no first-run identity is left anywhere in the swapped run.
#[test]
fn adversary_275_the_swapped_run_keeps_no_first_run_identity_under_another_name() {
    let suite = suite();
    for id in [RECORDED_SCENARIO, AMENDED_SCENARIO] {
        let (first, again) = halves(scenario(&suite, id));
        let old = sent_identities(first);
        let new = sent_identities(again);
        assert!(
            !old.is_empty() && !new.is_empty(),
            "{id} sends record_id in both runs"
        );
        assert!(old.is_disjoint(&new), "{id}: the swapped run draws afresh");
        let carried = strings(again);
        let left: Vec<&String> = carried.intersection(&old).collect();
        assert!(
            left.is_empty(),
            "{id}: the swapped run sends {new:?} but still carries the first run's {left:?}: {again:#?}"
        );
    }
}

/// `echo: input.record_id` — the swapped run's expected `Recorded` carries the identity it sent.
#[test]
fn adversary_275_an_event_field_echoing_the_identity_expects_the_swapped_identity() {
    let suite = suite();
    let (_, again) = halves(scenario(&suite, RECORDED_SCENARIO));
    let sent = sent_identities(again);
    let echoed: Vec<String> = again
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.records.Recorded" =>
            {
                payload
                    .get("echo")
                    .map(|value| serde_json::to_string(value).unwrap())
            }
            _ => None,
        })
        .collect();
    assert!(!echoed.is_empty(), "the swapped run expects Recorded.echo");
    for echo in &echoed {
        assert!(
            sent.contains(echo),
            "the swapped run sends record_id {sent:?} and expects Recorded.echo {echo}"
        );
    }
}

// ---- running the suite ------------------------------------------------------------------------

/// The model implemented as specified: refuses an identity already stored, copies the identity
/// into `origin` and `echo`, and amends the record `target` names.
#[derive(Default)]
struct Ledger {
    records: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u32>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Ledger {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("echoed-records-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.records.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let Some(caller) = request.caller.clone() else {
            return Err(TargetError::unavailable(
                "authenticating",
                "every command of this system is sent as a caller",
            ));
        };
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut records = self.records.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.records.RecordEntry" => {
                let (Some(id), Some(text)) = (
                    request.input.get("record_id").cloned(),
                    request.input.get("text").cloned(),
                ) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let Node::Text(key) = &id else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                if records.contains_key(key) {
                    SemanticCommandResult::took(outcome(&command, "already-recorded")).with_error(
                        DeclaredErrorValue::new("demo.records.AlreadyRecorded".parse().unwrap()),
                    )
                } else {
                    let account = caller["account_id"].clone();
                    records.insert(
                        key.clone(),
                        BTreeMap::from([
                            ("record_id".to_owned(), id.clone()),
                            ("account_id".to_owned(), account.clone()),
                            ("origin".to_owned(), id.clone()),
                            ("text".to_owned(), text.clone()),
                            ("state".to_owned(), Node::Text("Open".to_owned())),
                        ]),
                    );
                    SemanticCommandResult::took(outcome(&command, "recorded")).emitting(
                        ObservedEvent::new("demo.records.Recorded".parse().unwrap())
                            .with("record_id", id.clone())
                            .with("echo", id)
                            .with("account_id", account)
                            .with("text", text),
                    )
                }
            }
            "demo.records.Amend" => {
                let (Some(Node::Text(key)), Some(text)) = (
                    request.input.get("target").cloned(),
                    request.input.get("text").cloned(),
                ) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let Some(row) = records.get_mut(&key) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                row.insert("text".to_owned(), text.clone());
                SemanticCommandResult::took(outcome(&command, "amended")).emitting(
                    ObservedEvent::new("demo.records.Amended".parse().unwrap())
                        .with("record_id", Node::Text(key))
                        .with("text", text),
                )
            }
            other => panic!("no command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.records.borrow().values().cloned(),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Acceptance 1 on a model that echoes its caller-supplied identity: every scenario passes an
/// implementation of the model under both caller orders.
#[test]
fn adversary_275_every_scenario_passes_when_the_identity_is_echoed_under_other_names() {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Ledger::default())
        .into_report();
    assert_ne!(report.scenarios.len(), 0);
    let failed: Vec<(String, Status)> = report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .inspect(|result| eprintln!("{result:#?}"))
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    assert_eq!(failed, Vec::new());
}

/// Go parity on the same suite and target: the Go runner gives the Rust runner's verdicts.
#[test]
fn adversary_275_go_gives_the_reference_verdicts_on_the_echoed_suite() {
    let verdicts = support_go::assert_parity("adversary-275-echoed", &suite(), Ledger::default());
    assert!(!verdicts.is_empty());
}
