//! Adversary, pass 2, for synthesis of a command guarded by a row of another entity
//! (`when_related:`, ess/18, beyond10x/ess#211), after correction 1 threaded the arranging chain
//! through `invoke`, `created` and `advance`.
//!
//! Each case synthesizes a model and runs the suite against a hand-written in-memory target that
//! answers the model correctly: a scenario that fails against it is a scenario that fails a correct
//! implementation.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn sent<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
) -> Vec<&'a BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: named,
                input,
                ..
            } if named.to_string() == command => Some(input),
            _ => None,
        })
        .collect()
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

// ---- a generic in-memory target -------------------------------------------------------------

/// Rows by entity, each carrying its lifecycle state under `state`, in creation order.
#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, Vec<BTreeMap<String, Node>>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn insert(&self, entity: &str, state: &str, mut row: BTreeMap<String, Node>) {
        row.insert("state".to_owned(), Node::Text(state.to_owned()));
        self.rows
            .borrow_mut()
            .entry(entity.to_owned())
            .or_default()
            .push(row);
    }

    fn find(&self, entity: &str, key: &str, id: &str) -> Option<BTreeMap<String, Node>> {
        self.rows.borrow().get(entity).and_then(|rows| {
            rows.iter()
                .find(|row| row.get(key) == Some(&Node::Text(id.to_owned())))
                .cloned()
        })
    }

    fn set(&self, entity: &str, key: &str, id: &str, field: &str, value: Node) {
        let mut rows = self.rows.borrow_mut();
        let row = rows
            .get_mut(entity)
            .and_then(|rows| {
                rows.iter_mut()
                    .find(|row| row.get(key) == Some(&Node::Text(id.to_owned())))
            })
            .unwrap_or_else(|| panic!("no {entity} {id}"));
        row.insert(field.to_owned(), value);
    }
}

type Handler = Box<dyn Fn(&Store, &CommandRef, &BTreeMap<String, Node>) -> SemanticCommandResult>;

struct Service {
    handler: Handler,
    /// The entity each view projects.
    views: BTreeMap<&'static str, &'static str>,
    store: Store,
}

impl Service {
    fn new(views: &[(&'static str, &'static str)], handler: Handler) -> Self {
        Self {
            handler,
            views: views.iter().copied().collect(),
            store: Store::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn refusal(command: &CommandRef, name: &str, error: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name))
        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
}

fn took(
    command: &CommandRef,
    name: &str,
    event: &str,
    field: &str,
    id: &str,
) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name)).emitting(
        ObservedEvent::new(event.parse::<EventRef>().unwrap()).with(field, Node::Text(id.into())),
    )
}

impl ConformanceTarget for Service {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("related-guard-pass2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.rows.replace(BTreeMap::new());
        self.store.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = (self.handler)(&self.store, &request.command, &request.input);
        for event in &result.direct_events {
            self.store.published.borrow_mut().push(event.clone());
        }
        let token = self.store.mint();
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!("seq:{token}")).unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let name = request.view.to_string();
        let entity = self
            .views
            .get(name.as_str())
            .unwrap_or_else(|| panic!("no view {name}"));
        let rows = self
            .store
            .rows
            .borrow()
            .get(*entity)
            .cloned()
            .unwrap_or_default();
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
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

/// Every scenario whose id contains `command`, and its status against `target`.
fn statuses(result: &Synthesis, target: &Service, command: &str) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains(command))
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> BTreeMap<&String, &String> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .collect()
}

// ---- 1. a caller-supplied identity, arranged twice in one scenario ---------------------------

/// A caller-supplied order id, a create guarded by the customer existing, and a move admitted from
/// two states: the transition scenario arranges a second order in the other source state (#111).
const ORDERS: &str = "format: ess/18
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.CustomerId, kind: newtype, of: Uuid}
  - {name: demo.orders.Label, kind: newtype, of: String}
entities:
  - name: demo.orders.Customer
    identity: {name: customer_id, type: demo.orders.CustomerId}
    fields:
      - {name: label, type: demo.orders.Label}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer, type: demo.orders.CustomerId}
    lifecycle:
      initial: Open
      states: [Open, Held, Closed]
      terminal: [Closed]
      transitions:
        - {name: hold, from: [Open], to: Held}
        - {name: close, from: [Open, Held], to: Closed}
