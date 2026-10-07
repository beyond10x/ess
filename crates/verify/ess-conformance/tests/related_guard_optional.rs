//! An Optional command input may name the row a `when_related:` guard reads (beyond10x/ess#304,
//! `ess/22`). Absence performs no lookup and selects no related branch. Presence retains the three
//! existing answers: missing row, predicate refusal, or acceptance.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-optional.yaml");
const COMMAND: &str = "demo.release.PublishRelease";
const NO_CANDIDATE: &str = "demo.release.PublishRelease/outcome/no-candidate";
const NOT_ACCEPTED: &str = "demo.release.PublishRelease/outcome/not-accepted";
const PUBLISHED: &str = "demo.release.PublishRelease/outcome/published";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}\n{MODEL}"));
    let spec = Specification::assemble([(Source::new("optional-related.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{MODEL}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir())
}

fn related_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(COMMAND))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// Every invocation of `PublishRelease`, paired with the outcome asserted for that invocation.
fn invocations(
    scenario: &ConformanceScenario,
) -> Vec<(BTreeMap<String, ess_conformance::ScenarioValue>, String)> {
    let mut pending = None;
    let mut found = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == COMMAND =>
            {
                assert!(pending.is_none(), "an invocation is asserted before the next one");
                pending = Some(input.clone());
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                if let Some(input) = pending.take() {
                    found.push((input, outcome.to_string()));
                }
            }
            ScenarioStep::ExecuteCommand { .. } => {
                assert!(
                    pending.is_none(),
                    "a PublishRelease invocation has no outcome assertion: {:?}",
                    scenario.steps
                );
            }
            _ => {}
        }
    }
    assert!(pending.is_none(), "the final invocation has an outcome assertion");
    found
}

#[test]
fn issue_304_absent_present_and_missing_are_each_witnessed() {
    let result = synthesis();
    assert!(related_refusals(&result).is_empty(), "{:#?}", result.refusals);
    for id in [NO_CANDIDATE, NOT_ACCEPTED, PUBLISHED] {
        assert!(
            result.suite.scenarios.keys().any(|key| key.to_string() == id),
            "{id} exists; refusals: {:#?}",
            result.refusals
        );
    }

    let witnessed: Vec<_> = result
        .suite
        .scenarios
        .values()
        .flat_map(invocations)
        .collect();
    assert!(
        witnessed.iter().any(|(input, outcome)| {
            !input.contains_key("candidate") && outcome == PUBLISHED
        }),
        "absence skips the lookup and selects the accepting branch: {witnessed:#?}"
    );
    assert!(
        witnessed.iter().any(|(input, outcome)| {
            matches!(
                input.get("candidate"),
                Some(ess_conformance::ScenarioValue::Instance { .. })
            ) && outcome == NOT_ACCEPTED
        }),
        "a present row in the wrong state is refused: {witnessed:#?}"
    );
    assert!(
        witnessed.iter().any(|(input, outcome)| {
            matches!(
                input.get("candidate"),
                Some(ess_conformance::ScenarioValue::Instance { .. })
            ) && outcome == PUBLISHED
        }),
        "a present row in the required state is accepted: {witnessed:#?}"
    );
    assert!(
        witnessed.iter().any(|(input, outcome)| {
            matches!(
                input.get("candidate"),
                Some(ess_conformance::ScenarioValue::Literal { .. })
            ) && outcome == NO_CANDIDATE
        }),
        "a present identity no row carries selects the missing-row branch: {witnessed:#?}"
    );
}

#[derive(Clone, Copy)]
enum Fault {
    /// Treat an omitted Optional identity as a present identity that names no row.
    AbsentAsMissing,
    /// Remove every present Optional identity before evaluating the guard.
    IgnorePresent,
}

struct Faulty {
    fault: Fault,
    inner: Interpreted,
}

impl Faulty {
    fn new(model: EssIr, fault: Fault) -> Self {
        Self {
            fault,
            inner: Interpreted::for_model(model),
        }
    }
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }

    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }

    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }

    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() == COMMAND {
            match self.fault {
                Fault::AbsentAsMissing if !request.input.contains_key("candidate") => {
                    request.input.insert(
                        "candidate".to_owned(),
                        Node::Text("00000000-0000-4000-8000-ffffffffffff".to_owned()),
                    );
                }
                Fault::IgnorePresent => {
                    request.input.remove("candidate");
                }
                Fault::AbsentAsMissing => {}
            }
        }
        self.inner.execute_command(request)
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

fn statuses<T: ConformanceTarget>(
    result: &Synthesis,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("PublishRelease"))
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_honest_baseline(result: &Synthesis) {
    let honest = statuses(result, &Interpreted::for_model(ir()));
    for id in [NO_CANDIDATE, NOT_ACCEPTED, PUBLISHED] {
        assert_eq!(
            honest.get(id),
            Some(&Status::Passed),
            "{id} executes and passes before a mutant is meaningful: {honest:#?}"
        );
    }
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "the declared interpreter passes before a mutant is meaningful: {honest:#?}"
    );
}

#[test]
fn issue_304_a_target_treating_absent_as_missing_fails() {
    let result = synthesis();
    assert_honest_baseline(&result);
    let faulty = statuses(&result, &Faulty::new(ir(), Fault::AbsentAsMissing));
    assert_ne!(
        faulty.get(PUBLISHED),
        Some(&Status::Passed),
        "the accepting scenario executes the absent segment: {faulty:#?}"
    );
}

#[test]
fn issue_304_a_target_ignoring_a_present_reference_fails() {
    let result = synthesis();
    assert_honest_baseline(&result);
    let faulty = statuses(&result, &Faulty::new(ir(), Fault::IgnorePresent));
    for id in [NO_CANDIDATE, NOT_ACCEPTED] {
        assert_ne!(
            faulty.get(id),
            Some(&Status::Passed),
            "{id} distinguishes a present reference from absence: {faulty:#?}"
        );
    }
}
