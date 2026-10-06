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
                        reverse(
                            affect
                                .filter
                                .as_mut()
                                .and_then(|filter| filter.predicate_mut())
                                .expect("the fixture's filter parses"),
                        );
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

/// Deleting the selected rows (ess/23, beyond10x/ess#452): `RevokeTokens` deletes every token of a
/// user and scope, and `DeleteUser` deletes a user and, in an `affects:` entry, every token it owns.
/// Each removed row is read absent, each other row as arranged; one in-memory target implements
/// the model, and each mutation breaks it one way.
mod issue_452 {
    use super::*;

    const MODEL: &str =
        include_str!("../../../specify/ess-compiler/tests/fixtures/set-deletes.yaml");
    const REVOKED: &str = "demo.auth.RevokeTokens/outcome/revoked";
    const DELETED: &str = "demo.auth.DeleteUser/outcome/deleted";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Mutation {
        None,
        /// Removes nothing it selects, and reports the count it should have.
        DeleteNothing,
        /// Revokes every token of the user, whatever its scope.
        RevokeIgnoresScope,
        /// Revokes the first selected token only, and reports the count it should have.
        RevokeOnlyOne,
        /// Reports three revoked, whatever it revoked.
        RevokeReportsThree,
        /// Deleting a user removes every token, whoever owns it.
        DeleteEveryToken,
    }

    struct Token {
        id: String,
        user_id: String,
        scope: String,
    }

    struct AuthDesk {
        users: RefCell<Vec<(String, String)>>,
        tokens: RefCell<Vec<Token>>,
        next: RefCell<usize>,
        mutation: Mutation,
    }

    impl AuthDesk {
        fn new(mutation: Mutation) -> Self {
            Self {
                users: RefCell::default(),
                tokens: RefCell::default(),
                next: RefCell::new(4051),
                mutation,
            }
        }

        fn fresh(&self) -> String {
            let mut next = self.next.borrow_mut();
            *next += 13;
            format!("target-generated-{next}")
        }

        /// Removes the tokens `selected` picks, as the mutation allows, and says how many it
        /// should have removed.
        fn remove(&self, selected: impl Fn(&Token) -> bool) -> usize {
            let mut tokens = self.tokens.borrow_mut();
            let due = tokens.iter().filter(|token| selected(token)).count();
            if self.mutation == Mutation::DeleteNothing {
                return due;
            }
            let mut removed = 0;
            tokens.retain(|token| {
                let gone =
                    selected(token) && (self.mutation != Mutation::RevokeOnlyOne || removed == 0);
                removed += usize::from(gone);
                !gone
            });
            due
        }
    }