events:
  - name: demo.orders.CustomerAdded
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.OrderPlaced
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.OrderHeld
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.OrderClosed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
errors:
  - {name: demo.orders.OrderExists, summary: The order id is taken., fields: []}
  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.AddCustomer, demo.orders.PlaceOrder, demo.orders.HoldOrder, demo.orders.CloseOrder]
commands:
  - name: demo.orders.AddCustomer
    input:
      - {name: label, type: demo.orders.Label}
    outcomes:
      - name: added
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerAdded]
        payload:
          demo.orders.CustomerAdded: {customer_id: {generated: true}}
        sets:
          label: input.label
  - name: demo.orders.PlaceOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
    outcomes:
      - {name: duplicate, existing_instance: true, error: demo.orders.OrderExists}
      - name: no-customer
        when_related: {via: input.customer, exists: false}
        error: demo.orders.NoCustomer
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderPlaced]
        payload:
          demo.orders.OrderPlaced: {order_id: input.order_id}
        sets:
          customer: input.customer
  - name: demo.orders.HoldOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: held
        moves: demo.orders.Order.hold
        instance: order_id
        emits: [demo.orders.OrderHeld]
        payload: {demo.orders.OrderHeld: {order_id: input.order_id}}
  - name: demo.orders.CloseOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
    outcomes:
      - name: closed
        moves: demo.orders.Order.close
        instance: order_id
        emits: [demo.orders.OrderClosed]
        payload: {demo.orders.OrderClosed: {order_id: input.order_id}}
views:
  - name: demo.orders.Customers
    source: demo.orders.Customer
    consistency: read_your_writes
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: label, type: demo.orders.Label}
  - name: demo.orders.Orders
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: state, type: demo.orders.Order.State}
      - {name: customer, type: demo.orders.CustomerId}
";

fn orders_service() -> Service {
    Service::new(
        &[
            ("demo.orders.Customers", "Customer"),
            ("demo.orders.Orders", "Order"),
        ],
        Box::new(|store, command, input| match command.to_string().as_str() {
            "demo.orders.AddCustomer" => {
                let id = store.mint();
                store.insert(
                    "Customer",
                    "Active",
                    BTreeMap::from([
                        ("customer_id".to_owned(), Node::Text(id.clone())),
                        ("label".to_owned(), input["label"].clone()),
                    ]),
                );
                took(
                    command,
                    "added",
                    "demo.orders.CustomerAdded",
                    "customer_id",
                    &id,
                )
            }
            "demo.orders.PlaceOrder" => {
                let order = text(input.get("order_id"));
                let customer = text(input.get("customer"));
                if store.find("Order", "order_id", &order).is_some() {
                    return refusal(command, "duplicate", "demo.orders.OrderExists");
                }
                if store.find("Customer", "customer_id", &customer).is_none() {
                    return refusal(command, "no-customer", "demo.orders.NoCustomer");
                }
                store.insert(
                    "Order",
                    "Open",
                    BTreeMap::from([
                        ("order_id".to_owned(), Node::Text(order.clone())),
                        ("customer".to_owned(), Node::Text(customer)),
                    ]),
                );
                took(
                    command,
                    "placed",
                    "demo.orders.OrderPlaced",
                    "order_id",
                    &order,
                )
            }
            moving @ ("demo.orders.HoldOrder" | "demo.orders.CloseOrder") => {
                let order = text(input.get("order_id"));
                let (from, to, name, event): (&[&str], _, _, _) = if moving.ends_with("HoldOrder") {
                    (&["Open"], "Held", "held", "demo.orders.OrderHeld")
                } else {
                    (
                        &["Open", "Held"],
                        "Closed",
                        "closed",
                        "demo.orders.OrderClosed",
                    )
                };
                let row = store.find("Order", "order_id", &order).unwrap_or_else(|| {
                    panic!("{moving} for an order that does not exist: {order}")
                });
                let held = text(row.get("state"));
                if !from.contains(&held.as_str()) {
                    // An illegal move: the model declares no answer, so nothing happens.
                    return SemanticCommandResult::undeclared();
                }
                store.set("Order", "order_id", &order, "state", Node::Text(to.into()));
                took(command, name, event, "order_id", &order)
            }
            _ => SemanticCommandResult::undeclared(),
        }),
    )
}

