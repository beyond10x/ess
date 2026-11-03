//! Adversary pass on unit SYN2 of wave W2.
//!
//! Stories under attack: `story:every-witness-reads-back-its-outcome-effect`,
//! `story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists`,
//! `story:a-delete-may-have-a-no-op-branch-that-emits-nothing` and
//! `story:view-asserts-field-set-from-optional-input`.
//!
//! Every model is inline and brand-free, so no repository model the suite-bytes tables walk.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::{
    AbsentInputRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, InvocationObservationRequest, ObservedEvent, ObservedInvocation,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, Runner, ScenarioId, ScenarioStep, ScenarioValue,
    ViewExpectation,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn parsed(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = parsed(text).unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the model resolves:\n{d}"))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenario(&ScenarioId::parse(id).expect("an id"))
        .unwrap_or_else(|| {
            panic!(
                "`{id}` is synthesized; refusals: {:#?}",
                result
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
}

/// What a planted defect does to the row a send took `outcome` for, as the views show it.
#[derive(Clone, Copy)]
enum Fault {
    None,
    /// The row's `field` reads `"rewritten"` after the branch answered.
    Rewrites {
        outcome: &'static str,
        field: &'static str,
    },
    /// The row stops being listed after the branch answered.
    Deletes(&'static str),
    /// A row for the sent identity is listed after the branch answered.
    Stores(&'static str),
}

/// The model interpreter with [`Fault`] applied to the views' rows.
struct Target {
    inner: Interpreted,
    identity: &'static str,
    fault: Fault,
    touched: RefCell<BTreeSet<Node>>,
}

impl Target {
    fn new(model: &str, identity: &'static str, fault: Fault) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            identity,
            fault,
            touched: RefCell::new(BTreeSet::new()),
        }
    }

    fn faulted(&self) -> Option<&'static str> {
        match self.fault {
            Fault::None => None,
            Fault::Rewrites { outcome, .. } | Fault::Deletes(outcome) | Fault::Stores(outcome) => {
                Some(outcome)
            }
        }
    }
}

impl ConformanceTarget for Target {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.touched.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let sent = request.input.get(self.identity).cloned();
        let result = self.inner.execute_command(request)?;
        let taken = result
            .outcome
            .as_ref()
            .map(|taken| taken.outcome.as_str().to_owned());
        if let (Some(sent), Some(taken), Some(faulted)) = (sent, taken, self.faulted()) {
            if taken == faulted {
                self.touched.borrow_mut().insert(sent);
            }
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        let touched = self.touched.borrow();
        let id = self.identity;
        match self.fault {
            Fault::None => {}
            Fault::Deletes(_) => result
                .rows
                .retain(|row| !row.get(id).is_some_and(|value| touched.contains(value))),
            Fault::Rewrites { field, .. } => {
                for row in &mut result.rows {
                    if row.get(id).is_some_and(|value| touched.contains(value)) {
                        if let Some(value) = row.get_mut(field) {
                            *value = Node::from("rewritten");
                        }
                    }
                }
            }
            Fault::Stores(_) => {
                for value in touched.iter() {
                    if !result.rows.iter().any(|row| row.get(id) == Some(value)) {
                        let mut row = result.rows.first().cloned().unwrap_or_default();
                        row.insert(id.to_owned(), value.clone());
                        result.rows.push(row);
                    }
                }
            }
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
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

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

fn reads_complete(scenario: &ConformanceScenario) -> bool {
    scenario
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. }))
        && scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. }))
}

// ---------------------------------------------------------------------------------------------
// story:every-witness-reads-back-its-outcome-effect — input-guarded refusals (ess/34).
// ---------------------------------------------------------------------------------------------

/// `ClosePass` refuses an empty reason by input, closes an open pass and answers a closed one
/// with `{held}` — a `when_subject_state:` refusal or `wrong_state:`.
const HELD: &str = r#"format: ess/34
system: demo
version: v1
domain: demo.passes
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
entities:
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: demo.passes.EmptyReason, summary: The reason is empty.}
  - {name: demo.passes.NotOpen, summary: The pass is not open.}
