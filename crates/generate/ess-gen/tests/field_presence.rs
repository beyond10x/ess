//! A field's `presence:` projects into JSON Schema (beyond10x/ess#139).
//!
//! `null_when_absent` makes the property required and nullable; `omitted_when_absent` keeps it
//! optional and not nullable; an `Optional` that declares neither keeps exactly the projection it
//! had.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use serde_json::json;

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderReceipt
    kind: struct
    fields:
      - name: partner_ref
        type: Optional<String>
        presence: null_when_absent
      - name: discount_code
        type: Optional<String>
        presence: omitted_when_absent
      - name: note
        type: Optional<String>
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn receipt(text: &str) -> serde_json::Value {
    let artifacts = run(&JsonSchema, &compiled(text)).expect("generates");
    let path = "schema/types/demo.orders.OrderReceipt.schema.json";
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

#[test]
fn issue_139_both_policies_project_into_the_schema() {
    let schema = receipt(MODEL);
    let declared = &schema["$defs"]["demo.orders.OrderReceipt"];
    assert_eq!(declared["required"], json!(["partner_ref"]), "{schema:#}");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    for (instance, valid) in [
        (json!({"partner_ref": null}), true),
        (
            json!({"partner_ref": "p", "discount_code": "d", "note": "n"}),
            true,
        ),
        (json!({"partner_ref": null, "note": null}), false),
        (json!({}), false),
        (json!({"partner_ref": null, "discount_code": null}), false),
    ] {
        assert_eq!(
            validator.is_valid(&instance),
            valid,
            "{instance} against {schema:#}"
        );
    }
}

#[test]
fn an_optional_without_a_policy_keeps_its_projection() {
    let without = MODEL
        .replace("        presence: null_when_absent\n", "")
        .replace("        presence: omitted_when_absent\n", "");
    let before = receipt(&without.replace("format: ess/15", "format: ess/14"));
    let declared = &before["$defs"]["demo.orders.OrderReceipt"];
    let with = receipt(MODEL);
    assert_eq!(
        declared["properties"]["note"],
        with["$defs"]["demo.orders.OrderReceipt"]["properties"]["note"]
    );
    assert_eq!(
        declared["properties"]["discount_code"],
        with["$defs"]["demo.orders.OrderReceipt"]["properties"]["discount_code"],
        "omitted_when_absent is the projection an Optional field already had"
    );
}