/// Every `PlaceOrder` a `CloseOrder` scenario sends names an order id of its own: the move from
/// `Held` is witnessed on a second order, and a correct target refuses a second order under a taken
/// id as `duplicate`.
#[test]
fn adversary_pass2_two_orders_in_one_scenario_are_placed_under_distinct_supplied_ids() {
    let result = synthesis(ORDERS);
    let mut checked = 0;
    for (id, scenario) in &result.suite.scenarios {
        let placed: Vec<String> = sent(scenario, "demo.orders.PlaceOrder")
            .iter()
            .filter_map(|input| input.get("order_id"))
            .map(|value| format!("{value:?}"))
            .collect();
        if placed.len() < 2 || id.to_string().contains("PlaceOrder") {
            continue;
        }
        checked += 1;
        let distinct: BTreeSet<&String> = placed.iter().collect();
        assert_eq!(
            distinct.len(),
            placed.len(),
            "{id} places two orders under one supplied id: {placed:#?}"
        );
    }
    assert!(
        checked > 0,
        "some scenario places two orders: {:#?}",
        ids(&result)
    );
}

#[test]
fn adversary_pass2_a_correct_target_passes_every_move_scenario_on_a_related_guarded_row() {
    let result = synthesis(ORDERS);
    // ESS-SYNTH-012 is the model's own: it declares no `wrong_state:` answer for an illegal move.
    let refused: Vec<String> = refusals_about(&result, "demo.orders.CloseOrder")
        .into_iter()
        .filter(|refusal| !refusal.starts_with("ESS-SYNTH-012"))
        .collect();
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "the move from `Held` is witnessed (#111)"
    );
    let run = statuses(&result, &orders_service(), "demo.orders");
    assert!(
        run.keys().any(|id| id.contains("CloseOrder")),
        "CloseOrder is run: {run:#?}"
    );
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
}

/// [`ORDERS`] with the order view ranked: a ranked read is witnessed on further orders arranged
/// beside the scenario's own, each through the related-guarded `PlaceOrder`.
fn ranked_orders() -> String {
    replaced(
        ORDERS,
        "      - {name: state, type: demo.orders.Order.State}\n      - {name: customer, type: demo.orders.CustomerId}\n",
        "      - {name: state, type: demo.orders.Order.State}\n      - {name: customer, type: demo.orders.CustomerId}\n    order_by: [order_id asc]\n",
    )
}

#[test]
fn adversary_pass2_orders_arranged_beside_a_ranked_read_carry_distinct_supplied_ids() {
    let result = synthesis(&ranked_orders());
    let mut checked = 0;
    for (id, scenario) in &result.suite.scenarios {
        let placed: Vec<String> = sent(scenario, "demo.orders.PlaceOrder")
            .iter()
            .filter_map(|input| input.get("order_id"))
            .map(|value| format!("{value:?}"))
            .collect();
        if placed.len() < 2 || id.to_string().contains("outcome/duplicate") {
            continue;
        }
        checked += 1;
        let distinct: BTreeSet<&String> = placed.iter().collect();
        assert_eq!(
            distinct.len(),
            placed.len(),
            "{id} places two orders under one supplied id: {placed:#?}"
        );
    }
    let run = statuses(&result, &orders_service(), "demo.orders");
    assert!(
        checked > 0,
        "some scenario places several orders: {:#?}\nrefusals: {:#?}",
        ids(&result),
        refusals_about(&result, "demo.orders")
    );
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
}

#[test]
fn adversary_pass2_a_correct_target_passes_scenarios_with_ranked_companions() {
    let result = synthesis(&ranked_orders());
    let run = statuses(&result, &orders_service(), "demo.orders");
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
}

/// The control: the same ranked model without the related guard passes the same correct target, so
/// the failure above is the guard's arrangement and not the model or the target.
#[test]
fn adversary_pass2_control_without_the_related_guard_orders_carry_distinct_supplied_ids() {
    let text = replaced(
        &ranked_orders(),
        "      - name: no-customer\n        when_related: {via: input.customer, exists: false}\n        error: demo.orders.NoCustomer\n",
        "",
    );
    let result = synthesis(&text);
    let mut checked = 0;
    for (id, scenario) in &result.suite.scenarios {
        let placed: Vec<String> = sent(scenario, "demo.orders.PlaceOrder")
            .iter()
            .filter_map(|input| input.get("order_id"))
            .map(|value| format!("{value:?}"))
            .collect();
        if placed.len() < 2 || id.to_string().contains("outcome/duplicate") {
            continue;
        }
        checked += 1;
        let distinct: BTreeSet<&String> = placed.iter().collect();
        assert_eq!(distinct.len(), placed.len(), "{id}: {placed:#?}");
    }
    assert!(checked > 0, "{:#?}", ids(&result));
}

