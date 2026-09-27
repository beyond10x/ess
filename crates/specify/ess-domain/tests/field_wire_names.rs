//! A field keeps its wire name (beyond10x/ess#142).
//!
//! * beyond10x/ess#142 — a field's wire name on command inputs, event fields and struct fields.
//!   `wire:` written on the field itself was already read; the nested `naming: {wire: …}` spelling
//!   commands and events use for their own names is now read as a synonym of it, and writing both
//!   is refused rather than one silently winning.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::Field;

fn single(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("orders.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

// ---- #142 -------------------------------------------------------------------------------------

/// The issue's repro, with the header a document needs.
const ISSUE_142: &str = "format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - name: order_id
        type: demo.orders.OrderId
        naming: {wire: orderId}
events:
  - name: demo.orders.Placed
    fields:
      - name: order_id
        type: demo.orders.OrderId
        naming: {wire: orderId}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - name: order_id
        type: demo.orders.OrderId
        naming: {wire: orderId}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {order_id: input.order_id}
";

fn wire_names(spec: &Specification) -> Vec<Option<String>> {
    let event = spec
        .events()
        .get(&"demo.orders.Placed".parse().unwrap())
        .expect("declared");
    let command = spec
        .commands()
        .get(&"demo.orders.Place".parse().unwrap())
        .expect("declared");
    let receipt = spec
        .system()
        .types
        .get(&"demo.orders.Receipt".parse().unwrap())
        .expect("declared");
    vec![
        event.fields[0].naming.wire.clone(),
        command.input[0].naming.wire.clone(),
        receipt
            .field("order_id")
            .expect("a field")
            .naming
            .wire
            .clone(),
    ]
}

#[test]
fn issue_142_the_nested_naming_spelling_validates_on_every_field_position() {
    let spec = single(ISSUE_142).expect("#142's repro validates");
    assert_eq!(wire_names(&spec), vec![Some("orderId".to_owned()); 3]);
    assert_eq!(
        spec.events()
            .get(&"demo.orders.Placed".parse().unwrap())
            .unwrap()
            .fields[0]
            .name,
        "order_id",
        "the declared name is kept; only the wire spelling moves"
    );
}

#[test]
fn the_nested_spelling_reads_as_exactly_the_flat_one() {
    let flat = ISSUE_142.replace("naming: {wire: orderId}", "wire: orderId");
    let nested = single(ISSUE_142).expect("nested validates");
    let flat = single(&flat).expect("flat validates");
    assert_eq!(wire_names(&nested), wire_names(&flat));
    assert_eq!(
        format!("{:?}", nested.events()),
        format!("{:?}", flat.events()),
        "one model, two spellings"
    );
    assert_eq!(
        format!("{:?}", nested.commands()),
        format!("{:?}", flat.commands())
    );
}

#[test]
fn every_naming_key_is_read_from_the_nested_form() {
    let field: Field = serde_yaml::from_str(
        "name: order_id\ntype: String\nnaming: {wire: orderId, display: Order, summary: The order.}\n",
    )
    .expect("reads");
    assert_eq!(field.naming.wire.as_deref(), Some("orderId"));
    assert_eq!(field.naming.display.as_deref(), Some("Order"));
    assert_eq!(field.naming.summary.as_deref(), Some("The order."));
    let written = serde_yaml::to_string(&field).expect("writes");
    assert!(
        !written.contains("naming"),
        "a field is written back in the flat form, so its bytes do not depend on the spelling: \
         {written}"
    );
}

#[test]
fn writing_both_spellings_is_refused() {
    for text in [
        "name: order_id\ntype: String\nwire: orderId\nnaming: {wire: orderId}\n",
        "name: order_id\ntype: String\ndisplay: Order\nnaming: {wire: orderId}\n",
    ] {
        let error = serde_yaml::from_str::<Field>(text)
            .expect_err("two places to say one thing is refused")
            .to_string();
        assert!(error.contains("naming"), "{error}");
    }
    let both = ISSUE_142.replacen(
        "naming: {wire: orderId}",
        "wire: orderId\n        naming: {wire: orderId}",
        1,
    );
    let error = single(&both).expect_err("refused in a document too");
    assert!(error.contains("naming"), "{error}");
}

#[test]
fn a_misspelt_key_inside_the_nested_form_is_still_refused() {
    let error = serde_yaml::from_str::<Field>("name: a\ntype: String\nnaming: {wrie: b}\n")
        .expect_err("refused");
    assert!(error.to_string().contains("wrie"), "{error}");
}
