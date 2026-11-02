//! story:a-no-view-held-state-refusal-probes-the-record: an input refusal beside held-state
//! branches, on a model with no identity view, reads the record back after each held-state send
//! through a branch that runs from the state it was arranged in.
//!
//! * The correct target passes, and a target that moves the record on refusal fails.
//! * Where no branch has a valid send from that state, the scenario is withdrawn and refused as
//!   `ESS-SYNTH-001` naming the branches tried.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// A ticket is raised, held, then closed, all through `Advance`, whose branches read the held
/// state; a blank note is refused in every state, before the record is read. No view.
const TICKETS: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.ticket
summary: A ticket is raised, then advanced to held and to closed; a blank note is always refused.
entities:
  - name: demo.ticket.Ticket
    identity: {name: ticket_id, type: String}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Held, Closed]
      terminal: [Closed]
      transitions:
        - {name: hold, from: [Open], to: Held}
        - {name: close, from: [Held], to: Closed}
actors:
  - name: demo.ticket.Clerk
    may: [demo.ticket.Raise, demo.ticket.Advance]
errors:
  - {name: demo.ticket.BlankNote, fields: []}
  - {name: demo.ticket.AlreadyClosed, fields: []}
  - {name: demo.ticket.UnknownTicket, fields: []}
events:
  - name: demo.ticket.Raised
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Held
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Closed
    fields: [{name: ticket_id, type: String}]
commands:
  - name: demo.ticket.Raise
    input: []
    outcomes:
      - name: raised
        creates: demo.ticket.Ticket
        instance: ticket_id
        emits: [demo.ticket.Raised]
        payload: {demo.ticket.Raised: {ticket_id: {generated: true}}}
  - name: demo.ticket.Advance
    input:
      - {name: id, type: String}
      - {name: note, type: String}
    outcomes:
      - {name: blank-note, when: note == "", error: demo.ticket.BlankNote}
      - {name: unknown, unknown_instance: true, error: demo.ticket.UnknownTicket}
      - name: held
        when_subject_state: Open
        moves: demo.ticket.Ticket.hold
        instance: id
        emits: [demo.ticket.Held]
        payload: {demo.ticket.Held: {ticket_id: input.id}}
      - name: closed
        when_subject_state: Held
        moves: demo.ticket.Ticket.close
        instance: id
        emits: [demo.ticket.Closed]
        payload: {demo.ticket.Closed: {ticket_id: input.id}}
      - {name: already-closed, when_subject_state: Closed, error: demo.ticket.AlreadyClosed}
"#;