// ---- 2. the related row is the created row's owner -------------------------------------------

/// An entry is owned by an account and posted only into an account that exists: the related row is
/// the owner, one hop through the same input field.
const LEDGER: &str = r#"format: ess/18
system: ledger
version: v1
domain: ledger.book
types:
  - {name: ledger.book.AccountId, kind: newtype, of: Uuid}
  - {name: ledger.book.EntryId, kind: newtype, of: Uuid}
entities:
  - name: ledger.book.Account
    identity: {name: account_id, type: ledger.book.AccountId}
    fields:
      - {name: holder, type: String}
    relations:
      - {name: entries, kind: owns, target: ledger.book.Entry, cardinality: many, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: ledger.book.Entry
    identity: {name: entry_id, type: ledger.book.EntryId}
    fields:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted]}
actors:
  - name: ledger.book.Clerk
    may: [ledger.book.OpenAccount, ledger.book.PostEntry, ledger.book.AmendEntry]
errors:
  - {name: ledger.book.EmptyMemo, fields: []}
  - {name: ledger.book.NoAccount, fields: []}
events:
  - name: ledger.book.AccountOpened
    fields: [{name: account_id, type: ledger.book.AccountId}]
  - name: ledger.book.EntryPosted
    fields: [{name: entry_id, type: ledger.book.EntryId}]
  - name: ledger.book.EntryAmended
    fields: [{name: entry_id, type: ledger.book.EntryId}]
commands:
  - name: ledger.book.OpenAccount
    input: [{name: holder, type: String}]
    outcomes:
      - name: opened
        creates: ledger.book.Account
        instance: account_id
        sets: {holder: input.holder}
        emits: [ledger.book.AccountOpened]
        payload:
          ledger.book.AccountOpened: {account_id: {generated: true}}
  - name: ledger.book.PostEntry
    input:
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
    outcomes:
      - name: no-account
        when_related: {via: input.account_id, exists: false}
        error: ledger.book.NoAccount
      - name: posted
        creates: ledger.book.Entry
        instance: entry_id
        sets: {account_id: input.account_id, memo: input.memo}
        emits: [ledger.book.EntryPosted]
        payload:
          ledger.book.EntryPosted: {entry_id: {generated: true}}
  - name: ledger.book.AmendEntry
    input:
      - {name: entry_id, type: ledger.book.EntryId}
      - {name: memo, type: String}
    outcomes:
      - {name: empty-memo, when: memo == "", error: ledger.book.EmptyMemo}
      - name: amended
        updates: ledger.book.Entry
        instance: entry_id
        sets: {memo: input.memo}
        emits: [ledger.book.EntryAmended]
        payload:
          ledger.book.EntryAmended: {entry_id: input.entry_id}
views:
  - name: ledger.book.Entries
    source: ledger.book.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: ledger.book.EntryId}
      - {name: state, type: ledger.book.Entry.State}
      - {name: account_id, type: ledger.book.AccountId}
      - {name: memo, type: String}
"#;

fn ledger_service() -> Service {
    Service::new(
        &[("ledger.book.Entries", "Entry")],
        Box::new(|store, command, input| match command.to_string().as_str() {
            "ledger.book.OpenAccount" => {
                let id = store.mint();
                store.insert(
                    "Account",
                    "Open",
                    BTreeMap::from([
                        ("account_id".to_owned(), Node::Text(id.clone())),
                        ("holder".to_owned(), input["holder"].clone()),
                    ]),
                );
                took(
                    command,
                    "opened",
                    "ledger.book.AccountOpened",
                    "account_id",
                    &id,
                )
            }
            "ledger.book.PostEntry" => {
                let account = text(input.get("account_id"));
                if store.find("Account", "account_id", &account).is_none() {
                    return refusal(command, "no-account", "ledger.book.NoAccount");
                }
                let id = store.mint();
                store.insert(
                    "Entry",
                    "Posted",
                    BTreeMap::from([
                        ("entry_id".to_owned(), Node::Text(id.clone())),
                        ("account_id".to_owned(), Node::Text(account)),
                        ("memo".to_owned(), input["memo"].clone()),
                    ]),
                );
                took(
                    command,
                    "posted",
                    "ledger.book.EntryPosted",
                    "entry_id",
                    &id,
                )
            }
            "ledger.book.AmendEntry" => {
                let entry = text(input.get("entry_id"));
                if input.get("memo") == Some(&Node::Text(String::new())) {
                    return refusal(command, "empty-memo", "ledger.book.EmptyMemo");
                }
                store.set("Entry", "entry_id", &entry, "memo", input["memo"].clone());
                took(
                    command,
                    "amended",
                    "ledger.book.EntryAmended",
                    "entry_id",
                    &entry,
                )
            }
            _ => SemanticCommandResult::undeclared(),
        }),
    )
}