commands:
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.ClosePass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: reason, type: String}
    outcomes:
      - name: empty-reason
        when: 'reason == ""'
        error: demo.passes.EmptyReason
{held}
events:
  - name: demo.passes.PassOpened
    fields: [{name: pass_id, type: demo.passes.PassId}]
  - name: demo.passes.PassClosed
    fields: [{name: pass_id, type: demo.passes.PassId}]
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
      - {name: state, type: demo.passes.Pass.State}
"#;

const HELD_STATE: &str = "      - name: closed
        when_subject_state: Open
        moves: demo.passes.Pass.close
        instance: pass_id
        sets: {note: input.reason}
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
      - name: not-open
        when_subject_state: Closed
        error: demo.passes.NotOpen";

const WRONG_STATE: &str = "      - name: closed
        moves: demo.passes.Pass.close
        instance: pass_id
        sets: {note: input.reason}
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
      - name: not-open
        wrong_state: true
        error: demo.passes.NotOpen";

const EMPTY_REASON: &str = "demo.passes.ClosePass/outcome/empty-reason";

/// The Acceptance names "an input-guarded ... refusal of a command addressing an existing row".
/// `ClosePass` addresses an existing pass, and `complete_refusal` is minted for `empty-reason`
/// (`resolve.rs` `complete_refusal`, `guarded`), but `arranged` skips `addressed_row` on a command
/// with held-state guards (`synthesize.rs`, `guarded_complete(outcome) &&
/// !has_subject_guards(command)`).
#[test]
fn input_refusal_beside_a_held_state_guard_requires_the_row_unchanged() {
    let model = HELD.replace("{held}", HELD_STATE);
    let ir = ir(&model);
    let flagged = ir
        .commands()
        .values()
        .flat_map(|command| command.outcomes.iter())
        .any(|outcome| outcome.name.as_str() == "empty-reason" && outcome.complete_refusal);
    assert!(flagged, "the IR carries complete_refusal on `empty-reason`");
    let result = synthesize(&ir);
    let witness = scenario(&result, EMPTY_REASON);
    assert!(
        reads_complete(witness),
        "story:every-witness-reads-back-its-outcome-effect: `{EMPTY_REASON}` is flagged \
         complete_refusal on a command addressing an existing row and never snapshots the row \
         nor requires it unchanged: {:#?}",
        witness.steps
    );
    let rewrites = run(
        &result.suite,
        &Target::new(
            &model,
            "pass_id",
            Fault::Rewrites {
                outcome: "empty-reason",
                field: "note",
            },
        ),
    );
    assert_ne!(
        rewrites.get(EMPTY_REASON),
        Some(&Status::Passed),
        "a target rewriting the pass on `empty-reason` passes `{EMPTY_REASON}`"
    );
}

/// The same with `wrong_state:` in place of the held-state refusal.
#[test]
fn input_refusal_beside_wrong_state_requires_the_row_unchanged() {
    let model = HELD.replace("{held}", WRONG_STATE);
    let result = synthesize(&ir(&model));
    let witness = scenario(&result, EMPTY_REASON);
    assert!(
        reads_complete(witness),
        "story:every-witness-reads-back-its-outcome-effect: `{EMPTY_REASON}` beside \
         `wrong_state:` never snapshots the pass nor requires it unchanged: {:#?}",
        witness.steps
    );
    let honest = run(&result.suite, &Target::new(&model, "pass_id", Fault::None));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
}

/// The unit's own `GUARDED` model, driven by a target that changes the row on the refusal.
const GUARDED: &str = r#"format: {format}
system: demo
version: v1
domain: demo.passes
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
  - {name: demo.passes.CardId, kind: newtype, of: String}
entities:
  - name: demo.passes.Card
    identity: {name: card_id, type: demo.passes.CardId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.passes.EmptyNote, summary: The note is empty.}
  - {name: demo.passes.NoCard, summary: No card carries the identity.}
commands:
  - name: demo.passes.AddCard
    input:
      - {name: card_id, type: demo.passes.CardId}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.passes.Card
        instance: card_id
        sets: {label: input.label}
        emits: [demo.passes.CardAdded]
        payload:
          demo.passes.CardAdded: {card_id: input.card_id}
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.Annotate
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: card_id, type: demo.passes.CardId}
      - {name: note, type: String}
    outcomes:
      - name: empty-note
        when: 'note == ""'
        error: demo.passes.EmptyNote
      - name: no-card
        when_related: {via: input.card_id, exists: false}
        error: demo.passes.NoCard
      - name: annotated
        updates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassAnnotated]
        payload:
          demo.passes.PassAnnotated: {pass_id: input.pass_id}
