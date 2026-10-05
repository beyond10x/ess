//! Adversary pass 1 on beyond10x/ess#454 and #455.
//!
//! The model is #454's own fixture, the idiom its guide section recommends (`invalid-code` guarded
//! by `when_subject: {predicate: state == Active}`), with two plain input refusals added to the same
//! command: `reason-required: reason == ""` declared before `pause-refused: pause == true`. Their
//! guards overlap at `{reason: "", pause: true}`, where the first declared answers (#455). The guide
//! (`### Two input refusals whose guards overlap`) and `fields-and-invariants.md` say the suite
//! sends that overlap apart from the first refusal's own witness, so a target checking `pause`
//! first fails. Beside a held-state branch the refusals are sent on an arranged row, and that path
//! sends no refusal-pair overlap.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ErrorRef, OutcomeRef};
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, DeclaredErrorValue, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InvocationObservationRequest, ObservedEvent,
    ObservedInvocation, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner,
    ScenarioStep, ScenarioValue,
};
use ess_domain::command::OutcomeName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const FIXTURE: &str = include_str!("fixtures/held-state-input-refusal.yaml");

const REASON_REQUIRED: &str = "demo.session.Pause/outcome/reason-required";

/// The fixture with `reason` and `pause` inputs and the two plain refusals over them.
fn model() -> String {
    let model = FIXTURE
        .replace(
            "      - {name: code, type: String}\n    outcomes:\n",
            "      - {name: code, type: String}\n      - {name: reason, type: String}\n      \
             - {name: pause, type: Boolean}\n    outcomes:\n",
        )
        .replace(
            "      - {name: no-session, unknown_instance: true, error: demo.session.NoSession}\n",
            "      - {name: no-session, unknown_instance: true, error: demo.session.NoSession}\n      \
             - {name: reason-required, when: reason == \"\", error: demo.session.ReasonRequired}\n      \
             - {name: pause-refused, when: pause == true, error: demo.session.PauseRefused}\n",
        )
        .replace(
            "  - {name: demo.session.NotActive, fields: []}\n",
            "  - {name: demo.session.NotActive, fields: []}\n  \
             - {name: demo.session.PauseRefused, fields: []}\n  \
             - {name: demo.session.ReasonRequired, fields: []}\n",
        );
    assert_eq!(model.matches("pause-refused").count(), 1, "{model}");
    assert_eq!(
        model.matches("name: pause, type: Boolean").count(),
        1,
        "{model}"
    );
    assert_eq!(
        model.matches("demo.session.ReasonRequired").count(),
        2,
        "{model}"
    );
    model
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary-454-455.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(&model()))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn text(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match input.get(field).and_then(ScenarioValue::as_literal) {
        Some(Node::Text(text)) => Some(text.clone()),
        Some(Node::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

/// One `Pause` send: literal identity, `reason`, `pause`, and the outcome required after it.
type Pause = (bool, Option<String>, Option<String>, Option<String>);

/// Each `Pause` the scenario sends: whether its identity is a literal (no arranged record), its
/// `reason`, its `pause`, and the outcome the step after it requires.
fn pauses(scenario: &ConformanceScenario) -> Vec<Pause> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.session.Pause" {
            continue;
        }
        let literal_identity = input
            .get("session_id")
            .is_some_and(|value| value.as_literal().is_some());
        let required = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        out.push((
            literal_identity,
            text(input, "reason"),
            text(input, "pause"),
            required,
        ));
    }
    out
}

fn run(suite: &ConformanceSuite, target: &impl ConformanceTarget) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The interpreted model, or that model checking `pause` before `reason`: the declared order of
/// the two plain refusals swapped.
struct Target {
    inner: Interpreted,
    pause_first: bool,
}

impl ConformanceTarget for Target {
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
        let pause = request.command.to_string() == "demo.session.Pause"
            && request.input.get("pause") == Some(&Node::Bool(true));
        let command = request.command.clone();
        let mut result = self.inner.execute_command(request)?;
        let took_reason = result
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.outcome.to_string() == "reason-required");
        if self.pause_first && pause && took_reason {
            // Both refusals change nothing, so only the answer differs.
            result.outcome = Some(OutcomeRef::new(
                command,
                OutcomeName::new("pause-refused").expect("a name"),
            ));
            result.error = Some(DeclaredErrorValue::new(
                "demo.session.PauseRefused"
                    .parse::<ErrorRef>()
                    .expect("an error"),
            ));
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
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
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
}

/// #455's overlap send, on a command that also carries #454's held-state refusal.
#[test]
fn adv1_refusal_pair_overlap_is_sent_beside_a_held_state_refusal() {
    let result = synthesis();
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let sent = pauses(scenario(&result.suite, REASON_REQUIRED));
    assert!(
        sent.iter().any(|(_, reason, pause, required)| {
            reason.as_deref() == Some("")
                && pause.as_deref() == Some("true")
                && required.as_deref() == Some("reason-required")
        }),
        "`{{reason: \"\", pause: true}}` is sent requiring `reason-required`, the first declared: \
         {sent:#?}"
    );
}

/// A target taking the two plain refusals in the other order fails `reason-required`'s scenario.
#[test]
fn adv1_pause_first_target_fails_beside_a_held_state_refusal() {
    let suite = synthesis().suite;
    let honest = run(
        &suite,
        &Target {
            inner: Interpreted::for_model(ir(&model())),
            pause_first: false,
        },
    );
    assert_eq!(not_passed(&honest), Vec::<&str>::new(), "{honest:#?}");
    let swapped = run(
        &suite,
        &Target {
            inner: Interpreted::for_model(ir(&model())),
            pause_first: true,
        },
    );
    assert!(
        not_passed(&swapped).contains(&REASON_REQUIRED),
        "checking `pause` before `reason` fails `reason-required`: {swapped:#?}"
    );
}

/// Step 2 before step 3: a plain input refusal answers an identity nothing stores. Sent so on a
/// command with no stored-row branch (`plain_input_refusal_keeps_step_two`); here, beside the
/// held-state refusal the guide recommends, never.
#[test]
fn adv1_plain_refusal_is_sent_for_an_unknown_identity_beside_a_held_state_refusal() {
    let result = synthesis();
    let sent = pauses(scenario(&result.suite, REASON_REQUIRED));
    assert!(
        sent.iter().any(|(literal, reason, _, required)| {
            *literal
                && reason.as_deref() == Some("")
                && required.as_deref() == Some("reason-required")
        }),
        "a blank reason is sent for an identity nothing stores and requires `reason-required`: \
         {sent:#?}"
    );
}
