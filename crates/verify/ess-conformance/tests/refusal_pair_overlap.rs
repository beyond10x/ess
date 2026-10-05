//! Two input refusals whose guards overlap: the first declared answers, and the suite says so
//! apart from the first refusal's own witness (beyond10x/ess#455).
//!
//! `invalid-code: code == ""` is declared before `pause-refused: pause == true`. The first
//! refusal's primary send lies outside the second's guard where some input does, and the overlap
//! `{code: "", pause: true}` is sent once more, separately, requiring `invalid-code`. A target that
//! checks `pause` first then fails only that send; a target that never refuses a blank code fails
//! the primary one.
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

const MODEL: &str = include_str!("fixtures/refusal-pair-overlap.yaml");

const INVALID_CODE: &str = "demo.hold.Place/outcome/invalid-code";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("refusal-pair-overlap.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// A literal input value as text, for comparison.
fn literal(input: &BTreeMap<String, ScenarioValue>, field: &str) -> String {
    match input.get(field).and_then(ScenarioValue::as_literal) {
        Some(Node::Text(text)) => format!("{text:?}"),
        Some(Node::Bool(value)) => value.to_string(),
        other => panic!("a literal {field}, got {other:?}"),
    }
}

/// Each `Place` the scenario sends: its `code`, its `pause`, and the outcome the next step requires.
fn sends(scenario: &ConformanceScenario) -> Vec<(String, String, Option<String>)> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let required = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        out.push((literal(input, "code"), literal(input, "pause"), required));
    }
    out
}

fn send(code: &str, pause: bool, required: &str) -> (String, String, Option<String>) {
    (
        format!("{code:?}"),
        pause.to_string(),
        Some(required.to_owned()),
    )
}

