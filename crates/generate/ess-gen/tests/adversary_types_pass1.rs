//! Adversary pass 1 on the wave-2 `types` unit: presence policies on the positions #139 names
//! (a command response) in every projection family, spelled with a nested wire name; and a prefix
//! through a chain of newtypes.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::asyncapi::AsyncApi;
use ess_gen::openapi::OpenApi;
use ess_gen::schema::JsonSchema;
use serde_json::Value;

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Placed
    fields:
      - name: partner_ref
        type: Optional<String>
        naming: {wire: partnerRef}
        presence: null_when_absent
      - name: discount_code
        type: Optional<String>
        naming: {wire: discountCode}
        presence: omitted_when_absent
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
    response:
      - name: partner_ref
        type: Optional<String>
        naming: {wire: partnerRef}
        presence: null_when_absent
      - name: discount_code
        type: Optional<String>
        naming: {wire: discountCode}
        presence: omitted_when_absent
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {partner_ref: input.partner_ref, discount_code: input.discount_code}
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

fn artifacts(ir: &EssIr) -> Vec<(String, Value)> {
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
    all.into_iter()
        .filter_map(|(path, contents)| {
            serde_yaml::from_str::<Value>(&contents)
                .ok()
                .map(|value| (path, value))
        })
        .collect()
}

/// Every object schema whose `properties` names `property`.
fn owners<'a>(value: &'a Value, property: &str, into: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map
                .get("properties")
                .and_then(Value::as_object)
                .is_some_and(|properties| properties.contains_key(property))
            {
                into.push(value);
            }
            for child in map.values() {
                owners(child, property, into);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| owners(item, property, into)),
        _ => {}
    }
}

fn admits_null(schema: &Value) -> bool {
    schema["type"] == "null"
        || schema["anyOf"]
            .as_array()
            .is_some_and(|options| options.iter().any(admits_null))
        || schema["oneOf"]
            .as_array()
            .is_some_and(|options| options.iter().any(admits_null))
}

fn required(owner: &Value, property: &str) -> bool {
    owner["required"]
        .as_array()
        .is_some_and(|names| names.iter().any(|name| name == property))
}

#[test]
fn adversary_139_every_projection_holds_response_and_event_fields_to_their_policy() {
    let mut families = std::collections::BTreeSet::new();
    for (path, document) in artifacts(&compiled(MODEL)) {
        let mut found = Vec::new();
        owners(&document, "partnerRef", &mut found);
        for owner in &found {
            families.insert(if path.contains("openapi") {
                "openapi"
            } else if path.contains("asyncapi") {
                "asyncapi"
            } else {
                "schema"
            });
            assert!(
                required(owner, "partnerRef"),
                "`{path}`: null_when_absent is required: {owner:#}"
            );
            assert!(
                admits_null(&owner["properties"]["partnerRef"]),
                "`{path}`: null_when_absent is nullable: {owner:#}"
            );
            assert!(
                !required(owner, "discountCode"),
                "`{path}`: omitted_when_absent is optional: {owner:#}"
            );
            assert!(
                !admits_null(&owner["properties"]["discountCode"]),
                "`{path}`: omitted_when_absent is not nullable: {owner:#}"
            );
        }
    }
    // OpenAPI publishes a command's response fields only under an outcome that retains its
    // result, and this model has none, so only the other two families carry `partnerRef` here.
    assert!(
        families.contains("schema") && families.contains("asyncapi"),
        "{families:?}"
    );
}

/// An outer newtype over a prefixed one, narrowing it: every value starts with the longer prefix,
/// and the published schema refuses one that starts only with the shorter.
#[test]
fn adversary_146_a_narrowing_prefix_is_enforced_through_the_chain() {
    let text = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - name: demo.msgs.Channel
    kind: newtype
    of: String
    prefix: \"/\"
  - name: demo.msgs.OpsChannel
    kind: newtype
    of: demo.msgs.Channel
    prefix: \"/ops.\"
";
    let artifacts = run(&JsonSchema, &compiled(text)).expect("generates");
    let path = "schema/types/demo.msgs.OpsChannel.schema.json";
    let schema: Value = serde_json::from_str(
        &artifacts
            .get(path)
            .unwrap_or_else(|| panic!("no {path}: {:?}", artifacts.keys().collect::<Vec<_>>()))
            .contents,
    )
    .unwrap();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    assert!(
        validator.is_valid(&serde_json::json!("/ops.east")),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&serde_json::json!("/opsXeast")),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&serde_json::json!("/general")),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&serde_json::json!("ops.east")),
        "{schema:#}"
    );
}
