//! Adversary pass 2 against story:input-guard-overlap-precedence (beyond10x/ess#178).
//!
//! The unit's documents state the rule without a carve-out for the accepting branch's shape: "A
//! refusal with a `when:` over the input is taken before any accepting branch whose guard it
//! overlaps, whatever order they are written in" (`website/docs/guides/write-a-specification.md`),
//! and "A branch every input of which such a refusal claims is refused with `ESS-SYNTH-003`, naming
//! that refusal" (`website/docs/reference/predicates.md`, the paragraph that also names the
//! `when_subject:` and external shapes).
//!
//! The target cases run the synthesized suite against two ticket services that agree everywhere
//! except the overlap: one takes the refusal first (the stated precedence), the other the accepting
//! branch first. A suite that holds a target to the precedence passes the first and fails the
//! second.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const TICKETS: &str = include_str!("fixtures/input-guard-overlap.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tickets.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesis(text);
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    synthesis.suite
}

/// The fixture with `too-few: count < 5` refusing over an ordinary input, and `closed` guarded by
/// `guard` (the lines replacing `when: open == false`).
fn counted_with(guard: &str) -> String {
    let text = TICKETS
        .replace(
            "      - {name: open, type: Boolean}\n",
            "      - {name: open, type: Boolean}\n      - {name: count, type: Integer}\n",
        )
        .replace(
            "        error: demo.tickets.TicketIdRequired\n",
            "        error: demo.tickets.TicketIdRequired\n      - name: too-few\n        when: count < 5\n        error: demo.tickets.TooFew\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.tickets.TooFew\n    summary: The count is below five.\n    fields: []\n",
        )
        .replace("        when: open == false\n", guard);
    assert_ne!(text, TICKETS);
    text
}

/// The ticket carries a stored field `name: type`, set from `OpenTicket`'s input and shown by the
/// view.
fn stored(text: &str, name: &str, ty: &str) -> String {
    let out = text
        .replace(
            "    fields: []\n    lifecycle:",
            &format!("    fields:\n      - {{name: {name}, type: {ty}}}\n    lifecycle:"),
        )
        .replace(
            "  - name: demo.tickets.OpenTicket\n    input: []\n",
            &format!(
                "  - name: demo.tickets.OpenTicket\n    input:\n      - {{name: {name}, type: {ty}}}\n"
            ),
        )
        .replace(
            "        instance: ticket_id\n        emits: [demo.tickets.TicketOpened]\n",
            &format!(
                "        instance: ticket_id\n        sets: {{{name}: input.{name}}}\n        emits: [demo.tickets.TicketOpened]\n"
            ),
        )
        .replace(
            "      - {name: state, type: demo.tickets.Ticket.State}\n",
            &format!(
                "      - {{name: state, type: demo.tickets.Ticket.State}}\n      - {{name: {name}, type: {ty}}}\n"
            ),
        );
    assert_ne!(out, text);
    out
}

