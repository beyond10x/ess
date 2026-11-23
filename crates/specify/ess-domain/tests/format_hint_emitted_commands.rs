//! The format hints of command refusals, as emitted, name the source that declares the system
//! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError};

const CLAUSE: &str = "on the source that declares the system";

const EACH_MOVES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/each-moves.yaml");

/// An order shop whose `PlaceOrder` fills a payload field with `{generated: true}`, an `ess/4`
/// source, and whose `CancelOrder` guards on a list of held states, an `ess/18` shape.
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

fn refusals(text: &str) -> Vec<ValidationError> {
    let raw = RawSpecFile::parse(text).expect("parses");
    let errors = Specification::assemble([(Source::new("model.yaml"), raw)]).expect_err("refused");
    errors.as_slice().to_vec()
}

fn names_a_format(hint: &str) -> bool {
    hint.match_indices("ess/")
        .any(|(at, _)| hint[at + 4..].starts_with(|c: char| c.is_ascii_digit()))
}

#[test]
fn each_moves_below_ess_27_names_the_source_in_every_hint() {
    let text = EACH_MOVES.replacen("format: ess/27", "format: ess/26", 1);
    let errors = refusals(&text);
    let naming: Vec<&str> = errors
        .iter()
        .filter_map(|error| error.hint.as_deref())
        .filter(|hint| names_a_format(hint))
        .collect();
    assert!(!naming.is_empty(), "{errors:#?}");
    for hint in &naming {
        assert!(hint.contains(CLAUSE), "{hint}");
    }
    let at = |location: &str| {
        errors
            .iter()
            .find(|error| error.location == location)
            .unwrap_or_else(|| panic!("no error at {location}: {errors:#?}"))
    };
    let where_ = at("command.perrow.batch.Apply.outcomes.applied.affects[0].each.where");
    assert_eq!(where_.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        where_.hint.as_deref(),
        Some("write `format: ess/27` on the source that declares the system")
    );
    let each = at("command.perrow.batch.Apply.outcomes.applied.affects[0].each");
    assert_eq!(each.code, ValidationCode::ConflictingDeclaration);
    assert_eq!(
        each.hint.as_deref(),
        Some(
            "keep `each:` with `instance:` and `sets:`, or write the other effect as an entry of its own; write `format: ess/27` on the source that declares the system, from which the entry may move or delete the row each element names"
        )
    );
}

#[test]
fn a_generated_payload_source_below_ess_4_names_the_source() {
    let text = SHOP.replacen("format: ess/18", "format: ess/3", 1);
    let errors = refusals(&text);
    let error = errors
        .iter()
        .find(|error| {
            error.location
                == "command.demo.ship.PlaceOrder.outcomes.placed.payload.demo.ship.OrderPlaced.order_id"
        })
        .unwrap_or_else(|| panic!("{errors:#?}"));
    assert_eq!(error.code, ValidationCode::UnsupportedFormatVersion);
    assert_eq!(
        error.message,
        "explicit response/generated payload sources require specification format ess/4"
    );
    assert_eq!(
        error.hint.as_deref(),
        Some("write `format: ess/4` on the source that declares the system")
    );
}