    impl ConformanceTarget for AuthDesk {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            Ok(ImplementationIdentity::new("set-deletes", "1"))
        }
        fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            self.users.borrow_mut().clear();
            self.tokens.borrow_mut().clear();
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
                "demo.auth.AddUser" => {
                    let id = self.fresh();
                    self.users
                        .borrow_mut()
                        .push((id.clone(), text(input, "team")));
                    took(
                        command,
                        "added",
                        "demo.auth.UserAdded",
                        vec![("user_id", Node::Text(id))],
                    )
                }
                "demo.auth.IssueToken" => {
                    let id = self.fresh();
                    self.tokens.borrow_mut().push(Token {
                        id: id.clone(),
                        user_id: text(input, "user_id"),
                        scope: text(input, "scope"),
                    });
                    took(
                        command,
                        "issued",
                        "demo.auth.TokenIssued",
                        vec![("token_id", Node::Text(id))],
                    )
                }
                "demo.auth.RevokeTokens" => {
                    let (user_id, scope) = (text(input, "user_id"), text(input, "scope"));
                    let mut revoked = self.remove(|token| {
                        token.user_id == user_id
                            && (self.mutation == Mutation::RevokeIgnoresScope
                                || token.scope == scope)
                    });
                    if self.mutation == Mutation::RevokeReportsThree {
                        revoked = 3;
                    }
                    took(
                        command,
                        "revoked",
                        "demo.auth.TokensRevoked",
                        vec![
                            ("user_id", Node::Text(user_id)),
                            ("revoked", count(revoked)),
                        ],
                    )
                }
                "demo.auth.DeleteUser" => {
                    let id = text(input, "user_id");
                    let mut users = self.users.borrow_mut();
                    let Some(at) = users.iter().position(|(user, _)| *user == id) else {
                        let mut result = SemanticCommandResult::took(OutcomeRef::new(
                            command.clone(),
                            "no-such-user".parse().unwrap(),
                        ));
                        result.error = Some(DeclaredErrorValue::new(
                            "demo.auth.NoSuchUser".parse().unwrap(),
                        ));
                        result.consistency = Some(
                            ess_primitives::consistency::ConsistencyToken::new("write").unwrap(),
                        );
                        return Ok(result);
                    };
                    users.remove(at);
                    drop(users);
                    self.remove(|token| {
                        token.user_id == id || self.mutation == Mutation::DeleteEveryToken
                    });
                    took(
                        command,
                        "deleted",
                        "demo.auth.UserDeleted",
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
                "demo.auth.Users" => self
                    .users
                    .borrow()
                    .iter()
                    .map(|(id, team)| {
                        BTreeMap::from([
                            ("user_id".into(), Node::Text(id.clone())),
                            ("team".into(), Node::Text(team.clone())),
                        ])
                    })
                    .collect(),
                "demo.auth.Tokens" => self
                    .tokens
                    .borrow()
                    .iter()
                    .map(|token| {
                        BTreeMap::from([
                            ("token_id".into(), Node::Text(token.id.clone())),
                            ("user_id".into(), Node::Text(token.user_id.clone())),
                            ("scope".into(), Node::Text(token.scope.clone())),
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

    fn synthesis() -> ess_conformance::synthesize::Synthesis {
        let synthesis = synthesis_of(MODEL);
        assert!(
            synthesis.refusals.is_empty(),
            "nothing is refused: {:#?}",
            synthesis.refusals
        );
        synthesis
    }

    /// How many rows of `view` the scenario reads absent.
    fn absent(scenario: &ConformanceScenario, view: &str) -> usize {
        scenario
            .steps
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::ExpectSubjectAbsent { view: read, .. }
                    if read.to_string() == view)
            })
            .count()
    }

    /// Every `revoked` count the scenario requires, in order.
    fn counts(scenario: &ConformanceScenario) -> Vec<Option<Node>> {
        scenario
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExpectEvent { event, payload, .. }
                    if event.to_string() == "demo.auth.TokensRevoked" =>
                {
                    Some(payload.get("revoked").cloned())
                }
                _ => None,
            })
            .collect()
    }

    fn fails(mutation: Mutation, id: &str) {
        let statuses = run(&synthesis().suite, &AuthDesk::new(mutation));
        assert_eq!(
            statuses.get(id),
            Some(&Status::Failed),
            "{mutation:?} must fail {id}: {statuses:#?}"
        );
    }

    #[test]
    fn bulk_delete_removes_every_selected_row_and_keeps_the_rest() {
        let synthesis = synthesis();
        let revoked = scenario(&synthesis.suite, REVOKED);
        assert_eq!(
            absent(revoked, "demo.auth.Tokens"),
            3,
            "three selected rows read absent: {:#?}",
            revoked.steps
        );
        assert!(
            captured(revoked).len() >= 5,
            "three selected rows and one per conjunct left out: {:#?}",
            revoked.steps
        );
        assert_eq!(
            counts(revoked).first(),
            Some(&Some(count_node(3))),
            "{:#?}",
            revoked.steps
        );
        let statuses = run(&synthesis.suite, &AuthDesk::new(Mutation::None));
        assert_eq!(
            statuses.get(REVOKED),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
        for mutation in [
            Mutation::DeleteNothing,
            Mutation::RevokeIgnoresScope,
            Mutation::RevokeOnlyOne,
        ] {
            fails(mutation, REVOKED);
        }
    }

    #[test]
    fn bulk_delete_zero_match_is_accepted_with_count_zero() {
        let synthesis = synthesis();
        let revoked = scenario(&synthesis.suite, REVOKED);
        let sent = revoked
            .steps
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.auth.RevokeTokens")
            })
            .count();
        assert_eq!(
            sent, 2,
            "the command, then the zero-match call: {:#?}",
            revoked.steps
        );
        assert_eq!(
            counts(revoked),
            [Some(count_node(3)), Some(count_node(0))],
            "{:#?}",
            revoked.steps
        );
        assert_eq!(
            absent(revoked, "demo.auth.Tokens"),
            3,
            "the zero-match call reads no further row absent: {:#?}",
            revoked.steps
        );
        fails(Mutation::RevokeReportsThree, REVOKED);
    }

