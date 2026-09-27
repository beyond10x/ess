//! Adversary pass 2 against `story:interpreted-command-execution`: the `Generated::Given` correction.
//!
//! Each case drives [`execute_generating`] — the library step the linearizability checker will call
//! as its sequential model, handing it the values a recorded history published — and states what the
//! model, or the step's own documentation of `Generated::Given`, says it must answer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{
    execute_generating, Externals, Generated, GeneratedSlot, Step, Store,
};
use ess_conformance::mutate;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a well-formed name")
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn compiled(source: &str) -> EssIr {
    let raw = RawSpecFile::parse(source).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), source.to_owned());
    mutate::compile(vec![(Source::new("fixture.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!(
            "the model compiles: {} {} {}",
            stillborn.code, stillborn.cause, stillborn.message
        )
    })
}

fn outcomes(steps: &[Step]) -> BTreeSet<String> {
    steps
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
}

fn billing_model() -> EssIr {
    let base: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside")
            .display()
            .to_string();
        let source = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&source).expect("well formed");
        sources.insert(label.clone(), source);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    ess_compiler::resolve::compile(&specification, &sources).expect("billing resolves")
}

// ---- 1. a branch that mints more than another hides the branch the history recorded -------------

/// `billing.email.SendEmail`: `sent` mints `message_id` (one value), `failed` is external and mints
/// nothing. A history in which the provider rejected the address recorded `failed` and published no
/// value, so the checker, searching with every external branch open, hands the step exactly that:
/// nothing. `failed` is an outcome the model allows; the error for `sent` must not take it away.
#[test]
fn adversary2_a_branch_that_mints_more_than_was_recorded_does_not_hide_the_recorded_branch() {
    let ir = billing_model();
    let answered = execute_generating(
        &ir,
        &Store::default(),
        &name("billing.email.SendEmail"),
        &BTreeMap::from([
            ("recipient".to_owned(), text("someone@example.invalid")),
            ("template".to_owned(), text("reminder")),
        ]),
        &Externals::Open,
        &Generated::Given(BTreeMap::new()),
    );
    let allowed = answered.as_ref().map(|steps| outcomes(steps));
    assert!(
        allowed
            .as_ref()
            .is_ok_and(|allowed| allowed.contains("billing.email.SendEmail/failed")),
        "the recorded `failed` published no value and is an outcome the model allows under open \
         externals; one branch running out of given values refuses the whole step: {allowed:?}"
    );
}

// ---- 2. `sets: {generated: true}` consumes values the documented order gives to events ----------

/// `ess/14` admits `{generated: true}` in `sets:` (`docs/design/value-expressions.md` E3). `Given`
/// documents its order as "a created instance's identity first, then each generated field of each
/// emitted event in declared order" — `sets:` is not in it.
const STAMPS: &str = r"
format: ess/15
system: demo
version: v1
domain: demo.stamp
types:
  - {name: demo.stamp.TicketId, kind: newtype, of: Uuid}
  - {name: demo.stamp.Stamp, kind: newtype, of: Uuid}
  - {name: demo.stamp.Receipt, kind: newtype, of: Uuid}
entities:
  - name: demo.stamp.Ticket
    identity: {name: ticket_id, type: demo.stamp.TicketId}
    fields:
      - {name: stamp, type: demo.stamp.Stamp}
    lifecycle:
      initial: Open
      states: [Open, Stamped]
      terminal: [Stamped]
      transitions:
        - {name: stamp, from: [Open], to: Stamped}
commands:
  - name: demo.stamp.OpenTicket
    input: []
    outcomes:
      - name: opened
        creates: demo.stamp.Ticket
        instance: ticket_id
        emits: [demo.stamp.Opened]
        payload:
          demo.stamp.Opened: {ticket_id: {generated: true}}
  - name: demo.stamp.StampTicket
    input:
      - {name: ticket_id, type: demo.stamp.TicketId}
    outcomes:
      - name: stamped
        moves: demo.stamp.Ticket.stamp
        instance: ticket_id
        sets: {stamp: {generated: true}}
        emits: [demo.stamp.Stamped]
        payload:
          demo.stamp.Stamped: {ticket_id: input.ticket_id, receipt: {generated: true}}
