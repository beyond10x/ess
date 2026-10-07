//! A `when_subject:` guard over a field the creating command copied from a related row is arranged
//! (beyond10x/ess#307).
//!
//! `CreateRun` copies both Optional policy flags from its target. `Report` selects promotion or
//! rollback from the copied flag and its result input. Each branch and transition needs the actual
//! named source, while boundary witnesses must decide the complete row-and-input predicate.
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
const ROLLED_BACK: &str = "mini.m.Report/outcome/rolled-back";
const FINISHED: &str = "mini.m.Report/outcome/finished";
const POLICY_CASES: [&str; 4] = [
    PROMOTED,
    ROLLED_BACK,
    "mini.m.Run/transition/promote/by/mini.m.Report/promoted",
    "mini.m.Run/transition/rollback/by/mini.m.Report/rolled-back",
];

fn ir() -> EssIr {
    model_ir(MODEL)
}

fn model_ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
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
    for id in POLICY_CASES {
        scenario(&synthesis, id);
    }
    scenario(&synthesis, FINISHED);
}

/// The reports and policies reached through the actual captured source and subject identities.
fn policy_reports(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    let mut targets = BTreeMap::new();
    let mut runs = BTreeMap::new();
    let mut pending = None;
    let mut reports = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.AddTarget" =>
            {
                pending = Some(input.clone());
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Target" => {
                targets.insert(
                    instance.clone(),
                    pending.take().expect("target created first"),
                );
            }
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.CreateRun" =>
            {
                let ScenarioValue::Instance { instance } = &input["target_id"] else {
                    panic!("source identity was not captured: {input:?}");
                };
                pending = Some(
                    targets
                        .get(instance)
                        .expect("named source created first")
                        .clone(),
                );
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "mini.m.Run" => {
                runs.insert(instance.clone(), pending.take().expect("run created first"));
            }
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "mini.m.Report" =>
            {
                let ScenarioValue::Instance { instance } = &input["run_id"] else {
                    // Unknown-instance refusal scenarios intentionally name no arranged run.
                    continue;
                };
                let mut report = runs.get(instance).expect("named run created first").clone();
                report.insert("result".to_owned(), input["result"].clone());
                reports.push(report);
            }
            _ => {}
        }
    }
    reports
}

/// Both guarded branches and their transitions use the policy of the source actually named.
#[test]
fn issue_307_the_promoted_run_is_created_from_a_target_holding_true() {
    let synthesis = synthesize(&ir());
    for id in POLICY_CASES {
        let reports = policy_reports(scenario(&synthesis, id));
        let (flag, result) = if id.contains("rolled-back") {
            ("automatic_rollback", "Failed")
        } else {
            ("auto_promote", "Healthy")
        };
        assert!(
            reports.iter().any(|report| {
                report.get(flag) == Some(&ScenarioValue::literal(Node::Bool(true)))
                    && report.get("result")
                        == Some(&ScenarioValue::literal(Node::Text(result.to_owned())))
            }),
            "{id}: {reports:?}"
        );
    }
}

