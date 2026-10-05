//! The caller-swapped run of a synthesized scenario draws a fresh value for a struct-typed identity
//! input too, fresh in every member, and every copy of it follows: the event payload carrying the
//! whole struct and a view parameter carrying one member (beyond10x/ess#430, after #275).
//!
//! The interpreter does not interpret an `existing_instance:` branch (it answers every scenario
//! sending the command `unsupported`, as `tests/caller_fresh_identity.rs` says), so the reference
//! target here is a fixture that implements the model as specified: one row per identity, a
//! second creation of one refused.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::{CheckCode, Status};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::synthesize, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner,
    ScenarioResult, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

const MODEL: &str = include_str!("fixtures/caller-struct-identity.yaml");
const OPENED: &str = "demo.box.OpenBox/outcome/opened";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("box.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    synthesize(&ir(MODEL)).suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// One run of the scenario: the steps from one send under test to the next.
struct Run<'a> {
    /// The `slot` the send under test carries.
    slot: &'a Node,
    /// Every step of the run.
    steps: &'a [ScenarioStep],
}

/// The two runs, split at each send whose event expectation follows it: the first run's and the
/// caller-swapped one's.
fn runs(scenario: &ConformanceScenario) -> Vec<Run<'_>> {
    let starts: Vec<usize> = scenario
        .steps
        .iter()
        .enumerate()
        .filter(|(at, step)| {
            matches!(step, ScenarioStep::ExecuteCommand { .. })
                && matches!(
                    scenario.steps.get(at + 2),
                    Some(ScenarioStep::ExpectEvent { .. } | ScenarioStep::ExpectEventValues { .. })
                )
        })
        .map(|(at, _)| at)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(n, start)| {
            let end = starts.get(n + 1).copied().unwrap_or(scenario.steps.len());
            let ScenarioStep::ExecuteCommand { input, .. } = &scenario.steps[*start] else {
                unreachable!()
            };
            let ScenarioValue::Literal { value } = &input["slot"] else {
                panic!("the send under test carries a literal slot: {input:?}")
            };
            Run {
                slot: value,
                steps: &scenario.steps[*start..end],
            }
        })
        .collect()
}

fn member<'a>(value: &'a Node, name: &str) -> &'a Node {
    match value {
        Node::Map(members) => &members[name],
        other => panic!("not a struct: {other:?}"),
    }
}

#[test]
fn swapped_run_draws_fresh_struct_identity() {
    let suite = suite();
    let runs = runs(scenario(&suite, OPENED));
    assert_eq!(runs.len(), 2, "the first run and the caller-swapped one");
    for name in ["shelf", "label"] {
        assert_ne!(
            member(runs[0].slot, name),
            member(runs[1].slot, name),
            "the swapped run's `slot.{name}` is its own"
        );
    }
    // No send of either run reuses a slot another send created: each run's rows are its own.
    let mut created: Vec<&Node> = Vec::new();
    for step in &scenario(&suite, OPENED).steps {
        if let ScenarioStep::ExecuteCommand { input, .. } = step {
            if let Some(ScenarioValue::Literal { value }) = input.get("slot") {
                assert!(!created.contains(&value), "{value:?} is sent twice");
                created.push(value);
            }
        }
    }
    let verdicts = verdicts(Mode::Honest);
    assert_eq!(
        verdicts[OPENED].status,
        Status::Passed,
        "{:#?}",
        verdicts[OPENED]
    );
}

#[test]
fn swapped_struct_copy_in_payload_follows() {
    let suite = suite();
    for run in runs(scenario(&suite, OPENED)) {
        let carried = run
            .steps
            .iter()
            .find_map(|step| match step {
                ScenarioStep::ExpectEvent { event, payload, .. }
                    if event.to_string() == "demo.box.BoxOpened" =>
                {
                    payload.get("slot")
                }
                _ => None,
            })
            .expect("the run expects BoxOpened carrying its slot");
        assert_eq!(carried, run.slot);
    }
}

#[test]
fn swapped_struct_member_copy_follows() {
    let suite = suite();
    for run in runs(scenario(&suite, OPENED)) {
        let shelves: Vec<&Node> = run
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::QueryView { view, params }
                    if view.to_string() == "demo.box.ByShelf" =>
                {
                    match params.get("shelf") {
                        Some(ScenarioValue::Literal { value }) => Some(value),
                        other => panic!("shelf is sent as a literal: {other:?}"),
                    }
                }
                _ => None,
            })
            .collect();
        assert!(!shelves.is_empty(), "each run reads by its shelf");
        for shelf in shelves {
            assert_eq!(shelf, member(run.slot, "shelf"));
        }
    }
}

