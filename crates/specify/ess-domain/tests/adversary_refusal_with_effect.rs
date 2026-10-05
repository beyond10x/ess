//! Adversary cases for the admission rules of `compensates: true` (ess/22, beyond10x/ess#197,
//! `docs/design/refusal-with-effect.md`): one addressed instance, no fan-out.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/refusal-with-effect.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("order.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The design: "`instances:` beside it is refused by the existing set-effects rule (a refusal
/// changes no row `instances:` selects)". A compensating refusal that moves every order whose
/// `failure` equals the input, rather than the one `instance:` names.
#[test]
fn adversary_a_marked_refusal_moving_an_instances_set_is_refused() {
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "        moves: shop.order.Order.reset\n        instances: {where: failure == input.reason}\n        sets: {failure: input.reason}\n",
    );
    let outcome = assemble(&text);
    let errors = outcome.as_ref().err().unwrap_or_else(|| {
        panic!("a compensating refusal over an `instances:` set is admitted:\n{text}")
    });
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.location.contains("JoinOrder.outcomes.failed")),
        "{errors:#?}"
    );
}

/// The same with `updates:` over a set: field writes on every selected row.
#[test]
fn adversary_a_marked_refusal_updating_an_instances_set_is_refused() {
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "        updates: shop.order.Order\n        instances: {where: failure == input.reason}\n        sets: {failure: input.reason}\n",
    );
    let outcome = assemble(&text);
    let errors = outcome.as_ref().err().unwrap_or_else(|| {
        panic!("a compensating refusal over an `instances:` set is admitted:\n{text}")
    });
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.location.contains("JoinOrder.outcomes.failed")),
        "{errors:#?}"
    );
}
