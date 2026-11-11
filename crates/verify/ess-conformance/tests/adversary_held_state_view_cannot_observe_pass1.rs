//! Adversary, pass 1, story `a-held-state-refusal-reads-back-where-the-view-cannot-observe`.
//!
//! * A view that projects `state` but is filtered cannot observe a held record either: the
//!   held-state half reads it back, so a target that moves the record on refusal fails (pins the
//!   mutant that probes only where no view projects `state`).
//! * Two views, identity-only and with `state`: the state view observes, the mover fails.
//! * The refusal of a held-state half beside an identity view must not claim that no identity
//!   view shows the record.
//! * The arranged half of a command without held-state branches, beside an identity-only view,
//!   has the same gap the story closed for the held-state half.
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

/// A ticket held by a command of its own, and annotated through `Annotate`, which reads no held
/// state: its blank note is refused before the record is read.
const ANNOTATE: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.ticket
summary: A ticket is raised, held by a command of its own, and annotated; a blank note is refused.
entities:
  - name: demo.ticket.Ticket
    identity: {name: ticket_id, type: String}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Held]
      terminal: [Held]
      transitions:
        - {name: hold, from: [Open], to: Held}
actors:
  - name: demo.ticket.Clerk
    may: [demo.ticket.Raise, demo.ticket.Hold, demo.ticket.Annotate]
errors:
  - {name: demo.ticket.BlankNote, fields: []}
  - {name: demo.ticket.UnknownTicket, fields: []}
events:
  - name: demo.ticket.Raised
    fields: [{name: ticket_id, type: String}]
  - name: demo.ticket.Held
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
  - name: demo.ticket.Annotate
    input:
      - {name: id, type: String}
      - {name: note, type: String}
    outcomes:
      - {name: blank-note, when: note == "", error: demo.ticket.BlankNote}
      - {name: unknown, unknown_instance: true, error: demo.ticket.UnknownTicket}
      - name: noted
        updates: demo.ticket.Ticket
        instance: id
        emits: [demo.ticket.Noted]
        payload: {demo.ticket.Noted: {ticket_id: input.id}}
"#;

const ANNOTATE_BLANK: &str = "demo.ticket.Annotate/outcome/blank-note";

/// The model of the story's own test where no branch has a valid send from `Closed`.
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

/// Identity only.
const IDS_VIEW: &str = "  - name: demo.ticket.TicketIds
    source: demo.ticket.Ticket
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: String}
";

/// Identity and state, every row.
const STATE_VIEW: &str = "  - name: demo.ticket.Tickets
    source: demo.ticket.Ticket
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: String}
      - {name: state, type: demo.ticket.Ticket.State}
";

/// Identity and state, only open rows.
const OPEN_STATE_VIEW: &str = "  - name: demo.ticket.OpenTickets
    source: demo.ticket.Ticket
    consistency: read_your_writes
    filter: state == Open
    fields:
      - {name: ticket_id, type: String}
      - {name: state, type: demo.ticket.Ticket.State}
";

fn with(model: &str, views: &[&str]) -> String {
    format!("{model}views:\n{}", views.concat())
}

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
    Correct,
    /// A refused blank note on a stored ticket moves it one state on, with no event.
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

fn one_on(state: &mut String) {
    match state.as_str() {
        "Open" => "Held".clone_into(state),
        "Held" => "Closed".clone_into(state),
        _ => {}
    }
}

