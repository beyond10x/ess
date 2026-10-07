//! Set effects over filtered instances are witnessed on matching and non-matching rows (ess/16;
//! beyond10x/ess#167, #175, `docs/design/set-effects-over-filtered-instances.md`).
//!
//! `EndTeam` moves every `Open` session of a team to `Ended` and reports how many; `NoteTeam`
//! updates every session of a team; `Invite` takes one session off hold and puts every other
//! session of its team on hold. One in-memory target implements the model; each wrong mode breaks
//! it one way — a non-matching row changed, a matching row skipped, a wrong count, a row outside
//! the move's `from` states moved, `sets:` ignored, the subject included in `affects:` — and the
//! scenario of the branch it breaks fails, while a correct target passes the whole suite.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

const ENDED: &str = "demo.desk.EndTeam/outcome/ended";
const NOTED: &str = "demo.desk.NoteTeam/outcome/noted";
const INVITED: &str = "demo.desk.Invite/outcome/invited";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
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

// ---- the behaviour the issues describe, and ways to get it wrong ------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// Ends every `Open` session, whatever its team.
    EndIgnoresFilter,
    /// Ends the first matching session only.
    EndSkipsOne,
    /// Reports one more than it ended.
    EndMiscounts,
    /// Ends a matching `Parked` session as well.
    EndIgnoresFrom,
    /// Ends the matching sessions and leaves their note as it was.
    EndIgnoresSets,
    /// Notes every session, whatever its team.
    NoteIgnoresFilter,
    /// Notes the first matching session only.
    NoteSkipsOne,
    /// Reports none noted.
    NoteMiscounts,
    /// Puts the invited session on hold beside the others.
    InviteHoldsSubject,
    /// Puts nobody else on hold.
    InviteSkipsOthers,
    /// Puts every other session on hold, whatever its team.
    InviteHoldsEveryTeam,
}

#[derive(Clone, Debug)]
struct Row {
    id: String,
    team: String,
    note: String,
    on_hold: bool,
    state: &'static str,
}