events:
  - name: demo.passes.CardAdded
    fields: [{name: card_id, type: demo.passes.CardId}]
  - name: demo.passes.PassOpened
    fields: [{name: pass_id, type: demo.passes.PassId}]
  - name: demo.passes.PassAnnotated
    fields: [{name: pass_id, type: demo.passes.PassId}]
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
      - {name: state, type: demo.passes.Pass.State}
"#;

#[test]
fn guarded_refusals_that_change_the_row_fail_their_scenarios() {
    let model = GUARDED.replace("{format}", "ess/34");
    let result = synthesize(&ir(&model));
    for outcome in ["empty-note", "no-card"] {
        let id = format!("demo.passes.Annotate/outcome/{outcome}");
        let rewrites = run(
            &result.suite,
            &Target::new(
                &model,
                "pass_id",
                Fault::Rewrites {
                    outcome,
                    field: "note",
                },
            ),
        );
        assert_ne!(
            rewrites.get(&id),
            Some(&Status::Passed),
            "a target rewriting the pass on `{outcome}` passes `{id}`"
        );
        let deletes = run(
            &result.suite,
            &Target::new(&model, "pass_id", Fault::Deletes(outcome)),
        );
        assert_ne!(
            deletes.get(&id),
            Some(&Status::Passed),
            "a target deleting the pass on `{outcome}` passes `{id}`"
        );
    }
}

/// Below `ess/34` the guarded refusals keep the observation they had: no complete read-back.
#[test]
fn guarded_refusals_below_ess_34_take_no_complete_read_back() {
    let model = GUARDED.replace("{format}", "ess/33");
    let ir = ir(&model);
    assert!(
        ir.commands()
            .values()
            .flat_map(|command| command.outcomes.iter())
            .all(|outcome| !outcome.complete_refusal),
        "no outcome of the ess/33 model is complete_refusal"
    );
    let result = synthesize(&ir);
    for outcome in ["empty-note", "no-card"] {
        let id = format!("demo.passes.Annotate/outcome/{outcome}");
        let witness = scenario(&result, &id);
        assert!(
            !reads_complete(witness),
            "`{id}` at ess/33 takes the ess/34 observation: {:#?}",
            witness.steps
        );
    }
}

// ---------------------------------------------------------------------------------------------
// story:every-witness-reads-back-its-outcome-effect — the compensating delete (#585).
// ---------------------------------------------------------------------------------------------

const TICKETS: &str = r"format: {format}
system: demo
version: v1
domain: demo.tickets
types:
  - {name: demo.tickets.TicketId, kind: newtype, of: Uuid}
entities:
  - name: demo.tickets.Ticket
    identity: {name: ticket_id, type: demo.tickets.TicketId}
    fields:
      - {name: secret, type: String}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
errors:
  - {name: demo.tickets.ConsumedOnFailure, summary: The ticket was consumed and the grant failed.}
  - {name: demo.tickets.NoTicket, summary: No ticket carries the identity.}
commands:
  - name: demo.tickets.Issue
    input:
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: demo.tickets.Ticket
        instance: ticket_id
        sets: {secret: input.secret}
        emits: [demo.tickets.Issued]
        payload:
          demo.tickets.Issued: {ticket_id: {generated: true}}
  - name: demo.tickets.Redeem
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: proof, type: String}
    outcomes:
      - name: no-ticket
        unknown_instance: true
        error: demo.tickets.NoTicket
      - name: consumed-on-failure
        external: the grant fails after the ticket is consumed
        compensates: true
        deletes: demo.tickets.Ticket
        instance: ticket_id
{extra}        error: demo.tickets.ConsumedOnFailure
      - name: redeemed
        deletes: demo.tickets.Ticket
        instance: ticket_id
        emits: [demo.tickets.Redeemed]
        payload:
          demo.tickets.Redeemed: {ticket_id: input.ticket_id}
events:
  - name: demo.tickets.Issued
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
  - name: demo.tickets.Redeemed
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
views:
  - name: demo.tickets.Tickets
    source: demo.tickets.Ticket
    consistency: {consistency}
    fields:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: secret, type: String}
      - {name: state, type: demo.tickets.Ticket.State}
