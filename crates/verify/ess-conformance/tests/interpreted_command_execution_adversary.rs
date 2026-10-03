//! Adversary pass 1 against `story:interpreted-command-execution`.
//!
//! Each case drives [`execute`] — the library step the linearizability checker will call as its
//! sequential model — or the [`Interpreted`] target, and states what the model or the hand-written
//! reference says it must answer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Store, Undetermined,
};
use ess_conformance::mutate;
use ess_conformance::reference::Billing;
use ess_conformance::scenario::CommandRef;
use ess_conformance::target::{ConformanceTarget, SemanticCommandRequest};
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a well-formed name")
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    mutate::compile(vec![(Source::new("fixture.yaml"), raw)], &texts).unwrap_or_else(|stillborn| {
        panic!(
            "the model compiles: {} {} {}",
            stillborn.code, stillborn.cause, stillborn.message
        )
    })
}

fn outcomes(steps: &[ess_conformance::interpret::execute::Step]) -> BTreeSet<String> {
    steps
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
}

// ---- 1. a created identity the model publishes as `{generated: true}` ---------------------------

/// `creates:` with `instance: order_id`, and the announcement's `order_id` declared
/// `{generated: true}` — the shape `adversary_connective_and_source_mutants.rs` (`ORDERS`) uses.
/// The event's `order_id` *is* the new instance's identity (`instance:` says so); `generated` says
/// only that the implementation assigns it.
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

#[test]
fn adversary_a_created_identity_published_as_generated_is_the_identity_the_instance_is_held_under()
{
    let ir = compiled(ORDERS);
    let opened = execute(
        &ir,
        &Store::default(),
        &name("demo.orders.OpenOrder"),
        &BTreeMap::from([("note".to_owned(), Node::Text("n".to_owned()))]),
        &Externals::Withheld,
    )
    .expect("the model determines OpenOrder");
    assert_eq!(opened.len(), 1);
    let published = opened[0].events[0].payload["order_id"]
        .as_text()
        .expect("the identity is published as text")
        .to_owned();
    let held: Vec<String> = opened[0]
        .next
        .text_instances()
        .map(|(_, identity, _)| identity.to_owned())
        .collect();
    assert_eq!(
        held,
        std::slice::from_ref(&published),
        "`instance: order_id` makes the published `order_id` the created order's identity; \
         the interpreter holds the order under one minted value and announces another"
    );

    // What the divergence costs: the next command a client sends names the order it was told about,
    // and the model moves it `Open -> Closed`.
    let closed = execute(
        &ir,
        &opened[0].next,
        &name("demo.orders.CloseOrder"),
        &BTreeMap::from([("order_id".to_owned(), Node::Text(published))]),
        &Externals::Withheld,
    )
    .expect("the model determines CloseOrder");
    assert_eq!(
        outcomes(&closed),
        BTreeSet::from(["demo.orders.CloseOrder/closed".to_owned()]),
        "closing the order that was announced is `closed`, not a refusal the model does not \
         declare for an Open order"
    );
}

// ---- 2. two overlapping guards that both lead to the one declared refusal -----------------------

/// `switched` and `smashed` overlap over `force > 5` — refused by the model only where the input
/// domain is finite (`docs/design/mutation-audit-and-model-runner.md`), so over `Integer` it stands.
/// Neither transition starts from `On`, so from `On` either branch answers `wrong-state`: the model
/// leaves the branch open and the answer determined.
const LAMPS: &str = r"
format: ess/1
system: demo
version: v1
domain: demo.lamp
types:
  - {name: demo.lamp.LampId, kind: newtype, of: Uuid}
entities:
  - name: demo.lamp.Lamp
    identity: {name: lamp_id, type: demo.lamp.LampId}
    lifecycle:
      initial: Off
      states: [Off, On, Broken]
      terminal: [On, Broken]
      transitions:
        - {name: switch, from: [Off], to: On}
        - {name: smash, from: [Off], to: Broken}
errors:
  - name: demo.lamp.TooSoft
    summary: Nothing happened.
  - name: demo.lamp.LampStateConflict
    summary: Not from here.
