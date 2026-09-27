//! A suite reads a field written `naming: {wire: …}` exactly as one written `wire: …`
//! (beyond10x/ess#142).
//!
//! A suite addresses fields by their declared names and carries each field's naming wherever it
//! carries a field, so the target adapter is what spells the wire key. What the synonym must not do
//! is change the suite: both spellings are one model, so the suites are byte-identical, and the
//! naming a suite carries holds the wire name.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const NESTED: &str = "format: ess/13
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
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

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn issue_142_the_nested_spelling_synthesizes_the_same_suite_as_the_flat_one() {
    let nested = ess_conformance::synthesize::synthesize(&ir(NESTED));
    let flat = ess_conformance::synthesize::synthesize(&ir(
        &NESTED.replace("naming: {wire: orderId}", "wire: orderId")
    ));
    assert!(nested.refusals.is_empty(), "{:#?}", nested.refusals);
    assert!(!nested.suite.scenarios.is_empty());
    assert_eq!(
        nested.suite.to_canonical_json().expect("serialises"),
        flat.suite.to_canonical_json().expect("serialises"),
        "one model, one suite"
    );
}

#[test]
fn the_compiled_field_carries_the_wire_name_a_target_spells() {
    let model = ir(NESTED);
    let event = model
        .events()
        .get(&"demo.orders.Placed".parse().unwrap())
        .expect("declared");
    assert_eq!(event.fields[0].name, "order_id");
    assert_eq!(event.fields[0].naming.wire.as_deref(), Some("orderId"));
}
