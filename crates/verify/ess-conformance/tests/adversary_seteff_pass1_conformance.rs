//! Adversary pass 1 on `story:set-effects-over-filtered-instances` (ess/16; beyond10x/ess#167,
//! #175): wrong targets the synthesized suite should fail and does not.
//!
//! The model is the unit's own fixture, edited per case. One in-memory target implements it; each
//! wrong mode breaks it one way.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

const ENDED: &str = "demo.desk.EndTeam/outcome/ended";
const NOTED: &str = "demo.desk.NoteTeam/outcome/noted";
const INVITED: &str = "demo.desk.Invite/outcome/invited";
const TRANSFERRED: &str = "demo.desk.Transfer/outcome/transferred";

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds:\n{from}");
    text.replacen(from, to, 1)
}

fn synthesis_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).expect("the model compiles");
    ess_conformance::synthesize::synthesize(&ir)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// Notes every session, whatever its team, and reports only the matching ones as noted.
    NoteIgnoresFilter,
    /// Ends every `Open` session, whatever its team, and reports only the matching ones as ended.
    EndIgnoresFilter,
    /// Notes every session of the desk, whatever its team (drops one conjunct).
    NoteIgnoresTeam,
    /// Notes every session of the team, whatever its desk (drops the other conjunct).
    NoteIgnoresDesk,
    /// Holds every other session of the desk, whatever its team: ignores `subject.team`.
    InviteIgnoresSubjectTeam,
    /// Holds every other session of the team, whatever its desk: ignores `subject.desk`.
    InviteIgnoresSubjectDesk,
    /// Reads `subject.team` after the branch's own `sets:` rather than before.
    TransferReadsSubjectAfter,
    /// Answers with an undeclared refusal when no row matches, although zero matches is accepted.
    EndRefusesNone,
    /// Reports two ended, whatever it ended.
    EndCountsTwo,
}

#[derive(Clone, Debug)]
struct Row {
    id: String,
    team: String,
    desk: String,
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

