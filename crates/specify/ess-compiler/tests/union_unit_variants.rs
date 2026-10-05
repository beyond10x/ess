//! A unit variant through the compiler (ess/22, beyond10x/ess#418): it resolves to a variant with
//! no payload, written `null` in the IR, and a union without one keeps the bytes it had.

use ess_compiler::ir::{EssIr, ResolvedBody};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/union-unit-variants.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("work.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn status(ir: &EssIr) -> &ResolvedBody {
    &ir.types()[&"demo.work.Status".parse().unwrap()].body
}

#[test]
fn a_unit_variant_resolves_to_a_variant_with_no_payload() {
    let ir = ir(MODEL);
    let ResolvedBody::Union { tag, variants } = status(&ir) else {
        panic!("a union");
    };
    assert_eq!(tag, "kind");
    assert_eq!(variants.len(), 2);
    assert!(variants["Open"].is_none(), "{variants:?}");
    assert!(variants["Complete"].is_some(), "{variants:?}");
}

#[test]
fn the_ir_writes_a_unit_variant_as_null_beside_a_payload_variant() {
    let ir = ir(MODEL);
    let json: serde_json::Value = serde_json::from_str(&ir.to_canonical_json()).unwrap();
    let declared = json["types"]
        .as_object()
        .and_then(|types| types.get("demo.work.Status"))
        .unwrap_or_else(|| panic!("the union is in the IR: {}", json["types"]));
    assert_eq!(
        declared["body"]["variants"]["Open"],
        serde_json::Value::Null
    );
    assert!(
        declared["body"]["variants"]
            .as_object()
            .unwrap()
            .contains_key("Open"),
        "the unit variant is present, not dropped: {declared}"
    );
    assert_eq!(
        declared["body"]["variants"]["Complete"]["kind"], "declared",
        "{declared}"
    );
    assert_eq!(ir.to_canonical_json(), self::ir(MODEL).to_canonical_json());
}

#[test]
fn a_union_of_payload_variants_keeps_its_ir_shape() {
    let payload = MODEL.replace("      Open:\n", "      Open: String\n");
    let json: serde_json::Value = serde_json::from_str(&ir(&payload).to_canonical_json()).unwrap();
    let variants = &json["types"]["demo.work.Status"]["body"]["variants"];
    assert_eq!(
        variants["Open"],
        serde_json::json!({"kind": "primitive", "name": "string"}),
        "{variants}"
    );
}