#[test]
fn adversary_pass2_a_create_guarded_on_its_own_owner_is_witnessed() {
    let result = synthesis(LEDGER);
    let refused = [
        refusals_about(&result, "ledger.book.PostEntry"),
        refusals_about(&result, "ledger.book.AmendEntry"),
    ]
    .concat();
    let all = ids(&result);
    for id in [
        "ledger.book.PostEntry/outcome/posted",
        "ledger.book.PostEntry/outcome/no-account",
        "ledger.book.AmendEntry/outcome/amended",
        "ledger.book.AmendEntry/outcome/empty-memo",
    ] {
        assert!(
            all.iter().any(|have| have == id),
            "{id} is synthesized; refusals: {refused:#?}"
        );
    }
    assert_eq!(refused, Vec::<String>::new());
    let run = statuses(&result, &ledger_service(), "ledger.book");
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
}

// ---- 3. a cycle two entities long -------------------------------------------------------------

/// An alpha needs a beta, a beta needs an alpha, and an alpha can also be seeded with nothing: every
/// branch is reachable, through the seed.
const CYCLE: &str = "format: ess/18
system: demo
version: v1
domain: demo.cyc
types:
  - {name: demo.cyc.AlphaId, kind: newtype, of: Uuid}
  - {name: demo.cyc.BetaId, kind: newtype, of: Uuid}
