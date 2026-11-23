//! The format hints of type, view and admission refusals, as emitted, name the source that
//! declares the system (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError};

/// An order shop whose `CancelOrder` guards on held states, one a list and one on a refusal that
/// names no subject: an `ess/3` construct holding two `ess/18` shapes. Valid at `ess/18`.
const SHOP: &str = "format: ess/18
system: demo
version: v1
domain: demo.ship
types:
  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}
entities:
  - name: demo.ship.Order
    identity: {name: order_id, type: demo.ship.OrderId}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed, Shipped, Cancelled]
      terminal: [Cancelled]
      transitions:
        - {name: ship, from: [Placed], to: Shipped}
        - {name: cancel, from: [Placed, Shipped], to: Cancelled}
errors:
  - {name: demo.ship.Gone, fields: []}
events:
  - name: demo.ship.OrderPlaced
    fields: [{name: order_id, type: demo.ship.OrderId}]
  - name: demo.ship.OrderShipped
    fields: [{name: order_id, type: demo.ship.OrderId}]
  - name: demo.ship.OrderCancelled
    fields: [{name: order_id, type: demo.ship.OrderId}]
commands:
  - name: demo.ship.PlaceOrder
    input: []
    outcomes:
      - name: placed
        creates: demo.ship.Order
        instance: order_id
        emits: [demo.ship.OrderPlaced]
        payload: {demo.ship.OrderPlaced: {order_id: {generated: true}}}
  - name: demo.ship.ShipOrder
    input: [{name: order_id, type: demo.ship.OrderId}]
    outcomes:
      - name: shipped
        moves: demo.ship.Order.ship
        instance: order_id
        emits: [demo.ship.OrderShipped]
        payload: {demo.ship.OrderShipped: {order_id: input.order_id}}
  - name: demo.ship.CancelOrder
    input: [{name: order_id, type: demo.ship.OrderId}]
    outcomes:
      - name: cancelled
        when_subject_state: [Placed, Shipped]
        moves: demo.ship.Order.cancel
        instance: order_id
        emits: [demo.ship.OrderCancelled]
        payload: {demo.ship.OrderCancelled: {order_id: input.order_id}}
      - name: gone
        when_subject_state: Cancelled
        error: demo.ship.Gone
";

/// A struct that ignores undeclared fields, an `ess/24` key. Valid at `ess/24`.
const NOTE: &str = "format: ess/1
system: demo
version: v1
domain: demo.shop
types:
  - name: demo.shop.Note
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: text, type: String}
";

fn refusals(text: &str) -> Vec<ValidationError> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("parses: {error}"));
    Specification::assemble([(Source::new("model.yaml"), raw)])
        .expect_err("refused")
        .as_slice()
        .to_vec()
}

fn refusal_at<'a>(errors: &'a [ValidationError], location: &str) -> &'a ValidationError {
    errors
        .iter()
        .find(|error| error.location == location)
        .unwrap_or_else(|| panic!("no refusal at {location}: {errors:#?}"))
}

#[test]
fn held_state_guards_below_their_formats_name_the_source() {
    let errors = refusals(&SHOP.replacen("format: ess/18", "format: ess/2", 1));
    let guard = refusal_at(&errors, "command.demo.ship.CancelOrder.outcomes");
    assert_eq!(guard.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        guard.message,
        "subject-state outcome guards require specification format ess/3"
    );
    assert_eq!(
        guard.hint.as_deref(),
        Some("write `format: ess/3` on the source that declares the system")
    );
    for outcome in ["cancelled", "gone"] {
        let location =
            format!("command.demo.ship.CancelOrder.outcomes.{outcome}.when_subject_state");
        assert_eq!(
            refusal_at(&errors, &location).hint.as_deref(),
            Some("write `format: ess/18` on the source that declares the system"),
            "{location}"
        );
    }
}

#[test]
fn undeclared_fields_below_ess_24_name_the_source() {
    let errors = refusals(NOTE);
    let refusal = refusal_at(&errors, "type.demo.shop.Note.undeclared_fields");
    assert_eq!(refusal.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        refusal.hint.as_deref(),
        Some(
            "write `format: ess/24` on the source that declares the system, or drop \
             `undeclared_fields:`"
        )
    );
}

#[test]
fn a_binary64_field_at_ess_1_names_the_source() {
    let text = NOTE
        .replace("    undeclared_fields: ignored\n", "")
        .replace("type: String", "type: Binary64");
    let errors = refusals(&text);
    let found: Vec<_> = errors
        .iter()
        .filter(|error| error.code == ValidationCode::UnsupportedFormatVersion)
        .collect();
    assert_eq!(found.len(), 1, "{errors:#?}");
    assert_eq!(
        found[0].message,
        "Binary64 requires specification format ess/2"
    );
    assert_eq!(
        found[0].hint.as_deref(),
        Some("write `format: ess/2` on the source that declares the system")
    );
}
