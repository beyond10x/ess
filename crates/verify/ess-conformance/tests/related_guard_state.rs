//! `when_related:` reading the related row's held state (beyond10x/ess#229, `ess/20`): a release
//! is published only for a candidate in state `Accepted`. The refusal is witnessed on a candidate
//! that exists in another state, between decoys that are accepted, and the move on an accepted
//! one; a target that checks only that the candidate exists fails the refusal.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const RELEASE: &str = include_str!("fixtures/related-guard-release.yaml");

const NO_CANDIDATE: &str = "demo.release.PublishRelease/outcome/no-candidate";
const NOT_ACCEPTED: &str = "demo.release.PublishRelease/outcome/not-accepted";
const PUBLISHED: &str = "demo.release.PublishRelease/outcome/published";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("release.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(RELEASE))
}

/// The refusals filed under the scenario id `id`, each with its cause as a sentence.
fn refusals_for(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{refusal:?} {}", refusal.cause))
        .collect()
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
                    "no scenario {id}\n refusals: {:#?}",
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{refusal:?}"))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The commands the scenario sends, in order.
fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

/// The input of the last `PublishRelease` the scenario sends.
fn publishing(scenario: &ConformanceScenario) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.release.PublishRelease" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the scenario publishes a release")
}

/// The state a `Candidates` view expectation of the scenario says `candidate` holds.
fn observed_state(scenario: &ConformanceScenario, candidate: &ScenarioValue) -> Option<String> {
    let candidate = serde_json::to_value(candidate).unwrap();
    scenario.steps.iter().find_map(|step| {
        let step = serde_json::to_value(step).unwrap();
        let fields = &step["expectation"]["fields"];
        (step["step"] == "expect_view"
            && step["view"] == "demo.release.Candidates"
            && fields["candidate_id"] == candidate)
            .then(|| fields["state"]["value"].as_str().map(str::to_owned))
            .flatten()
    })
}

#[test]
fn issue_229_both_sides_of_the_related_state_are_synthesized() {
    let result = synthesis();
    for id in [NO_CANDIDATE, NOT_ACCEPTED, PUBLISHED] {
        scenario(&result, id);
        assert!(
            refusals_for(&result, id).is_empty(),
            "{id}: {:#?}",
            refusals_for(&result, id)
        );
    }
    for id in [NOT_ACCEPTED, PUBLISHED] {
        let input = publishing(scenario(&result, id));
        for field in ["release_id", "candidate"] {
            assert!(
                matches!(input.get(field), Some(ScenarioValue::Instance { .. })),
                "{id}: {field} names an arranged row: {input:?}"
            );
        }
    }
    // Each witness observes the named candidate's state before publishing: the refusal on one that
    // is not accepted, the move on one that is.
    for (id, state) in [(NOT_ACCEPTED, "Proposed"), (PUBLISHED, "Accepted")] {
        let named = publishing(scenario(&result, id));
        let observed = observed_state(scenario(&result, id), &named["candidate"]);
        assert_eq!(observed.as_deref(), Some(state), "{id}");
    }
    // The accepting witness moves its candidate to `Accepted` before publishing.
    assert!(
        sent(scenario(&result, PUBLISHED))
            .iter()
            .any(|command| command == "demo.release.AcceptCandidate"),
        "{:?}",
        sent(scenario(&result, PUBLISHED))
    );
}

// ---- the suite, run ------------------------------------------------------------------------

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    candidates: RefCell<BTreeMap<String, String>>,
    releases: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

/// A release service that checks the candidate's state, or — as a mutant — only that it exists.
struct Releasing {
    checks_state: bool,
    store: Store,
}

impl Releasing {
    fn new(checks_state: bool) -> Self {
        Self {
            checks_state,
            store: Store::default(),
        }
    }

    fn mint(&self) -> String {
        self.store.minted.set(self.store.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.store.minted.get())
    }

