//! A non-creating command's effect on related records (beyond10x/ess#229, the effect half;
//! `story:related-record-effects`, ess/22): `moves: <Entity>.<transition>` inside an `affects:`
//! entry, with the `instances:` set-move semantics.
//!
//! `DeactivateUser` moves its user to `Inactive` and ends every `Live` session of that user,
//! marking each revoked; a session of the user resting in `Expired` is skipped, and another user's
//! sessions are left alone. The interpreter applies the move; synthesis witnesses it on rows
//! arranged on both sides of the filter and outside the move's `from` states; and an in-memory
//! target that skips or misapplies the move in any of the ways below fails the branch's scenario
//! while a correct one passes the whole suite.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/22
system: demo
version: v1
domain: demo.users
types:
  - {name: demo.users.UserId, kind: newtype, of: String}
  - {name: demo.users.SessionId, kind: newtype, of: String}
entities:
  - name: demo.users.User
    identity: {name: user_id, type: demo.users.UserId}
    fields: [{name: team, type: String}]
    lifecycle:
      initial: Active
      states: [Active, Inactive]
      terminal: [Inactive]
      transitions: [{name: deactivate, from: [Active], to: Inactive}]
  - name: demo.users.Session
    identity: {name: session_id, type: demo.users.SessionId}
    fields:
      - {name: user_id, type: demo.users.UserId}
      - {name: revoked, type: Boolean}
    lifecycle:
      initial: Live
      states: [Live, Ended, Expired]
      terminal: [Ended, Expired]
      transitions:
        - {name: end, from: [Live], to: Ended}
        - {name: expire, from: [Live], to: Expired}
events:
  - name: demo.users.UserAdded
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.UserDeactivated
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.SessionStarted
    fields: [{name: session_id, type: demo.users.SessionId}]
  - name: demo.users.SessionExpired
    fields: [{name: session_id, type: demo.users.SessionId}]
errors: [{name: demo.users.NotActive}, {name: demo.users.NotLive}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.StartSession, demo.users.ExpireSession, demo.users.DeactivateUser]
commands:
  - name: demo.users.AddUser
    input: [{name: team, type: String}]
    outcomes:
      - name: added
        creates: demo.users.User
        instance: user_id
        emits: [demo.users.UserAdded]
        payload: {demo.users.UserAdded: {user_id: {generated: true}}}
        sets: {team: input.team}
  - name: demo.users.StartSession
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: started
        creates: demo.users.Session
        instance: session_id
        emits: [demo.users.SessionStarted]
        payload: {demo.users.SessionStarted: {session_id: {generated: true}}}
        sets: {user_id: input.user_id, revoked: false}
  - name: demo.users.ExpireSession
    input: [{name: session_id, type: demo.users.SessionId}]
    outcomes:
      - name: expired
        moves: demo.users.Session.expire
        instance: session_id
        emits: [demo.users.SessionExpired]
        payload: {demo.users.SessionExpired: {session_id: input.session_id}}
      - {name: not-live, wrong_state: true, error: demo.users.NotLive}
  - name: demo.users.DeactivateUser
    input: [{name: user_id, type: demo.users.UserId}]
    outcomes:
      - name: deactivated
        moves: demo.users.User.deactivate
        instance: user_id
        emits: [demo.users.UserDeactivated]
        payload: {demo.users.UserDeactivated: {user_id: input.user_id}}
        affects:
          - entity: demo.users.Session
            where: user_id == subject.user_id
            moves: demo.users.Session.end
            sets: {revoked: true}
      - {name: not-active, wrong_state: true, error: demo.users.NotActive}
views:
  - name: demo.users.Users
    source: demo.users.User
    consistency: read_your_writes
    fields: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: state, type: demo.users.User.State}]
  - name: demo.users.Sessions
    source: demo.users.Session
    consistency: read_your_writes
    fields: [{name: session_id, type: demo.users.SessionId}, {name: user_id, type: demo.users.UserId}, {name: revoked, type: Boolean}, {name: state, type: demo.users.Session.State}]
