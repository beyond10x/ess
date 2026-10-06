//! An input refusal that holds only for a live record is guarded by the held state as well
//! (beyond10x/ess#454): `when_subject: {predicate: state == Active}` beside its `when:` makes it a
//! held-state branch, answered after existence and after `wrong_state`.
//!
//! The synthesized suite witnesses that order: the unknown-identity scenario and the wrong-state
//! scenario each also send an input the held-state refusal claims, and require the existence or
//! wrong-state answer. A target that checks the input before it looks the record up fails them; the
//! model interpreter, which answers in the declared order, passes every scenario.
use std::collections::BTreeMap;
use std::path::Path;

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

const MODEL: &str = include_str!("fixtures/held-state-input-refusal.yaml");

/// The line that makes `invalid-code` a held-state branch; without it the refusal is a plain input
/// refusal, answered before existence.
const HELD: &str = "        when_subject: {predicate: state == Active}\n";

const NO_SESSION: &str = "demo.session.Pause/outcome/no-session";
const INVALID_CODE: &str = "demo.session.Pause/outcome/invalid-code";
const PAUSED_REFUSES: &str = "demo.session.Session/state/Paused/refuses/demo.session.Pause";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("held-state-input-refusal.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result.refusals
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Each `Pause` the scenario sends: whether its identity is a literal (no arranged record), the
/// code it carries, and the outcome the step after it requires.
fn pauses(scenario: &ConformanceScenario) -> Vec<(bool, String, Option<String>)> {
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
        let code = match input.get("code").and_then(ScenarioValue::as_literal) {
            Some(Node::Text(code)) => code.clone(),
            other => panic!("a literal text code, got {other:?}"),
        };
        let required = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        out.push((literal_identity, code, required));
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

/// The interpreted model, or that model changed to check the code before it looks the record up.
struct Target {
    inner: Interpreted,
    code_first: bool,
}

impl Target {
    fn new(model: &str, code_first: bool) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            code_first,
        }
    }
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
        let blank = self.code_first
            && request.command.to_string() == "demo.session.Pause"
            && request.input.get("code") == Some(&Node::Text(String::new()));
        let command = request.command.clone();
        let mut result = self.inner.execute_command(request)?;
        if blank {
            // The model changes nothing for a blank code in any state, so only the answer differs:
            // the code is refused before the record is looked up.
            result.outcome = Some(OutcomeRef::new(
                command,
                OutcomeName::new("invalid-code").expect("a name"),
            ));
            result.error = Some(DeclaredErrorValue::new(
                "demo.session.InvalidCode"
                    .parse::<ErrorRef>()
                    .expect("an error"),
            ));
            result.direct_events.clear();
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

#[test]
fn held_state_input_refusal_answers_after_existence() {
    let result = synthesis(MODEL);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let sent = pauses(scenario(&result, NO_SESSION));
    assert!(
        sent.contains(&(true, String::new(), Some("no-session".to_owned()))),
        "an unknown identity is also sent the blank code the held-state refusal claims, and \
         still takes `no-session`: {sent:#?}"
    );
    assert!(
        sent.iter()
            .all(|(literal, _, required)| *literal && required.as_deref() == Some("no-session")),
        "every send is for an unknown identity and requires `no-session`: {sent:#?}"
    );
}

#[test]
fn held_state_input_refusal_answers_after_wrong_state() {
    let result = synthesis(MODEL);
    let sent = pauses(scenario(&result, PAUSED_REFUSES));
    assert!(
        sent.contains(&(false, String::new(), Some("not-active".to_owned()))),
        "the paused record is also sent the blank code and takes `not-active`: {sent:#?}"
    );
    let invalid = pauses(scenario(&result, INVALID_CODE));
    assert!(
        invalid.contains(&(false, String::new(), Some("invalid-code".to_owned()))),
        "the refusal's own witness stays on an active record: {invalid:#?}"
    );
}

#[test]
fn code_first_target_fails() {
    let result = synthesis(MODEL);
    let honest = run(&result.suite, &Target::new(MODEL, false));
    assert_eq!(
        not_passed(&honest),
        Vec::<&str>::new(),
        "the interpreted model answers in the declared order and passes: {honest:#?}"
    );
    let code_first = run(&result.suite, &Target::new(MODEL, true));
    let failed = not_passed(&code_first);
    assert!(
        failed.contains(&NO_SESSION),
        "checking the code before the lookup fails `no-session`: {code_first:#?}"
    );
    assert!(
        failed.contains(&PAUSED_REFUSES),
        "and the paused record's wrong-state scenario: {code_first:#?}"
    );
}

#[test]
fn plain_input_refusal_keeps_step_two() {
    let plain = MODEL.replace(HELD, "");
    assert_ne!(plain, MODEL, "the held-state guard is removed");
    let result = synthesis(&plain);
    let own = pauses(scenario(&result, INVALID_CODE));
    assert_eq!(
        own.first(),
        Some(&(true, String::new(), Some("invalid-code".to_owned()))),
        "a refusal guarded by `when:` alone is sent for an unknown identity first: {own:#?}"
    );
    for id in [NO_SESSION, PAUSED_REFUSES] {
        // The wrong-state scenario's arrangement pauses its record first; that send is not the
        // witness.
        let sent: Vec<_> = pauses(scenario(&result, id))
            .into_iter()
            .filter(|(_, _, required)| required.as_deref() != Some("paused"))
            .collect();
        assert_eq!(
            sent.len(),
            1,
            "{id} sends one input, as it did before the held-state overlap: {sent:#?}"
        );
        assert_ne!(
            sent[0].1, "",
            "{id} is not sent the refused code: {sent:#?}"
        );
    }
}

/// The `## Which branch answers` section of the guide, up to the next level-two heading.
fn section<'p>(page: &'p str, heading: &str) -> Option<&'p str> {
    let start = page.find(&format!("\n{heading}\n"))? + 1;
    let rest = &page[start + heading.len()..];
    let end = rest.find("\n## ").map_or(rest.len(), |at| at + 1);
    Some(&page[start..start + heading.len() + end])
}

/// Every fenced YAML block's body in `text`, in order.
fn yaml_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut open: Option<String> = None;
    for line in text.lines() {
        match &mut open {
            None if line.trim_start().starts_with("```yaml") => open = Some(String::new()),
            Some(body) if line.trim_start() == "```" => {
                blocks.push(std::mem::take(body));
                open = None;
            }
            Some(body) => {
                body.push_str(line);
                body.push('\n');
            }
            None => {}
        }
    }
    blocks
}

#[test]
fn which_branch_answers_section_states_the_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/guards-and-predicates.md");
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let section = section(&page, "## Which branch answers")
        .unwrap_or_else(|| panic!("the guide has the heading `## Which branch answers`"));
    for phrase in [
        "input refusals answer first",
        "unknown instance",
        "`wrong_state`",
        "`when_subject: {predicate: state ==",
        "after existence",
    ] {
        assert!(
            section.contains(phrase),
            "`## Which branch answers` says {phrase:?}:\n{section}"
        );
    }
    assert!(
        yaml_blocks(section).iter().any(|block| block == MODEL),
        "`## Which branch answers` shows the fixture \
         `tests/fixtures/held-state-input-refusal.yaml`, byte for byte:\n{section}"
    );
}