";

const CONSUMED: &str = "demo.tickets.Redeem/outcome/consumed-on-failure";

fn tickets(format: &str, extra: &str, consistency: &str) -> String {
    TICKETS
        .replace("{format}", format)
        .replace("{extra}", extra)
        .replace("{consistency}", consistency)
}

/// At `ess/22` (the first format admitting `compensates:`) through `ess/33` the delete is refused
/// naming `ess/34`; below `ess/22` the marker itself is refused.
#[test]
fn a_compensating_delete_is_refused_at_every_format_below_ess_34() {
    for major in 22..34 {
        let format = format!("ess/{major}");
        let errors = parsed(&tickets(&format, "", "read_your_writes"))
            .expect_err("a compensating delete is refused below ess/34");
        assert!(
            errors.contains("ess/34") && errors.contains("consumed-on-failure"),
            "{format}: the refusal names the outcome and ess/34: {errors}"
        );
    }
    parsed(&tickets("ess/21", "", "read_your_writes"))
        .expect_err("`compensates:` is refused below ess/22");
    parsed(&tickets("ess/34", "", "read_your_writes"))
        .unwrap_or_else(|errors| panic!("ess/34 admits the compensating delete: {errors}"));
}

/// The design admits a compensating delete of the addressed row and nothing beside it; a field
/// write on a row the branch deletes is a change no read-back can observe.
#[test]
fn a_compensating_delete_that_also_writes_fields_is_refused() {
    let text = tickets(
        "ess/34",
        "        sets: {secret: input.proof}\n",
        "read_your_writes",
    );
    assert!(
        parsed(&text).is_err(),
        "a compensating delete beside `sets:` validates at ess/34"
    );
}

/// Out of the Acceptance: at `ess/34` a `compensates: true` refusal declaring no change is told
/// the change it may declare; `deletes:` is one of them from this format, and the refusal's message
/// and hint (`validate_compensation`, `MissingDeclaration` arm) still name only moves and writes.
#[test]
fn a_compensation_declaring_no_change_is_pointed_at_deletes_from_ess_34() {
    let text = tickets("ess/34", "", "read_your_writes").replace(
        "        deletes: demo.tickets.Ticket\n        instance: ticket_id\n        error: \
         demo.tickets.ConsumedOnFailure",
        "        error: demo.tickets.ConsumedOnFailure",
    );
    assert!(
        !text.contains("deletes: demo.tickets.Ticket\n        instance: ticket_id\n        error"),
        "the compensating change is removed"
    );
    let errors = parsed(&text).expect_err("a compensation declaring no change is refused");
    assert!(
        errors.contains("compensates") && errors.contains("deletes:"),
        "at ess/34 the refusal of a compensation declaring no change names `deletes:`: {errors}"
    );
}

/// No immediate view: the compensating delete's change cannot be read back, so no scenario is
/// written for it and a refusal names the branch.
#[test]
fn a_compensating_delete_no_view_reads_is_refused_not_written() {
    let model = tickets("ess/34", "", "eventual");
    let result = synthesize(&ir(&model));
    let id = ScenarioId::parse(CONSUMED).expect("an id");
    assert!(
        result.suite.scenario(&id).is_none(),
        "`{CONSUMED}` is written with no view to read the deletion back"
    );
    assert!(
        result
            .refusals
            .iter()
            .any(|refusal| refusal.to_string().contains("consumed-on-failure")),
        "a refusal names the unread compensating delete: {:#?}",
        result
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// The compensating delete's scenario on a target that keeps the ticket fails, and the honest
/// interpreter passes the whole suite.
#[test]
fn a_compensating_delete_kept_by_the_target_fails() {
    let model = tickets("ess/34", "", "read_your_writes");
    let result = synthesize(&ir(&model));
    let honest = run(
        &result.suite,
        &Target::new(&model, "ticket_id", Fault::None),
    );
    assert_eq!(not_passed(&honest), Vec::<String>::new());
    let witness = scenario(&result, CONSUMED);
    let absent = witness.steps.iter().any(|step| {
        matches!(
            step,
            ScenarioStep::ExpectSubjectAbsent { .. }
                | ScenarioStep::ExpectView {
                    expectation: ViewExpectation::Excludes { .. },
                    ..
                }
        )
    });
    assert!(absent, "`{CONSUMED}` reads the ticket back absent");
}

// ---------------------------------------------------------------------------------------------
// story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists
// ---------------------------------------------------------------------------------------------

const NOTES: &str = r"format: ess/33
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.notes.NoNote, summary: No note carries the identity.}
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
commands:
  - name: demo.notes.OpenNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: opened
        creates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteOpened]
        payload:
          demo.notes.NoteOpened: {note_id: input.note_id}
  - name: demo.notes.CloseNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: count, type: Integer}
    outcomes:
{branches}
events:
  - name: demo.notes.NoteOpened
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.NoteClosed
    fields:
      - {name: note_id, type: demo.notes.NoteId}