";

const DEACTIVATED: &str = "demo.users.DeactivateUser/outcome/deactivated";

/// The same model with the entry's `sets:` left out: a move alone.
fn moves_only() -> String {
    let from = "            sets: {revoked: true}\n";
    assert!(MODEL.contains(from));
    MODEL.replacen(from, "", 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("users.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
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

// ---- a target, and the ways it can get the effect wrong ----------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// Deactivates the user and leaves every session as it was.
    SkipsTheEffect,
    /// Marks the user's sessions revoked and leaves them `Live`: the effect read as `sets:` alone.
    SetsWithoutMoving,
    /// Moves the user's sessions to `Expired` rather than `Ended`.
    WrongArrival,
    /// Ends the user's `Expired` session too.
    IgnoresFrom,
    /// Ends every `Live` session, whoever's it is.
    IgnoresFilter,
    /// Ends the first of the user's sessions only.
    EndsOne,
    /// Ends the sessions and leaves them unrevoked.
    IgnoresSets,
}

#[derive(Clone, Debug)]
struct Session {
    id: String,
    user_id: String,
    revoked: bool,
    state: &'static str,
}

#[derive(Clone, Debug)]
struct User {
    id: String,
    team: String,
    inactive: bool,
}

struct Desk {
    mode: Mode,
    /// Whether the model the suite was synthesized from writes `revoked: true`.
    revokes: bool,
    users: RefCell<Vec<User>>,
    sessions: RefCell<Vec<Session>>,
    next: RefCell<usize>,
}

fn text(input: &BTreeMap<String, Node>, field: &str) -> String {
    input
        .get(field)
        .and_then(Node::as_text)
        .unwrap_or_default()
        .to_owned()
}

fn took(
    command: &CommandRef,
    outcome: &str,
    event: &str,
    payload: Vec<(&str, Node)>,
) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    for (key, value) in payload {
        observed.payload.insert(key.to_owned(), value);
    }
    result.direct_events.push(observed);
    result
}

fn refused(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

impl Desk {
    fn new(mode: Mode, revokes: bool) -> Self {
        Self {
            mode,
            revokes,
            users: RefCell::default(),
            sessions: RefCell::default(),
            next: RefCell::new(0),
        }
    }

    fn fresh(&self, prefix: &str) -> String {
        let mut next = self.next.borrow_mut();
        *next += 1;
        format!("{prefix}-{next}")
    }

    fn deactivate(&self, command: &CommandRef, id: &str) -> SemanticCommandResult {
        let mode = self.mode;
        let mut users = self.users.borrow_mut();
        // An identity no user carries is answered as one resting outside `deactivate`'s `from`.
        let Some(user) = users
            .iter_mut()
            .find(|user| user.id == id)
            .filter(|user| !user.inactive)
        else {
            return refused(command, "not-active", "demo.users.NotActive");
        };
        user.inactive = true;
        let mut ended = 0;
        for session in self.sessions.borrow_mut().iter_mut() {
            let selected = mode == Mode::IgnoresFilter || session.user_id == id;
            let movable = session.state == "Live"
                || (mode == Mode::IgnoresFrom && session.state == "Expired");
            if !(selected && movable) || mode == Mode::SkipsTheEffect {
                continue;
            }
            if mode == Mode::EndsOne && ended == 1 {
                continue;
            }
            session.state = match mode {
                Mode::SetsWithoutMoving => session.state,
                Mode::WrongArrival => "Expired",
                _ => "Ended",
            };
            if self.revokes && mode != Mode::IgnoresSets {
                session.revoked = true;
            }
            ended += 1;
        }
        took(
            command,
            "deactivated",
            "demo.users.UserDeactivated",
            vec![("user_id", Node::Text(id.to_owned()))],
        )
    }
}

impl ConformanceTarget for Desk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("related-record-effects", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.users.borrow_mut().clear();
        self.sessions.borrow_mut().clear();
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
        let input = &request.input;
        Ok(match command.to_string().as_str() {
            "demo.users.AddUser" => {
                let id = self.fresh("user");
                self.users.borrow_mut().push(User {
                    id: id.clone(),
                    team: text(input, "team"),
                    inactive: false,
                });
                took(
                    &command,
                    "added",
                    "demo.users.UserAdded",
                    vec![("user_id", Node::Text(id))],
                )
            }
            "demo.users.StartSession" => {
                let id = self.fresh("session");
                self.sessions.borrow_mut().push(Session {
                    id: id.clone(),
                    user_id: text(input, "user_id"),
                    revoked: false,
                    state: "Live",
                });
                took(
                    &command,
                    "started",
                    "demo.users.SessionStarted",
                    vec![("session_id", Node::Text(id))],
                )
            }
            "demo.users.ExpireSession" => {
                let id = text(input, "session_id");
                let mut sessions = self.sessions.borrow_mut();
                let Some(session) = sessions
                    .iter_mut()
                    .find(|session| session.id == id)
                    .filter(|session| session.state == "Live")
                else {
                    return Ok(refused(&command, "not-live", "demo.users.NotLive"));
                };
                session.state = "Expired";
                took(
                    &command,
                    "expired",
                    "demo.users.SessionExpired",
                    vec![("session_id", Node::Text(id))],
                )
            }
            "demo.users.DeactivateUser" => self.deactivate(&command, &text(input, "user_id")),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = match request.view.to_string().as_str() {
            "demo.users.Users" => self
                .users
                .borrow()
                .iter()
                .map(|user| {
                    BTreeMap::from([
                        ("user_id".to_owned(), Node::Text(user.id.clone())),
                        ("team".to_owned(), Node::Text(user.team.clone())),
                        (
                            "state".to_owned(),
                            Node::Text(if user.inactive { "Inactive" } else { "Active" }.into()),
                        ),
                    ])
                })
                .collect(),
            "demo.users.Sessions" => self
                .sessions
                .borrow()
                .iter()
                .map(|session| {
                    BTreeMap::from([
                        ("session_id".to_owned(), Node::Text(session.id.clone())),
                        ("user_id".to_owned(), Node::Text(session.user_id.clone())),
                        ("revoked".to_owned(), Node::Bool(session.revoked)),
                        ("state".to_owned(), Node::Text(session.state.to_owned())),
                    ])
                })
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult { rows, total: None })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
}

// ---- synthesis ---------------------------------------------------------------------------------

#[test]
fn issue_229_the_related_move_is_synthesized_without_a_refusal() {
    for text in [MODEL.to_owned(), moves_only()] {
        let synthesis = synthesis_of(&text);
        scenario(&synthesis.suite, DEACTIVATED);
        let about: Vec<String> = synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{refusal:?}"))
            .collect();
        assert!(
            about
                .iter()
                .all(|refusal| !refusal.contains("DeactivateUser")),
            "nothing about the branch is refused: {about:#?}"
        );
        // The one refusal left is the rule the design keeps: a related move is a cause of its
        // transition but no driver of an arrangement, so `Ended`, which only it reaches, cannot be
        // arranged for `ExpireSession`'s refusal there.
        let [only] = about.as_slice() else {
            panic!("one refusal: {about:#?}")
        };
        assert!(
            only.contains("ExpireSession") && only.contains("Ended") && only.contains("NoPath"),
            "{only}"
        );
    }
}

/// How many steps of `scenario` send `command`.
fn sends(scenario: &ConformanceScenario, command: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
                if sent.to_string() == command)
        })
        .count()
}

/// The view reads of `scenario` requiring a row in `state`.
fn reads_in(scenario: &ConformanceScenario, state: &str) -> usize {
    let needle = format!("Text(\"{state}\")");
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExpectView { view, .. }
                if view.to_string() == "demo.users.Sessions")
                && format!("{step:?}").contains(&needle)
        })
        .count()
}