    /// `PublishRelease`: a missing candidate, then — unless this is the mutant — one not accepted,
    /// then the move.
    fn publish(
        &self,
        command: &CommandRef,
        request: &SemanticCommandRequest,
    ) -> SemanticCommandResult {
        let error = |name: &str| DeclaredErrorValue::new(name.parse::<ErrorRef>().unwrap());
        let id = text(request.input.get("release_id"));
        let candidate = text(request.input.get("candidate"));
        let held = self.store.releases.borrow().get(&id).cloned();
        let state = self.store.candidates.borrow().get(&candidate).cloned();
        match state.as_deref() {
            None => SemanticCommandResult::took(branch(command, "no-candidate"))
                .with_error(error("demo.release.NoCandidate")),
            Some(state) if self.checks_state && state != "Accepted" => {
                SemanticCommandResult::took(branch(command, "not-accepted"))
                    .with_error(error("demo.release.CandidateNotAccepted"))
            }
            Some(_) if held.as_deref() == Some("Draft") => {
                self.store
                    .releases
                    .borrow_mut()
                    .insert(id.clone(), "Published".to_owned());
                SemanticCommandResult::took(branch(command, "published")).emitting(
                    ObservedEvent::new(
                        "demo.release.ReleasePublished".parse::<EventRef>().unwrap(),
                    )
                    .with("release_id", Node::Text(id)),
                )
            }
            Some(_) => SemanticCommandResult::undeclared(),
        }
    }
}