";

const CLOSED_DELETE: &str = "        deletes: demo.notes.Note
        instance: note_id
        emits: [demo.notes.NoteClosed]
        payload:
          demo.notes.NoteClosed: {note_id: input.note_id}";

const CLOSED: &str = "demo.notes.CloseNote/outcome/closed";

/// Each `CloseNote` send requiring `closed`: the note it names.
fn closing_notes(steps: &[ScenarioStep]) -> Vec<ScenarioValue> {
    steps
        .windows(2)
        .filter_map(|pair| match pair {
            [ScenarioStep::ExecuteCommand { command, input, .. }, ScenarioStep::ExpectOutcome { outcome }]
                if command.to_string() == "demo.notes.CloseNote"
                    && outcome.outcome.as_str() == "closed" =>
            {
                input.get("note_id").cloned()
            }
            _ => None,
        })
        .collect()
}

fn boundary_case(label: &str, branches: &str) {
    let model = NOTES.replace("{branches}", branches);
    let result = synthesize(&ir(&model));
    let witness = scenario(&result, CLOSED);
    let notes = closing_notes(&witness.steps);
    assert!(
        notes.len() > 1,
        "{label}: `{CLOSED}` is further witnessed at its guard's boundary: {:#?}",
        witness.steps
    );
    for (at, note) in notes.iter().enumerate() {
        assert!(
            !notes[..at].contains(note),
            "{label}: a closing send names a note an earlier send deleted: {:#?}",
            witness.steps
        );
    }
    let honest = run(&result.suite, &Target::new(&model, "note_id", Fault::None));
    assert_eq!(not_passed(&honest), Vec::<String>::new(), "{label}");
}

/// The deleting branch carries the guard itself, the default accepts nothing.
#[test]
fn a_guarded_deleting_branch_is_sent_its_boundary_row_for_a_note_that_exists() {
    boundary_case(
        "guarded delete",
        &format!(
            "      - name: closed\n        when: 'count <= 3'\n{CLOSED_DELETE}\n      - name: \
             dropped\n        accepts: nothing"
        ),
    );
}

/// With a non-creating `unknown_instance:` answering first, the identity-row arrangement is the
/// one that runs.
#[test]
fn a_deleting_branch_below_unknown_instance_is_sent_its_boundary_row_for_a_note_that_exists() {
    boundary_case(
        "below unknown_instance",
        &format!(
            "      - {{name: no-note, unknown_instance: true, error: demo.notes.NoNote}}\n      - \
             name: dropped\n        when: 'count > 3'\n        accepts: nothing\n      - name: \
             closed\n{CLOSED_DELETE}"
        ),
    );
}

// ---------------------------------------------------------------------------------------------
// story:a-delete-may-have-a-no-op-branch-that-emits-nothing
// ---------------------------------------------------------------------------------------------

const JOBS: &str = "format: ess/27
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: String}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Queued, states: [Queued], terminal: [Queued]}
commands:
  - name: demo.jobs.CreateJob
    input:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
    outcomes:
      - name: created
        creates: demo.jobs.Job
        instance: job_id
        sets: {title: input.title}
        emits: [demo.jobs.JobCreated]
        payload:
          demo.jobs.JobCreated: {job_id: input.job_id}
  - name: demo.jobs.RenameJob
    input:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
    outcomes:
      - {name: absent, unknown_instance: true, refuses: false}
      - name: renamed
        updates: demo.jobs.Job
        instance: job_id
        sets: {title: input.title}
        emits: [demo.jobs.JobRenamed]
        payload:
          demo.jobs.JobRenamed: {job_id: input.job_id}