#[test]
fn issue_229_the_move_is_witnessed_on_moved_kept_and_skipped_rows() {
    let synthesis = synthesis_of(MODEL);
    let deactivated = scenario(&synthesis.suite, DEACTIVATED);
    // Three of the subject's sessions the move takes, one of another user's it leaves, and one of
    // the subject's resting outside the move's `from` states.
    assert!(
        sends(deactivated, "demo.users.StartSession") >= 5,
        "{:#?}",
        deactivated.steps
    );
    assert!(
        sends(deactivated, "demo.users.ExpireSession") >= 1,
        "one selected session rests outside `from`: {:#?}",
        deactivated.steps
    );
    assert!(
        reads_in(deactivated, "Ended") >= 3,
        "the moved sessions are read back where the move leaves them: {:#?}",
        deactivated.steps
    );
    assert!(
        reads_in(deactivated, "Expired") >= 1,
        "the skipped session is read back where it rested: {:#?}",
        deactivated.steps
    );
}

// ---- the interpreter ---------------------------------------------------------------------------

#[test]
fn issue_229_the_interpreter_applies_the_move_to_the_related_rows() {
    for text in [MODEL.to_owned(), moves_only()] {
        let ir = ir_of(&text);
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        let statuses = run(
            &synthesis.suite,
            &ess_conformance::interpret::Interpreted::for_model(ir),
        );
        assert_eq!(
            statuses.get(DEACTIVATED),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
        assert_eq!(not_passed(&statuses).len(), 0, "{statuses:#?}");
    }
}

// ---- targets -----------------------------------------------------------------------------------

#[test]
fn issue_229_a_correct_target_passes_the_whole_suite() {
    for (text, revokes) in [(MODEL.to_owned(), true), (moves_only(), false)] {
        let synthesis = synthesis_of(&text);
        let statuses = run(&synthesis.suite, &Desk::new(Mode::Correct, revokes));
        assert_eq!(
            statuses.get(DEACTIVATED),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
        assert_eq!(not_passed(&statuses).len(), 0, "{statuses:#?}");
    }
}

#[test]
fn issue_229_a_target_skipping_or_misapplying_the_move_fails_its_branch() {
    let synthesis = synthesis_of(MODEL);
    for mode in [
        Mode::SkipsTheEffect,
        Mode::SetsWithoutMoving,
        Mode::WrongArrival,
        Mode::IgnoresFrom,
        Mode::IgnoresFilter,
        Mode::EndsOne,
        Mode::IgnoresSets,
    ] {
        let statuses = run(&synthesis.suite, &Desk::new(mode, true));
        assert_eq!(
            statuses.get(DEACTIVATED),
            Some(&Status::Failed),
            "{mode:?}: {statuses:#?}"
        );
    }
}

#[test]
fn issue_229_a_move_alone_is_still_witnessed_by_the_state_it_leaves() {
    let synthesis = synthesis_of(&moves_only());
    for mode in [
        Mode::SkipsTheEffect,
        Mode::WrongArrival,
        Mode::IgnoresFrom,
        Mode::IgnoresFilter,
        Mode::EndsOne,
    ] {
        let statuses = run(&synthesis.suite, &Desk::new(mode, false));
        assert_eq!(
            statuses.get(DEACTIVATED),
            Some(&Status::Failed),
            "{mode:?}: {statuses:#?}"
        );
    }
}

// ---- correction round 1 (adversary pass 1) -----------------------------------------------------

/// The model with the entry split in two over the same rows: one moves them, the next marks them
/// revoked. Each row both select is read back once moved and revoked, as the interpreter leaves it.
fn move_then_sets() -> String {
    let from = "            moves: demo.users.Session.end\n            sets: {revoked: true}\n";
    assert!(MODEL.contains(from));
    MODEL.replacen(
        from,
        "            moves: demo.users.Session.end\n          - entity: demo.users.Session\n            where: user_id == subject.user_id\n            sets: {revoked: true}\n",
        1,
    )
}

#[test]
fn issue_229_a_moving_and_a_setting_entry_over_one_row_are_read_back_combined() {
    let ir = ir_of(&move_then_sets());
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let deactivated = scenario(&synthesis.suite, DEACTIVATED);
    assert!(
        reads_in(deactivated, "Ended") >= 6,
        "both entries' rows are read back moved: {:#?}",
        deactivated.steps
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(
        statuses.get(DEACTIVATED),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    // A target taking the move and not the second entry's writes fails it.
    let skipping = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir_of(&moves_only())),
    );
    assert_eq!(
        skipping.get(DEACTIVATED),
        Some(&Status::Failed),
        "{skipping:#?}"
    );
}

#[test]
fn issue_229_a_move_seen_only_in_its_arrival_state_and_writing_nothing_is_refused_by_name() {
    // `touch` starts only from `Live` and arrives in `Live`: no arranged row can show it taken.
    let model = moves_only()
        .replacen(
            "        - {name: end, from: [Live], to: Ended}\n",
            "        - {name: touch, from: [Live], to: Live}\n",
            1,
        )
        .replacen(
            "      states: [Live, Ended, Expired]\n",
            "      states: [Live, Expired]\n",
            1,
        )
        .replacen(
            "      terminal: [Ended, Expired]\n",
            "      terminal: [Expired]\n",
            1,
        )
        .replacen(
            "            moves: demo.users.Session.end\n",
            "            moves: demo.users.Session.touch\n",
            1,
        );
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    let about: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect();
    assert!(
        about
            .iter()
            .any(|refusal| refusal.contains("DeactivateUser")
                && refusal.contains("arrives in")
                && refusal.contains("skips the move")),
        "{about:#?}"
    );
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == DEACTIVATED),
        "no scenario a skipping target passes"
    );
}

