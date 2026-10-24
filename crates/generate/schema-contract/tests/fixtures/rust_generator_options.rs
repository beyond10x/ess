//! The models, schemas and readers the Rust generator-option tests share: the option scenarios
//! in `rust_generator_options.rs`, and the ones that build the generated crate in
//! `model_ergonomics.rs`, the test file allowed to run cargo.

use std::collections::{BTreeMap, BTreeSet};

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::{Plan, Realization};
use serde_json::{json, Value};

/// A model with a struct, an enum, a newtype, a tagged union, a list and optional fields.
pub const SHOP: &str = r"format: ess/22
system: probe
version: v1
domain: probe.shop
types:
  - name: probe.shop.Colour
    kind: enum
    variants:
      - {name: Red, wire: red}
      - {name: Blue, wire: blue}
  - name: probe.shop.Sku
    kind: newtype
    of: String
  - name: probe.shop.Amount
    kind: union
    tag: kind
    variants:
      count: probe.shop.Sku
      text: String
  - name: probe.shop.Line
    kind: struct
    fields:
      - {name: sku, type: probe.shop.Sku}
      - {name: count, type: Integer}
      - {name: colour, type: 'Optional<probe.shop.Colour>'}
      - {name: note, type: 'Optional<String>'}
      - {name: amount, type: probe.shop.Amount}
  - name: probe.shop.Order
    kind: struct
    fields:
      - {name: lines, type: 'List<probe.shop.Line>'}
      - {name: payload, type: Json}
";

/// A struct holding a binary64 float, which implements neither `Eq` nor `Hash`.
pub const LAB: &str = r"format: ess/22
system: probe
version: v1
domain: probe.lab
types:
  - name: probe.lab.Reading
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: ratio, type: Binary64}
";

/// Open records with declared fields, a field-less map and a closed record, as a bundle.
pub fn open_schemas() -> Value {
    json!({
        "Open": {"type": "object", "properties": {"x": {"type": "string"}}},
        "Counted": {"type": "object", "required": ["n"],
            "properties": {"n": {"type": "integer"}}, "additionalProperties": {"type": "integer"}},
        "Map": {"type": "object", "additionalProperties": {"type": "string"}},
        "Closed": {"type": "object", "additionalProperties": false,
            "properties": {"y": {"type": ["string", "null"]}}}
    })
}

pub fn shop_plan() -> Plan {
    Plan::from_model(&crate::model::selection(SHOP, &["probe.shop.Order"])).expect("plan")
}

pub fn bundle_plan(schemas: &Value) -> Plan {
    let roots = schemas
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let bundle = import(
        &json!({"components": {"schemas": schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    Plan::from_bundle(&bundle, &roots).unwrap()
}

/// Each generated declaration's text, by the name it declares, helpers left out.
pub fn declarations(realization: &Realization) -> BTreeMap<String, String> {
    realization
        .declarations
        .split("// Source: ")
        .skip(1)
        .map(|block| {
            let name = block
                .lines()
                .find_map(|line| {
                    line.strip_prefix("pub struct ")
                        .or_else(|| line.strip_prefix("pub enum "))
                })
                .unwrap_or_else(|| panic!("no declaration in {block}"))
                .split(|c: char| !c.is_ascii_alphanumeric())
                .next()
                .unwrap()
                .to_owned();
            (name, block.to_owned())
        })
        .collect()
}

/// The `#[derive(...)]` lines of one declaration.
pub fn derive_lines(block: &str) -> Vec<&str> {
    block
        .lines()
        .filter(|line| line.starts_with("#[derive("))
        .collect()
}
