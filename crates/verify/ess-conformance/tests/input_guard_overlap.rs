//! An input-guarded refusal is taken before any accepting branch whose guard it overlaps
//! (beyond10x/ess#178, `docs/design/input-guard-overlap-precedence.md`).
//!
//! `closed: open == false` and `id-required: ticket_id == ""` both hold of `{ticket_id: "", open:
//! false}`. `validate` accepts the command — the accepting branch may not read the identity to step
//! aside (ESS-COMMAND-003) — so the precedence is stated rather than refused, and synthesis holds a
//! target to it twice over:
//!
//! 1. the refusal is sent again at the overlap point and required there;
//! 2. an accepting branch's witness refutes every sibling input-guarded refusal, default or no
//!    default, so a target that honours the precedence is never asked to take the accepting branch
//!    for an input the refusal claims.
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
    synthesize::synthesize,
    target::*,
    AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const TICKETS: &str = include_str!("fixtures/input-guard-overlap.yaml");
const ID_REQUIRED: &str = "demo.tickets.SetTicketOpen/outcome/id-required";
const CLOSED: &str = "demo.tickets.SetTicketOpen/outcome/closed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tickets.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn suite() -> ConformanceSuite {
    suite_of(TICKETS)
}

/// The fixture with a second input-guarded refusal that reads an ordinary input rather than the
/// identity: `count < 5` refuses, and `closed` still reads only `open`.
fn counted() -> String {
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
        );
    assert_ne!(text, TICKETS);
    text
}

/// Every `SetTicketOpen` invocation of one scenario, with the branch it requires.
fn invocations(
    suite: &ConformanceSuite,
    id: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, String)> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1;
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.tickets.SetTicketOpen" {
            continue;
        }
        // A subject-fact scenario may send the command before its own invocation to observe the
        // arranged row; only the invocations that require a branch are the ones listed here.
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        out.push((input.clone(), outcome.outcome.to_string()));
    }
    out
}

fn literal(value: Node) -> ScenarioValue {
    ScenarioValue::literal(value)
}

/// Which branch a ticket service decides first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Order {
    /// Every input-guarded refusal before any accepting branch: the stated precedence.
    RefusalsFirst,
    /// `open` before the identity and the count: the accepting branch wins the overlap.
    AcceptingFirst,
}

type Row = BTreeMap<String, Node>;