// ---- a cascade onto rows whose creator is guarded (an adopter's report, row 14) ----------------

/// Cancelling a feed discards its pending items. `Post` creates the items, and the refusals the
/// cases below put above its creating branch decide whether an item can be posted to the feed the
/// cancellation names.
const FEED: &str = r"format: ess/25
system: cascade
version: v1
domain: cascade.feed
types:
  - {name: cascade.feed.FeedId, kind: newtype, of: Uuid}
  - {name: cascade.feed.ItemId, kind: newtype, of: Uuid}
entities:
  - name: cascade.feed.Feed
    identity: {name: feed_id, type: cascade.feed.FeedId}
    fields: [{name: label, type: String}]
    lifecycle:
      initial: Open
      states: [Open, Cancelled]
      terminal: [Cancelled]
      transitions: [{name: cancel, from: [Open], to: Cancelled}]
  - name: cascade.feed.Item
    identity: {name: item_id, type: cascade.feed.ItemId}
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
    relations:
      - {name: feed, kind: references, target: cascade.feed.Feed, cardinality: one, via: feed_id}
    lifecycle:
      initial: Pending
      states: [Pending, Discarded]
      terminal: [Discarded]
      transitions: [{name: discard, from: [Pending], to: Discarded}]
errors:
  - {name: cascade.feed.FeedNotFound, fields: []}
  - {name: cascade.feed.WrongState, fields: []}
  - {name: cascade.feed.Blank, fields: []}
