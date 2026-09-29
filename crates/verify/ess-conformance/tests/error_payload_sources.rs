//! An error whose fields the specification gives sources (ess/19, `story:error-payload-sources`):
//! the model interpreter carries the declared fields on the error it reports, and the synthesized
//! suite asserts them. An error field with no declared source is still carried as none.
//!
//! The model is `fixtures/error-payload-sources.yaml`: an input-guarded refusal filling its error
//! from the input and two literals, an `unknown_instance:` refusal from the input, and a
//! `wrong_state:` refusal from the input and the row it is answered for.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::scenario::ScenarioStep;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const ORDERS: &str = include_str!("fixtures/error-payload-sources.yaml");

const ORDER: &str = "00000000-0000-4000-8000-000000000001";

fn edit(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn number(value: i64) -> Node {
    Node::Number(value.into())
}

fn one(ir: &EssIr, store: &Store, command: &str, input: &[(&str, Node)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let mut steps = execute(
        ir,
        store,
        &command.parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|why| panic!("{command}: {why}"));
    assert_eq!(steps.len(), 1, "{steps:#?}");
    steps.remove(0)
}

fn carried(step: &Step) -> (String, BTreeMap<String, Node>) {
    let error = step.error.as_ref().expect("the step reports an error");
    (error.error.to_string(), error.fields.clone())
}

fn fields(pairs: &[(&str, Node)]) -> BTreeMap<String, Node> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

/// A store holding one open order of `quantity` items, and its identity.
fn placed(ir: &EssIr, quantity: i64) -> (Store, Node) {
    let step = one(
        ir,
        &Store::default(),
        "demo.order.PlaceOrder",
        &[("quantity", number(quantity))],
    );
    let identity = step.events[0].payload["order_id"].clone();
    (step.next, identity)
}

#[test]
fn an_input_guarded_refusal_carries_its_input_and_literal_fields() {
    let ir = compiled(ORDERS);
    let step = one(
        &ir,
        &Store::default(),
        "demo.order.PlaceOrder",
        &[("quantity", number(11))],
    );
    assert_eq!(
        carried(&step),
        (
            "demo.order.TooMany".to_owned(),
            fields(&[
                ("limit", number(10)),
                ("reason", Node::Text("Quantity".to_owned())),
                ("requested", number(11)),
            ])
        )
    );
}

#[test]
fn an_unknown_instance_refusal_carries_the_identity_it_was_asked_for() {
    let ir = compiled(ORDERS);
    let step = one(
        &ir,
        &Store::default(),
        "demo.order.CloseOrder",
        &[("order_id", Node::Text(ORDER.to_owned()))],
    );
    assert_eq!(
        carried(&step),
        (
            "demo.order.NoSuchOrder".to_owned(),
            fields(&[("order_id", Node::Text(ORDER.to_owned()))])
        )
    );
}

#[test]
fn a_wrong_state_refusal_carries_a_field_of_the_row_it_is_answered_for() {
    let ir = compiled(ORDERS);
    let (store, order) = placed(&ir, 7);
    let closed = one(
        &ir,
        &store,
        "demo.order.CloseOrder",
        &[("order_id", order.clone())],
    );
    assert!(closed.error.is_none());
    let again = one(
        &ir,
        &closed.next,
        "demo.order.CloseOrder",
        &[("order_id", order.clone())],
    );
    assert_eq!(
        carried(&again),
        (
            "demo.order.AlreadyClosed".to_owned(),
            fields(&[("order_id", order), ("quantity", number(7))])
        )
    );
    assert_eq!(again.next, closed.next, "a refusal changes nothing");
}

#[test]
fn an_error_with_no_declared_source_is_still_carried_as_none() {
    let ir = compiled(&edit(
        ORDERS,
        "        payload:\n          demo.order.TooMany: {requested: input.quantity, limit: 10, reason: Quantity}\n",
        "",
    ));
    let step = one(
        &ir,
        &Store::default(),
        "demo.order.PlaceOrder",
        &[("quantity", number(11))],
    );
    assert_eq!(
        carried(&step),
        ("demo.order.TooMany".to_owned(), BTreeMap::new())
    );
}

/// Every `expect_error` step of the synthesized suite, by error, with the fields it compares.
fn expected_errors(ir: &EssIr) -> Vec<(String, BTreeMap<String, Node>)> {
    let synthesis = ess_conformance::synthesize(ir);
    synthesis
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .filter_map(|step| match step {
            ScenarioStep::ExpectError { error, fields } => {
                Some((error.to_string(), fields.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_synthesized_suite_asserts_every_declared_error_field() {
    let ir = compiled(ORDERS);
    let expected = expected_errors(&ir);
    let of = |name: &str| -> Vec<&BTreeMap<String, Node>> {
        expected
            .iter()
            .filter(|(error, _)| error == name)
            .map(|(_, fields)| fields)
            .collect()
    };
    let too_many = of("demo.order.TooMany");
    assert!(!too_many.is_empty(), "{expected:#?}");
    for fields in &too_many {
        assert_eq!(
            fields.keys().map(String::as_str).collect::<Vec<_>>(),
            ["limit", "reason", "requested"],
            "{expected:#?}"
        );
        assert_eq!(fields["limit"], number(10));
        assert_eq!(fields["reason"], Node::Text("Quantity".to_owned()));
    }
    let unknown = of("demo.order.NoSuchOrder");
    assert!(!unknown.is_empty(), "{expected:#?}");
    for fields in &unknown {
        assert!(fields.contains_key("order_id"), "{expected:#?}");
    }
    let closed = of("demo.order.AlreadyClosed");
    assert!(!closed.is_empty(), "{expected:#?}");
    for fields in &closed {
        // The row's value, as the arrangement created it.
        assert!(fields.contains_key("quantity"), "{expected:#?}");
        // The identity the command names is a bound instance the target minted, which the suite
        // knows only as a reference and `expect_error` compares values: covered by name, as an
        // event field read from a bound input is.
        assert!(!fields.contains_key("order_id"), "{expected:#?}");
    }
}

#[test]
fn a_suite_for_errors_without_sources_asserts_no_field() {
    let text = edit(
        ORDERS,
        "        payload:\n          demo.order.TooMany: {requested: input.quantity, limit: 10, reason: Quantity}\n",
        "",
    );
    let text = edit(
        &text,
        "        payload:\n          demo.order.NoSuchOrder: {order_id: input.order_id}\n",
        "",
    );
    let text = edit(
        &text,
        "        payload:\n          demo.order.AlreadyClosed: {order_id: input.order_id, quantity: {subject: quantity}}\n",
        "",
    );
    let expected = expected_errors(&compiled(&text));
    assert!(!expected.is_empty());
    assert!(
        expected.iter().all(|(_, fields)| fields.is_empty()),
        "{expected:#?}"
    );
}
