//! Every projection spells a field by its wire name (beyond10x/ess#142).
//!
//! The issue's repro writes `naming: {wire: orderId}` on a command input, an event field and a
//! struct field. JSON Schema, `OpenAPI` and `AsyncAPI` must carry `orderId` as the property and
//! never the declared `order_id`; and the nested spelling must be the same model as the flat
//! `wire:`, so every artifact is byte-identical between the two.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::asyncapi::AsyncApi;
use ess_gen::openapi::OpenApi;
use ess_gen::schema::JsonSchema;

const NESTED: &str = "format: ess/13
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
      - {name: receipt, type: Optional<demo.orders.Receipt>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {order_id: input.order_id}
components:
  - component: orders
    owns: {domains: [demo.orders]}
    accepts: {commands: [demo.orders.Place]}
    publishes: {events: [demo.orders.Placed]}
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// Every mapping key anywhere in a JSON or YAML document.
fn keys(value: &serde_yaml::Value, into: &mut Vec<String>) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, value) in map {
                if let Some(key) = key.as_str() {
                    into.push(key.to_owned());
                }
                keys(value, into);
            }
        }
        serde_yaml::Value::Sequence(items) => items.iter().for_each(|item| keys(item, into)),
        serde_yaml::Value::Tagged(tagged) => keys(&tagged.value, into),
        _ => {}
    }
}

fn artifacts(ir: &EssIr) -> Vec<(String, String)> {
    let mut all = Vec::new();
    for (path, artifact) in run(&JsonSchema, ir).expect("schema generates") {
        all.push((path, artifact.contents));
    }
    for (path, artifact) in run(&OpenApi, ir).expect("openapi generates") {
        all.push((path, artifact.contents));
    }
    for (path, artifact) in run(&AsyncApi, ir).expect("asyncapi generates") {
        all.push((path, artifact.contents));
    }
    all
}

#[test]
fn schema_openapi_and_asyncapi_key_the_field_by_its_wire_name() {
    let ir = compiled(NESTED);
    let artifacts = artifacts(&ir);
    let mut seen = std::collections::BTreeSet::new();
    for (path, contents) in &artifacts {
        let Ok(document) = serde_yaml::from_str::<serde_yaml::Value>(contents) else {
            continue;
        };
        let mut found = Vec::new();
        keys(&document, &mut found);
        assert!(
            !found.iter().any(|key| key == "order_id"),
            "`{path}` keys a property by the declared name `order_id`:\n{contents}"
        );
        if found.iter().any(|key| key == "orderId") {
            let family = if path.contains("openapi") {
                "openapi"
            } else if path.contains("asyncapi") {
                "asyncapi"
            } else {
                "schema"
            };
            seen.insert(family);
        }
    }
    assert_eq!(
        seen.into_iter().collect::<Vec<_>>(),
        vec!["asyncapi", "openapi", "schema"],
        "every projection family carries `orderId`; artifacts: {:?}",
        artifacts.iter().map(|(path, _)| path).collect::<Vec<_>>()
    );
}

#[test]
fn the_nested_and_flat_spellings_project_byte_identically() {
    let flat = NESTED.replace("naming: {wire: orderId}", "wire: orderId");
    let nested = artifacts(&compiled(NESTED));
    let flat = artifacts(&compiled(&flat));
    assert_eq!(nested.len(), flat.len());
    for ((path, left), (other, right)) in nested.iter().zip(&flat) {
        assert_eq!(path, other);
        assert_eq!(left, right, "`{path}` differs between the two spellings");
    }
}