fn with_priority(text: &str) -> String {
    let out = text.replace(
        "types:\n",
        "types:\n  - name: demo.tickets.Priority\n    kind: enum\n    variants: [Low, Normal, High]\n",
    );
    stored(&out, "priority", "demo.tickets.Priority")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Order {
    RefusalsFirst,
    AcceptingFirst,
}

/// What `closed` reads, besides the refusals.
#[derive(Debug, Clone, Copy)]
enum Closing {
    /// `when_subject: {predicate: level > 0}` and no `when:`.
    StoredLevelOnly,
    /// `when_subject: {field: priority, equals: High}` beside `when: open == false`.
    HighAndInput,
}

type Row = BTreeMap<String, Node>;

struct Tickets {
    order: Order,
    closing: Closing,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Tickets {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tickets-pass2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
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
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.tickets.OpenTicket" => {
                let id = Node::Text(format!("ticket-{}", self.minted.get()));
                let mut row = Row::from([
                    ("ticket_id".to_owned(), id.clone()),
                    ("state".to_owned(), Node::Text("Open".into())),
                ]);
                for field in ["level", "priority"] {
                    if let Some(value) = request.input.get(field) {
                        row.insert(field.into(), value.clone());
                    }
                }
                rows.push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.tickets.TicketOpened".parse().unwrap())
                        .with("ticket_id", id),
                )
            }
            "demo.tickets.SetTicketOpen" => {
                let id = request.input["ticket_id"].clone();
                let row = rows.iter().find(|row| row["ticket_id"] == id).cloned();
                let closing = match (self.closing, &row) {
                    (_, None) => false,
                    (Closing::StoredLevelOnly, Some(row)) => row.get("level").is_some_and(
                        |level| matches!(level, Node::Number(n) if *n > Number::from(0_i64)),
                    ),
                    (Closing::HighAndInput, Some(row)) => {
                        row.get("priority") == Some(&Node::Text("High".into()))
                            && request.input.get("open") == Some(&Node::Bool(false))
                    }
                };
                let refusal = if id == Node::Text(String::new()) {
                    Some(("id-required", "demo.tickets.TicketIdRequired"))
                } else if request.input.get("count").is_some_and(
                    |count| matches!(count, Node::Number(n) if *n < Number::from(5_i64)),
                ) {
                    Some(("too-few", "demo.tickets.TooFew"))
                } else {
                    None
                };
                let refused = |(name, error): (&str, &str)| {
                    SemanticCommandResult::took(outcome(&command, name))
                        .with_error(DeclaredErrorValue::new(error.parse().unwrap()))
                };
                let refuse_now = match self.order {
                    Order::RefusalsFirst => refusal,
                    Order::AcceptingFirst => refusal.filter(|_| !closing),
                };
                if let Some(refusal) = refuse_now {
                    return Ok(refused(refusal).with_consistency(token));
                }
                let (name, state, event) = if closing {
                    ("closed", "Closed", "demo.tickets.TicketClosed")
                } else {
                    ("reopened", "Open", "demo.tickets.TicketReopened")
                };
                if let Some(row) = rows.iter_mut().find(|row| row["ticket_id"] == id) {
                    row.insert("state".into(), Node::Text(state.into()));
                }
                SemanticCommandResult::took(outcome(&command, name))
                    .emitting(ObservedEvent::new(event.parse().unwrap()).with("ticket_id", id))
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        match request.view.to_string().as_str() {
            "demo.tickets.Tickets" => Ok(SemanticViewResult::of(self.rows.borrow().clone())),
            other => Err(TargetError::unsupported("view", other)),
        }
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn failing(suite: &ConformanceSuite, order: Order, closing: Closing) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let target = Tickets {
        order,
        closing,
        rows: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &target)
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| eprintln!("{order:?}: {scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

fn held_to_the_precedence(text: &str, closing: Closing) {
    let suite = suite_of(text);
    assert_eq!(
        failing(&suite, Order::RefusalsFirst, closing),
        BTreeSet::new(),
        "a service honouring the precedence fails its own suite"
    );
    let failed = failing(&suite, Order::AcceptingFirst, closing);
    assert!(
        failed.contains("demo.tickets.SetTicketOpen/outcome/too-few"),
        "a service that decides `closed` before `too-few` passes the `too-few` scenario, so the \
         overlap is never sent: failed {failed:?}"
    );
}

/// `closed: when_subject: {predicate: level > 0}` with no `when:` at all overlaps `too-few` for
/// every count below five on a ticket whose level is positive.
#[test]
fn adversary_overlap_pass2_a_stored_only_accepting_branch_is_held_to_the_precedence() {
    let text = stored(
        &counted_with("        when_subject: {predicate: 'level > 0'}\n"),
        "level",
        "Integer",
    );
    held_to_the_precedence(&text, Closing::StoredLevelOnly);
}

/// The `{field, equals}` spelling of `when_subject:` (an enum stored fact, `SubjectField`) beside
/// `when: open == false`: `too-few` overlaps it on a `High` ticket at `{open: false, count < 5}`.
#[test]
fn adversary_overlap_pass2_an_enum_subject_field_branch_is_held_to_the_precedence() {
    let text = with_priority(&counted_with(
        "        when_subject: {field: priority, equals: High}\n        when: open == false\n",
    ));
    held_to_the_precedence(&text, Closing::HighAndInput);
}

/// The refusals of one synthesis that mention `name`.
fn refusals_about(synthesis: &Synthesis, name: &str) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|text| text.contains(name))
        .collect()
}

/// Pass 1's shadow correction, on the `when_subject:` shape. `closed: when_subject: level > 0,
/// when: count == 0` beside `too-few: count <= 0` and a default: every input `closed` accepts,
/// `too-few` claims, so `closed` is unreachable. The documented refusal names `too-few`; the
/// `Shadow` naming lives only in `reach`, which a subject-fact branch does not go through.
#[test]
fn adversary_overlap_pass2_a_shadowed_subject_fact_branch_is_refused_naming_the_refusal() {
    let text = stored(
        &counted_with("        when_subject: {predicate: 'level > 0'}\n        when: count == 0\n")
            .replace("        when: count < 5\n", "        when: count <= 0\n"),
        "level",
        "Integer",
    );
    let synthesis = synthesis(&text);
    let about_closed = refusals_about(&synthesis, "closed");
    assert!(
        !about_closed.is_empty(),
        "the shadowed `closed` is refused: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(
        about_closed.iter().any(|text| text.contains("too-few")),
        "the refusal for the shadowed subject-fact `closed` does not name `too-few`: \
         {about_closed:#?}"
    );
}

/// The same on the external shape. `deferred: when: retry == true, external` beside
/// `retry-refused: when: retry == true` and a default: the refusal claims every input the external
/// branch accepts. `reach_external` renders only the branch's own guard.
#[test]
fn adversary_overlap_pass2_a_shadowed_external_branch_is_refused_naming_the_refusal() {
    const MODEL: &str = r"
format: ess/16
system: demo
version: v1
domain: demo.mail
events:
  - name: demo.mail.Sent
    fields: []
  - name: demo.mail.Deferred
    fields: []
errors:
  - name: demo.mail.RetryRefused
    fields: []
commands:
  - name: demo.mail.Send
    input:
      - {name: retry, type: Boolean}
      - {name: attempts, type: Integer}
    outcomes:
      - name: sent
        emits: [demo.mail.Sent]
      - name: deferred
        when: attempts == 0
        external: the provider defers the delivery
        emits: [demo.mail.Deferred]
      - name: retry-refused
        when: attempts <= 0
        error: demo.mail.RetryRefused
";
    let synthesis = synthesis(MODEL);
    let about = refusals_about(&synthesis, "deferred");
    assert!(
        !about.is_empty(),
        "the shadowed `deferred` is refused: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(
        about.iter().any(|text| text.contains("retry-refused")),
        "the refusal for the shadowed external `deferred` does not name `retry-refused`: {about:#?}"
    );
}
