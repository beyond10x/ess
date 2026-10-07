//! Adversary pass 1 against beyond10x/ess#304 (`when_related` through an Optional input, ess/22).
//!
//! Each model variant below is a valid ess/22 document that differs from the unit's one fixture in
//! one respect the unit's tests do not exercise: declaration order, no `wrong_state`, an accepting
//! related branch, an input-guarded refusal beside the related guard, a creating command. For each,
//! synthesis must refuse nothing for the command, the declared interpreter must pass every
//! scenario of it, and a target that reads absence as a missing row must fail one.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-optional.yaml");
const PUBLISH: &str = "demo.release.PublishRelease";
const DRAFT: &str = "demo.release.DraftRelease";

const NO_CANDIDATE: &str = "      - name: no-candidate\n        when_related: {via: input.candidate, exists: false}\n        error: demo.release.NoCandidate\n";
const NOT_ACCEPTED: &str = "      - name: not-accepted\n        when_related: {via: input.candidate, predicate: state != Accepted}\n        error: demo.release.CandidateNotAccepted\n";
const WRONG_STATE: &str =
    "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n";
const PUBLISHED: &str = "      - name: published\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn publish_outcomes(body: &str) -> String {
    let original = format!("{NO_CANDIDATE}{NOT_ACCEPTED}{WRONG_STATE}{PUBLISHED}");
    replaced(MODEL, &original, body)
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("adversary-304.yaml"), raw)])
}

fn ir(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals_for(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| {
            format!(
                "{} [{}]: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

/// The same document with the reference required: the twin the unit's base already admitted.
fn required(text: &str) -> String {
    replaced(
        text,
        "type: Optional<demo.release.CandidateId>}",
        "type: demo.release.CandidateId}",
    )
}

/// The refusals synthesis makes for `command` in the required-input twin: those are the base's
/// own limits, not the Optional form's, and are subtracted from it.
fn twin_refusals(text: &str, command: &str) -> Vec<String> {
    let model = ir(&required(text));
    refusals_for(&ess_conformance::synthesize::synthesize(&model), command)
}

#[derive(Clone, Copy, Debug)]
enum Fault {
    AbsentAsMissing,
    IgnorePresent,
}

struct Faulty {
    command: &'static str,
    fault: Fault,
    inner: Interpreted,
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
        if request.command.to_string() == self.command {
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
    command: &str,
) -> BTreeMap<String, Status> {
    let short = command.rsplit('.').next().unwrap();
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains(short))
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// The whole check for one variant: no refusal, an honest pass, and both faults caught.
fn attack(label: &str, text: &str, command: &'static str) {
    let model = ir(text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let twin = twin_refusals(text, command);
    let refused: Vec<String> = refusals_for(&result, command)
        .into_iter()
        .filter(|refusal| !twin.contains(refusal))
        .collect();
    assert!(
        refused.is_empty(),
        "{label}: synthesis refuses scenarios of {command} that the required-input twin \
         synthesizes: {refused:#?}\n(twin refuses only: {twin:#?})"
    );
    let absent_sent = result.suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(
                step,
                ess_conformance::scenario::ScenarioStep::ExecuteCommand { command: sent, input, .. }
                    if sent.to_string() == command && !input.contains_key("candidate")
            )
        })
    });
    assert!(
        absent_sent,
        "{label}: no scenario sends {command} with the Optional reference absent"
    );
    let honest = statuses(&result, &Interpreted::for_model(model.clone()), command);
    assert!(!honest.is_empty(), "{label}: {command} has scenarios");
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "{label}: the declared interpreter disagrees with synthesis: {honest:#?}"
    );
    for fault in [Fault::AbsentAsMissing, Fault::IgnorePresent] {
        let faulty = statuses(
            &result,
            &Faulty {
                command,
                fault,
                inner: Interpreted::for_model(model.clone()),
            },
            command,
        );
        assert!(
            faulty.values().any(|status| *status != Status::Passed),
            "{label}: a target with {fault:?} passes every scenario of {command}: {faulty:#?}"
        );
    }
}

#[test]
fn adversary_304_wrong_state_declared_first_synthesizes_and_agrees() {
    attack(
        "wrong_state first",
        &publish_outcomes(&format!(
            "{WRONG_STATE}{NOT_ACCEPTED}{NO_CANDIDATE}{PUBLISHED}"
        )),
        PUBLISH,
    );
}

#[test]
fn adversary_304_without_wrong_state_synthesizes_and_agrees() {
    attack(
        "no wrong_state",
        &publish_outcomes(&format!("{NO_CANDIDATE}{NOT_ACCEPTED}{PUBLISHED}")),
        PUBLISH,
    );
}

#[test]
fn adversary_304_accepting_related_branch_with_refusing_default_synthesizes_and_agrees() {
    let accepting = "      - name: published\n        when_related: {via: input.candidate, predicate: state == Accepted}\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n";
    let default_refusal =
        "      - {name: not-accepted, error: demo.release.CandidateNotAccepted}\n";
    attack(
        "accepting related branch, refusing default",
        &publish_outcomes(&format!("{NO_CANDIDATE}{accepting}{default_refusal}")),
        PUBLISH,
    );
}

#[test]
fn adversary_304_input_guarded_refusal_beside_optional_via_synthesizes_and_agrees() {
    let held = "      - {name: held, when: hold == true, error: demo.release.Held}\n";
    let text = publish_outcomes(&format!(
        "{NO_CANDIDATE}{NOT_ACCEPTED}{held}{WRONG_STATE}{PUBLISHED}"
    ));
    let text = replaced(
        &text,
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n",
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n      - {name: hold, type: Boolean}\n",
    );
    let text = replaced(
        &text,
        "  - {name: demo.release.NoCandidate,",
        "  - {name: demo.release.Held, summary: The release is held back., fields: []}\n  - {name: demo.release.NoCandidate,",
    );
    attack("input-guarded refusal beside", &text, PUBLISH);
}

/// The #304 reproduction's shape: a creating command whose Optional input names a related row.
fn creating_variant() -> String {
    replaced(
        MODEL,
        "  - name: demo.release.DraftRelease\n    input: []\n    outcomes:\n      - name: drafted\n",
        "  - name: demo.release.DraftRelease\n    input:\n      - {name: candidate, type: Optional<demo.release.CandidateId>}\n    outcomes:\n      - name: no-candidate\n        when_related: {via: input.candidate, exists: false}\n        error: demo.release.NoCandidate\n      - name: not-accepted\n        when_related: {via: input.candidate, predicate: state != Accepted}\n        error: demo.release.CandidateNotAccepted\n      - name: drafted\n",
    )
}

#[test]
fn adversary_304_creating_command_with_optional_via_synthesizes_and_agrees() {
    attack("creating command", &creating_variant(), DRAFT);
}

#[test]
fn adversary_304_creating_command_keeps_dependent_command_green() {
    let model = ir(&creating_variant());
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals_for(&result, PUBLISH);
    assert!(refused.is_empty(), "{refused:#?}");
    let honest = statuses(&result, &Interpreted::for_model(model.clone()), PUBLISH);
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "{honest:#?}"
    );
}