struct Desk {
    mode: Mode,
    rows: RefCell<Vec<Row>>,
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

fn count(n: usize) -> Node {
    Node::Number(Number::from(i64::try_from(n).unwrap()))
}

impl Desk {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::new(Vec::new()),
            next: RefCell::new(0),
        }
    }

    fn open(&self, command: &CommandRef, team: String, note: String) -> SemanticCommandResult {
        let mut next = self.next.borrow_mut();
        *next += 1;
        let id = format!("session-{next}");
        self.rows.borrow_mut().push(Row {
            id: id.clone(),
            team: team.clone(),
            note,
            on_hold: false,
            state: "Open",
        });
        took(
            command,
            "opened",
            "demo.desk.SessionOpened",
            vec![("session_id", Node::Text(id)), ("team", Node::Text(team))],
        )
    }

    /// `Park` and `Close`: one named session moves out of `Open`, or the command answers `NotOpen`.
    fn single(
        &self,
        command: &CommandRef,
        id: &str,
        (outcome, event, to): (&str, &str, &'static str),
    ) -> SemanticCommandResult {
        let mut rows = self.rows.borrow_mut();
        let row = rows.iter_mut().find(|row| row.id == id);
        let Some(row) = row.filter(|row| row.state == "Open") else {
            let mut result = SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                "not-open".parse().unwrap(),
            ));
            result.error = Some(DeclaredErrorValue::new(
                "demo.desk.NotOpen".parse().unwrap(),
            ));
            result.consistency =
                Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
            return result;
        };
        row.state = to;
        took(
            command,
            outcome,
            event,
            vec![("session_id", Node::Text(id.to_owned()))],
        )
    }

    fn end(&self, command: &CommandRef, team: &str, note: &str) -> SemanticCommandResult {
        let mode = self.mode;
        let mut rows = self.rows.borrow_mut();
        let mut ended = 0;
        for row in rows.iter_mut() {
            let matches = mode == Mode::EndIgnoresFilter || row.team == team;
            let movable =
                row.state == "Open" || (mode == Mode::EndIgnoresFrom && row.state == "Parked");
            if !(matches && movable) || (mode == Mode::EndSkipsOne && ended == 1) {
                continue;
            }
            row.state = "Ended";
            if mode != Mode::EndIgnoresSets {
                note.clone_into(&mut row.note);
            }
            ended += 1;
        }
        if mode == Mode::EndMiscounts {
            ended += 1;
        }
        took(
            command,
            "ended",
            "demo.desk.TeamEnded",
            vec![
                ("team", Node::Text(team.to_owned())),
                ("ended", count(ended)),
            ],
        )
    }

    fn note(&self, command: &CommandRef, team: &str, note: &str) -> SemanticCommandResult {
        let mode = self.mode;
        let mut rows = self.rows.borrow_mut();
        let mut noted = 0;
        for row in rows.iter_mut() {
            let matches = mode == Mode::NoteIgnoresFilter || row.team == team;
            if !matches || (mode == Mode::NoteSkipsOne && noted == 1) {
                continue;
            }
            note.clone_into(&mut row.note);
            noted += 1;
        }
        if mode == Mode::NoteMiscounts {
            noted = 0;
        }
        took(
            command,
            "noted",
            "demo.desk.TeamNoted",
            vec![
                ("team", Node::Text(team.to_owned())),
                ("noted", count(noted)),
            ],
        )
    }

    fn invite(&self, command: &CommandRef, id: &str) -> SemanticCommandResult {
        let mode = self.mode;
        let mut rows = self.rows.borrow_mut();
        let Some(team) = rows
            .iter()
            .find(|row| row.id == id)
            .map(|row| row.team.clone())
        else {
            return SemanticCommandResult::undeclared();
        };
        for row in rows.iter_mut() {
            if row.id == id {
                row.on_hold = mode == Mode::InviteHoldsSubject;
                continue;
            }
            let selected = mode == Mode::InviteHoldsEveryTeam || row.team == team;
            if selected && mode != Mode::InviteSkipsOthers {
                row.on_hold = true;
            }
        }
        took(
            command,
            "invited",
            "demo.desk.Invited",
            vec![("session_id", Node::Text(id.to_owned()))],
        )
    }
}

impl ConformanceTarget for Desk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("set-effects", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
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
            "demo.desk.Open" => self.open(&command, text(input, "team"), text(input, "note")),
            "demo.desk.Park" => self.single(
                &command,
                &text(input, "session_id"),
                ("parked", "demo.desk.SessionParked", "Parked"),
            ),
            "demo.desk.Close" => self.single(
                &command,
                &text(input, "session_id"),
                ("closed", "demo.desk.SessionClosed", "Ended"),
            ),
            "demo.desk.EndTeam" => self.end(&command, &text(input, "team"), &text(input, "note")),
            "demo.desk.NoteTeam" => self.note(&command, &text(input, "team"), &text(input, "note")),
            "demo.desk.Invite" => self.invite(&command, &text(input, "session_id")),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), "demo.desk.SessionDetails");
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|row| {
                    BTreeMap::from([
                        ("session_id".to_owned(), Node::Text(row.id.clone())),
                        ("team".to_owned(), Node::Text(row.team.clone())),
                        ("note".to_owned(), Node::Text(row.note.clone())),
                        ("on_hold".to_owned(), Node::Bool(row.on_hold)),
                        ("state".to_owned(), Node::Text(row.state.to_owned())),
                    ])
                })
                .collect(),
            total: None,
        })
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

// ---- the suite ----------------------------------------------------------------------------------

#[test]
fn every_set_effect_is_synthesized_without_a_refusal() {
    let synthesis = synthesis_of(MODEL);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    for id in [ENDED, NOTED, INVITED] {
        scenario(&synthesis.suite, id);
    }
}