events:
  - name: cascade.feed.FeedOpened
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
  - name: cascade.feed.ItemPosted
    fields: [{name: item_id, type: cascade.feed.ItemId}]
  - name: cascade.feed.FeedCancelled
    fields: [{name: feed_id, type: cascade.feed.FeedId}]
actors:
  - name: cascade.feed.Operator
    may: [cascade.feed.Open, cascade.feed.Post, cascade.feed.Cancel]
commands:
  - name: cascade.feed.Open
    input: [{name: label, type: String}]
    outcomes:
      - name: opened
        creates: cascade.feed.Feed
        instance: feed_id
        emits: [cascade.feed.FeedOpened]
        payload: {cascade.feed.FeedOpened: {feed_id: {generated: true}}}
        sets: {label: input.label}
  - name: cascade.feed.Post
    input:
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: urgent, type: Boolean}
    outcomes:
      - name: posted
        creates: cascade.feed.Item
        instance: item_id
        emits: [cascade.feed.ItemPosted]
        payload: {cascade.feed.ItemPosted: {item_id: {generated: true}}}
        sets: {feed_id: input.feed_id}
  - name: cascade.feed.Cancel
    input: [{name: feed_id, type: cascade.feed.FeedId}]
    outcomes:
      - name: cancelled
        moves: cascade.feed.Feed.cancel
        instance: feed_id
        emits: [cascade.feed.FeedCancelled]
        payload: {cascade.feed.FeedCancelled: {feed_id: input.feed_id}}
        affects:
          - entity: cascade.feed.Item
            where: feed_id == input.feed_id
            moves: cascade.feed.Item.discard
      - {name: wrong-state, wrong_state: true, error: cascade.feed.WrongState}