#[test]
fn issue_307_optional_policy_witnesses_keep_true_false_and_absent_distinct() {
    let synthesis = synthesize(&ir());
    let reports: Vec<_> = synthesis
        .suite
        .scenarios
        .values()
        .flat_map(policy_reports)
        .collect();
    for flag in ["auto_promote", "automatic_rollback"] {
        for value in [Node::Null, Node::Bool(false), Node::Bool(true)] {
            assert!(
                reports.iter().any(|report| {
                    report
                        .get(flag)
                        .cloned()
                        .unwrap_or_else(|| ScenarioValue::literal(Node::Null))
                        == ScenarioValue::literal(value.clone())
                }),
                "{flag} never holds {value:?}: {reports:#?}"
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Stores no flag on the run, so a report never promotes.
    CopiesNothing,
    DropsPromote,
    DropsRollback,
    SwapsPolicies,
    IgnoresPromotePolicy,
    IgnoresRollbackPolicy,
    AbsentPromoteEnabled,
    AbsentRollbackEnabled,
    IgnoresResult,
    OmitsTransition,
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
        let flag = run.get("auto_promote") == Some(&Node::Bool(true))
            || self.mutant == Mutant::IgnoresPromotePolicy
            || (self.mutant == Mutant::AbsentPromoteEnabled
                && run.get("auto_promote") == Some(&Node::Null));
        let rollback = run.get("automatic_rollback") == Some(&Node::Bool(true))
            || self.mutant == Mutant::IgnoresRollbackPolicy
            || (self.mutant == Mutant::AbsentRollbackEnabled
                && run.get("automatic_rollback") == Some(&Node::Null));
        let promote = (healthy || self.mutant == Mutant::IgnoresResult) && flag;
        let (state, name, emitted) = if promote {
            ("Promoted", "promoted", "mini.m.Promoted")
        } else if !healthy && rollback {
            ("RolledBack", "rolled-back", "mini.m.RolledBack")
        } else {
            ("Done", "finished", "mini.m.Finished")
        };
        if self.mutant != Mutant::OmitsTransition {
            run.insert("state".to_owned(), Node::Text(state.to_owned()));
        }
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
                    (
                        "automatic_rollback".to_owned(),
                        input
                            .get("automatic_rollback")
                            .cloned()
                            .unwrap_or(Node::Null),
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
                let copy = |field: &str| {
                    if self.mutant == Mutant::CopiesNothing
                        || (self.mutant == Mutant::DropsPromote && field == "auto_promote")
                        || (self.mutant == Mutant::DropsRollback && field == "automatic_rollback")
                    {
                        return Node::Null;
                    }
                    let field = if self.mutant == Mutant::SwapsPolicies {
                        if field == "auto_promote" {
                            "automatic_rollback"
                        } else {
                            "auto_promote"
                        }
                    } else {
                        field
                    };
                    self.targets
                        .borrow()
                        .iter()
                        .find(|row| row["target_id"] == target)
                        .map_or(Node::Null, |row| row[field].clone())
                };
                self.runs.borrow_mut().push(Row::from([
                    ("run_id".to_owned(), id.clone()),
                    ("target_id".to_owned(), target.clone()),
                    ("auto_promote".to_owned(), copy("auto_promote")),
                    ("automatic_rollback".to_owned(), copy("automatic_rollback")),
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
            "mini.m.RunsForTarget" => self
                .runs
                .borrow()
                .iter()
                .filter(|row| row.get("target_id") == request.params.get("target"))
                .cloned()
                .collect(),
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
    model_statuses(MODEL, mutant)
}

fn model_statuses(text: &str, mutant: Mutant) -> BTreeMap<String, Status> {
    let suite = synthesize(&model_ir(text)).suite;
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

#[test]
fn copied_field_guard_and_view_share_arrangement() {
    let model = format!(
        "{}{}",
        MODEL.replace(
            "target_id: input.target_id",
            "target_id: {related: {via: input.target_id, field: target_id}}"
        ),
        r"
  - name: mini.m.RunsForTarget
    source: mini.m.Run
    consistency: read_your_writes
    params: [{name: target, type: mini.m.TargetId}]
    filter: target_id == param.target
    fields:
      - {name: run_id, type: mini.m.RunId}
      - {name: target_id, type: mini.m.TargetId}
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
      - {name: state, type: mini.m.Run.State}
"
    );
    let synthesis = synthesize(&model_ir(&model));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    for id in POLICY_CASES {
        let scenario = scenario(&synthesis, id);
        assert_ne!(policy_reports(scenario).len(), 0);
        assert!(scenario.steps.iter().any(|step| matches!(step,
            ScenarioStep::QueryView { view, params } if view.to_string() == "mini.m.RunsForTarget"
                && matches!(params.get("target"), Some(ScenarioValue::Instance { .. }))
        )), "{id}: {scenario:?}");
    }
    let statuses = model_statuses(&model, Mutant::None);
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:?}"
    );
}

/// A correct hand-written target passes every scenario of the suite.
#[test]
fn issue_307_a_correct_target_passes_the_suite() {
    let statuses = statuses(Mutant::None);
    for id in POLICY_CASES {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "{statuses:#?}");
    }
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

#[test]
fn issue_307_each_policy_copy_and_branch_has_a_decisive_mutant() {
    let mut survivors = Vec::new();
    for mutant in [
        Mutant::DropsPromote,
        Mutant::DropsRollback,
        Mutant::SwapsPolicies,
        Mutant::IgnoresPromotePolicy,
        Mutant::IgnoresRollbackPolicy,
        Mutant::AbsentPromoteEnabled,
        Mutant::AbsentRollbackEnabled,
        Mutant::IgnoresResult,
        Mutant::OmitsTransition,
    ] {
        let statuses = statuses(mutant);
        if !statuses
            .iter()
            .any(|(id, status)| id.contains("Report") && *status == Status::Failed)
        {
            survivors.push(mutant);
        }
    }
    assert!(
        survivors.is_empty(),
        "policy mutants survived: {survivors:?}"
    );
}

/// The native interpreter executes every branch that reads a copied related-source field.
#[test]
fn issue_307_the_interpreter_executes_related_sources() {
    let model = ir();
    let result = synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let expected = [PROMOTED, ROLLED_BACK, FINISHED];
    let selected: Vec<_> = report
        .scenarios
        .iter()
        .filter(|run| expected.contains(&run.scenario.to_string().as_str()))
        .collect();
    assert!(
        !selected.is_empty(),
        "no copied-field outcomes: {report:#?}"
    );
    assert_eq!(selected.len(), expected.len(), "{selected:#?}");
    for id in expected {
        assert!(
            selected
                .iter()
                .any(|run| run.scenario.to_string() == id && run.status == Status::Passed),
            "{id}: {selected:#?}"
        );
    }
    assert!(
        selected.iter().all(|run| run.status == Status::Passed),
        "{selected:#?}"
    );
    let failed: Vec<_> = report
        .scenarios
        .iter()
        .filter(|run| run.status != Status::Passed)
        .collect();
    assert!(
        failed.is_empty(),
        "native interpreter failures: {failed:#?}"
    );
}