commands:
  - name: demo.lamp.Make
    input:
      - {name: label, type: String}
    outcomes:
      - name: made
        creates: demo.lamp.Lamp
        instance: lamp_id
        emits: [demo.lamp.Made]
  - name: demo.lamp.Hit
    input:
      - {name: lamp_id, type: demo.lamp.LampId}
      - {name: force, type: Integer}
    outcomes:
      - name: switched
        when: force > 0
        moves: demo.lamp.Lamp.switch
        instance: lamp_id
        emits: [demo.lamp.Switched]
        payload:
          demo.lamp.Switched: {lamp_id: input.lamp_id}
      - name: smashed
        when: force > 5
        moves: demo.lamp.Lamp.smash
        instance: lamp_id
        emits: [demo.lamp.Smashed]
        payload:
          demo.lamp.Smashed: {lamp_id: input.lamp_id}
      - name: soft
        error: demo.lamp.TooSoft
      - {name: wrong-state, wrong_state: true, error: demo.lamp.LampStateConflict}
events:
  - name: demo.lamp.Made
    fields:
      - {name: lamp_id, type: demo.lamp.LampId}
  - name: demo.lamp.Switched
    fields:
      - {name: lamp_id, type: demo.lamp.LampId}
  - name: demo.lamp.Smashed
    fields:
      - {name: lamp_id, type: demo.lamp.LampId}
";

#[test]
fn adversary_a_determined_refusal_reached_through_two_open_branches_is_answered() {
    use ess_conformance::interpret::Interpreted;
    use ess_conformance::target::ScenarioContext;

    let ir = compiled(LAMPS);
    let target = Interpreted::for_model(ir.clone());
    let correlation = CorrelationId::new("adversary-lamp").expect("an id");
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.lamp.Hit/outcome/wrong-state"
                .parse()
                .expect("a scenario id"),
            correlation.clone(),
        ))
        .expect("a context");
    let send = |command: &str, input: BTreeMap<String, Node>| {
        target.execute_command(SemanticCommandRequest {
            command: CommandRef::new(name(command)),
            actor: None,
            caller: None,
            input,
            correlation: correlation.clone(),
        })
    };
    let made = send(
        "demo.lamp.Make",
        BTreeMap::from([("label".to_owned(), Node::Text("l".to_owned()))]),
    )
    .expect("Make is determined");
    let lamp = made.direct_events[0].payload["lamp_id"].clone();
    let hit = |force: i64| {
        send(
            "demo.lamp.Hit",
            BTreeMap::from([
                ("lamp_id".to_owned(), lamp.clone()),
                ("force".to_owned(), Node::Number(Number::from(force))),
            ]),
        )
    };
    let on = hit(1).expect("force 1 selects only `switched`");
    assert_eq!(
        on.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.lamp.Hit/switched")
    );

    // From `On`, `force: 10` selects both branches, and both answer the one declared refusal.
    let again = hit(10);
    assert_eq!(
        again
            .as_ref()
            .ok()
            .and_then(|result| result.outcome.as_ref())
            .map(ToString::to_string)
            .as_deref(),
        Some("demo.lamp.Hit/wrong-state"),
        "every branch the model leaves open answers `wrong-state`, so the answer is determined and \
         the interpreter need not choose: {again:?}"
    );
}

// ---- 3. a sequential history of the unfaulted reference, replayed through the step --------------

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
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    ess_compiler::resolve::compile(&specification, &sources).expect("billing resolves")
}

fn money(amount: i64) -> Node {
    Node::Map(BTreeMap::from([
        ("amount".to_owned(), Node::Number(Number::from(amount))),
        ("currency".to_owned(), Node::Text("EUR".to_owned())),
    ]))
}

fn create_input() -> BTreeMap<String, Node> {
    BTreeMap::from([
        (
            "account_id".to_owned(),
            Node::Text("00000000-0000-4000-8000-00000000abcd".to_owned()),
        ),
        (
            "customer_email".to_owned(),
            Node::Text("someone@example.invalid".to_owned()),
        ),
        ("amount".to_owned(), money(5)),
    ])
}

