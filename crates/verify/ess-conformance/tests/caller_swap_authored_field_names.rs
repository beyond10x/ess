//! beyond10x/ess#766: the caller-swapped run renames only the instance and instant names it binds.
//! An authored field named `instance` or `instant`, in any authored position, keeps the value the
//! run sends, so the interpreter, which is correct, passes every synthesized scenario.
//!
//! The same run expects the identity it sends wherever a copy of it sits in an event payload or an
//! error's fields, at any depth: a struct with a member the scenario does not determine is asserted
//! member by member under a dotted key (`data.id`), and the swapped run's fresh identity replaces
//! the first run's there too.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/19
system: example
version: v1
summary: Fields named instance on a model whose commands read the caller.
domain: example.order
types:
  - {name: example.order.OrderId, kind: newtype, of: Uuid}
  - {name: example.order.AccountId, kind: newtype, of: Uuid}
  - name: example.order.Pair
    kind: struct
    fields: [{name: kind, type: String}, {name: instance, type: String}]
  - name: example.order.Wrap
    kind: struct
    fields: [{name: instance, type: String}]
entities:
  - name: example.order.Order
    identity: {name: order_id, type: example.order.OrderId}
    fields:
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: holder, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: example.order.AccountUser
    attributes:
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
    may: [example.order.PlaceOrder]
errors:
  - name: example.order.Refused
    summary: The value is not the caller's.
    fields:
      - {name: instance, type: String}
commands:
  - name: example.order.PlaceOrder
    input:
      - {name: instance, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
    outcomes:
      - name: refused
        when: instance != caller.instance
        error: example.order.Refused
        payload:
          example.order.Refused: {instance: {caller: instance}}
      - name: placed
        creates: example.order.Order
        instance: order_id
        sets:
          account_id: {caller: account_id}
          instance: input.instance
          holder: {caller: instance}
          pair: input.pair
          wrap: input.wrap
        emits: [example.order.OrderPlaced]
        payload:
          example.order.OrderPlaced:
            order_id: {generated: true}
            account_id: {caller: account_id}
            instance: input.instance
            pair: input.pair
            wrap: input.wrap
events:
  - name: example.order.OrderPlaced
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
views:
  - name: example.order.OrderDetails
    source: example.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: holder, type: String}
      - {name: pair, type: example.order.Pair}
      - {name: wrap, type: example.order.Wrap}
      - {name: state, type: example.order.Order.State}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(suite: &ConformanceSuite, ir: &EssIr) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn assert_all_pass(text: &str) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    let results = statuses(&synthesis.suite, &ir);
    assert!(!results.is_empty(), "scenarios are synthesized");
    let failed: Vec<_> = results
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "every scenario passes: {failed:#?}");
    let suite = serde_json::to_string(&synthesis.suite).unwrap();
    assert!(suite.contains("-swapped"), "a swapped run is synthesized");
}

#[test]
fn an_authored_field_named_instance_keeps_its_value_in_the_swapped_run() {
    assert_all_pass(MODEL);
}

#[test]
fn an_authored_field_named_instant_keeps_its_value_in_the_swapped_run() {
    let text = MODEL
        .replace("instance", "instant")
        .replace("instant: order_id", "instance: order_id");
    assert_all_pass(&text);
}

/// A command two actors, each with a `desk` attribute and no role in common, are both granted,
/// creating a ticket from an input identity it echoes into a struct member of the event it emits;
/// the struct's `label` is generated, so the member is asserted under the dotted key `data.id`. The
/// swapped run exchanges the callers and draws a fresh identity.
const TICKETS: &str = r"format: ess/37
system: demo
version: v1
summary: An input identity echoed into a nested event member, sent by two actors.
domain: demo.tickets
types:
  - {name: demo.tickets.TicketId, kind: newtype, of: Uuid}
  - name: demo.tickets.Receipt
    kind: struct
    fields:
      - {name: id, type: demo.tickets.TicketId}
      - {name: label, type: String}
entities:
  - name: demo.tickets.Ticket
    identity: {name: ticket_id, type: demo.tickets.TicketId}
    fields:
      - {name: note, type: String}
      - {name: desk, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.tickets.TicketSubmitted
    fields:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: data, type: demo.tickets.Receipt}
actors:
  - name: demo.tickets.Clerk
    attributes: [{name: desk, type: String}]
    may: [demo.tickets.SubmitTicket]
  - name: demo.tickets.Agent
    attributes: [{name: desk, type: String}]
    may: [demo.tickets.SubmitTicket]