impl ConformanceTarget for Tickets {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tickets-adversary", "1"))
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
        let event = |event: &str, id: &str| {
            ObservedEvent::new(event.parse::<EventRef>().unwrap())
                .with("ticket_id", Node::Text(id.to_owned()))
        };
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.ticket.Raise" => {
                let id = format!("ticket-{n}");
                rows.insert(id.clone(), "Open".to_owned());
                SemanticCommandResult::took(branch(&command, "raised"))
                    .emitting(event("demo.ticket.Raised", &id))
            }
            "demo.ticket.Hold" => {
                let id = text(request.input.get("id"));
                match rows.get_mut(&id) {
                    Some(state) if state == "Open" => {
                        "Held".clone_into(state);
                        SemanticCommandResult::took(branch(&command, "held"))
                            .emitting(event("demo.ticket.Held", &id))
                    }
                    _ => return Err(TargetError::unavailable("hold", "the ticket is not open")),
                }
            }
            "demo.ticket.Annotate" => {
                let id = text(request.input.get("id"));
                let note = text(request.input.get("note"));
                if note.is_empty() {
                    if let Some(state) = rows.get_mut(&id) {
                        if self.mode == Mode::AdvancesOnRefusal {
                            one_on(state);
                        }
                    }
                    refuse("blank-note", "demo.ticket.BlankNote")
                } else if rows.contains_key(&id) {
                    SemanticCommandResult::took(branch(&command, "noted"))
                        .emitting(event("demo.ticket.Noted", &id))
                } else {
                    refuse("unknown", "demo.ticket.UnknownTicket")
                }
            }
            "demo.ticket.Advance" => {
                let id = text(request.input.get("id"));
                let note = text(request.input.get("note"));
                if note.is_empty() {
                    if let Some(state) = rows.get_mut(&id) {
                        if self.mode == Mode::AdvancesOnRefusal {
                            one_on(state);
                        }
                    }
                    refuse("blank-note", "demo.ticket.BlankNote")
                } else {
                    match rows.get_mut(&id) {
                        None => refuse("unknown", "demo.ticket.UnknownTicket"),
                        Some(state) if state == "Open" => {
                            "Held".clone_into(state);
                            SemanticCommandResult::took(branch(&command, "held"))
                                .emitting(event("demo.ticket.Held", &id))
                        }
                        Some(state) if state == "Held" => {
                            "Closed".clone_into(state);
                            SemanticCommandResult::took(branch(&command, "closed"))
                                .emitting(event("demo.ticket.Closed", &id))
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
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.to_string();
        let rows = self.rows.borrow();
        let rows = rows
            .iter()
            .filter(|(_, state)| view != "demo.ticket.OpenTickets" || state.as_str() == "Open")
            .map(|(id, state)| {
                let mut row = BTreeMap::from([("ticket_id".to_owned(), Node::Text(id.clone()))]);
                if view != "demo.ticket.TicketIds" {
                    row.insert("state".to_owned(), Node::Text(state.clone()));
                }
                row
            })
            .collect::<Vec<_>>();
        Ok(SemanticViewResult::of(rows))
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

/// The status and checks of scenario `id` run against `target`.
fn run_of(text: &str, id: &str, target: &impl ConformanceTarget) -> (Status, String) {
    let result = synthesis(text);
    assert!(
        filed(&result, id),
        "{id} not filed: {:#?}",
        refusals(&result)
    );
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let run = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .find(|run| run.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("{id} not run"));
    (run.status, format!("{:?}", run.checks))
}

/// The control and the mover for `id` on `text`: the interpreted target and the fixture's
/// correct mode pass, and the mover must fail.
fn mover_fails(text: &str, id: &str) {
    let interpreted = ess_conformance::interpret::Interpreted::for_model(compiled(text));
    let (status, checks) = run_of(text, id, &interpreted);
    assert_eq!(status, Status::Passed, "interpreted control: {checks}");
    let (status, checks) = run_of(text, id, &Tickets::new(Mode::Correct));
    assert_eq!(status, Status::Passed, "fixture control: {checks}");
    let (status, checks) = run_of(text, id, &Tickets::new(Mode::AdvancesOnRefusal));
    assert_ne!(
        status,
        Status::Passed,
        "{id} passed a target that moves the stored ticket on refusal and publishes nothing: \
         {checks}"
    );
}

/// A view that projects `state` but is filtered to `Open` cannot observe a `Held` or `Closed`
/// record, so each held-state half reads the record back; the mutant that probes only where no
/// view projects `state` lets the mover through here.
#[test]
fn a_filtered_state_view_held_state_half_catches_the_mover() {
    mover_fails(&with(TICKETS, &[OPEN_STATE_VIEW]), BLANK);
}

/// An identity-only view beside a state view: the state view observes, the mover fails.
#[test]
fn an_identity_only_view_beside_a_state_view_catches_the_mover() {
    mover_fails(&with(TICKETS, &[IDS_VIEW, STATE_VIEW]), BLANK);
}

/// The refusal names a cause that holds for the model: an identity view does show the record.
#[test]
fn a_withdrawn_held_state_half_beside_an_identity_view_does_not_claim_none_shows_it() {
    let result = synthesis(&with(NO_SEND, &[IDS_VIEW]));
    let lines = refusals(&result);
    let ours: Vec<_> = lines
        .iter()
        .filter(|line| line.contains("demo.ticket.Annotate/outcome/refused-note"))
        .collect();
    assert!(!ours.is_empty(), "{lines:#?}");
    assert!(
        ours.iter()
            .all(|line| !line.contains("no identity view shows that record")),
        "the refusal says no identity view shows the record, but demo.ticket.TicketIds does; \
         the cause is that no view observes it in its state: {ours:#?}"
    );
}

/// The arranged half of a command without held-state branches, beside a view that projects the
/// identity but not `state`: the same gap, outside the held-state path. Either the half reads the
/// record back and the mover fails, or it is withdrawn.
#[test]
#[ignore = "story:an-arranged-input-refusal-reads-back-where-the-view-cannot-observe (backlog)"]
fn an_identity_only_view_arranged_half_catches_the_mover_or_is_withdrawn() {
    let model = with(ANNOTATE, &[IDS_VIEW]);
    let result = synthesis(&model);
    if !filed(&result, ANNOTATE_BLANK) {
        return;
    }
    mover_fails(&model, ANNOTATE_BLANK);
}