/// The linearizability checker's second acceptance line: "a recorded two-client history against the
/// unfaulted `Billing` target is `Linearizable`". The history here is sequential — one client, three
/// operations — so its only linearization is the order it was recorded in. The checker holds only
/// what the history recorded: the inputs, and the identities `Billing` published.
#[test]
fn adversary_a_sequential_history_of_the_unfaulted_reference_is_one_the_step_allows() {
    let billing = Billing::new();
    let correlation = CorrelationId::new("adversary-history").expect("an id");
    let send = |command: &str, input: BTreeMap<String, Node>| {
        billing
            .execute_command(SemanticCommandRequest {
                command: CommandRef::new(name(command)),
                actor: None,
                caller: None,
                input,
                correlation: correlation.clone(),
            })
            .expect("billing answers")
    };
    let first = send("billing.invoice.CreateInvoice", create_input());
    let second = send("billing.invoice.CreateInvoice", create_input());
    let second_id = second.direct_events[0].payload["invoice_id"].clone();
    let issue = BTreeMap::from([
        ("invoice_id".to_owned(), second_id),
        (
            "issued_at".to_owned(),
            Node::Text("2026-01-05T09:00:01Z".to_owned()),
        ),
    ]);
    let recorded = send("billing.invoice.IssueInvoice", issue.clone());
    assert_eq!(
        recorded
            .outcome
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("billing.invoice.IssueInvoice/issued"),
        "the reference issued the second invoice"
    );
    assert!(first.outcome.is_some());

    // The same three operations, in the same order, through the step.
    // Each creation is handed the identity `Billing` published for it, which the history recorded.
    let ir = billing_model();
    let mut store = Store::default();
    for published in [&first, &second] {
        let recorded_id = published.direct_events[0].payload["invoice_id"].clone();
        let steps = execute_generating(
            &ir,
            &store,
            &name("billing.invoice.CreateInvoice"),
            &create_input(),
            &Externals::Withheld,
            &Generated::Given(BTreeMap::from([(
                GeneratedSlot::new(name("billing.invoice.InvoiceCreated"), "invoice_id"),
                recorded_id,
            )])),
        )
        .expect("the model determines CreateInvoice");
        store = steps[0].next.clone();
    }
    let allowed = outcomes(
        &execute(
            &ir,
            &store,
            &name("billing.invoice.IssueInvoice"),
            &issue,
            &Externals::Withheld,
        )
        .expect("the model determines IssueInvoice"),
    );
    assert!(
        allowed.contains("billing.invoice.IssueInvoice/issued"),
        "the recorded answer `issued` is one the model allows after two creations; the step \
         allows only {allowed:?}, because it fixed the identities the implementation assigns"
    );
}

// ---- 4. mutants the committed suite does not reach (green now; each kills one) ------------------

/// `invariants:` at rest, and `creates: … into:`. Billing never leaves an instance violating an
/// invariant and declares no `into:`, so deleting `at_rest` or ignoring `into` passes every other
/// case.
const COUNTERS: &str = r"
format: ess/15
system: demo
version: v1
domain: demo.count
types:
  - {name: demo.count.CounterId, kind: newtype, of: Uuid}
entities:
  - name: demo.count.Counter
    identity: {name: counter_id, type: demo.count.CounterId}
    fields:
      - {name: value, type: Integer}
    invariants:
      - value >= 0
    lifecycle:
      initial: Idle
      states: [Idle, Running]
      terminal: [Running]
      transitions:
        - {name: run, from: [Idle], to: Running}
commands:
  - name: demo.count.Start
    input:
      - {name: value, type: Integer}
    outcomes:
      - name: started
        creates: demo.count.Counter
        into: Running
        instance: counter_id
        sets: {value: input.value}
        emits: [demo.count.Started]
        payload:
          demo.count.Started: {counter_id: {generated: true}}
  - name: demo.count.Run
    input:
      - {name: counter_id, type: demo.count.CounterId}
    outcomes:
      - name: ran
        moves: demo.count.Counter.run
        instance: counter_id
        emits: [demo.count.Ran]
        payload:
          demo.count.Ran: {counter_id: input.counter_id}
events:
  - name: demo.count.Ran
    fields:
      - {name: counter_id, type: demo.count.CounterId}
  - name: demo.count.Started
    fields:
      - {name: counter_id, type: demo.count.CounterId}
";

#[test]
fn adversary_a_step_that_leaves_an_instance_violating_its_invariant_is_not_answered() {
    let ir = compiled(COUNTERS);
    let start = |value: i64| {
        execute(
            &ir,
            &Store::default(),
            &name("demo.count.Start"),
            &BTreeMap::from([("value".to_owned(), Node::Number(Number::from(value)))]),
            &Externals::Withheld,
        )
    };
    let ok = start(1).expect("a non-negative value rests");
    let (_, _, held) = ok[0].next.text_instances().next().expect("created");
    assert_eq!(
        held.state.as_str(),
        "Running",
        "`into: Running`, not `initial`"
    );
    assert!(
        matches!(start(-1), Err(Undetermined::BrokenInvariant { .. })),
        "`value >= 0` is violated by what the model's own outcome wrote: {:?}",
        start(-1)
    );
}