    #[test]
    fn affects_delete_entry_removes_owned_rows_with_subject() {
        let synthesis = synthesis();
        let deleted = scenario(&synthesis.suite, DELETED);
        assert!(
            absent(deleted, "demo.auth.Tokens") >= 3,
            "every owned token reads absent: {:#?}",
            deleted.steps
        );
        assert!(
            absent(deleted, "demo.auth.Users") >= 1,
            "the deleted subject reads absent: {:#?}",
            deleted.steps
        );
        let statuses = run(&synthesis.suite, &AuthDesk::new(Mutation::None));
        assert_eq!(
            statuses.get(DELETED),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
        for mutation in [Mutation::DeleteNothing, Mutation::DeleteEveryToken] {
            fails(mutation, DELETED);
        }
    }

    #[test]
    fn set_delete_targets_refuse_by_name() {
        // The conformance half: the interpreted model passes both scenarios, and a target that
        // deletes nothing fails each. Entity Runtime's `SetEffectUnsupported` and the code
        // targets' `MissingRepresentation` are cases of the same name in their own crates.
        let ir = ir_of(MODEL);
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
        let statuses = run(
            &synthesis.suite,
            &ess_conformance::interpret::Interpreted::for_model(ir),
        );
        assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
        for id in [REVOKED, DELETED] {
            assert_eq!(statuses.get(id), Some(&Status::Passed), "{id}");
        }
        let mutant = run(&synthesis.suite, &AuthDesk::new(Mutation::DeleteNothing));
        for id in [REVOKED, DELETED] {
            assert_eq!(mutant.get(id), Some(&Status::Failed), "{id}: {mutant:#?}");
        }
        assert!(
            not_passed(&run(&synthesis.suite, &AuthDesk::new(Mutation::None))).is_empty(),
            "the honest target passes the whole suite"
        );
    }
}

/// One record per element of an input list (ess/23, beyond10x/ess#459): `RunSource` updates its
/// source and, per element of `applied`, updates the `SeenDocument` the element names if held and
/// creates it if not. One in-memory target implements the model; each mutation breaks it one way.
mod issue_459 {
    use super::*;

    const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
    const RAN: &str = "demo.feed.RunSource/outcome/ran";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Mutation {
        None,
        /// Creates a row for an element naming no held row, and leaves a held row as it was.
        CreateOnly,
        /// Updates a held row, and creates none.
        UpdateOnly,
        /// Adds a row for every element, whether or not one is held under its identity.
        AddsRow,
    }

    #[derive(Clone)]
    struct Seen {
        id: Node,
        source: Node,
        hash: Node,
        revision: Node,
    }

    struct FeedDesk {
        sources: RefCell<Vec<(String, String)>>,
        seen: RefCell<Vec<Seen>>,
        next: RefCell<usize>,
        mutation: Mutation,
    }

    impl FeedDesk {
        fn new(mutation: Mutation) -> Self {
            Self {
                sources: RefCell::default(),
                seen: RefCell::default(),
                next: RefCell::new(7001),
                mutation,
            }
        }

        fn refused(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
            let mut result = SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                outcome.parse().unwrap(),
            ));
            result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
            result.consistency =
                Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
            result
        }