events:
  - name: demo.stamp.Opened
    fields:
      - {name: ticket_id, type: demo.stamp.TicketId}
  - name: demo.stamp.Stamped
    fields:
      - {name: ticket_id, type: demo.stamp.TicketId}
      - {name: receipt, type: demo.stamp.Receipt}
";

const TICKET: &str = "00000000-0000-4000-8000-00000000000a";
const RECEIPT: &str = "00000000-0000-4000-8000-00000000000b";

#[test]
fn adversary2_given_values_go_to_event_fields_in_the_order_given_documents() {
    let ir = compiled(STAMPS);
    let opened = execute_generating(
        &ir,
        &Store::default(),
        &name("demo.stamp.OpenTicket"),
        &BTreeMap::new(),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new(name("demo.stamp.Opened"), "ticket_id"),
            text(TICKET),
        )])),
    )
    .expect("the model determines OpenTicket");
    let stamped = execute_generating(
        &ir,
        &opened[0].next,
        &name("demo.stamp.StampTicket"),
        &BTreeMap::from([("ticket_id".to_owned(), text(TICKET))]),
        &Externals::Withheld,
        // What the history recorded: the one generated field `Stamped` published.
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new(name("demo.stamp.Stamped"), "receipt"),
            text(RECEIPT),
        )])),
    );
    let receipt = stamped
        .as_ref()
        .ok()
        .and_then(|steps| steps.first())
        .and_then(|step| step.events.first())
        .and_then(|event| event.payload.get("receipt"))
        .cloned();
    assert_eq!(
        receipt,
        Some(text(RECEIPT)),
        "`Generated::Given` says values go to the created identity and then to generated event \
         fields in declared order; the `sets:` write took the recorded receipt first: {stamped:?}"
    );
}

// ---- 3. a given identity already held replaces the instance holding it --------------------------

const ORDERS: &str = r"
format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
  - {name: demo.orders.Note, kind: newtype, of: String}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: note, type: demo.orders.Note}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.OpenOrder, demo.orders.CloseOrder]}
errors:
  - name: demo.orders.OrderStateConflict
    summary: Already closed.
commands:
  - name: demo.orders.OpenOrder
    input:
      - {name: note, type: demo.orders.Note}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        sets: {note: input.note}
        emits: [demo.orders.OrderOpened]
        payload:
          demo.orders.OrderOpened: {note: input.note, order_id: {generated: true}}
  - name: demo.orders.CloseOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.OrderClosed]
        payload:
          demo.orders.OrderClosed: {order_id: input.order_id}
      - {name: wrong-state, wrong_state: true, error: demo.orders.OrderStateConflict}
events:
  - name: demo.orders.OrderOpened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: note, type: demo.orders.Note}
  - name: demo.orders.OrderClosed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
";

const ORDER: &str = "00000000-0000-4000-8000-0000000000aa";

fn open_order(ir: &EssIr, store: &Store, generated: Vec<Node>) -> Result<Vec<Step>, String> {
    execute_generating(
        ir,
        store,
        &name("demo.orders.OpenOrder"),
        &BTreeMap::from([("note".to_owned(), text("n"))]),
        &Externals::Withheld,
        // Each value is the one `OrderOpened` published for the new order's identity.
        &Generated::Given(
            generated
                .into_iter()
                .map(|value| {
                    (
                        GeneratedSlot::new(name("demo.orders.OrderOpened"), "order_id"),
                        value,
                    )
                })
                .collect(),
        ),
    )
    .map_err(|why| why.to_string())
}

