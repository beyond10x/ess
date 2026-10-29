//! An entity's invariants reach its JSON Schema document (beyond10x/ess#688).
//!
//! The entity document publishes every invariant verbatim under `x-ess-invariants`, and an
//! `Integer` comparison becomes `minimum`, `maximum` or `const` on its property, exactly as a
//! struct's do (beyond10x/ess#394). A command input, an event or a view row that carries a value
//! the entity also holds is not the entity, and gains nothing.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::{JsonSchema, ModelTypes};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const MODEL: &str = "format: ess/20
system: demo
version: v1
domain: demo.stock
types:
  - {name: demo.stock.ItemId, kind: newtype, of: Uuid}
entities:
  - name: demo.stock.Item
    identity: {name: item_id, type: demo.stock.ItemId}
    fields:
      - {name: format_version, type: Integer}
      - {name: quantity, type: Integer}
      - {name: revision, type: Optional<Integer>}
      - {name: label, type: String}
    invariants:
      - format_version > 0
      - quantity >= 0
      - quantity <= 1000
      - revision == 3
      - label != ''
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
commands:
  - name: demo.stock.AddItem
    input:
      - {name: format_version, type: Integer}
      - {name: quantity, type: Integer}
      - {name: label, type: String}
    outcomes:
      - name: added
        creates: demo.stock.Item
        instance: item_id
        sets: {format_version: input.format_version, quantity: input.quantity, label: input.label}
        emits: [demo.stock.ItemAdded]
        payload:
          demo.stock.ItemAdded: {item_id: {generated: true}, format_version: input.format_version, quantity: input.quantity}
events:
  - name: demo.stock.ItemAdded
    fields:
      - {name: item_id, type: demo.stock.ItemId}
      - {name: format_version, type: Integer}
      - {name: quantity, type: Integer}
views:
  - name: demo.stock.ItemStock
    source: demo.stock.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.stock.ItemId}
      - {name: format_version, type: Integer}
      - {name: quantity, type: Integer}
components:
  - component: stock-service
    owns:
      domains: [demo.stock]
    accepts:
      commands: [demo.stock.AddItem]
    publishes:
      events: [demo.stock.ItemAdded]
";

const ENTITY: &str = "schema/entities/demo.stock.Item.schema.json";

const VIEW_ROW: &str = "demo.stock.ItemStock.row";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("stock.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("stock.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn documents() -> BTreeMap<String, Value> {
    let ir = compiled(MODEL);
    let mut documents: BTreeMap<String, Value> = run(&JsonSchema, &ir)
        .expect("generates")
        .into_iter()
        .map(|(path, artifact)| {
            let value = serde_json::from_str(&artifact.contents).expect("JSON");
            (path, value)
        })
        .collect();
    // A view row has no schema document of its own; `ess generate types --root <view>.row`
    // publishes it from the same `types::message`.
    let selection = ModelTypes::select(&ir, &BTreeSet::from([VIEW_ROW.to_owned()]))
        .unwrap_or_else(|errors| panic!("selects: {errors:?}"));
    let types: Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    documents.insert(
        format!("schema/views/{VIEW_ROW}"),
        types["$defs"][VIEW_ROW].clone(),
    );
    documents
}

fn entity() -> Value {
    let mut documents = documents();
    documents.remove(ENTITY).unwrap_or_else(|| {
        panic!(
            "no `{ENTITY}` among {:?}",
            documents.keys().collect::<Vec<_>>()
        )
    })
}

#[test]
fn an_entity_schema_publishes_its_invariants_verbatim() {
    let schema = entity();
    assert_eq!(
        schema["x-ess-invariants"],
        json!([
            "format_version > 0",
            "quantity >= 0",
            "quantity <= 1000",
            "revision == 3",
            "label != ''"
        ]),
        "{schema:#}"
    );
}

#[test]
fn an_entity_integer_bound_becomes_a_keyword_on_its_property() {
    let schema = entity();
    let property = |name: &str| schema["properties"][name].clone();
    assert_eq!(
        property("format_version")["minimum"],
        json!(1),
        "{schema:#}"
    );
    assert_eq!(property("quantity")["minimum"], json!(0), "{schema:#}");
    assert_eq!(property("quantity")["maximum"], json!(1000), "{schema:#}");
    let revision = property("revision");
    assert!(revision.get("const").is_none(), "{revision:#}");
    assert_eq!(revision["minimum"], json!(3), "{revision:#}");
    assert_eq!(revision["maximum"], json!(3), "{revision:#}");
    let label = property("label");
    for keyword in ["minimum", "maximum", "const", "minLength", "maxLength"] {
        assert!(label.get(keyword).is_none(), "`label` gained `{keyword}`");
    }
}

#[test]
fn a_validator_refuses_an_entity_record_its_invariants_refuse() {
    let schema = entity();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let valid = json!({
        "item_id": "6f1c2d3e-4a5b-4c6d-8e7f-0a1b2c3d4e5f",
        "format_version": 1,
        "quantity": 0,
        "label": "bolt",
        "state": "Open"
    });
    assert!(validator.is_valid(&valid), "{valid} against {schema:#}");
    for (field, value) in [
        ("format_version", json!(0)),
        ("quantity", json!(-1)),
        ("quantity", json!(1001)),
        ("revision", json!(4)),
    ] {
        let mut instance = valid.clone();
        instance[field] = value.clone();
        assert!(
            !validator.is_valid(&instance),
            "{field} = {value} was accepted"
        );
    }
}

#[test]
fn a_message_carrying_entity_fields_gains_no_entity_invariant() {
    let documents = documents();
    let messages: Vec<(&String, &Value)> = documents
        .iter()
        .filter(|(path, _)| {
            ["schema/commands/", "schema/events/", "schema/views/"]
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .collect();
    let kinds: Vec<&str> = messages
        .iter()
        .filter_map(|(_, document)| document["x-ess-kind"].as_str())
        .collect();
    for kind in ["command-input", "event-payload", "view-row"] {
        assert!(
            kinds.contains(&kind),
            "no `{kind}` document among {:?}",
            documents.keys().collect::<Vec<_>>()
        );
    }
    for (path, document) in messages {
        assert!(
            document.get("x-ess-invariants").is_none(),
            "`{path}` gained invariants: {document:#}"
        );
        for name in ["format_version", "quantity"] {
            let Some(property) = document["properties"].get(name) else {
                continue;
            };
            for keyword in ["minimum", "maximum", "const"] {
                assert!(
                    property.get(keyword).is_none(),
                    "`{path}` `{name}` gained `{keyword}`: {document:#}"
                );
            }
        }
    }
}