/// The instances a scenario's arrangement captured, in order.
fn captured(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance { instance, .. } => Some(instance.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_set_move_arranges_three_matching_rows_a_non_matching_one_and_one_outside_the_from_states() {
    let synthesis = synthesis_of(MODEL);
    let ended = scenario(&synthesis.suite, ENDED);
    assert!(
        captured(ended).len() >= 5,
        "three matching, one not matching, one parked: {:#?}",
        ended.steps
    );
    assert!(
        ended.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "demo.desk.Park")),
        "one matching row is parked first: {:#?}",
        ended.steps
    );
    let count = ended.steps.iter().find_map(|step| match step {
        ScenarioStep::ExpectEvent { event, payload, .. }
            if event.to_string() == "demo.desk.TeamEnded" =>
        {
            payload.get("ended").cloned()
        }
        _ => None,
    });
    assert_eq!(count, Some(count_node(3)), "{:#?}", ended.steps);
}

fn count_node(n: i64) -> Node {
    Node::Number(Number::from(n))
}

#[test]
fn a_correct_target_passes_the_whole_suite() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, &Desk::new(Mode::Correct));
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
    for id in [ENDED, NOTED, INVITED] {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "{id}");
    }
}

#[test]
fn each_wrong_set_effect_fails_the_scenario_of_its_branch() {
    let synthesis = synthesis_of(MODEL);
    for (mode, id) in [
        (Mode::EndIgnoresFilter, ENDED),
        (Mode::EndSkipsOne, ENDED),
        (Mode::EndMiscounts, ENDED),
        (Mode::EndIgnoresFrom, ENDED),
        (Mode::EndIgnoresSets, ENDED),
        (Mode::NoteIgnoresFilter, NOTED),
        (Mode::NoteSkipsOne, NOTED),
        (Mode::NoteMiscounts, NOTED),
        (Mode::InviteHoldsSubject, INVITED),
        (Mode::InviteSkipsOthers, INVITED),
        (Mode::InviteHoldsEveryTeam, INVITED),
    ] {
        let statuses = run(&synthesis.suite, &Desk::new(mode));
        assert_ne!(
            statuses.get(id),
            Some(&Status::Passed),
            "{mode:?} must fail {id}: {statuses:#?}"
        );
    }
}

#[test]
fn a_set_effect_no_immediate_view_observes_is_named() {
    let text = MODEL.replace(
        "  - name: demo.desk.SessionDetails
    source: demo.desk.Session
    consistency: read_your_writes",
        "  - name: demo.desk.SessionDetails
    source: demo.desk.Session
    consistency: eventual",
    );
    assert_ne!(text, MODEL);
    let synthesis = synthesis_of(&text);
    let unobserved = synthesis
        .notes
        .iter()
        .any(|note| note.to_string().contains("EndTeam"))
        || synthesis
            .refusals
            .iter()
            .any(|refusal| refusal.to_string().contains("EndTeam"));
    assert!(
        unobserved,
        "a set effect no immediate view observes is named, never passed silently: notes {:#?} \
         refusals {:#?}",
        synthesis.notes, synthesis.refusals
    );
}

#[test]
fn each_set_outcome_is_sent_again_where_its_filter_selects_no_row_and_reports_zero() {
    let synthesis = synthesis_of(MODEL);
    for (id, command, event, field) in [
        (ENDED, "demo.desk.EndTeam", "demo.desk.TeamEnded", "ended"),
        (NOTED, "demo.desk.NoteTeam", "demo.desk.TeamNoted", "noted"),
    ] {
        let scenario = scenario(&synthesis.suite, id);
        let sent = scenario
            .steps
            .iter()
            .filter(|step| {
                matches!(step,
                ScenarioStep::ExecuteCommand { command: sent, .. } if sent.to_string() == command)
            })
            .count();
        assert_eq!(
            sent, 2,
            "the command, then the zero-match call: {:#?}",
            scenario.steps
        );
        let counts: Vec<Option<Node>> = scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExpectEvent {
                    event: named,
                    payload,
                    ..
                } if named.to_string() == event => Some(payload.get(field).cloned()),
                _ => None,
            })
            .collect();
        assert_eq!(counts, [Some(count_node(3)), Some(count_node(0))], "{id}");
    }
}