struct Tickets {
    order: Order,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Tickets {
    fn new(order: Order) -> Self {
        Self {
            order,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Tickets {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tickets-fixture", "1"))
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
                if let Some(level) = request.input.get("level") {
                    row.insert("level".into(), level.clone());
                }
                rows.push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.tickets.TicketOpened".parse().unwrap())
                        .with("ticket_id", id),
                )
            }
            "demo.tickets.SetTicketOpen" => {
                let id = request.input["ticket_id"].clone();
                // `closed` also needs a positive stored `level` where the ticket carries one (the
                // `when_subject:` variant).
                let level_allows = rows
                    .iter()
                    .find(|row| row["ticket_id"] == id)
                    .and_then(|row| row.get("level"))
                    .is_none_or(
                        |level| matches!(level, Node::Number(n) if *n > Number::from(0_i64)),
                    );
                let closing = request.input["open"] == Node::Bool(false) && level_allows;
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
                if let (Order::RefusalsFirst, Some(refusal)) = (self.order, refusal) {
                    return Ok(refused(refusal).with_consistency(token));
                }
                if let (Order::AcceptingFirst, false, Some(refusal)) =
                    (self.order, closing, refusal)
                {
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

/// The scenarios that do not pass against a ticket service deciding in this order.
fn failing(suite: &ConformanceSuite, order: Order) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Tickets::new(order))
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

/// #178's acceptance: `{ticket_id: "", open: false}` is sent, and `id-required` is required for it.
#[test]
fn issue_178_the_overlap_point_is_sent_and_requires_the_refusal() {
    let sent = invocations(&suite(), ID_REQUIRED);
    let overlap = BTreeMap::from([
        ("ticket_id".to_owned(), literal(Node::Text(String::new()))),
        ("open".to_owned(), literal(Node::Bool(false))),
    ]);
    assert!(
        sent.contains(&(overlap, "id-required".to_owned())),
        "the overlap point is not required to refuse: {sent:#?}"
    );
}

#[test]
fn issue_178_a_service_that_refuses_first_passes_its_own_suite() {
    assert_eq!(failing(&suite(), Order::RefusalsFirst), BTreeSet::new());
}

#[test]
fn issue_178_a_service_that_reads_open_before_the_identity_fails_the_refusal() {
    assert_eq!(
        failing(&suite(), Order::AcceptingFirst),
        BTreeSet::from([ID_REQUIRED.to_owned()])
    );
}

/// An accepting witness refutes a sibling refusal although a default exists: `closed` is never sent
/// a count the refusal claims.
#[test]
fn an_accepting_witness_refutes_every_sibling_refusal_beside_a_default() {
    let suite = suite_of(&counted());
    let sent = invocations(&suite, CLOSED);
    assert!(!sent.is_empty());
    for (input, branch) in &sent {
        if branch != "closed" {
            continue;
        }
        let Some(ScenarioValue::Literal {
            value: Node::Number(count),
        }) = input.get("count")
        else {
            panic!("`closed` is sent a literal count: {input:?}");
        };
        assert!(
            *count >= Number::from(5_i64),
            "`closed` sent a count `too-few` claims: {input:?}"
        );
    }
}

/// With a refusal over an ordinary input, a service honouring the precedence passes, and one that
/// reads `open` first fails exactly the two refusals' overlap checks.
#[test]
fn a_refusal_over_an_ordinary_input_is_held_to_the_precedence() {
    let suite = suite_of(&counted());
    assert_eq!(failing(&suite, Order::RefusalsFirst), BTreeSet::new());
    assert_eq!(
        failing(&suite, Order::AcceptingFirst),
        BTreeSet::from([
            ID_REQUIRED.to_owned(),
            "demo.tickets.SetTicketOpen/outcome/too-few".to_owned(),
        ])
    );
}

/// A command whose input-guarded refusal overlaps no accepting branch gains no invocation.
#[test]
fn a_refusal_disjoint_from_every_accepting_branch_is_sent_once() {
    let text = TICKETS.replace(
        "when: ticket_id == \"\"",
        "when: {all: [open == true, 'ticket_id == \"\"']}",
    );
    assert_ne!(text, TICKETS);
    let sent = invocations(&suite_of(&text), ID_REQUIRED);
    assert_eq!(sent.len(), 1, "{sent:#?}");
}

/// The fixture with `closed` also reading a stored field: `when_subject: {predicate: level > 0}`
/// beside its `when: open == false`, the level set when the ticket is opened.
fn with_subject_fact(text: &str) -> String {
    let out = text
        .replace(
            "    fields: []\n    lifecycle:",
            "    fields:\n      - {name: level, type: Integer}\n    lifecycle:",
        )
        .replace(
            "  - name: demo.tickets.OpenTicket\n    input: []\n",
            "  - name: demo.tickets.OpenTicket\n    input:\n      - {name: level, type: Integer}\n",
        )
        .replace(
            "        instance: ticket_id\n        emits: [demo.tickets.TicketOpened]\n",
            "        instance: ticket_id\n        sets: {level: input.level}\n        emits: [demo.tickets.TicketOpened]\n",
        )
        .replace(
            "      - name: closed\n        when: open == false\n",
            "      - name: closed\n        when_subject: {predicate: 'level > 0'}\n        when: open == false\n",
        )
        .replace(
            "      - {name: state, type: demo.tickets.Ticket.State}\n",
            "      - {name: state, type: demo.tickets.Ticket.State}\n      - {name: level, type: Integer}\n",
        );
    assert_ne!(out, text);
    out
}

/// A refusal over an ordinary input beside a `when_subject:` accepting branch is sent for an
/// arranged row that branch's stored guard admits, with an input both input guards admit, and is
/// required there.
#[test]
fn a_refusal_for_an_arranged_row_is_sent_the_overlap_with_a_subject_fact_branch() {
    let suite = suite_of(&with_subject_fact(&counted()));
    let sent = invocations(&suite, "demo.tickets.SetTicketOpen/outcome/too-few");
    assert!(
        sent.iter().any(|(input, branch)| branch == "too-few"
            && matches!(input.get("ticket_id"), Some(ScenarioValue::Instance { .. }))
            && input.get("open") == Some(&literal(Node::Bool(false)))
            && matches!(
                input.get("count"),
                Some(ScenarioValue::Literal { value: Node::Number(n) }) if *n < Number::from(5_i64)
            )),
        "`too-few` is not sent its overlap with `closed` for an arranged row: {sent:#?}"
    );
    assert_eq!(failing(&suite, Order::RefusalsFirst), BTreeSet::new());
    let failed = failing(&suite, Order::AcceptingFirst);
    assert!(
        failed.contains(ID_REQUIRED)
            && failed.contains("demo.tickets.SetTicketOpen/outcome/too-few"),
        "{failed:?}"
    );
}

/// An external branch's input guard overlaps a refusal too: the provider is never asked for an
/// input the refusal claims, so the refusal is sent the overlap without configuring the provider,
/// and the external branch's witness refutes the refusal.
#[test]
fn a_refusal_overlapping_an_external_branch_is_sent_the_overlap() {
    const MODEL: &str = r#"
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
  - name: demo.mail.RecipientRequired
    fields: []
commands:
  - name: demo.mail.Send
    input:
      - {name: retry, type: Boolean}
      - {name: recipient, type: String}
    outcomes:
      - name: sent
        emits: [demo.mail.Sent]
      - name: deferred
        when: retry == true
        external: the provider defers the delivery
        emits: [demo.mail.Deferred]
      - name: recipient-required
        when: recipient != "nobody"
        error: demo.mail.RecipientRequired
"#;
    let suite = suite_of(MODEL);
    let sent_to = |id: &str| -> Vec<BTreeMap<String, ScenarioValue>> {
        suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)
            .unwrap_or_else(|| panic!("no scenario {id}"))
            .1
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => Some(input.clone()),
                _ => None,
            })
            .collect()
    };
    let nobody = literal(Node::Text("nobody".into()));
    assert!(
        sent_to("demo.mail.Send/outcome/recipient-required")
            .iter()
            .any(
                |input| matches!(input.get("recipient"), Some(value) if value != &nobody)
                    && input.get("retry") == Some(&literal(Node::Bool(true)))
            ),
        "the refusal is not sent its overlap with `deferred`"
    );
    for input in sent_to("demo.mail.Send/outcome/deferred") {
        assert_eq!(input.get("recipient"), Some(&nobody), "{input:?}");
    }
}