entities:
  - name: demo.cyc.Alpha
    identity: {name: alpha_id, type: demo.cyc.AlphaId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: demo.cyc.Beta
    identity: {name: beta_id, type: demo.cyc.BetaId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
errors:
  - {name: demo.cyc.NoAlpha, summary: No such alpha., fields: []}
  - {name: demo.cyc.NoBeta, summary: No such beta., fields: []}
events:
  - name: demo.cyc.AlphaAdded
    fields: [{name: alpha_id, type: demo.cyc.AlphaId}]
  - name: demo.cyc.BetaAdded
    fields: [{name: beta_id, type: demo.cyc.BetaId}]
actors:
  - name: demo.cyc.Maker
    may: [demo.cyc.AddAlpha, demo.cyc.AddBeta, demo.cyc.SeedAlpha]
commands:
  - name: demo.cyc.AddAlpha
    input:
      - {name: beta, type: demo.cyc.BetaId}
    outcomes:
      - name: no-beta
        when_related: {via: input.beta, exists: false}
        error: demo.cyc.NoBeta
      - name: added
        creates: demo.cyc.Alpha
        instance: alpha_id
        emits: [demo.cyc.AlphaAdded]
        payload: {demo.cyc.AlphaAdded: {alpha_id: {generated: true}}}
  - name: demo.cyc.AddBeta
    input:
      - {name: alpha, type: demo.cyc.AlphaId}
    outcomes:
      - name: no-alpha
        when_related: {via: input.alpha, exists: false}
        error: demo.cyc.NoAlpha
      - name: added
        creates: demo.cyc.Beta
        instance: beta_id
        emits: [demo.cyc.BetaAdded]
        payload: {demo.cyc.BetaAdded: {beta_id: {generated: true}}}
  - name: demo.cyc.SeedAlpha
    input: []
    outcomes:
      - name: seeded
        creates: demo.cyc.Alpha
        instance: alpha_id
        emits: [demo.cyc.AlphaAdded]
        payload: {demo.cyc.AlphaAdded: {alpha_id: {generated: true}}}
views:
  - name: demo.cyc.Alphas
    source: demo.cyc.Alpha
    consistency: read_your_writes
    fields: [{name: alpha_id, type: demo.cyc.AlphaId}]
  - name: demo.cyc.Betas
    source: demo.cyc.Beta
    consistency: read_your_writes
    fields: [{name: beta_id, type: demo.cyc.BetaId}]
";

fn cycle_service() -> Service {
    Service::new(
        &[("demo.cyc.Alphas", "Alpha"), ("demo.cyc.Betas", "Beta")],
        Box::new(|store, command, input| {
            let add = |entity: &str, key: &str, event: &str, name: &str| {
                let id = store.mint();
                store.insert(
                    entity,
                    "Live",
                    BTreeMap::from([(key.to_owned(), Node::Text(id.clone()))]),
                );
                took(command, name, event, key, &id)
            };
            match command.to_string().as_str() {
                "demo.cyc.SeedAlpha" => add("Alpha", "alpha_id", "demo.cyc.AlphaAdded", "seeded"),
                "demo.cyc.AddAlpha" => {
                    if store
                        .find("Beta", "beta_id", &text(input.get("beta")))
                        .is_none()
                    {
                        return refusal(command, "no-beta", "demo.cyc.NoBeta");
                    }
                    add("Alpha", "alpha_id", "demo.cyc.AlphaAdded", "added")
                }
                "demo.cyc.AddBeta" => {
                    if store
                        .find("Alpha", "alpha_id", &text(input.get("alpha")))
                        .is_none()
                    {
                        return refusal(command, "no-alpha", "demo.cyc.NoAlpha");
                    }
                    add("Beta", "beta_id", "demo.cyc.BetaAdded", "added")
                }
                _ => SemanticCommandResult::undeclared(),
            }
        }),
    )
}

#[test]
fn adversary_pass2_a_cycle_two_entities_long_stops_and_is_witnessed_through_the_seed() {
    let result = synthesis(CYCLE);
    let refused = [
        refusals_about(&result, "demo.cyc.AddAlpha"),
        refusals_about(&result, "demo.cyc.AddBeta"),
    ]
    .concat();
    let all = ids(&result);
    for id in [
        "demo.cyc.AddAlpha/outcome/no-beta",
        "demo.cyc.AddAlpha/outcome/added",
        "demo.cyc.AddBeta/outcome/no-alpha",
        "demo.cyc.AddBeta/outcome/added",
    ] {
        assert!(
            all.iter().any(|have| have == id),
            "{id} is synthesized; refusals: {refused:#?}"
        );
    }
    let run = statuses(&result, &cycle_service(), "demo.cyc");
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
    // The same model twice gives the same suite.
    assert_eq!(
        result.suite,
        synthesis(CYCLE).suite,
        "synthesis is deterministic"
    );
}

// ---- 4. a predicate over two related fields --------------------------------------------------

/// The sign-in fixture with the configuration on a plan and in a region, and the refusal reading
/// both stored fields: "a basic configuration in the north".
fn two_fields() -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - name: demo.signin.Plan\n    kind: enum\n    variants: [Basic, Premium]\n  - name: demo.signin.Region\n    kind: enum\n    variants: [North, South]\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n      - {name: region, type: demo.signin.Region}\n    lifecycle",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    outcomes:",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n      - {name: region, type: demo.signin.Region}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "          redirect_client: input.redirect_client\n",
        "          redirect_client: input.redirect_client\n          plan: input.plan\n          region: input.region\n",
    );
    replaced(
        &text,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n",
        "        when_related: {via: input.tenant, predicate: {all: [plan == Basic, region == North]}}\n",
    )
}

fn sign_in_service(conjuncts: &'static [(&'static str, &'static str)]) -> Service {
    sign_in_reading(false, conjuncts)
}

/// `conjuncts`: which stored fields the service reads, so a mutant can drop one; `first_row`: the
/// mutant reading the first configuration rather than the one `input.tenant` names, where one exists.
fn sign_in_reading(first_row: bool, conjuncts: &'static [(&'static str, &'static str)]) -> Service {
    Service::new(
        &[
            ("demo.signin.Configurations", "Configuration"),
            ("demo.signin.SignIns", "SignIn"),
        ],
        Box::new(
            move |store, command, input| match command.to_string().as_str() {
                "demo.signin.ConfigureTenant" => {
                    let tenant = store.mint();
                    let mut row =
                        BTreeMap::from([("tenant".to_owned(), Node::Text(tenant.clone()))]);
                    for field in ["redirect_client", "plan", "region"] {
                        row.insert(field.to_owned(), input[field].clone());
                    }
                    store.insert("Configuration", "Active", row);
                    took(
                        command,
                        "configured",
                        "demo.signin.TenantConfigured",
                        "tenant",
                        &tenant,
                    )
                }
                "demo.signin.InitiateSignIn" => {
                    let tenant = text(input.get("tenant"));
                    let Some(named) = store.find("Configuration", "tenant", &tenant) else {
                        return refusal(command, "no-configuration", "demo.signin.NoConfiguration");
                    };
                    let row = if first_row {
                        store.rows.borrow()["Configuration"][0].clone()
                    } else {
                        named
                    };
                    if conjuncts
                        .iter()
                        .all(|(field, value)| row.get(*field) == Some(&Node::Text((*value).into())))
                    {
                        return refusal(
                            command,
                            "no-redirect-entry",
                            "demo.signin.NoRedirectEntry",
                        );
                    }
                    let id = store.mint();
                    store.insert(
                        "SignIn",
                        "Initiated",
                        BTreeMap::from([
                            ("sign_in_id".to_owned(), Node::Text(id.clone())),
                            ("tenant".to_owned(), Node::Text(tenant)),
                            ("client".to_owned(), input["client"].clone()),
                        ]),
                    );
                    took(
                        command,
                        "initiated",
                        "demo.signin.SignInInitiated",
                        "sign_in_id",
                        &id,
                    )
                }
                _ => SemanticCommandResult::undeclared(),
            },
        ),
    )
}