impl ConformanceTarget for Releasing {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("releasing-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.candidates.replace(BTreeMap::new());
        self.store.releases.replace(BTreeMap::new());
        self.store.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let event = |name: &str, field: &str, id: &str| {
            ObservedEvent::new(name.parse::<EventRef>().unwrap())
                .with(field, Node::Text(id.to_owned()))
        };
        let result = match command.to_string().as_str() {
            "demo.release.ProposeCandidate" => {
                let id = self.mint();
                self.store
                    .candidates
                    .borrow_mut()
                    .insert(id.clone(), "Proposed".to_owned());
                SemanticCommandResult::took(branch(&command, "proposed")).emitting(event(
                    "demo.release.CandidateProposed",
                    "candidate_id",
                    &id,
                ))
            }
            "demo.release.AcceptCandidate" => {
                let id = text(request.input.get("candidate_id"));
                let held = self.store.candidates.borrow().get(&id).cloned();
                // The variant whose acceptance reads a parent candidate (`own_entity`).
                let parent = request.input.get("parent").map(|parent| {
                    self.store
                        .candidates
                        .borrow()
                        .get(&text(Some(parent)))
                        .cloned()
                });
                if parent == Some(None) {
                    SemanticCommandResult::took(branch(&command, "no-parent")).with_error(
                        DeclaredErrorValue::new(
                            "demo.release.NoCandidate".parse::<ErrorRef>().unwrap(),
                        ),
                    )
                } else if parent
                    .as_ref()
                    .is_some_and(|state| state.as_deref() == Some("Accepted"))
                {
                    SemanticCommandResult::took(branch(&command, "parent-not-accepted")).with_error(
                        DeclaredErrorValue::new(
                            "demo.release.CandidateNotAccepted"
                                .parse::<ErrorRef>()
                                .unwrap(),
                        ),
                    )
                } else if held.as_deref() == Some("Proposed") {
                    self.store
                        .candidates
                        .borrow_mut()
                        .insert(id.clone(), "Accepted".to_owned());
                    SemanticCommandResult::took(branch(&command, "accepted")).emitting(event(
                        "demo.release.CandidateAccepted",
                        "candidate_id",
                        &id,
                    ))
                } else {
                    SemanticCommandResult::undeclared()
                }
            }
            "demo.release.DraftRelease" => {
                let id = self.mint();
                self.store
                    .releases
                    .borrow_mut()
                    .insert(id.clone(), "Draft".to_owned());
                SemanticCommandResult::took(branch(&command, "drafted")).emitting(event(
                    "demo.release.ReleaseDrafted",
                    "release_id",
                    &id,
                ))
            }
            "demo.release.PublishRelease" => self.publish(&command, &request),
            _ => SemanticCommandResult::undeclared(),
        };
        for published in &result.direct_events {
            self.store.published.borrow_mut().push(published.clone());
        }
        self.store.minted.set(self.store.minted.get() + 1);
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.store.minted.get()
            ))
            .unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = |held: &BTreeMap<String, String>, key: &str| -> Vec<BTreeMap<String, Node>> {
            held.iter()
                .map(|(id, state)| {
                    BTreeMap::from([
                        (key.to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text(state.clone())),
                    ])
                })
                .collect()
        };
        let rows = match request.view.to_string().as_str() {
            "demo.release.Candidates" => rows(&self.store.candidates.borrow(), "candidate_id"),
            "demo.release.Releases" => rows(&self.store.releases.borrow(), "release_id"),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn statuses(result: &Synthesis, target: &Releasing) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

#[test]
fn issue_229_a_target_reading_the_candidates_state_passes_and_one_ignoring_it_fails() {
    let result = synthesis();
    let correct = statuses(&result, &Releasing::new(true));
    for id in [NO_CANDIDATE, NOT_ACCEPTED, PUBLISHED] {
        assert_eq!(
            correct.get(id).map(String::as_str),
            Some("passed"),
            "{correct:#?}"
        );
    }
    assert!(
        correct.values().all(|status| status == "passed"),
        "{correct:#?}"
    );
    let ignoring = statuses(&result, &Releasing::new(false));
    assert_ne!(
        ignoring.get(NOT_ACCEPTED).map(String::as_str),
        Some("passed"),
        "{ignoring:#?}"
    );
}

// ---- a mover of the related row that reads a related row of the same entity ---------------

/// The release fixture where accepting a candidate names a `parent` candidate that must exist and
/// must not be accepted yet: the row `PublishRelease` reads is moved into `Accepted` by a command
/// whose own related row is a candidate too (adversary pass 1, beyond10x/ess#229).
fn own_entity() -> String {
    let from = "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n";
    let to = "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: parent, type: demo.release.CandidateId}\n    outcomes:\n      - name: no-parent\n        when_related: {via: input.parent, exists: false}\n        error: demo.release.NoCandidate\n      - name: parent-not-accepted\n        when_related: {via: input.parent, predicate: state == Accepted}\n        error: demo.release.CandidateNotAccepted\n";
    let text = RELEASE.replacen(from, to, 1);
    assert_ne!(text, RELEASE);
    text
}

#[test]
fn issue_229_a_same_entity_mover_is_arranged_one_level_deep_and_its_suite_runs() {
    let result = ess_conformance::synthesize::synthesize(&ir(&own_entity()));
    for id in [
        NOT_ACCEPTED,
        PUBLISHED,
        "demo.release.AcceptCandidate/outcome/parent-not-accepted",
        "demo.release.AcceptCandidate/outcome/accepted",
    ] {
        scenario(&result, id);
        assert!(
            refusals_for(&result, id).is_empty(),
            "{id}: {:#?}",
            refusals_for(&result, id)
        );
    }
    // Every scenario binds each instance once, so the suite admits; and a target honouring both
    // guards passes all of it, decoys and nested parents included.
    let correct = statuses(&result, &Releasing::new(true));
    assert!(
        correct.values().all(|status| status == "passed"),
        "{correct:#?}"
    );
    let ignoring = statuses(&result, &Releasing::new(false));
    assert_ne!(
        ignoring.get(NOT_ACCEPTED).map(String::as_str),
        Some("passed"),
        "{ignoring:#?}"
    );
}

#[test]
fn issue_229_a_mover_needing_a_moved_row_of_its_own_entity_is_refused_naming_the_bound() {
    // Accepting needs an accepted parent: no first candidate is ever accepted, and the nested
    // parent, arranged where its lifecycle starts, does not select `accepted`.
    let text = own_entity().replacen(
        "predicate: state == Accepted}",
        "predicate: state != Accepted}",
        1,
    );
    let result = ess_conformance::synthesize::synthesize(&ir(&text));
    for id in [PUBLISHED, "demo.release.AcceptCandidate/outcome/accepted"] {
        assert!(
            !result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id),
            "{id} has no witness"
        );
        let named = refusals_for(&result, id);
        assert!(
            named.iter().any(|text| text.contains("one level deep")),
            "{id}: {named:#?}"
        );
    }
}