events:
  - name: demo.jobs.JobCreated
    fields: [{name: job_id, type: demo.jobs.JobId}]
  - name: demo.jobs.JobRenamed
    fields: [{name: job_id, type: demo.jobs.JobId}]
views:
  - name: demo.jobs.Jobs
    source: demo.jobs.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
";

/// The no-op read-back applies to an `updates:` command's no-op too: an upsert-like target that
/// stores the row on the no-op branch fails, and the honest interpreter passes.
#[test]
fn a_no_op_rename_that_stores_a_row_fails() {
    let id = "demo.jobs.RenameJob/outcome/absent";
    let result = synthesize(&ir(JOBS));
    let honest = run(&result.suite, &Target::new(JOBS, "job_id", Fault::None));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
    let stores = run(
        &result.suite,
        &Target::new(JOBS, "job_id", Fault::Stores("absent")),
    );
    assert_ne!(
        stores.get(id),
        Some(&Status::Passed),
        "a target storing a job on the no-op branch passes `{id}`"
    );
}

// ---------------------------------------------------------------------------------------------
// A related-row refusal whose related row is of the addressed entity (the re-keying vault).
// ---------------------------------------------------------------------------------------------

const VAULT: &str = include_str!("fixtures/identity-changing-updates.yaml");

/// At `ess/34` the vault's `taken` (a row-set `when_related:` refusal reading another `Secret`)
/// reads the addressed secret back unchanged — through the whole-view snapshot the row-set path
/// already takes (`SnapshotView` … `ExpectViewUnchanged`), or the complete-subject one — and the
/// honest interpreter passes. (First run asserted the complete-subject steps only, which was the
/// adversary's mistake: the whole-view observation covers the row.)
#[test]
fn a_related_refusal_over_the_addressed_entity_reads_back_and_passes() {
    let model = VAULT.replace("format: ess/23", "format: ess/34");
    let result = synthesize(&ir(&model));
    let taken = scenario(&result, "demo.vault.RenameSecret/outcome/taken");
    let whole_view = taken
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::SnapshotView { .. }))
        && taken
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectViewUnchanged { .. }));
    assert!(
        reads_complete(taken) || whole_view,
        "`taken` at ess/34 reads the addressed secret back: {:#?}",
        taken.steps
    );
    let honest = run(&result.suite, &Target::new(&model, "name", Fault::None));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
}

/// Out of the Acceptance (deletes only): #581's further witness of a compound guard's complement,
/// on a re-keying `updates:` — the row leaves its old identity as a deleted one does. The vault's
/// row-set `taken` is replaced by the compound guard (validation admits one related-row kind per
/// command; the first run kept both and was refused by validation, not a finding).
#[test]
fn a_rekeying_branch_further_witnessed_by_a_compound_guard_passes() {
    let model = VAULT
        .replace(
            "      - {name: new_name, type: demo.vault.SecretName}\n    outcomes:\n      - name: \
             taken\n        when_related:\n          entity: demo.vault.Secret\n          where: \
             name == input.new_name\n          exists: true\n        error: \
             demo.vault.NameTaken\n",
            "      - {name: new_name, type: demo.vault.SecretName}\n      - {name: proof, type: \
             String}\n    outcomes:\n      - name: wrong-proof\n        when_subject: {predicate: \
             {all: ['value != \"\"', 'value != input.proof']}}\n        error: \
             demo.vault.WrongProof\n",
        )
        .replace(
            "  - {name: demo.vault.NoSuchSecret,",
            "  - {name: demo.vault.WrongProof, summary: The proof is wrong., fields: []}\n  - \
             {name: demo.vault.NoSuchSecret,",
        );
    assert!(model.contains("wrong-proof"), "the guard is planted");
    // Second run: validation requires the row-set collision answer beside a re-key and refuses a
    // row set beside `when_subject:`, so no valid model reaches #581's shape on a re-key. The case
    // records that the shape is unreachable instead of synthesizing it.
    let refused = parsed(&model).expect_err("a re-key without its collision answer is refused");
    assert!(
        refused.contains("re-keys the record"),
        "the re-key shape #581 would need is refused by validation: {refused}"
    );
}