views:
  - name: cascade.feed.Items
    source: cascade.feed.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: cascade.feed.ItemId}
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: state, type: cascade.feed.Item.State}
  - name: cascade.feed.Feeds
    source: cascade.feed.Feed
    consistency: read_your_writes
    fields:
      - {name: feed_id, type: cascade.feed.FeedId}
      - {name: state, type: cascade.feed.Feed.State}
";

const CANCELLED: &str = "cascade.feed.Cancel/outcome/cancelled";

/// An unknown feed refused before `posted`.
const UNKNOWN_FEED: &str = "      - name: unknown-feed
        when_related: {via: input.feed_id, exists: false}
        error: cascade.feed.FeedNotFound
";

/// An input refusal before `posted`, reading nothing of the feed.
const BLANK: &str = "      - {name: blank, when: urgent == true, error: cascade.feed.Blank}
";

/// A feed that is not open refused before `posted`: the state the cancellation needs it in.
const STALE_FEED: &str = "      - name: stale-feed
        when_related: {via: input.feed_id, predicate: state != Open}
        error: cascade.feed.WrongState
";

/// An open feed refused before `posted`: items are posted only to a cancelled feed, so no item of
/// the feed a cancellation names can be posted while it is still open.
const OPEN_FEED: &str = "      - name: open-feed
        when_related: {via: input.feed_id, predicate: state == Open}
        error: cascade.feed.WrongState
";

/// [`FEED`] with `refusals` declared above `Post`'s creating branch.
fn feed(refusals: &[&str]) -> String {
    let at = "      - name: posted\n";
    assert!(FEED.contains(at));
    FEED.replacen(at, &format!("{}{at}", refusals.concat()), 1)
}

/// `model` with the cancellation's `affects:` entry left out: a target that skips the move. A
/// command no scenario of `model` sends takes the transition instead, so the model stays admitted.
fn skipping(model: &str) -> String {
    let entry = "        affects:
          - entity: cascade.feed.Item
            where: feed_id == input.feed_id
            moves: cascade.feed.Item.discard
";
    let may = "cascade.feed.Cancel]\n";
    let views = "views:\n";
    assert!(model.contains(entry) && model.contains(may) && model.contains(views));
    model
        .replacen(entry, "", 1)
        .replacen(may, "cascade.feed.Cancel, cascade.feed.Discard]\n", 1)
        .replacen(
            views,
            "  - name: cascade.feed.Discard
    input: [{name: item_id, type: cascade.feed.ItemId}]
    outcomes:
      - name: discarded
        moves: cascade.feed.Item.discard
        instance: item_id
        emits: [cascade.feed.ItemPosted]
        payload: {cascade.feed.ItemPosted: {item_id: input.item_id}}
      - {name: not-pending, wrong_state: true, error: cascade.feed.WrongState}
views:
",
            1,
        )
}