// A foreign key need not declare ownership. These IDs are generated by the target, so a witness
// cannot guess the user a session belongs to from the command's sample input.
mod issue_288 {
    use super::*;

    const MODEL: &str = r"
format: ess/16
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
      - {name: team, type: String}
      - {name: ended, type: Boolean}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.users.UserAdded
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.UserDeactivated
    fields: [{name: user_id, type: demo.users.UserId}]
  - name: demo.users.SessionStarted
    fields: [{name: session_id, type: demo.users.SessionId}]
errors: [{name: demo.users.NotActive}]
actors:
  - name: demo.users.Admin
    may: [demo.users.AddUser, demo.users.DeactivateUser, demo.users.StartSession]
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
    input: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}]
    outcomes:
      - name: started
        creates: demo.users.Session
        instance: session_id
        emits: [demo.users.SessionStarted]
        payload: {demo.users.SessionStarted: {session_id: {generated: true}}}
        sets: {user_id: input.user_id, team: input.team, ended: false}
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
            sets: {ended: true}
      - {name: not-active, wrong_state: true, error: demo.users.NotActive}
views:
  - name: demo.users.Users
    source: demo.users.User
    consistency: read_your_writes
    fields: [{name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: state, type: demo.users.User.State}]
  - name: demo.users.Sessions
    source: demo.users.Session
    consistency: read_your_writes
    fields: [{name: session_id, type: demo.users.SessionId}, {name: user_id, type: demo.users.UserId}, {name: team, type: String}, {name: ended, type: Boolean}, {name: state, type: demo.users.Session.State}]
