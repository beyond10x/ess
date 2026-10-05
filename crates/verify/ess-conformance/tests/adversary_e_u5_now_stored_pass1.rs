//! Adversary, unit E-U5 (`now` over stored and related rows), pass 1.
//!
//! The unit's acceptance: "a related row looked up by a wrong identity" is one of the A3 faulty
//! targets, "each a faulty target failing a synthesized scenario". The suite controls the unit
//! wrote (`tests/now_stored_rows_suite.rs`, `tests/now_stored_rows_runtimes.rs`) run only the
//! clock faults against the synthesized suite. Here the healthy interpreter, its clock staged as
//! the unit's own controls stage it, is wrapped so that `Lend` decides by a member row other than
//! the one its input names, and the synthesized suite must fail at least one `Lend` scenario.

mod support_now_stored;
mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::sync::Mutex;

use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::*;
use ess_conformance::{now_offset, AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;
use support_now_stored::{Clocking, Staged, LEASES, WALL_MS};
use support_occurrence_clock::model;

const LEND: [&str; 3] = [
    "demo.leases.Lend/outcome/banned",
    "demo.leases.Lend/outcome/embargoed",
    "demo.leases.Lend/outcome/lent",
];

/// Which stored member a faulty `Lend` decides by instead of the one its input names.
#[derive(Clone, Copy, Debug)]
enum Pick {
    First,
    Last,
}

/// The healthy, staged interpreter, with `Lend` reading the wrong member row.
struct WrongMember {
    inner: Staged,
    pick: Pick,
    members: Mutex<Vec<String>>,
}

impl WrongMember {
    fn rewrite(&self, mut request: SemanticCommandRequest) -> SemanticCommandRequest {
        if request.command.to_string() == "demo.leases.Lend" {
            let members = self.members.lock().unwrap();
            let chosen = match self.pick {
                Pick::First => members.first(),
                Pick::Last => members.last(),
            };
            if let Some(chosen) = chosen {
                request
                    .input
                    .insert("member_id".to_owned(), Node::Text(chosen.clone()));
            }
        }
        request
    }

    fn note(&self, command: &str, answer: &Result<SemanticCommandResult, TargetError>) {
        if command != "demo.leases.RegisterMember" {
            return;
        }
        if let Ok(answer) = answer {
            if let Some(Node::Text(member)) = answer
                .direct_events
                .first()
                .and_then(|event| event.payload.get("member_id"))
            {
                self.members.lock().unwrap().push(member.clone());
            }
        }
    }
}

impl ConformanceTarget for WrongMember {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.members.lock().unwrap().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        let answer = self.inner.execute_command(self.rewrite(request));
        self.note(&command, &answer);
        answer
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        let command = request.command.to_string();
        let completion = self.inner.execute_command_recorded(self.rewrite(request));
        self.note(&command, &completion.answer);
        completion
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

fn run<T: ConformanceTarget>(target: &T) -> BTreeMap<String, Status> {
    let suite = synthesize(&model(LEASES)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let runner = Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(&suite),
    );
    runner
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn wrong_member_is_caught(pick: Pick) {
    let suite = synthesize(&model(LEASES)).suite;
    let healthy = Staged::new(model(LEASES), &suite, Clocking::Healthy);
    let statuses = run(&healthy);
    for id in LEND {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "healthy {id}");
    }
    let faulty = WrongMember {
        inner: Staged::new(model(LEASES), &suite, Clocking::Healthy),
        pick,
        members: Mutex::new(Vec::new()),
    };
    let statuses = run(&faulty);
    let failed: Vec<&str> = LEND
        .iter()
        .copied()
        .filter(|id| statuses.get(*id) != Some(&Status::Passed))
        .collect();
    assert_ne!(
        failed.len(),
        0,
        "a Lend deciding by the {pick:?} stored member passes every synthesized Lend scenario: \
         {statuses:#?}"
    );
}

#[test]
fn adversary_a3_lend_deciding_by_the_first_member_fails_a_synthesized_scenario() {
    wrong_member_is_caught(Pick::First);
}

#[test]
fn adversary_a3_lend_deciding_by_the_last_member_fails_a_synthesized_scenario() {
    wrong_member_is_caught(Pick::Last);
}

/// `LEASES` with a `banned_until` on every book too, which the implementation assigns and no
/// predicate reads: only the member's `banned_until` is ordered against `now`.
fn books_with_their_own_banned_until() -> String {
    [
        (
            "      - {name: embargo_until, type: Timestamp}\n    lifecycle:",
            "      - {name: embargo_until, type: Timestamp}\n      - {name: banned_until, type: Timestamp}\n    lifecycle:",
        ),
        (
            "sets: {embargo_until: input.embargo_until}",
            "sets: {embargo_until: input.embargo_until, banned_until: {generated: true}}",
        ),
    ]
    .iter()
    .fold(LEASES.to_owned(), |text, (before, after)| {
        assert!(text.contains(before), "{before}");
        text.replacen(before, after, 1)
    })
}

/// A book's row is arranged by the book's own predicates: the member's ordering of the member's
/// `banned_until` against `now` says nothing about a book field that shares its name.
#[test]
fn adversary_a3_a_book_is_not_refused_for_the_members_now_ordering() {
    let synthesis = synthesize(&model(&books_with_their_own_banned_until()));
    let misattributed: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {:?}", refusal.scenario, refusal.cause))
        .filter(|line| line.contains("demo.leases.Book.banned_until"))
        .collect();
    assert_eq!(
        misattributed,
        Vec::<String>::new(),
        "a book refused naming a field no book predicate orders against `now`"
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for id in LEND {
        assert!(
            ids.iter().any(|known| known == id),
            "{id} is synthesized: {ids:#?}"
        );
    }
}

/// `LEASES` with an `embargo_until` on every member too, which the implementation assigns and no
/// predicate reads: only the book's `embargo_until` is ordered against `now`.
fn members_with_their_own_embargo_until() -> String {
    [
        (
            "      - {name: banned_until, type: Timestamp}\n    lifecycle:",
            "      - {name: banned_until, type: Timestamp}\n      - {name: embargo_until, type: Timestamp}\n    lifecycle:",
        ),
        (
            "sets: {banned_until: input.banned_until}",
            "sets: {banned_until: input.banned_until, embargo_until: {generated: true}}",
        ),
    ]
    .iter()
    .fold(LEASES.to_owned(), |text, (before, after)| {
        assert!(text.contains(before), "{before}");
        text.replacen(before, after, 1)
    })
}

/// A member's row is arranged by the member's own predicates: the book's ordering of the book's
/// `embargo_until` against `now` says nothing about a member field that shares its name.
#[test]
fn adversary_a3_a_member_is_not_refused_for_the_books_now_ordering() {
    let synthesis = synthesize(&model(&members_with_their_own_embargo_until()));
    let misattributed: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {:?}", refusal.scenario, refusal.cause))
        .filter(|line| line.contains("demo.leases.Member.embargo_until"))
        .collect();
    assert_eq!(
        misattributed,
        Vec::<String>::new(),
        "a member refused naming a field no member predicate orders against `now`"
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for id in LEND {
        assert!(
            ids.iter().any(|known| known == id),
            "{id} is synthesized: {ids:#?}"
        );
    }
}