/// A faulty implementation that hands out an identity it already assigned. `creates:` brings a new
/// instance into existence; the step instead silently replaces the closed order with a fresh open one,
/// so a checker would accept the history as one the model allows.
#[test]
fn adversary2_a_given_identity_already_held_never_replaces_the_instance_holding_it() {
    let ir = compiled(ORDERS);
    let opened = open_order(&ir, &Store::default(), vec![text(ORDER)]).expect("opened");
    let closed = execute_generating(
        &ir,
        &opened[0].next,
        &name("demo.orders.CloseOrder"),
        &BTreeMap::from([("order_id".to_owned(), text(ORDER))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::new()),
    )
    .expect("closed");
    let store = closed[0].next.clone();
    let order = name("demo.orders.Order");
    assert_eq!(
        store.instance(&order, ORDER).map(|it| it.state.as_str()),
        Some("Closed")
    );

    let reopened = open_order(&ir, &store, vec![text(ORDER)]);
    let replaced: Vec<String> = reopened
        .iter()
        .flatten()
        .filter(|step| step.next.instance(&order, ORDER) != store.instance(&order, ORDER))
        .map(|step| format!("{:?}", step.next.instance(&order, ORDER)))
        .collect();
    assert!(
        replaced.is_empty(),
        "an identity the store already holds is not a new instance; the step overwrote the closed \
         order with {replaced:?}"
    );
}

// ---- 4. a given value is not checked against the type it fills -----------------------------------

/// The IR's `Generated` is "explicit implementation ownership, retaining ordinary type assertions".
/// A faulty implementation publishing text that is not a UUID for a `Uuid` identity is not a history
/// the model allows.
#[test]
fn adversary2_a_given_value_that_is_not_of_the_declared_type_is_not_assigned() {
    let ir = compiled(ORDERS);
    let answered = open_order(&ir, &Store::default(), vec![text("not-a-uuid")]);
    assert!(
        answered.is_err(),
        "`not-a-uuid` is not a value of `demo.orders.OrderId` (`Uuid`), and the step assigned it: \
         {answered:?}"
    );
}

// ---- 5. an optional generated field the implementation published --------------------------------

const SLIPS: &str = r"
format: ess/15
system: demo
version: v1
domain: demo.slip
types:
  - {name: demo.slip.SlipId, kind: newtype, of: Uuid}
entities:
  - name: demo.slip.Slip
    identity: {name: slip_id, type: demo.slip.SlipId}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
commands:
  - name: demo.slip.Write
    input: []
    outcomes:
      - name: written
        creates: demo.slip.Slip
        instance: slip_id
        emits: [demo.slip.Written]
        payload:
          demo.slip.Written:
            slip_id: {generated: true}
            trace: {generated: true}
            number: {generated: true}
events:
  - name: demo.slip.Written
    fields:
      - {name: slip_id, type: demo.slip.SlipId}
      - {name: trace, type: Optional<Uuid>}
      - {name: number, type: Uuid}
";

/// The implementation published `trace` — an allowed value of `Optional<Uuid>` — and `number`. A
/// checker hands the step what the event carried, in declared order. The step publishes no `trace`
/// and gives `number` the value recorded for `trace`.
#[test]
fn adversary2_an_optional_generated_field_that_was_published_does_not_shift_the_next_value() {
    let ir = compiled(SLIPS);
    let slip = "00000000-0000-4000-8000-000000000001";
    let trace = "00000000-0000-4000-8000-000000000002";
    let number = "00000000-0000-4000-8000-000000000003";
    let written = execute_generating(
        &ir,
        &Store::default(),
        &name("demo.slip.Write"),
        &BTreeMap::new(),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([
            (
                GeneratedSlot::new(name("demo.slip.Written"), "slip_id"),
                text(slip),
            ),
            (
                GeneratedSlot::new(name("demo.slip.Written"), "trace"),
                text(trace),
            ),
            (
                GeneratedSlot::new(name("demo.slip.Written"), "number"),
                text(number),
            ),
        ])),
    )
    .expect("the model determines Write");
    let payload = &written[0].events[0].payload;
    assert_eq!(
        (payload.get("trace"), payload.get("number")),
        (Some(&text(trace)), Some(&text(number))),
        "the recorded event is one the model allows; the step answers {payload:?}"
    );
}