// ---- interpreter -------------------------------------------------------------------------------

fn step(
    model: &EssIr,
    store: &Store,
    command: &str,
    input: &BTreeMap<String, Node>,
) -> (String, Store) {
    let mut steps = execute(
        model,
        store,
        &command.parse::<QualifiedName>().unwrap(),
        input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{command} is interpreted: {error}"));
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    let identity = step
        .events
        .iter()
        .flat_map(|event| event.payload.values())
        .find_map(Node::as_text)
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    (
        step.outcome
            .as_ref()
            .map_or_else(|| "none".to_owned(), ToString::to_string)
            + "|"
            + &identity,
        step.next,
    )
}

#[test]
fn adversary_304_an_explicit_null_reference_is_absent() {
    let model = ir(MODEL);
    let (drafted, store) = step(&model, &Store::default(), DRAFT, &BTreeMap::new());
    let release = drafted.split('|').nth(1).unwrap().to_owned();
    let (outcome, _) = step(
        &model,
        &store,
        PUBLISH,
        &BTreeMap::from([
            ("release_id".to_owned(), Node::Text(release)),
            ("candidate".to_owned(), Node::Null),
        ]),
    );
    assert!(
        outcome.starts_with("demo.release.PublishRelease/published"),
        "null is the absent Optional reference: {outcome}"
    );
}

// ---- validation --------------------------------------------------------------------------------

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

#[test]
fn adversary_304_the_unit_fixture_below_ess_22_is_refused_naming_ess_22() {
    for format in ["ess/21", "ess/20"] {
        let text = replaced(MODEL, "format: ess/22\n", &format!("format: {format}\n"));
        let errors = assemble(&text)
            .err()
            .unwrap_or_else(|| panic!("{format}: must refuse"));
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.to_string().contains("ess/22")),
            "{format}: the Optional via with its wrong_state sibling is refused without naming \
             ess/22, the format that admits it: {errors}"
        );
    }
}

#[test]
fn adversary_304_two_optional_vias_are_one_related_row_refusal() {
    let text = replaced(
        MODEL,
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n",
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n      - {name: other, type: Optional<demo.release.CandidateId>}\n",
    );
    let text = replaced(
        &text,
        "        when_related: {via: input.candidate, predicate: state != Accepted}\n",
        "        when_related: {via: input.other, predicate: state != Accepted}\n",
    );
    let errors = assemble(&text).expect_err("two related rows are refused");
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "one related row"
        ),
        "{errors}"
    );
}

#[test]
fn adversary_304_absent_reference_with_two_accepting_when_branches_must_cover() {
    // No default: the absent case reaches the `when:` branches over `hold`; both values covered.
    let text = publish_outcomes(&format!(
        "{NO_CANDIDATE}{NOT_ACCEPTED}{WRONG_STATE}      - {{name: held, when: hold == true, error: demo.release.Held}}\n      - name: published\n        when: hold == false\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {{demo.release.ReleasePublished: {{release_id: input.release_id}}}}\n"
    ));
    let text = replaced(
        &text,
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n",
        "      - {name: candidate, type: Optional<demo.release.CandidateId>}\n      - {name: hold, type: Boolean}\n",
    );
    let text = replaced(
        &text,
        "  - {name: demo.release.NoCandidate,",
        "  - {name: demo.release.Held, summary: The release is held back., fields: []}\n  - {name: demo.release.NoCandidate,",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("covered absent case is accepted: {errors}"));
    // Dropping the `hold == false` branch's guard coverage leaves `hold = false` unanswered.
    let uncovered = replaced(
        &text,
        "        when: hold == false\n",
        "        when: hold == true\n",
    );
    let errors = assemble(&uncovered).expect_err("hold = false is unanswered");
    assert!(
        has(&errors, ValidationCode::NonExhaustiveBranches, "absent")
            || has(&errors, ValidationCode::NonExhaustiveBranches, "hold"),
        "{errors}"
    );
}