#[test]
fn adversary_pass2_a_predicate_over_two_related_fields_is_witnessed_and_passes_a_correct_target() {
    let result = synthesis(&two_fields());
    assert_eq!(
        refusals_about(&result, "InitiateSignIn"),
        Vec::<String>::new()
    );
    let run = statuses(
        &result,
        &sign_in_service(&[("plan", "Basic"), ("region", "North")]),
        "InitiateSignIn",
    );
    for id in [
        "demo.signin.InitiateSignIn/outcome/no-configuration",
        "demo.signin.InitiateSignIn/outcome/no-redirect-entry",
        "demo.signin.InitiateSignIn/outcome/initiated",
    ] {
        assert!(run.contains_key(id), "{id} is run: {run:#?}");
    }
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
}

/// A target reading the predicate on the first configuration it holds rather than the one the
/// input names is caught on both predicate branches: the decoys either side of the named row are
/// chosen to select another branch (F2).
#[test]
fn adversary_pass2_a_target_reading_the_first_row_for_a_two_field_predicate_fails() {
    let result = synthesis(&two_fields());
    let run = statuses(
        &result,
        &sign_in_reading(true, &[("plan", "Basic"), ("region", "North")]),
        "InitiateSignIn",
    );
    for id in [
        "demo.signin.InitiateSignIn/outcome/no-redirect-entry",
        "demo.signin.InitiateSignIn/outcome/initiated",
    ] {
        assert!(
            failed(&run).contains_key(&id.to_owned()),
            "a target reading the first row passes {id}: {run:#?}"
        );
    }
}

/// A target that reads one conjunct of `plan == Basic && region == North` and drops the other is
/// caught: some scenario sends a row on which the two readings disagree.
#[test]
fn adversary_pass2_a_target_reading_one_conjunct_of_the_related_predicate_fails_a_scenario() {
    let result = synthesis(&two_fields());
    for kept in [&[("plan", "Basic")][..], &[("region", "North")][..]] {
        let run = statuses(&result, &sign_in_service(kept), "InitiateSignIn");
        assert!(
            !failed(&run).is_empty(),
            "a target reading only {kept:?} passes every scenario: {run:#?}"
        );
    }
}

// ---- 5. `existing_instance:` beside both `when_related` shapes -------------------------------

/// A caller-supplied order id, a customer that must exist, and a refusal for a blocked customer.
fn blocked_orders() -> String {
    let text = replaced(
        ORDERS,
        "  - {name: demo.orders.Label, kind: newtype, of: String}\n",
        "  - {name: demo.orders.Label, kind: newtype, of: String}\n  - {name: demo.orders.Standing, kind: enum, variants: [Good, Blocked]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: label, type: demo.orders.Label}\n    lifecycle: {initial: Active",
        "      - {name: label, type: demo.orders.Label}\n      - {name: standing, type: demo.orders.Standing}\n    lifecycle: {initial: Active",
    );
    let text = replaced(
        &text,
        "  - name: demo.orders.AddCustomer\n    input:\n      - {name: label, type: demo.orders.Label}\n",
        "  - name: demo.orders.AddCustomer\n    input:\n      - {name: label, type: demo.orders.Label}\n      - {name: standing, type: demo.orders.Standing}\n",
    );
    let text = replaced(
        &text,
        "          label: input.label\n",
        "          label: input.label\n          standing: input.standing\n",
    );
    let text = replaced(
        &text,
        "  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}\n",
        "  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}\n  - {name: demo.orders.Blocked, summary: The customer is blocked., fields: []}\n",
    );
    replaced(
        &text,
        "        error: demo.orders.NoCustomer\n",
        "        error: demo.orders.NoCustomer\n      - name: blocked\n        when_related: {via: input.customer, predicate: standing == Blocked}\n        error: demo.orders.Blocked\n",
    )
}