        fn run(
            &self,
            command: &CommandRef,
            input: &BTreeMap<String, Node>,
        ) -> SemanticCommandResult {
            let applied = match input.get("applied") {
                Some(Node::Seq(items)) => items.clone(),
                _ => Vec::new(),
            };
            let member = |item: &Node, name: &str| match item {
                Node::Map(members) => members.get(name).cloned().unwrap_or(Node::Null),
                _ => Node::Null,
            };
            let ids: Vec<Node> = applied
                .iter()
                .map(|item| member(item, "document_id"))
                .collect();
            if ids
                .iter()
                .enumerate()
                .any(|(at, id)| ids[..at].contains(id))
            {
                return Self::refused(command, "duplicated", "demo.feed.DuplicateDocument");
            }
            let source = text(input, "source_id");
            let mut sources = self.sources.borrow_mut();
            let Some(held) = sources.iter_mut().find(|(id, _)| *id == source) else {
                return Self::refused(command, "no-such-source", "demo.feed.NoSuchSource");
            };
            held.1 = text(input, "label");
            let mut seen = self.seen.borrow_mut();
            for item in &applied {
                let row = Seen {
                    id: member(item, "document_id"),
                    source: Node::Text(source.clone()),
                    hash: member(item, "content_hash"),
                    revision: member(item, "revision"),
                };
                let at = seen.iter().position(|held| held.id == row.id);
                match (at, self.mutation) {
                    (_, Mutation::AddsRow) => seen.push(row),
                    (Some(_), Mutation::CreateOnly) | (None, Mutation::UpdateOnly) => {}
                    (Some(at), _) => seen[at] = row,
                    (None, _) => seen.push(row),
                }
            }
            took(
                command,
                "ran",
                "demo.feed.SourceRan",
                vec![("source_id", Node::Text(source))],
            )
        }
    }

    impl ConformanceTarget for FeedDesk {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            Ok(ImplementationIdentity::new("set-each", "1"))
        }
        fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            self.sources.borrow_mut().clear();
            self.seen.borrow_mut().clear();
            Ok(())
        }
        fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
            Ok(())
        }
        fn execute_command(
            &self,
            request: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            let command = &request.command;
            Ok(match command.to_string().as_str() {
                "demo.feed.AddSource" => {
                    let mut next = self.next.borrow_mut();
                    *next += 11;
                    let id = format!("target-source-{next}");
                    self.sources
                        .borrow_mut()
                        .push((id.clone(), text(&request.input, "label")));
                    took(
                        command,
                        "added",
                        "demo.feed.SourceAdded",
                        vec![("source_id", Node::Text(id))],
                    )
                }
                "demo.feed.RunSource" => self.run(command, &request.input),
                other => panic!("unexpected command {other}"),
            })
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            let rows = match request.view.to_string().as_str() {
                "demo.feed.Sources" => self
                    .sources
                    .borrow()
                    .iter()
                    .map(|(id, label)| {
                        BTreeMap::from([
                            ("source_id".into(), Node::Text(id.clone())),
                            ("label".into(), Node::Text(label.clone())),
                        ])
                    })
                    .collect(),
                "demo.feed.SeenDocuments" => self
                    .seen
                    .borrow()
                    .iter()
                    .map(|row| {
                        BTreeMap::from([
                            ("document_id".into(), row.id.clone()),
                            ("source_id".into(), row.source.clone()),
                            ("content_hash".into(), row.hash.clone()),
                            ("revision".into(), row.revision.clone()),
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

    fn synthesis() -> ess_conformance::synthesize::Synthesis {
        let synthesis = synthesis_of(MODEL);
        assert!(
            synthesis.refusals.is_empty(),
            "nothing is refused: {:#?}",
            synthesis.refusals
        );
        synthesis
    }

    /// The `RunSource` calls of a scenario, and its one-row snapshots of `SeenDocuments`.
    fn shape(scenario: &ConformanceScenario) -> (usize, usize) {
        let sent = scenario
            .steps
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.feed.RunSource")
            })
            .count();
        let snapshots = scenario
            .steps
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::SnapshotSubject { view, .. }
                    if view.to_string() == "demo.feed.SeenDocuments")
            })
            .count();
        (sent, snapshots)
    }

    #[test]
    fn each_entry_scenario_reads_held_new_and_decoy_rows() {
        let synthesis = synthesis();
        let ran = scenario(&synthesis.suite, RAN);
        let (sent, snapshots) = shape(ran);
        assert!(
            sent >= 2,
            "a first call puts the held row and the decoy in place, then the command: {:#?}",
            ran.steps
        );
        assert!(
            snapshots >= 2,
            "one row per element's identity: {:#?}",
            ran.steps
        );
        let statuses = run(&synthesis.suite, &FeedDesk::new(Mutation::AddsRow));
        assert_eq!(
            statuses.get(RAN),
            Some(&Status::Failed),
            "a second row for a held identity fails: {statuses:#?}"
        );
    }

    /// The fixture with an enum member of `variants` on the element and the row, which the entry
    /// reads.
    fn with_enum(variants: &str) -> String {
        let model = MODEL
            .replacen(
                "  - name: demo.feed.AppliedDocument\n",
                &format!(
                    "  - {{name: demo.feed.Kind, kind: enum, variants: [{variants}]}}\n  - name: demo.feed.AppliedDocument\n"
                ),
                1,
            )
            .replacen(
                "      - {name: revision, type: Integer}\nentities:",
                "      - {name: revision, type: Integer}\n      - {name: kind, type: demo.feed.Kind}\nentities:",
                1,
            )
            .replacen(
                "      - {name: revision, type: Integer}\n    lifecycle: {initial: Seen",
                "      - {name: revision, type: Integer}\n      - {name: kind, type: demo.feed.Kind}\n    lifecycle: {initial: Seen",
                1,
            )
            .replacen("revision: doc.revision}", "revision: doc.revision, kind: doc.kind}", 1);
        format!("{model}      - {{name: kind, type: demo.feed.Kind}}\n")
    }

    /// The held row's two elements sit at adjacent distinctions, so an enum member of an even
    /// number of variants differs between them as an odd one does, and the interpreter passes the
    /// scenario that reads it.
    #[test]
    fn each_entry_reading_an_even_variant_enum_member_is_witnessed() {
        for variants in ["Pdf, Html", "Pdf, Html, Text", "Pdf, Html, Text, Csv"] {
            let model = with_enum(variants);
            assert!(model.contains("kind: doc.kind"), "{model}");
            let ir = ir_of(&model);
            let synthesis = ess_conformance::synthesize::synthesize(&ir);
            assert!(
                synthesis.refusals.is_empty(),
                "[{variants}]: {:#?}",
                synthesis.refusals
            );
            let statuses = run(
                &synthesis.suite,
                &ess_conformance::interpret::Interpreted::for_model(ir),
            );
            assert_eq!(
                statuses.get(RAN),
                Some(&Status::Passed),
                "[{variants}]: {statuses:#?}"
            );
        }
    }

    #[test]
    fn each_entry_targets_refuse_by_name() {
        // The conformance half: the interpreted model passes the scenario, and a create-only and
        // an update-only target each fail it. Entity Runtime's `SetEffectUnsupported` and the code
        // targets' `MissingRepresentation` are cases of the same name in their own crates.
        let ir = ir_of(MODEL);
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
        let statuses = run(
            &synthesis.suite,
            &ess_conformance::interpret::Interpreted::for_model(ir),
        );
        assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
        assert_eq!(statuses.get(RAN), Some(&Status::Passed));
        let honest = run(&synthesis.suite, &FeedDesk::new(Mutation::None));
        assert!(
            not_passed(&honest).is_empty(),
            "the honest target passes: {honest:#?}"
        );
        for mutation in [Mutation::CreateOnly, Mutation::UpdateOnly] {
            let mutant = run(&synthesis.suite, &FeedDesk::new(mutation));
            assert_eq!(
                mutant.get(RAN),
                Some(&Status::Failed),
                "{mutation:?}: {mutant:#?}"
            );
        }
    }
}
