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

/// A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) is the adapter's to spell: the
/// suite addresses the state by its declared name, so it keeps the bytes of the bare lifecycle.
///
/// All but the two digests of its provenance, which name the specification and the contract it
/// was synthesized from: the spelling is part of both, so a suite stays bound to the model that
/// declares it.
#[test]
fn a_state_wire_name_leaves_the_suite_bytes_unchanged() {
    let spelled = include_str!("../../../generate/ess-gen/tests/fixtures/state-wire.yaml");
    let bare = spelled.replace(
        "states: [{name: Indexing, wire: indexing}, Ready]",
        "states: [Indexing, Ready]",
    );
    assert_ne!(bare, spelled);
    let wired = ess_conformance::synthesize::synthesize(&ir(spelled));
    let plain = ess_conformance::synthesize::synthesize(&ir(&bare));
    assert!(!wired.suite.scenarios.is_empty());
    let bytes = wired.suite.to_canonical_json().expect("serialises");
    let unbound = |json: &str| {
        let mut value: serde_json::Value = serde_json::from_str(json).expect("a suite is JSON");
        let provenance = value["provenance"].as_object_mut().expect("a provenance");
        for digest in ["spec_digest", "contract_digest"] {
            assert!(provenance.remove(digest).is_some(), "carries `{digest}`");
        }
        value.to_string()
    };
    assert_eq!(
        unbound(&bytes),
        unbound(&plain.suite.to_canonical_json().expect("serialises")),
        "one suite"
    );
    assert!(
        bytes.contains("\"Indexing\""),
        "the suite carries the declared state"
    );
    assert!(!bytes.contains("\"indexing\""), "and not its wire name");
}
