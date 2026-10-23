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
use ess_domain::types::Presence;
use ess_gen::artifact::run;
use ess_gen::schema::{JsonSchema, ModelTypes};
use serde_json::json;
use std::collections::BTreeSet;

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

/// The view's row schema (`ess generate types --root <view>.row`), made a document whose root is
/// the row, so a validator reads it.
fn view_row(text: &str) -> serde_json::Value {
    let ir = compiled(text);
    let selection = ModelTypes::select(&ir, &BTreeSet::from([VIEW_ROW.to_owned()]))
        .unwrap_or_else(|errors| panic!("selects: {errors:?}"));
    let mut schema: serde_json::Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    schema["$ref"] = json!(format!("#/$defs/{VIEW_ROW}"));
    schema
}

/// `presence: omitted_or_null` (beyond10x/ess#678): the `anyOf [T, null]` of `null_when_absent`,
/// and not required.
#[test]
fn omitted_or_null_projects_not_required_and_nullable() {
    let text = MODEL
        .replace("format: ess/15", "format: ess/31")
        .replace("presence: omitted_when_absent", "presence: omitted_or_null");
    let schema = receipt(&text);
    let declared = &schema["$defs"]["demo.orders.OrderReceipt"];
    assert_eq!(declared["required"], json!(["partner_ref"]), "{schema:#}");
    assert_eq!(
        declared["properties"]["discount_code"], declared["properties"]["partner_ref"],
        "the nullable projection `null_when_absent` already writes: {schema:#}"
    );
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    for (instance, valid) in [
        (json!({"partner_ref": null}), true),
        (json!({"partner_ref": null, "discount_code": null}), true),
        (json!({"partner_ref": null, "discount_code": "d"}), true),
        (json!({"partner_ref": null, "discount_code": 7}), false),
    ] {
        assert_eq!(
            validator.is_valid(&instance),
            valid,
            "{instance} against {schema:#}"
        );
    }
}

/// A view field written field by field carries its source field's presence (beyond10x/ess#677),
/// from `ess/31`.
const VIEW: &str = "format: ess/31
system: demo
version: v1
domain: demo.notes
entities:
  - name: demo.notes.Note
    identity: {name: id, type: Uuid}
    fields:
      - {name: embed_token, type: Optional<String>, presence: null_when_absent}
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
      - {name: hint, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: id, type: Uuid}
      - {name: embed_token, type: Optional<String>}
      - {name: reason, type: Optional<String>}
      - {name: hint, type: Optional<String>}
      - {name: note, type: Optional<String>}
";

const VIEW_ROW: &str = "demo.notes.Notes.row";

#[test]
fn issue_677_a_view_field_carries_its_source_fields_presence() {
    let ir = compiled(VIEW);
    let view = ir
        .views()
        .get(&"demo.notes.Notes".parse().unwrap())
        .expect("the view");
    let carried = |name: &str| view.field(name).expect("a view field").naming.presence;
    assert_eq!(carried("embed_token"), Some(Presence::NullWhenAbsent));
    assert_eq!(carried("reason"), Some(Presence::OmittedOrNull));
    assert_eq!(carried("hint"), Some(Presence::OmittedWhenAbsent));
    assert_eq!(carried("note"), None);
    assert_eq!(carried("id"), None);

    let schema = view_row(VIEW);
    let row = &schema["$defs"][VIEW_ROW];
    assert_eq!(row["required"], json!(["id", "embed_token"]), "{schema:#}");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let id = "00000000-0000-4000-8000-000000000000";
    for (instance, valid) in [
        (json!({"id": id, "embed_token": null}), true),
        (json!({"id": id, "embed_token": null, "reason": null}), true),
        (json!({"id": id}), false),
        (json!({"id": id, "embed_token": null, "hint": null}), false),
    ] {
        assert_eq!(
            validator.is_valid(&instance),
            valid,
            "{instance} against {schema:#}"
        );
    }
}

#[test]
fn below_ess_31_a_view_field_carries_no_presence() {
    let text = VIEW
        .replace("format: ess/31", "format: ess/30")
        .replace(
            "      - {name: reason, type: Optional<String>, presence: omitted_or_null}\n",
            "",
        )
        .replace("      - {name: reason, type: Optional<String>}\n", "");
    let ir = compiled(&text);
    let view = ir
        .views()
        .get(&"demo.notes.Notes".parse().unwrap())
        .expect("the view");
    assert!(
        view.fields
            .iter()
            .all(|field| field.naming.presence.is_none()),
        "{:?}",
        view.fields
    );
    let schema = view_row(&text);
    let row = &schema["$defs"][VIEW_ROW];
    assert_eq!(row["required"], json!(["id"]), "{schema:#}");
}