/// The suite with `invalid-code`'s scenario cut to the steps of one of its sends: `0` the primary
/// send, `1` the next.
fn only_send(suite: &ConformanceSuite, nth: usize) -> ConformanceSuite {
    let mut cut = suite.clone();
    let (_, scenario) = cut
        .scenarios
        .iter_mut()
        .find(|(key, _)| key.to_string() == INVALID_CODE)
        .expect("the scenario is filed");
    let starts: Vec<usize> = scenario
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .map(|(at, _)| at)
        .collect();
    let end = starts.get(nth + 1).copied().unwrap_or(scenario.steps.len());
    scenario.steps = scenario.steps[starts[nth]..end].to_vec();
    cut
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Change {
    /// The model as declared.
    None,
    /// Checks `pause` before `code`: the declared order swapped.
    PauseFirst,
    /// Never refuses a blank code: it reads the code as if one were given.
    NeverBlank,
}

struct Target {
    inner: Interpreted,
    change: Change,
}

impl Target {
    fn new(change: Change) -> Self {
        Self {
            inner: Interpreted::for_model(ir(MODEL)),
            change,
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
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let blank = request.input.get("code") == Some(&Node::Text(String::new()));
        let pause = request.input.get("pause") == Some(&Node::Bool(true));
        if self.change == Change::NeverBlank && blank {
            request
                .input
                .insert("code".to_owned(), Node::Text("code".to_owned()));
        }
        let command = request.command.clone();
        let mut result = self.inner.execute_command(request)?;
        if self.change == Change::PauseFirst && pause {
            // The model refuses every `pause: true` input and changes nothing, so only the answer
            // differs.
            result.outcome = Some(OutcomeRef::new(
                command,
                OutcomeName::new("pause-refused").expect("a name"),
            ));
            result.error = Some(DeclaredErrorValue::new(
                "demo.hold.PauseRefused"
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
fn earlier_refusal_witness_avoids_later_sibling() {
    let result = synthesis(MODEL);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let sent = sends(scenario(&result.suite, INVALID_CODE));
    assert_eq!(
        sent.first(),
        Some(&send("", false, "invalid-code")),
        "the primary send is a blank code outside `pause-refused`'s guard: {sent:#?}"
    );
}

#[test]
fn refusal_pair_overlap_sent_separately() {
    let result = synthesis(MODEL);
    let sent = sends(scenario(&result.suite, INVALID_CODE));
    assert_eq!(
        sent,
        vec![
            send("", false, "invalid-code"),
            send("", true, "invalid-code")
        ],
        "the overlap is sent once more, after the primary send, requiring the first declared"
    );
}

#[test]
fn swapped_order_target_fails_only_overlap() {
    let suite = synthesis(MODEL).suite;
    let honest = run(&suite, &Target::new(Change::None));
    assert_eq!(not_passed(&honest), Vec::<&str>::new(), "{honest:#?}");

    let swapped = Target::new(Change::PauseFirst);
    assert_eq!(
        not_passed(&run(&suite, &swapped)),
        vec![INVALID_CODE],
        "checking `pause` first fails the first refusal's scenario and nothing else"
    );
    assert_eq!(
        not_passed(&run(&only_send(&suite, 0), &swapped)),
        Vec::<&str>::new(),
        "its primary send passes"
    );
    assert_eq!(
        not_passed(&run(&only_send(&suite, 1), &swapped)),
        vec![INVALID_CODE],
        "its overlap send fails"
    );

    let never_blank = Target::new(Change::NeverBlank);
    assert!(
        not_passed(&run(&only_send(&suite, 0), &never_blank)).contains(&INVALID_CODE),
        "a target never refusing a blank code fails the primary send"
    );
}

#[test]
fn inseparable_refusals_keep_overlap_witness() {
    // Every blank code is also not "x": no input separates the two guards.
    let inseparable = MODEL.replace("when: pause == true", "when: code != \"x\"");
    assert_ne!(inseparable, MODEL);
    let result = synthesis(&inseparable);
    let sent: Vec<_> = sends(scenario(&result.suite, INVALID_CODE))
        .into_iter()
        .map(|(code, _, required)| (code, required))
        .collect();
    assert_eq!(
        sent,
        vec![("\"\"".to_owned(), Some("invalid-code".to_owned()))],
        "the one witness is the overlap, sent once"
    );
}

#[test]
fn no_overlap_models_bytes_unchanged() {
    // Disjoint guards: the first refusal's witness already lies outside the second's.
    let disjoint = MODEL.replace(
        "when: pause == true",
        "when: {all: [pause == true, code != \"\"]}",
    );
    assert_ne!(disjoint, MODEL);
    let result = synthesis(&disjoint);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let sent = sends(scenario(&result.suite, INVALID_CODE));
    assert_eq!(
        sent.len(),
        1,
        "no overlap, so one send, as before: {sent:#?}"
    );
    assert_eq!(sent[0].0, "\"\"", "{sent:#?}");
    assert_eq!(sent[0].2.as_deref(), Some("invalid-code"), "{sent:#?}");
}

/// The `### Two input refusals whose guards overlap` subsection of `## Which branch answers`.
fn subsection(page: &str) -> Option<&str> {
    let section = page.find("\n## Which branch answers\n")? + 1;
    let rest = &page[section..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |at| at + 4);
    let section = &rest[..end];
    let start = section.find("\n### Two input refusals whose guards overlap\n")? + 1;
    let tail = &section[start..];
    let stop = tail[4..].find("\n### ").map_or(tail.len(), |at| at + 5);
    Some(&tail[..stop])
}

#[test]
fn refusal_pair_paragraph_states_declaration_order() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/guards-and-predicates.md");
    let page = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let text = subsection(&page).unwrap_or_else(|| {
        panic!(
            "`## Which branch answers` has the heading `### Two input refusals whose guards overlap`"
        )
    });
    for phrase in [
        "the first declared answers",
        "make the guards disjoint",
        "`all:`",
        "sent once more",
    ] {
        assert!(
            text.contains(phrase),
            "the subsection says {phrase:?}:\n{text}"
        );
    }
}
