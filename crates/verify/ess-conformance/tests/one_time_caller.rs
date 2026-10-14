//! Disclosure scenarios of a one-time outcome that records who issued it (beyond10x/ess#550).
//!
//! The outcome `keys.api.IssueKey/issued` sets `issued_by: {caller: user_id}` and protects its
//! `secret`. Every disclosure scenario sends the command, so each must send it authenticated as a
//! caller carrying `user_id`, as the outcome's own scenario does; without one the interpreted
//! target has no value for the read and the scenario comes back unsupported.
use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::{
    ConformanceTarget, EntitySetupRequest, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RecordedCommandCompletion, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_conformance::{report::Status, ScenarioId, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const WITH_CALLER: &str = include_str!("fixtures/one-time-caller/with-caller.yaml");
const WITHOUT_CALLER: &str = include_str!("fixtures/one-time-caller/without-caller.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn run(ir: &EssIr, target_ir: EssIr) -> ess_conformance::report::ConformanceReport {
    let synthesis = ess_conformance::synthesize(ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = ess_conformance::interpret::Interpreted::for_model(target_ir);
    ess_conformance::Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
}

fn statuses(report: &ess_conformance::report::ConformanceReport) -> Vec<(String, Status)> {
    report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

#[test]
fn the_disclosure_scenarios_bind_the_caller_the_outcome_reads() {
    let ir = model(WITH_CALLER);
    let synthesis = ess_conformance::synthesize(&ir);
    let disclosures: Vec<_> = synthesis
        .suite
        .scenarios
        .iter()
        .filter(|(id, _)| matches!(id, ScenarioId::Disclosure { .. }))
        .collect();
    assert_eq!(
        disclosures.len(),
        4,
        "{:?}",
        synthesis.suite.scenarios.keys()
    );
    for (id, scenario) in disclosures {
        let commands: Vec<_> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand { caller, .. }
                | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } => Some(caller),
                _ => None,
            })
            .collect();
        assert!(!commands.is_empty(), "{id} sends no command");
        for caller in commands {
            assert!(
                caller.contains_key("user_id"),
                "{id} sends a command without the caller attribute the outcome reads"
            );
        }
    }
    let report = run(&ir, ir.clone());
    let statuses = statuses(&report);
    assert_eq!(statuses.len(), 5, "{statuses:?}");
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:?}\n{}",
        serde_json::to_string_pretty(&report.scenarios).unwrap()
    );
}

#[test]
fn a_model_reading_no_caller_value_passes_its_disclosure_scenarios_as_before() {
    let ir = model(WITHOUT_CALLER);
    // No command reads the caller, so no disclosure step is sent as one: the suite keeps the bytes
    // it was synthesized with before the disclosure cells learned to bind a caller.
    let synthesis = ess_conformance::synthesize(&ir);
    for (id, scenario) in &synthesis.suite.scenarios {
        if !matches!(id, ScenarioId::Disclosure { .. }) {
            continue;
        }
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { caller, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } = step
            {
                assert!(caller.is_empty(), "{id} gained a caller");
            }
        }
    }
    let statuses = statuses(&run(&ir, ir.clone()));
    assert_eq!(statuses.len(), 5, "{statuses:?}");
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:?}"
    );
}

/// How the [`Mutant`] departs from the interpreted target.
#[derive(Clone, Copy, Debug)]
enum Defect {
    /// Ignores the caller each command is sent as, so it has no value to record as `issued_by`.
    DropsTheCaller,
    /// Answers every issuance after the first in a scenario with the first one's secret.
    DisclosesTwice,
}

/// The interpreted target, wrong in one way.
struct Mutant {
    inner: Interpreted,
    defect: Defect,
    first: RefCell<Option<Node>>,
}

impl Mutant {
    fn recorded(&self, mut request: SemanticCommandRequest) -> RecordedCommandCompletion {
        if matches!(self.defect, Defect::DropsTheCaller) {
            request.caller = None;
        }
        let mut completion = self.inner.execute_command_recorded(request);
        if let (Defect::DisclosesTwice, Ok(result)) = (self.defect, &mut completion.answer) {
            if let Some(secret) = result
                .response
                .as_mut()
                .and_then(|response| response.get_mut("secret"))
            {
                let mut first = self.first.borrow_mut();
                match &*first {
                    Some(earlier) => secret.clone_from(earlier),
                    None => *first = Some(secret.clone()),
                }
            }
        }
        completion
    }
}

impl ConformanceTarget for Mutant {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.first.borrow_mut() = None;
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.recorded(request).answer
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        self.recorded(request)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

#[test]
fn a_target_dropping_the_caller_or_disclosing_twice_fails_a_disclosure_scenario() {
    let ir = model(WITH_CALLER);
    let synthesis = ess_conformance::synthesize(&ir);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let mut verdicts = BTreeMap::new();
    for defect in [Defect::DropsTheCaller, Defect::DisclosesTwice] {
        let target = Mutant {
            inner: Interpreted::for_model(ir.clone()),
            defect,
            first: RefCell::new(None),
        };
        let report = ess_conformance::Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &target)
            .into_report();
        let statuses = statuses(&report);
        let disclosures: Vec<_> = statuses
            .iter()
            .filter(|(id, _)| id.contains("/disclosure/"))
            .collect();
        assert_eq!(disclosures.len(), 4, "{defect:?}: {statuses:?}");
        assert!(
            disclosures
                .iter()
                .any(|(_, status)| *status != Status::Passed),
            "{defect:?} passed every disclosure scenario: {statuses:?}"
        );
        verdicts.insert(format!("{defect:?}"), statuses);
    }
    // Disclosing the secret twice is a failure of the declared rule, not a target that could
    // not answer: a disclosure scenario that sends the command twice fails.
    assert!(
        verdicts["DisclosesTwice"]
            .iter()
            .any(|(id, status)| id.contains("/disclosure/") && *status == Status::Failed),
        "{verdicts:?}"
    );
    // With no caller to record, no disclosure scenario passes.
    assert!(
        verdicts["DropsTheCaller"]
            .iter()
            .filter(|(id, _)| id.contains("/disclosure/"))
            .all(|(_, status)| *status != Status::Passed),
        "{verdicts:?}"
    );
}
