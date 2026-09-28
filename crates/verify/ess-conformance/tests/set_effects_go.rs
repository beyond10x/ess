//! The generated Go runtime gives the reference verdicts for set effects over filtered instances
//! (`instances: {where}`, `affects:`, `{count: changed}`; ess/16, beyond10x/ess#167, #175) —
//! constructs that need no suite vocabulary of their own (beyond10x/ess#188).
//!
//! The target is `tests/set_effects.rs`'s, with every wrong mode, recorded once and replayed to the
//! Go runtime.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn go_gives_the_reference_verdict_for_every_set_effect_mode() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    for mode in [
        Mode::Correct,
        Mode::EndIgnoresFilter,
        Mode::EndSkipsOne,
        Mode::EndMiscounts,
        Mode::EndIgnoresFrom,
        Mode::EndIgnoresSets,
        Mode::NoteIgnoresFilter,
        Mode::NoteSkipsOne,
        Mode::NoteMiscounts,
        Mode::InviteHoldsSubject,
        Mode::InviteSkipsOthers,
        Mode::InviteHoldsEveryTeam,
    ] {
        let verdicts = support_go::assert_parity(
            &format!("seteffects-{mode:?}").to_lowercase(),
            &suite,
            Desk::new(mode),
        );
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            mode == Mode::Correct,
            "{mode:?}: {verdicts:?}"
        );
    }
}

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
