//! An entity invariant may not read a required field that a creating outcome leaves unset (ess#112).
//!
//! `items >= 0` over an `Integer` field that `PlaceOrder`'s `creates:` branch does not set used to
//! validate. After `PlaceOrder`, `items` has no specified value, so the invariant held only if the
//! implementation happened to pick one that satisfies it, and the generated "still satisfies what
//! it declares" scenario passed or failed on that undeclared choice. The refusal names both repairs:
//! set the field on the outcome, or declare it `Optional`.

use ess_primitives::error::{ValidationCode, ValidationErrors};

fn spec(body: &str) -> Result<ess_domain::Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).expect("the fixture is well formed");
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("shop.yaml"), raw)])
}

/// One entity with a required `items`, an optional `note`, and the invariant `{invariant}`; one
/// creating branch whose extra keys are `{placed}`, and a second creating branch whose extra keys
/// are `{imported}`.
fn model(invariant: &str, placed: &str, imported: &str) -> String {
    format!(
        "format: ess/4
system: shop
version: v1
domain: shop.orders
entities:
  - name: shop.orders.Order
    identity:
      name: order_id
      type: Uuid
    fields:
      - name: items
        type: Integer
      - name: note
        type: Optional<String>
    invariants:
      - {invariant}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed
events:
  - name: shop.orders.OrderPlaced
    fields:
      - name: order_id
        type: Uuid
  - name: shop.orders.OrderClosed
    fields: []
commands:
  - name: shop.orders.PlaceOrder
    input:
      - name: count
        type: Integer
    outcomes:
      - name: placed
        creates: shop.orders.Order
        instance: order_id
        emits: [shop.orders.OrderPlaced]
        payload:
          shop.orders.OrderPlaced:
            order_id: {{generated: true}}
{placed}
  - name: shop.orders.ImportOrder
    input:
      - name: count
        type: Integer
    outcomes:
      - name: imported
        creates: shop.orders.Order
        instance: order_id
        emits: [shop.orders.OrderPlaced]
        payload:
          shop.orders.OrderPlaced:
            order_id: {{generated: true}}
{imported}
  - name: shop.orders.CloseOrder
    input:
      - name: order_id
        type: Uuid
    outcomes:
      - name: closed
        moves: shop.orders.Order.close
        instance: order_id
        emits: [shop.orders.OrderClosed]
"
    )
}

const SETS_ZERO: &str = "        sets:\n          items: \"0\"";
const SETS_INPUT: &str = "        sets:\n          items: input.count";
const NOTHING: &str = "";

fn refusals(errors: &ValidationErrors) -> Vec<&ess_primitives::error::ValidationError> {
    errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::InvariantReadsUnsetField)
        .collect()
}

#[test]
fn an_invariant_over_a_required_field_no_create_sets_is_refused_with_its_own_code() {
    let errors = spec(&model("items >= 0", NOTHING, SETS_ZERO))
        .expect_err("`placed` leaves `items` unset and the invariant reads it");
    let refused = refusals(&errors);
    assert_eq!(refused.len(), 1, "exactly the one branch:\n{errors}");
    let refusal = refused[0];
    assert_eq!(
        refusal.code.as_str(),
        "invariant_reads_unset_field",
        "a stable code of its own"
    );
    for required in ["shop.orders.PlaceOrder", "placed"] {
        assert!(
            refusal.location.contains(required),
            "{required:?} missing from the site `{}`",
            refusal.location
        );
    }
    for required in ["`items`", "shop.orders.Order", "items >= 0"] {
        assert!(
            refusal.message.contains(required),
            "{required:?} missing from:\n{}",
            refusal.message
        );
    }
    let hint = refusal
        .hint
        .as_deref()
        .expect("the refusal names its repairs");
    assert!(
        hint.contains("sets: {items:"),
        "names the `sets:` repair: {hint}"
    );
    assert!(
        hint.contains("Optional<Integer>"),
        "names the `Optional` repair: {hint}"
    );
}

#[test]
fn every_creating_branch_that_leaves_the_field_unset_is_named() {
    let errors = spec(&model("items >= 0", NOTHING, NOTHING)).expect_err("both leave it unset");
    let refused = refusals(&errors);
    assert_eq!(refused.len(), 2, "{errors}");
    assert!(refused
        .iter()
        .any(|error| error.location.contains("PlaceOrder")));
    assert!(refused
        .iter()
        .any(|error| error.location.contains("ImportOrder")));
}

#[test]
fn a_field_every_create_sets_compiles() {
    spec(&model("items >= 0", SETS_ZERO, SETS_INPUT))
        .expect("a literal and an input read each set `items`");
}

#[test]
fn an_optional_field_the_state_and_the_identity_need_no_create_to_set_them() {
    for invariant in [
        "'note != \"\"'",
        "{state: [Open, Closed]}",
        "{order_id: {exists: true}}",
    ] {
        spec(&model(invariant, NOTHING, NOTHING))
            .unwrap_or_else(|errors| panic!("`{invariant}` reads nothing unset:\n{errors}"));
    }
}

#[test]
fn a_nested_read_of_an_unset_field_is_still_a_read_of_it() {
    // `items` read under a disjunction is still read; the rule does not try to decide which branch
    // of the invariant an unset value would take, because that is the implementation's choice.
    let errors = spec(&model(
        "{any: [{note: {exists: true}}, 'items >= 0']}",
        NOTHING,
        SETS_ZERO,
    ))
    .expect_err("`items` is read");
    assert_eq!(refusals(&errors).len(), 1, "{errors}");
}