    fn open(&self, command: &CommandRef, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let mut next = self.next.borrow_mut();
        *next += 1;
        let id = format!("session-{next}");
        let team = text(input, "team");
        self.rows.borrow_mut().push(Row {
            id: id.clone(),
            team: team.clone(),
            desk: text(input, "desk"),
            note: text(input, "note"),
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

    fn end(&self, command: &CommandRef, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let (team, note) = (text(input, "team"), text(input, "note"));
        let mode = self.mode;
        let mut rows = self.rows.borrow_mut();
        let mut ended = 0;
        for row in rows.iter_mut() {
            let matches = mode == Mode::EndIgnoresFilter || row.team == team;
            if !(matches && row.state == "Open") {
                continue;
            }
            row.state = "Ended";
            note.clone_into(&mut row.note);
            ended += usize::from(row.team == team);
        }
        if mode == Mode::EndRefusesNone && ended == 0 {
            return SemanticCommandResult::undeclared();
        }
        if mode == Mode::EndCountsTwo {
            ended = 2;
        }
        took(
            command,
            "ended",
            "demo.desk.TeamEnded",
            vec![("team", Node::Text(team)), ("ended", count(ended))],
        )
    }

    fn note(&self, command: &CommandRef, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let (team, desk, note) = (
            text(input, "team"),
            text(input, "desk"),
            text(input, "note"),
        );
        let mode = self.mode;
        let mut rows = self.rows.borrow_mut();
        let mut noted = 0;
        for row in rows.iter_mut() {
            let team_ok =
                matches!(mode, Mode::NoteIgnoresFilter | Mode::NoteIgnoresTeam) || row.team == team;
            let desk_ok = input.get("desk").is_none()
                || matches!(mode, Mode::NoteIgnoresFilter | Mode::NoteIgnoresDesk)
                || row.desk == desk;
            if !(team_ok && desk_ok) {
                continue;
            }
            note.clone_into(&mut row.note);
            noted +=
                usize::from(row.team == team && (input.get("desk").is_none() || row.desk == desk));
        }
        took(
            command,
            "noted",
            "demo.desk.TeamNoted",
            vec![("team", Node::Text(team)), ("noted", count(noted))],
        )
    }

    /// `Invite` (hold the others of the subject's team, and desk where the filter says so) and
    /// `Transfer` (move the subject to another team, then hold the others of its old team).
    fn hold_others(
        &self,
        command: &CommandRef,
        input: &BTreeMap<String, Node>,
        by_desk: bool,
        (outcome, event): (&str, &str),
    ) -> SemanticCommandResult {
        let mode = self.mode;
        let id = text(input, "session_id");
        let mut rows = self.rows.borrow_mut();
        let Some((before_team, before_desk)) = rows
            .iter()
            .find(|row| row.id == id)
            .map(|row| (row.team.clone(), row.desk.clone()))
        else {
            return SemanticCommandResult::undeclared();
        };
        let transfer = outcome == "transferred";
        let subject_team = if transfer && mode == Mode::TransferReadsSubjectAfter {
            text(input, "team")
        } else {
            before_team
        };
        for row in rows.iter_mut() {
            if row.id == id {
                if transfer {
                    row.team = text(input, "team");
                } else {
                    row.on_hold = false;
                }
                continue;
            }
            let team_ok = mode == Mode::InviteIgnoresSubjectTeam || row.team == subject_team;
            let desk_ok =
                !by_desk || mode == Mode::InviteIgnoresSubjectDesk || row.desk == before_desk;
            if team_ok && desk_ok {
                row.on_hold = true;
            }
        }
        took(
            command,
            outcome,
            event,
            vec![("session_id", Node::Text(id.clone()))],
        )
    }
}

struct Target {
    desk: Desk,
    by_desk: bool,
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("set-effects-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.desk.rows.borrow_mut().clear();
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
        let desk = &self.desk;
        Ok(match command.to_string().as_str() {
            "demo.desk.Open" => desk.open(&command, input),
            "demo.desk.Park" => desk.single(
                &command,
                &text(input, "session_id"),
                ("parked", "demo.desk.SessionParked", "Parked"),
            ),
            "demo.desk.Close" => desk.single(
                &command,
                &text(input, "session_id"),
                ("closed", "demo.desk.SessionClosed", "Ended"),
            ),
            "demo.desk.EndTeam" => desk.end(&command, input),
            "demo.desk.NoteTeam" => desk.note(&command, input),
            "demo.desk.Invite" => desk.hold_others(
                &command,
                input,
                self.by_desk,
                ("invited", "demo.desk.Invited"),
            ),
            "demo.desk.Transfer" => desk.hold_others(
                &command,
                input,
                false,
                ("transferred", "demo.desk.Transferred"),
            ),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .desk
                .rows
                .borrow()
                .iter()
                .map(|row| {
                    BTreeMap::from([
                        ("session_id".to_owned(), Node::Text(row.id.clone())),
                        ("team".to_owned(), Node::Text(row.team.clone())),
                        ("desk".to_owned(), Node::Text(row.desk.clone())),
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
        Err(TargetError::unsupported("external", "none declared"))
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

fn run(suite: &ConformanceSuite, mode: Mode, by_desk: bool) -> BTreeMap<String, Status> {
    let target = Target {
        desk: Desk::new(mode),
        by_desk,
    };
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn has(suite: &ConformanceSuite, id: &str) -> bool {
    suite.scenarios.keys().any(|key| key.to_string() == id)
}

/// Either the branch's scenario is refused (never green without an observation), or a correct
/// target passes it and the wrong one does not.
fn wrong_mode_is_caught(text: &str, id: &str, mode: Mode, by_desk: bool) {
    let synthesis = synthesis_of(text);
    if !has(&synthesis.suite, id) {
        let command = id.split('/').next().unwrap();
        assert!(
            synthesis
                .refusals
                .iter()
                .any(|refusal| refusal.to_string().contains(command))
                || synthesis
                    .notes
                    .iter()
                    .any(|note| note.to_string().contains(command)),
            "{id} is neither filed nor refused: {:#?}",
            synthesis.refusals
        );
        eprintln!("{id}: refused, not filed");
        return;
    }
    let correct = run(&synthesis.suite, Mode::Correct, by_desk);
    assert_eq!(
        correct.get(id),
        Some(&Status::Passed),
        "a correct target passes {id}: {correct:#?}"
    );
    let wrong = run(&synthesis.suite, mode, by_desk);
    eprintln!("{id}: filed; {mode:?} -> {:?}", wrong.get(id));
    assert_ne!(
        wrong.get(id),
        Some(&Status::Passed),
        "{mode:?} must fail {id}, and the scenario filed for it passes"
    );
}

// ---- the only immediate view does not publish what `sets:` writes -----------------------------

const VIEW_NOTE: &str = "      - {name: note, type: demo.desk.Note}
      - {name: on_hold, type: Boolean}
      - {name: state, type: demo.desk.Session.State}";

/// `NoteTeam` updates `note` on every session of a team; the one view that can read a row back does
/// not publish `note`. Nothing the scenario reads can tell a noted row from an un-noted one, so the
/// scenario must be refused rather than filed and passed by a target that notes every team.
#[test]
fn a_set_update_whose_sets_no_view_publishes_is_not_green_without_an_observation() {
    let text = edited(
        MODEL,
        VIEW_NOTE,
        "      - {name: on_hold, type: Boolean}
      - {name: state, type: demo.desk.Session.State}",
    );
    wrong_mode_is_caught(&text, NOTED, Mode::NoteIgnoresFilter, false);
}

/// Control: with the fixture's own view the same wrong targets are caught by the rows read back.
#[test]
fn control_the_leaking_targets_are_caught_when_the_view_publishes_the_fields() {
    wrong_mode_is_caught(MODEL, NOTED, Mode::NoteIgnoresFilter, false);
    wrong_mode_is_caught(MODEL, ENDED, Mode::EndIgnoresFilter, false);
}

/// The same for a set move whose view publishes neither the state nor what `sets:` writes.
#[test]
fn a_set_move_no_view_can_see_is_not_green_without_an_observation() {
    let text = edited(MODEL, VIEW_NOTE, "      - {name: on_hold, type: Boolean}");
    wrong_mode_is_caught(&text, ENDED, Mode::EndIgnoresFilter, false);
}

// ---- a filter of two conjuncts ------------------------------------------------------------------

fn with_desk() -> String {
    let text = edited(
        MODEL,
        "      - {name: team, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}
      - {name: on_hold, type: Boolean}
    lifecycle:",
        "      - {name: team, type: demo.desk.Team}
      - {name: desk, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}
      - {name: on_hold, type: Boolean}
    lifecycle:",
    );
    let text = edited(
        &text,
        "  - name: demo.desk.Open
    input:
      - {name: team, type: demo.desk.Team}",
        "  - name: demo.desk.Open
    input:
      - {name: team, type: demo.desk.Team}
      - {name: desk, type: demo.desk.Team}",
    );
    let text = edited(
        &text,
        "          team: input.team\n          note: input.note\n          on_hold: false",
        "          team: input.team\n          desk: input.desk\n          note: input.note\n          on_hold: false",
    );
    let text = edited(
        &text,
        "  - name: demo.desk.NoteTeam
    input:
      - {name: team, type: demo.desk.Team}",
        "  - name: demo.desk.NoteTeam
    input:
      - {name: team, type: demo.desk.Team}
      - {name: desk, type: demo.desk.Team}",
    );
    let text = edited(
        &text,
        "        updates: demo.desk.Session\n        instances: {where: team == input.team}",
        "        updates: demo.desk.Session\n        instances: {where: {all: [team == input.team, desk == input.desk]}}",
    );
    let text = edited(
        &text,
        "where: team == subject.team",
        "where: {all: [team == subject.team, desk == subject.desk]}",
    );
    edited(
        &text,
        "      - {name: team, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}
      - {name: on_hold, type: Boolean}
      - {name: state",
        "      - {name: team, type: demo.desk.Team}
      - {name: desk, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}
      - {name: on_hold, type: Boolean}
      - {name: state",
    )
}

/// (a) A target changing a row the filter leaves out — here one of the right desk and another team
/// — must fail. One non-matching row falsifies one conjunct, so a target that drops the other passes.
#[test]
fn a_set_update_dropping_either_conjunct_of_its_filter_fails() {
    let text = with_desk();
    wrong_mode_is_caught(&text, NOTED, Mode::NoteIgnoresTeam, true);
    wrong_mode_is_caught(&text, NOTED, Mode::NoteIgnoresDesk, true);
}

/// (e) An `affects:` target ignoring one `subject.<field>` of its filter must fail.
#[test]
fn an_affects_target_ignoring_either_subject_field_of_its_filter_fails() {
    let text = with_desk();
    wrong_mode_is_caught(&text, INVITED, Mode::InviteIgnoresSubjectTeam, true);
    wrong_mode_is_caught(&text, INVITED, Mode::InviteIgnoresSubjectDesk, true);
}

// ---- `subject.` is the subject before the branch ------------------------------------------------

/// `Transfer` moves its subject to another team and holds every other session of the team it
/// left. A target reading `subject.team` after the branch's own `sets:` holds the wrong team.
#[test]
fn an_affects_target_reading_the_subject_after_the_branch_fails() {
    let text = edited(
        MODEL,
        "views:\n",
        "  - name: demo.desk.Transfer
    input:
      - {name: session_id, type: demo.desk.SessionId}
      - {name: team, type: demo.desk.Team}
    outcomes:
      - name: transferred
        updates: demo.desk.Session
        instance: session_id
        emits: [demo.desk.Transferred]
        payload:
          demo.desk.Transferred: {session_id: input.session_id}
        sets:
          team: input.team
        affects:
          - entity: demo.desk.Session
            where: team == subject.team
            sets:
              on_hold: true
views:\n",
    );
    let text = edited(
        &text,
        "  - name: demo.desk.SessionClosed",
        "  - name: demo.desk.Transferred
    fields:
      - {name: session_id, type: demo.desk.SessionId}
  - name: demo.desk.SessionClosed",
    );
    let text = edited(
        &text,
        "demo.desk.Invite]",
        "demo.desk.Invite, demo.desk.Transfer]",
    );
    wrong_mode_is_caught(&text, TRANSFERRED, Mode::TransferReadsSubjectAfter, false);
}

// ---- the count ------------------------------------------------------------------------------------

/// Decision 1: "Zero matches is accepted." A target refusing a team with no open session must fail
/// some scenario.
#[test]
fn a_target_refusing_zero_matches_fails_some_scenario() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, Mode::EndRefusesNone, false);
    assert!(
        statuses.values().any(|status| *status != Status::Passed),
        "a target answering an undeclared refusal when no row matches passes every scenario: \
         {statuses:#?}"
    );
}

/// `{count: changed}` is asserted at one value only; a target reporting that constant passes.
#[test]
fn a_target_reporting_a_constant_count_fails_some_scenario() {
    let synthesis = synthesis_of(MODEL);
    let statuses = run(&synthesis.suite, Mode::EndCountsTwo, false);
    assert!(
        statuses.values().any(|status| *status != Status::Passed),
        "a target reporting `ended: 2` whatever it ended passes every scenario: {statuses:#?}"
    );
}
