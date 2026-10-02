//! A `when_subject:` guard over a field the creating command copied from a related row is arranged
//! (beyond10x/ess#307).
//!
//! `CreateRun` copies its target's `auto_promote` onto the run it creates; `Report` promotes a run
//! only where that copied flag holds `true`. The only way to hold the run's flag at `true` is to add
//! a target with it first and create the run from that target, so the stored-row search arranges
//! the target before the run, with the value the branch needs.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr, refs::CommandRef, refs::OutcomeRef, resolve::compile, source::SourceMap,
};
use ess_conformance::report::Status;
use ess_conformance::{
    scenario::{ErrorRef, ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceScenario, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/subject-guard-copied-field.yaml");

const PROMOTED: &str = "mini.m.Report/outcome/promoted";
const FINISHED: &str = "mini.m.Report/outcome/finished";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("mini.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)))
}

/// Each command the scenario sends, with the input it sends.
fn sent(scenario: &ConformanceScenario) -> Vec<(String, BTreeMap<String, ScenarioValue>)> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                Some((command.to_string(), input.clone()))
            }
            _ => None,
        })
        .collect()
}

/// The branches reading the copied flag are synthesized, with no ESS-SYNTH-003 or ESS-SYNTH-004
/// anywhere in the suite.
#[test]
fn issue_307_a_guard_over_a_copied_field_is_arranged() {
    let synthesis = synthesize(&ir());
    let refused = refusals(&synthesis);
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-003") || refusal.contains("ESS-SYNTH-004")),
        "{refused:#?}"
    );
    scenario(&synthesis, PROMOTED);
    scenario(&synthesis, FINISHED);
}

/// The promoted run is created from a target added with `auto_promote: true` before it, and the
/// run the scenario reports on is that one.
#[test]
fn issue_307_the_promoted_run_is_created_from_a_target_holding_true() {
    let synthesis = synthesize(&ir());
    let sent = sent(scenario(&synthesis, PROMOTED));
    let added = sent
        .iter()
        .position(|(command, input)| {
            command == "mini.m.AddTarget"
                && matches!(
                    input.get("auto_promote"),
                    Some(ScenarioValue::Literal {
                        value: Node::Bool(true)
                    })
                )
        })
        .unwrap_or_else(|| panic!("a target holding true is added: {sent:#?}"));
    let created = sent
        .iter()
        .position(|(command, input)| {
            command == "mini.m.CreateRun"
                && matches!(input.get("target_id"), Some(ScenarioValue::Instance { .. }))
        })
        .unwrap_or_else(|| panic!("the run is created naming an added target: {sent:#?}"));
    assert!(added < created, "{sent:#?}");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Stores no flag on the run, so a report never promotes.
    CopiesNothing,
}

type Row = BTreeMap<String, Node>;

/// A hand-written target holding targets and runs.
struct Store {
    mutant: Mutant,
    targets: RefCell<Vec<Row>>,
    runs: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn event(name: &str, field: &str, value: Node) -> ObservedEvent {
    ObservedEvent::new(name.parse().unwrap()).with(field, value)
}

impl Store {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            targets: RefCell::default(),
            runs: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }

    fn report(&self, command: &CommandRef, input: &Row) -> SemanticCommandResult {
        let named = input.get("run_id").cloned().unwrap_or(Node::Null);
        let mut runs = self.runs.borrow_mut();
        let found = runs.iter_mut().find(|run| run["run_id"] == named);
        let Some(run) = found.filter(|run| run["state"] == Node::Text("Running".to_owned())) else {
            return SemanticCommandResult::took(outcome(command, "wrong-state")).with_error(
                DeclaredErrorValue::new("mini.m.Conflict".parse::<ErrorRef>().unwrap()),
            );
        };
        let healthy = input.get("result") == Some(&Node::Text("Healthy".to_owned()));
        let flag = run.get("auto_promote") == Some(&Node::Bool(true));
        let promote = healthy && flag;
        let (state, name, emitted) = if promote {
            ("Promoted", "promoted", "mini.m.Promoted")
        } else {
            ("Done", "finished", "mini.m.Finished")
        };
        run.insert("state".to_owned(), Node::Text(state.to_owned()));
        SemanticCommandResult::took(outcome(command, name))
            .emitting(event(emitted, "run_id", named))
    }
}

impl ConformanceTarget for Store {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("runs-307", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.targets.replace(Vec::new());
        self.runs.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let input = &request.input;
        let result = match command.to_string().as_str() {
            "mini.m.AddTarget" => {
                let id = self.mint();
                self.targets.borrow_mut().push(Row::from([
                    ("target_id".to_owned(), id.clone()),
                    (
                        "auto_promote".to_owned(),
                        input.get("auto_promote").cloned().unwrap_or(Node::Null),
                    ),
                ]));
                SemanticCommandResult::took(outcome(&command, "added")).emitting(event(
                    "mini.m.TargetAdded",
                    "target_id",
                    id,
                ))
            }
            "mini.m.CreateRun" => {
                let id = self.mint();
                let target = input.get("target_id").cloned().unwrap_or(Node::Null);
                let flag = match self.mutant {
                    Mutant::CopiesNothing => Node::Null,
                    Mutant::None => self
                        .targets
                        .borrow()
                        .iter()
                        .find(|row| row["target_id"] == target)
                        .map_or(Node::Null, |row| row["auto_promote"].clone()),
                };
                self.runs.borrow_mut().push(Row::from([
                    ("run_id".to_owned(), id.clone()),
                    ("target_id".to_owned(), target),
                    ("auto_promote".to_owned(), flag),
                    ("state".to_owned(), Node::Text("Running".to_owned())),
                ]));
                SemanticCommandResult::took(outcome(&command, "created")).emitting(event(
                    "mini.m.RunCreated",
                    "run_id",
                    id,
                ))
            }
            "mini.m.Report" => self.report(&command, input),
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = match request.view.to_string().as_str() {
            "mini.m.Runs" => self.runs.borrow().clone(),
            _ => Vec::new(),
        };
        Ok(SemanticViewResult::of(rows))
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

fn statuses(mutant: Mutant) -> BTreeMap<String, Status> {
    let suite = synthesize(&ir()).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|e| panic!("{e}"));
    let target = Store::new(mutant);
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// A correct hand-written target passes every scenario of the suite.
#[test]
fn issue_307_a_correct_target_passes_the_suite() {
    let statuses = statuses(Mutant::None);
    assert!(statuses.contains_key(PROMOTED), "{statuses:#?}");
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// A target that does not copy the flag fails a scenario the copy decides.
#[test]
fn issue_307_a_target_misreading_the_copy_fails() {
    let statuses = statuses(Mutant::CopiesNothing);
    assert!(
        [PROMOTED, FINISHED]
            .iter()
            .any(|id| statuses.get(*id) == Some(&Status::Failed)),
        "{statuses:#?}"
    );
}

/// The interpreted target fails no scenario of the suite.
#[test]
fn issue_307_the_interpreted_target_fails_no_scenario() {
    let model = ir();
    let result = synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    for run in &report.scenarios {
        assert!(
            matches!(run.status, Status::Passed | Status::Unsupported),
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
}