";
    const DEACTIVATED: &str = "demo.users.DeactivateUser/outcome/deactivated";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Mutation {
        None,
        IgnoreIdentity,
        IgnoreTeam,
        OnlyOne,
    }

    #[derive(Default)]
    struct User {
        id: String,
        team: String,
        inactive: bool,
    }

    struct Session {
        id: String,
        user_id: String,
        team: String,
        ended: bool,
    }

    struct UserDesk {
        users: RefCell<Vec<User>>,
        sessions: RefCell<Vec<Session>>,
        next: RefCell<usize>,
        observations: RefCell<Vec<(usize, usize, usize)>>,
        mixed: bool,
        mutation: Mutation,
    }

    impl UserDesk {
        fn new(mixed: bool, mutation: Mutation) -> Self {
            Self {
                users: RefCell::default(),
                sessions: RefCell::default(),
                next: RefCell::new(7919),
                observations: RefCell::default(),
                mixed,
                mutation,
            }
        }

        fn fresh(&self) -> String {
            let mut next = self.next.borrow_mut();
            *next += 17;
            format!("target-generated-{next}")
        }
    }

    impl ConformanceTarget for UserDesk {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            Ok(ImplementationIdentity::new("captured-user-affects", "1"))
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
            let input = &request.input;
            let command = &request.command;
            Ok(match command.to_string().as_str() {
                "demo.users.AddUser" => {
                    let id = self.fresh();
                    self.users.borrow_mut().push(User {
                        id: id.clone(),
                        team: text(input, "team"),
                        inactive: false,
                    });
                    took(
                        command,
                        "added",
                        "demo.users.UserAdded",
                        vec![("user_id", Node::Text(id))],
                    )
                }
                "demo.users.StartSession" => {
                    let id = self.fresh();
                    let user_id = text(input, "user_id");
                    // This assertion audits the witness: even the decoy must name a real user.
                    assert!(
                        self.users.borrow().iter().any(|user| user.id == user_id),
                        "session input guessed an unarranged user: {user_id}"
                    );
                    self.sessions.borrow_mut().push(Session {
                        id: id.clone(),
                        user_id,
                        team: text(input, "team"),
                        ended: false,
                    });
                    took(
                        command,
                        "started",
                        "demo.users.SessionStarted",
                        vec![("session_id", Node::Text(id))],
                    )
                }
                "demo.users.DeactivateUser" => {
                    let id = text(input, "user_id");
                    let mut users = self.users.borrow_mut();
                    let user = users
                        .iter_mut()
                        .find(|user| user.id == id)
                        .expect("captured user");
                    user.inactive = true;
                    let mut sessions = self.sessions.borrow_mut();
                    let matching = sessions
                        .iter()
                        .filter(|row| row.user_id == id && (!self.mixed || row.team == user.team))
                        .count();
                    let other_user = sessions
                        .iter()
                        .filter(|row| row.user_id != id && (!self.mixed || row.team == user.team))
                        .count();
                    let other_team = sessions
                        .iter()
                        .filter(|row| row.user_id == id && row.team != user.team)
                        .count();
                    self.observations
                        .borrow_mut()
                        .push((matching, other_user, other_team));
                    let mut changed = 0;
                    for row in sessions.iter_mut() {
                        let selected = (row.user_id == id
                            || self.mutation == Mutation::IgnoreIdentity)
                            && (!self.mixed
                                || row.team == user.team
                                || self.mutation == Mutation::IgnoreTeam);
                        if selected && (self.mutation != Mutation::OnlyOne || changed == 0) {
                            row.ended = true;
                            changed += 1;
                        }
                    }
                    took(
                        command,
                        "deactivated",
                        "demo.users.UserDeactivated",
                        vec![("user_id", Node::Text(id))],
                    )
                }
                other => panic!("unexpected command {other}"),
            })
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            let rows = match request.view.to_string().as_str() {
                "demo.users.Users" => self
                    .users
                    .borrow()
                    .iter()
                    .map(|row| {
                        BTreeMap::from([
                            ("user_id".into(), Node::Text(row.id.clone())),
                            ("team".into(), Node::Text(row.team.clone())),
                            (
                                "state".into(),
                                Node::Text(if row.inactive { "Inactive" } else { "Active" }.into()),
                            ),
                        ])
                    })
                    .collect(),
                "demo.users.Sessions" => self
                    .sessions
                    .borrow()
                    .iter()
                    .map(|row| {
                        BTreeMap::from([
                            ("session_id".into(), Node::Text(row.id.clone())),
                            ("user_id".into(), Node::Text(row.user_id.clone())),
                            ("team".into(), Node::Text(row.team.clone())),
                            ("ended".into(), Node::Bool(row.ended)),
                            ("state".into(), Node::Text("Live".into())),
                        ])
                    })
                    .collect(),
                other => panic!("unexpected view {other}"),
            };
            Ok(SemanticViewResult { rows, total: None })
        }
        fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
            Err(TargetError::unsupported("external", "unused"))
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

    fn reverse(predicate: &mut ess_primitives::predicate::Predicate) {
        use ess_primitives::predicate::Predicate;
        match predicate {
            Predicate::All(children) => children.iter_mut().for_each(reverse),
            Predicate::Compare { left, right, .. } => std::mem::swap(left, right),
            other => panic!("unexpected reversal {other:?}"),
        }
    }

    fn compiled(filter: &str) -> EssIr {
        // Compact syntax reads a bare RHS word as a literal. Build reversed fact operands in
        // the public typed source AST, then run the same domain validation and compilation.
        let reversed = filter.contains("subject.user_id == user_id")
            || filter.contains("input.user_id == user_id");
        let filter = filter
            .replace("subject.user_id == user_id", "user_id == subject.user_id")
            .replace("input.user_id == user_id", "user_id == input.user_id")
            .replace("subject.team == team", "team == subject.team");
        let model = MODEL.replace("user_id == subject.user_id", &filter);
        let mut raw = RawSpecFile::parse(&model).expect("parses");
        if reversed {
            for command in &mut raw.commands {
                for outcome in &mut command.outcomes {
                    for affect in &mut outcome.affects {
                        reverse(&mut affect.filter);
                    }
                }
            }
        }
        let spec = Specification::assemble([(Source::new("captured-user-affects.yaml"), raw)])
            .expect("admitted");
        compile(&spec, &SourceMap::new()).expect("compiled")
    }

    fn suite(filter: &str) -> ConformanceSuite {
        let ir = compiled(filter);
        let mut synthesis = ess_conformance::synthesize::synthesize(&ir);
        assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
        scenario(&synthesis.suite, DEACTIVATED);
        synthesis
            .suite
            .scenarios
            .retain(|id, _| id.to_string() == DEACTIVATED);
        synthesis.suite
    }

    #[test]
    fn the_interpreter_executes_captured_subject_identity_filters() {
        for filter in [
            "user_id == subject.user_id",
            "subject.user_id == user_id",
            "user_id == input.user_id",
            "input.user_id == user_id",
            "{all: [user_id == subject.user_id, team == subject.team]}",
            "{all: [subject.user_id == user_id, subject.team == team]}",
        ] {
            let target = ess_conformance::interpret::Interpreted::for_model(compiled(filter));
            assert_eq!(
                run(&suite(filter), &target).get(DEACTIVATED),
                Some(&Status::Passed),
                "{filter}"
            );
        }
    }

    fn honest(filter: &str, mixed: bool) {
        let suite = suite(filter);
        let target = UserDesk::new(mixed, Mutation::None);
        assert_eq!(run(&suite, &target).get(DEACTIVATED), Some(&Status::Passed));
        assert!(
            target
                .observations
                .borrow()
                .iter()
                .any(|&(matching, decoy, team)| matching >= 2
                    && decoy >= 1
                    && (!mixed || team >= 1)),
            "must actually execute multiple matches and each conjunct's decoy: {:?}",
            target.observations
        );
    }

    #[test]
    fn issue_288_affects_subject_identity_uses_the_captured_user() {
        honest("user_id == subject.user_id", false);
        honest("subject.user_id == user_id", false);
    }

    #[test]
    fn issue_288_affects_does_not_touch_another_subjects_rows() {
        let suite = suite("user_id == subject.user_id");
        for mutation in [Mutation::IgnoreIdentity, Mutation::OnlyOne] {
            let target = UserDesk::new(false, mutation);
            assert_eq!(
                run(&suite, &target).get(DEACTIVATED),
                Some(&Status::Failed),
                "{mutation:?}"
            );
        }
    }

    #[test]
    fn issue_288_subject_identity_and_input_identity_witness_equivalently() {
        for filter in [
            "user_id == input.user_id",
            "input.user_id == user_id",
            "user_id == subject.user_id",
        ] {
            honest(filter, false);
        }
    }

    #[test]
    fn issue_288_subject_identity_keeps_each_stored_conjunct_witness() {
        for filter in [
            "{all: [user_id == subject.user_id, team == subject.team]}",
            "{all: [subject.user_id == user_id, subject.team == team]}",
        ] {
            honest(filter, true);
            let suite = suite(filter);
            for mutation in [
                Mutation::IgnoreIdentity,
                Mutation::IgnoreTeam,
                Mutation::OnlyOne,
            ] {
                assert_eq!(
                    run(&suite, &UserDesk::new(true, mutation)).get(DEACTIVATED),
                    Some(&Status::Failed),
                    "{filter}; {mutation:?}"
                );
            }
        }
    }
}

#[test]
fn the_actual_interpreter_executes_the_complete_set_effect_suite() {
    let ir = ir_of(MODEL);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert_eq!(synthesis.refusals.len(), 0);
    let statuses = run(
        &synthesis.suite,
        &ess_conformance::interpret::Interpreted::for_model(ir),
    );
    assert_eq!(statuses.len(), 14);
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
    for id in [ENDED, NOTED, INVITED] {
        assert_eq!(statuses.get(id), Some(&Status::Passed));
    }
}