/// `related_first`: the mutant that reads the customer before the order id.
fn blocked_service(related_first: bool) -> Service {
    Service::new(
        &[
            ("demo.orders.Customers", "Customer"),
            ("demo.orders.Orders", "Order"),
        ],
        Box::new(
            move |store, command, input| match command.to_string().as_str() {
                "demo.orders.AddCustomer" => {
                    let id = store.mint();
                    store.insert(
                        "Customer",
                        "Active",
                        BTreeMap::from([
                            ("customer_id".to_owned(), Node::Text(id.clone())),
                            ("label".to_owned(), input["label"].clone()),
                            ("standing".to_owned(), input["standing"].clone()),
                        ]),
                    );
                    took(
                        command,
                        "added",
                        "demo.orders.CustomerAdded",
                        "customer_id",
                        &id,
                    )
                }
                "demo.orders.PlaceOrder" => {
                    let order = text(input.get("order_id"));
                    let customer = text(input.get("customer"));
                    let taken = store.find("Order", "order_id", &order).is_some();
                    let row = store.find("Customer", "customer_id", &customer);
                    let related = || match &row {
                        None => Some(refusal(command, "no-customer", "demo.orders.NoCustomer")),
                        Some(row) if row.get("standing") == Some(&Node::Text("Blocked".into())) => {
                            Some(refusal(command, "blocked", "demo.orders.Blocked"))
                        }
                        Some(_) => None,
                    };
                    if related_first {
                        if let Some(answer) = related() {
                            return answer;
                        }
                    }
                    if taken {
                        return refusal(command, "duplicate", "demo.orders.OrderExists");
                    }
                    if let Some(answer) = related() {
                        return answer;
                    }
                    store.insert(
                        "Order",
                        "Open",
                        BTreeMap::from([
                            ("order_id".to_owned(), Node::Text(order.clone())),
                            ("customer".to_owned(), Node::Text(customer)),
                        ]),
                    );
                    took(
                        command,
                        "placed",
                        "demo.orders.OrderPlaced",
                        "order_id",
                        &order,
                    )
                }
                _ => orders_service().handler.as_ref()(store, command, input),
            },
        ),
    )
}

#[test]
fn adversary_pass2_existing_instance_beside_exists_false_and_a_predicate_answers_first() {
    let result = synthesis(&blocked_orders());
    assert_eq!(
        refusals_about(&result, "demo.orders.PlaceOrder"),
        Vec::<String>::new()
    );
    let run = statuses(&result, &blocked_service(false), "demo.orders.PlaceOrder");
    for id in [
        "demo.orders.PlaceOrder/outcome/duplicate",
        "demo.orders.PlaceOrder/outcome/no-customer",
        "demo.orders.PlaceOrder/outcome/blocked",
        "demo.orders.PlaceOrder/outcome/placed",
    ] {
        assert!(run.contains_key(id), "{id} is run: {run:#?}");
    }
    assert!(
        failed(&run).is_empty(),
        "a correct target fails: {:#?}",
        failed(&run)
    );
    let mutant = statuses(&result, &blocked_service(true), "demo.orders.PlaceOrder");
    assert!(
        failed(&mutant).contains_key(&"demo.orders.PlaceOrder/outcome/duplicate".to_owned()),
        "a target reading the customer first passes: {mutant:#?}"
    );
}

// ---- 6. determinism ---------------------------------------------------------------------------

#[test]
fn adversary_pass2_synthesis_of_related_guards_is_deterministic() {
    for text in [
        ORDERS.to_owned(),
        LEDGER.to_owned(),
        two_fields(),
        blocked_orders(),
    ] {
        let first = synthesis(&text);
        let second = synthesis(&text);
        assert_eq!(first.suite, second.suite, "two syntheses differ");
        assert_eq!(
            format!("{:?}", first.refusals),
            format!("{:?}", second.refusals),
            "two syntheses refuse differently"
        );
    }
}
