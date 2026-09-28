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