const BLANK: &str = "demo.ticket.Advance/outcome/blank-note";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tickets.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&compiled(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn filed(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// The blank note first, then existence, then the held state.
    Correct,
    /// [`Mode::Correct`], except a refused blank note on a stored ticket moves it one state on
    /// (`Open` to `Held`, `Held` to `Closed`), with no event.
    AdvancesOnRefusal,
}

struct Tickets {
    mode: Mode,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Tickets {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            minted: Cell::new(0),
            rows: RefCell::default(),
            published: RefCell::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

impl ConformanceTarget for Tickets {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tickets-fixture-d368-u2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let refuse = |name: &str, error: &str| {
            SemanticCommandResult::took(branch(&command, name))
                .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
        };
        let moved = |event: &str, id: &str| {
            ObservedEvent::new(event.parse::<EventRef>().unwrap())
                .with("ticket_id", Node::Text(id.to_owned()))
        };
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.ticket.Raise" => {
                let id = format!("ticket-{n}");
                rows.insert(id.clone(), "Open".to_owned());
                SemanticCommandResult::took(branch(&command, "raised"))
                    .emitting(moved("demo.ticket.Raised", &id))
            }
            "demo.ticket.Advance" => {
                let id = text(request.input.get("id"));
                let note = text(request.input.get("note"));
                if note.is_empty() {
                    if let Some(state) = rows.get_mut(&id) {
                        if self.mode == Mode::AdvancesOnRefusal {
                            match state.as_str() {
                                "Open" => "Held".clone_into(state),
                                "Held" => "Closed".clone_into(state),
                                _ => {}
                            }
                        }
                    }
                    refuse("blank-note", "demo.ticket.BlankNote")
                } else {
                    match rows.get_mut(&id) {
                        None => refuse("unknown", "demo.ticket.UnknownTicket"),
                        Some(state) if state == "Open" => {
                            "Held".clone_into(state);
                            SemanticCommandResult::took(branch(&command, "held"))
                                .emitting(moved("demo.ticket.Held", &id))
                        }
                        Some(state) if state == "Held" => {
                            "Closed".clone_into(state);
                            SemanticCommandResult::took(branch(&command, "closed"))
                                .emitting(moved("demo.ticket.Closed", &id))
                        }
                        Some(_) => refuse("already-closed", "demo.ticket.AlreadyClosed"),
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        panic!("no view is declared")
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
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

fn failures(text: &str, mode: Mode) -> BTreeMap<String, String> {
    let result = synthesis(text);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Tickets::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.status != Status::Passed)
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?}: {:?}", run.status, run.checks),
            )
        })
        .collect()
}

/// The control: the blank-note scenario is filed with a half in each held state, and the correct
/// target passes every scenario, so a failure below is the mutant's.
#[test]
fn the_correct_target_passes_the_no_view_held_state_refusal() {
    let result = synthesis(TICKETS);
    assert!(filed(&result, BLANK), "{:#?}", refusals(&result));
    let failed = failures(TICKETS, Mode::Correct);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The acceptance: with no identity view, each held-state half reads the record back through a
/// branch that runs from the state it was arranged in, so a target that moves the record on
/// refusal, publishing nothing, answers that read with another branch and fails.
#[test]
fn a_target_moving_the_record_on_refusal_fails_the_no_view_held_state_refusal() {
    let failed = failures(TICKETS, Mode::AdvancesOnRefusal);
    assert!(
        failed.contains_key(BLANK),
        "{BLANK} passed a target that moves the stored ticket on refusal with no view: each \
         held-state half must read the record back through a branch that runs from its state \
         (story:a-no-view-held-state-refusal-probes-the-record): {failed:#?}"
    );
}

/// A ticket held and closed by commands of their own, which declare no `wrong_state:` answer, and
/// annotated through `Annotate`, whose branches read the held state and whose every input a
/// synthesizer builds is refused first (`note.count < 100000`). Every state is arranged; from
/// `Closed` no branch has a valid send: `Annotate`'s are shadowed, and `Hold` and `Close` answer a
/// closed ticket with a refusal the model does not declare.
const NO_SEND: &str = r"format: ess/18
system: demo
version: v1
domain: demo.ticket
summary: A ticket is raised, held and closed by commands of their own; every note is refused.
entities:
  - name: demo.ticket.Ticket
    identity: {name: ticket_id, type: String}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Held, Closed]
      terminal: [Closed]
      transitions:
        - {name: hold, from: [Open], to: Held}
        - {name: close, from: [Held], to: Closed}
actors:
  - name: demo.ticket.Clerk
    may: [demo.ticket.Raise, demo.ticket.Hold, demo.ticket.Close, demo.ticket.Annotate]
errors:
  - {name: demo.ticket.NoteRefused, fields: []}
  - {name: demo.ticket.AlreadyClosed, fields: []}
  - {name: demo.ticket.UnknownTicket, fields: []}
events:
  - name: demo.ticket.Raised
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Held
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Closed
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Noted
    fields: [{name: ticket_id, type: String}]
commands:
  - name: demo.ticket.Raise
    input: []
    outcomes:
      - name: raised
        creates: demo.ticket.Ticket
        instance: ticket_id
        emits: [demo.ticket.Raised]
        payload: {demo.ticket.Raised: {ticket_id: {generated: true}}}
  - name: demo.ticket.Hold
    input:
      - {name: id, type: String}
    outcomes:
      - name: held
        moves: demo.ticket.Ticket.hold
        instance: id
        emits: [demo.ticket.Held]
        payload: {demo.ticket.Held: {ticket_id: input.id}}
  - name: demo.ticket.Close
    input:
      - {name: id, type: String}
    outcomes:
      - name: closed
        moves: demo.ticket.Ticket.close
        instance: id
        emits: [demo.ticket.Closed]
        payload: {demo.ticket.Closed: {ticket_id: input.id}}
  - name: demo.ticket.Annotate
    input:
      - {name: id, type: String}
      - {name: note, type: String}
    outcomes:
      - {name: refused-note, when: note.count < 100000, error: demo.ticket.NoteRefused}
      - {name: unknown, unknown_instance: true, error: demo.ticket.UnknownTicket}
      - name: noted-open
        when_subject_state: Open
        updates: demo.ticket.Ticket
        instance: id
        emits: [demo.ticket.Noted]
        payload: {demo.ticket.Noted: {ticket_id: input.id}}
      - name: noted-held
        when_subject_state: Held
        updates: demo.ticket.Ticket
        instance: id
        emits: [demo.ticket.Noted]
        payload: {demo.ticket.Noted: {ticket_id: input.id}}
      - {name: already-closed, when_subject_state: Closed, error: demo.ticket.AlreadyClosed}
";

const REFUSED_NOTE: &str = "demo.ticket.Annotate/outcome/refused-note";

/// Where no branch has a valid send from a held state, nothing would show a target that moves
/// the record on refusal there, so the scenario is withdrawn and refused as `ESS-SYNTH-001` naming
/// the branches tried, never filed with a half that passes such a target.
#[test]
fn a_no_view_held_state_refusal_without_a_valid_send_is_withdrawn_naming_the_branches_tried() {
    let result = synthesis(NO_SEND);
    let lines = refusals(&result);
    assert!(
        !filed(&result, REFUSED_NOTE),
        "{REFUSED_NOTE} filed with no view and no valid send from `Closed` to read the record \
         back through (story:a-no-view-held-state-refusal-probes-the-record): {lines:#?}"
    );
    assert!(
        lines.iter().any(|line| line.contains("SYNTH-001")
            && line.contains(REFUSED_NOTE)
            && line.contains("demo.ticket.Annotate.already-closed")
            && line.contains("demo.ticket.Hold.held")
            && line.contains("demo.ticket.Close.closed")
            && line.contains("no identity view shows that record")),
        "{REFUSED_NOTE} withdrawn without naming the branches it could not read the record back \
         through (story:a-no-view-held-state-refusal-probes-the-record): {lines:#?}"
    );
}