commands:
  - name: demo.tickets.SubmitTicket
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: note, type: String}
    outcomes:
      - name: submitted
        creates: demo.tickets.Ticket
        instance: ticket_id
        sets: {note: input.note, desk: {caller: desk}}
        emits: [demo.tickets.TicketSubmitted]
        payload:
          demo.tickets.TicketSubmitted:
            ticket_id: input.ticket_id
            data: {id: input.ticket_id, label: {generated: true}}
";

const SUBMITTED: &str = "demo.tickets.SubmitTicket/outcome/submitted";
const NESTED: &str = "data: {id: input.ticket_id, label: {generated: true}}";

/// The identity each send of `SubmitTicket` carries, and what each event or error expectation of
/// the scenario `id` holds under `key` (the member `member` of it, where it holds a struct).
fn sent_and_expected(
    sender: &str,
    text: &str,
    id: &str,
    key: &str,
    member: Option<&str>,
) -> (Vec<Node>, Vec<Node>) {
    let ir = ir(text);
    let synthesis = synthesize(&ir);
    let scenario = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(name, _)| name.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    let (mut sent, mut expected) = (Vec::new(), Vec::new());
    let mut held = |value: Option<&Node>| {
        let Some(value) = value else { return };
        expected.push(match (member, value) {
            (Some(name), Node::Map(members)) => members[name].clone(),
            _ => value.clone(),
        });
    };
    for step in &scenario.steps {
        match step {
            // A run creates its ticket by sending a literal identity and addresses it afterwards
            // by the instance it created; a refusal for a ticket that does not exist sends one.
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string().ends_with(sender) =>
            {
                if let Some(ScenarioValue::Literal { value }) = input.get("ticket_id") {
                    sent.push(value.clone());
                }
            }
            ScenarioStep::ExpectEvent { payload, .. } => held(payload.get(key)),
            ScenarioStep::ExpectEventValues { payload, .. } => {
                if let Some(ScenarioValue::Literal { value }) = payload.get(key) {
                    held(Some(value));
                }
            }
            ScenarioStep::ExpectError { fields, .. } => held(fields.get(key)),
            _ => {}
        }
    }
    (sent, expected)
}

fn assert_each_run_expects_what_it_sends(
    sender: &str,
    text: &str,
    id: &str,
    key: &str,
    member: Option<&str>,
) {
    let (sent, expected) = sent_and_expected(sender, text, id, key, member);
    assert_eq!(
        sent.len(),
        2,
        "the first run and the reversed one: {sent:?}"
    );
    assert_ne!(sent[0], sent[1], "the reversed run sends its own identity");
    assert_eq!(
        sent, expected,
        "each run expects under `{key}` the identity it sent"
    );
    let ir = ir(text);
    let results = statuses(&synthesize(&ir).suite, &ir);
    let ran: Vec<_> = results.iter().filter(|(name, _)| name == id).collect();
    assert_eq!(ran.len(), 1, "{id} ran once: {results:#?}");
    assert_eq!(ran[0].1, Status::Passed, "the interpreter passes {id}");
}

#[test]
fn a_reversed_run_expects_its_own_identity_in_a_nested_event_member() {
    assert_each_run_expects_what_it_sends("SubmitTicket", TICKETS, SUBMITTED, "data.id", None);
}

#[test]
fn a_reversed_run_expects_its_own_identity_in_a_nested_error_member() {
    let text = TICKETS
        .replace(
            "commands:\n",
            "errors:\n  - name: demo.tickets.Refused\n    summary: The note is refused.\n    fields:\n      - {name: data, type: demo.tickets.Receipt}\ncommands:\n",
        )
        .replace(
            "    outcomes:\n      - name: submitted\n",
            "    outcomes:\n      - name: refused\n        when: note == \"bad\"\n        error: demo.tickets.Refused\n        payload:\n          demo.tickets.Refused:\n            data: {id: input.ticket_id, label: {generated: true}}\n      - name: submitted\n",
        );
    assert_each_run_expects_what_it_sends(
        "SubmitTicket",
        &text,
        "demo.tickets.SubmitTicket/outcome/refused",
        "data.id",
        None,
    );
}

#[test]
fn a_reversed_run_expects_its_own_identity_in_a_top_level_event_field() {
    let text = TICKETS
        .replace(
            "- {name: data, type: demo.tickets.Receipt}",
            "- {name: id, type: demo.tickets.TicketId}",
        )
        .replace(NESTED, "id: input.ticket_id");
    assert_each_run_expects_what_it_sends("SubmitTicket", &text, SUBMITTED, "id", None);
}

#[test]
fn a_reversed_run_expects_its_own_identity_in_a_struct_set_whole() {
    let text = TICKETS.replace("label: {generated: true}", "label: submitted");
    assert_each_run_expects_what_it_sends("SubmitTicket", &text, SUBMITTED, "data", Some("id"));
}
