//! Typed enum attributes (`ess/23`, beyond10x/ess#450) in the projections: the JSON Schema and
//! `OpenAPI` enum carries `x-ess-attributes`, the documentation lists each variant's values, and an
//! enum without attributes keeps its bytes.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = "format: ess/23
system: demo
version: v1
domain: demo.rules
types:
  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
      - {name: arity, type: Optional<Integer>}
      - {name: weight, type: Decimal}
      - {name: label, type: String}
    variants:
      - {name: GreaterThan, wire: gt, attributes: {takes_number: true, arity: 2, weight: 0.5, label: '>'}}
      - {name: Contains, attributes: {takes_number: false, weight: 1.0, label: contains}}
  - name: demo.rules.Plain
    kind: enum
    variants: [One, Two]
events:
  - name: demo.rules.RuleAdded
    fields:
      - {name: operator, type: demo.rules.Operator}
      - {name: plain, type: demo.rules.Plain}
commands:
  - name: demo.rules.AddRule
    input:
      - {name: operator, type: demo.rules.Operator}
      - {name: plain, type: demo.rules.Plain}
    outcomes:
      - name: added
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {operator: input.operator, plain: input.plain}
components:
  - component: rules-service
    summary: Keeps the rules.
    owns:
      domains: [demo.rules]
    accepts:
      commands: [demo.rules.AddRule]
    publishes:
      events: [demo.rules.RuleAdded]
    reached_by: network
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("rules.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifacts(name: &str) -> Vec<(String, String)> {
    let generator = ess_gen::generator(name).expect("the generator exists");
    ess_gen::artifact::run(generator.as_ref(), &ir())
        .expect("generates")
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

/// Every object in `value` whose `x-ess-name` is `name`.
fn declarations<'a>(
    value: &'a serde_json::Value,
    name: &str,
    out: &mut Vec<&'a serde_json::Value>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("x-ess-name").and_then(serde_json::Value::as_str) == Some(name) {
                out.push(value);
            }
            for value in map.values() {
                declarations(value, name, out);
            }
        }
        serde_json::Value::Array(items) => {
            for value in items {
                declarations(value, name, out);
            }
        }
        _ => {}
    }
}

#[test]
fn enum_attributes_projected() {
    let expected = serde_json::json!([
        {"name": "takes_number", "type": "Boolean", "kind": "boolean", "values": {"gt": true, "Contains": false}},
        {"name": "arity", "type": "Optional<Integer>", "kind": "integer", "values": {"gt": 2}},
        {"name": "weight", "type": "Decimal", "kind": "decimal", "values": {"gt": "0.5", "Contains": "1.0"}},
        {"name": "label", "type": "String", "kind": "string", "values": {"gt": ">", "Contains": "contains"}},
    ]);
    for generator in ["schema", "openapi"] {
        let mut seen = 0;
        for (path, contents) in artifacts(generator) {
            let document: serde_json::Value = if std::path::Path::new(&path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                serde_json::from_str(&contents).expect("JSON")
            } else {
                serde_yaml::from_str(&contents).expect("YAML")
            };
            let mut operators = Vec::new();
            declarations(&document, "demo.rules.Operator", &mut operators);
            for operator in operators {
                assert_eq!(operator["x-ess-attributes"], expected, "{generator} {path}");
                seen += 1;
            }
            let mut plains = Vec::new();
            declarations(&document, "demo.rules.Plain", &mut plains);
            for plain in plains {
                assert!(
                    plain.get("x-ess-attributes").is_none(),
                    "an enum without attributes keeps its bytes: {plain}"
                );
            }
        }
        assert!(seen > 0, "{generator} projects the attributed enum");
    }

    let docs: String = artifacts("docs")
        .into_iter()
        .map(|(_, contents)| contents)
        .collect();
    for row in [
        "| variant | `takes_number` | `arity` | `weight` | `label` |",
        "| `GreaterThan` | `true` | `2` | `0.5` | `>` |",
        "| `Contains` | `false` | — | `1.0` | `contains` |",
    ] {
        assert!(docs.contains(row), "the docs table has `{row}`:\n{docs}");
    }
}