fn causes(synthesis: &ess_conformance::synthesize::Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal}: {}", refusal.cause))
        .collect()
}

/// `model` synthesizes with no refusal, and its cancellation's scenario passes the interpreter and
/// fails one that skips the move.
fn cascade_witnessed(model: &str) {
    let ir = ir_of(model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert_eq!(causes(&synthesis), Vec::<String>::new(), "{model}");
    let cancelled = scenario(&synthesis.suite, CANCELLED);
    assert!(
        sends(cancelled, "cascade.feed.Post") >= 3,
        "{:#?}",
        cancelled.steps
    );
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(not_passed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
    let skipped = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir_of(&skipping(model))),
    );
    assert_eq!(
        skipped.get(CANCELLED),
        Some(&Status::Failed),
        "{skipped:#?}"
    );
}

#[test]
fn a_cascade_onto_rows_of_a_guarded_creator_is_witnessed() {
    cascade_witnessed(&feed(&[]));
    cascade_witnessed(&feed(&[UNKNOWN_FEED]));
    cascade_witnessed(&feed(&[BLANK]));
    cascade_witnessed(&feed(&[UNKNOWN_FEED, BLANK]));
}

#[test]
fn a_cascade_onto_rows_of_a_creator_refusing_a_related_state_is_witnessed() {
    // A predicate over the related row is admitted only beside its missing-row branch.
    cascade_witnessed(&feed(&[UNKNOWN_FEED, STALE_FEED]));
    cascade_witnessed(&feed(&[STALE_FEED, UNKNOWN_FEED]));
    cascade_witnessed(&feed(&[UNKNOWN_FEED, STALE_FEED, BLANK]));
}

/// The refusal of the cancellation, where no item of its feed can be posted.
fn undriven() -> ess_conformance::synthesize::Refusal {
    let synthesis = synthesis_of(&feed(&[UNKNOWN_FEED, OPEN_FEED]));
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == CANCELLED),
        "no scenario a skipping target passes"
    );
    let found = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains("Cancel/outcome/cancelled"))
        .cloned();
    found.unwrap_or_else(|| panic!("{:#?}", causes(&synthesis)))
}

#[test]
fn a_cascade_refusal_names_the_creator_it_could_not_drive() {
    let refusal = undriven();
    let cause = refusal.cause.to_string();
    assert!(
        cause.contains("cascade.feed.Post/posted") && cause.contains("cancelled.affects[0]"),
        "{cause}"
    );
    assert!(!cause.contains("arrives in"), "{cause}");
}

#[test]
fn a_cascade_set_effect_refusal_carries_its_own_help() {
    let fallback = "give the field a type that has a finite value";
    let help = undriven().hint();
    assert!(!help.contains(fallback) && help.contains("creat"), "{help}");
    // A move seen only in the state it arrives in, writing nothing.
    let model = moves_only()
        .replacen(
            "        - {name: end, from: [Live], to: Ended}\n",
            "        - {name: touch, from: [Live], to: Live}\n",
            1,
        )
        .replacen(
            "      states: [Live, Ended, Expired]\n",
            "      states: [Live, Expired]\n",
            1,
        )
        .replacen(
            "      terminal: [Ended, Expired]\n",
            "      terminal: [Expired]\n",
            1,
        )
        .replacen(
            "            moves: demo.users.Session.end\n",
            "            moves: demo.users.Session.touch\n",
            1,
        );
    let synthesis = synthesis_of(&model);
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| refusal.to_string().contains("DeactivateUser"))
        .unwrap_or_else(|| panic!("{:#?}", causes(&synthesis)));
    let help = refusal.hint();
    assert!(!help.contains(fallback) && help.contains("move"), "{help}");
}