#[test]
fn struct_key_serialization_matches() {
    // The swapped run is rewritten as JSON: a struct identity is found by the serialization of the
    // `Node` it was drawn as, compared with the serialization of the JSON value the suite holds.
    let node = Node::Map(BTreeMap::from([
        ("shelf".to_owned(), Node::from("slot.shelf-1048592")),
        ("label".to_owned(), Node::from("slot.label-1048592")),
        (
            "zone".to_owned(),
            Node::Map(BTreeMap::from([("n".to_owned(), Node::from("a"))])),
        ),
    ]));
    let keyed = serde_json::to_string(&node).unwrap();
    let held: serde_json::Value = serde_json::from_str(&keyed).unwrap();
    assert_eq!(held.to_string(), keyed);
    assert_eq!(serde_json::to_value(&node).unwrap().to_string(), keyed);
    assert_eq!(
        keyed,
        r#"{"label":"slot.label-1048592","shelf":"slot.shelf-1048592","zone":{"n":"a"}}"#
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// One row per identity; a second creation of one is refused.
    Honest,
    /// From the third send on, ignores the sent slot and answers from the record two sends back.
    ReusesFirstRun,
}

/// The box model implemented by hand.
struct Boxes {
    mode: Mode,
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    sends: Cell<usize>,
}

impl ConformanceTarget for Boxes {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("boxes-430", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.sends.set(0);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command: &CommandRef = &request.command;
        let Some(caller) = request.caller.clone() else {
            return Err(TargetError::unavailable(
                "authenticating",
                "sent as a caller",
            ));
        };
        let answer = |name: &str| OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap());
        self.sends.set(self.sends.get() + 1);
        let consistency = ConsistencyToken::new(format!("seq:{}", self.sends.get())).unwrap();
        let mut slot = request.input["slot"].clone();
        let mut rows = self.rows.borrow_mut();
        if self.mode == Mode::ReusesFirstRun && self.sends.get() > 2 {
            slot = rows[self.sends.get() - 3]["slot"].clone();
            return Ok(SemanticCommandResult::took(answer("opened"))
                .emitting(
                    ObservedEvent::new("demo.box.BoxOpened".parse().unwrap()).with("slot", slot),
                )
                .with_consistency(consistency));
        }
        if rows.iter().any(|row| row["slot"] == slot) {
            return Ok(SemanticCommandResult::took(answer("exists"))
                .with_error(DeclaredErrorValue::new("demo.box.Taken".parse().unwrap()))
                .with_consistency(consistency));
        }
        rows.push(BTreeMap::from([
            ("slot".to_owned(), slot.clone()),
            ("opened_by".to_owned(), caller["subject"].clone()),
            ("state".to_owned(), Node::from("Open")),
        ]));
        Ok(SemanticCommandResult::took(answer("opened"))
            .emitting(ObservedEvent::new("demo.box.BoxOpened".parse().unwrap()).with("slot", slot))
            .with_consistency(consistency))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let wanted = request.params.get("shelf");
        Ok(SemanticViewResult::of(
            self.rows
                .borrow()
                .iter()
                .filter(|row| Some(member(&row["slot"], "shelf")) == wanted)
                .cloned()
                .collect::<Vec<_>>(),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("binding", "none"))
    }
}

fn verdicts(mode: Mode) -> BTreeMap<String, ScenarioResult> {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Boxes {
        mode,
        rows: RefCell::default(),
        sends: Cell::new(0),
    };
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

#[test]
fn reused_identity_mutant_fails() {
    let honest = verdicts(Mode::Honest);
    assert_eq!(
        honest[OPENED].status,
        Status::Passed,
        "{:#?}",
        honest[OPENED]
    );
    let reused = verdicts(Mode::ReusesFirstRun);
    let result = &reused[OPENED];
    assert_eq!(result.status, Status::Failed, "{result:#?}");
    assert!(
        result
            .checks
            .iter()
            .any(|check| check.status == Status::Failed
                && check.code == CheckCode::Payload
                && check.about.contains("demo.box.BoxOpened")),
        "the swapped run's BoxOpened.slot expectation fails: {result:#?}"
    );
}
