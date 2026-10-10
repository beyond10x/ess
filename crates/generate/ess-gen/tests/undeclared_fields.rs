//! `undeclared_fields: ignored` in the projections (`ess/24`, beyond10x/ess#500).
//!
//! At a command response declared `ignored`, and at every object of a struct type declared
//! `ignored`, the JSON Schema tree, `OpenAPI` and `AsyncAPI` write `"additionalProperties": true`
//! explicitly — a keyword is an assertion, and an absent one reads as an oversight. A closed
//! sibling keeps `false`, and a model that never writes the key keeps every byte it had.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::{json, Value};

/// `{command}` and `{record}` are the keys written on `PlaceOrder` and on `Extension`.
const MODEL: &str = "\
format: ess/24
system: catalog
version: v1
domain: catalog.orders
types:
  - name: catalog.orders.Extension
    kind: struct
{record}    fields:
      - name: vendor
        type: String
  - name: catalog.orders.Closed
    kind: struct
    fields:
      - name: note
        type: String
events:
  - name: catalog.orders.OrderPlaced
    fields:
      - name: extension
        type: catalog.orders.Extension
      - name: closed
        type: catalog.orders.Closed
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder
      - catalog.orders.CheckOrder
commands:
  - name: catalog.orders.PlaceOrder
{command}    input:
      - name: item
        type: String
    response:
      - name: order_ref
        type: String
      - name: extension
        type: catalog.orders.Extension
      - name: closed
        type: catalog.orders.Closed
    outcomes:
      - name: placed
        returns: true
        emits: [catalog.orders.OrderPlaced]
        payload:
          catalog.orders.OrderPlaced:
            extension: {generated: true}
            closed: {generated: true}
  - name: catalog.orders.CheckOrder
    input:
      - name: item
        type: String
    response:
      - name: order_ref
        type: String
    outcomes:
      - name: checked
        returns: true
components:
  - component: orders
    owns: {domains: [catalog.orders]}
    accepts: {commands: [catalog.orders.PlaceOrder, catalog.orders.CheckOrder]}
    publishes: {events: [catalog.orders.OrderPlaced]}
    reached_by: network
";

fn text(command: Option<&str>, record: Option<&str>) -> String {
    let key = |value: Option<&str>| {
        value.map_or_else(String::new, |value| {
            format!("    undeclared_fields: {value}\n")
        })
    };
    MODEL
        .replace("{command}", &key(command))
        .replace("{record}", &key(record))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn outputs(model: &EssIr) -> std::collections::BTreeMap<String, ess_gen::Artifact> {
    ess_gen::generate_all(model).unwrap_or_else(|error| panic!("{error}"))
}

fn schema(outputs: &std::collections::BTreeMap<String, ess_gen::Artifact>, path: &str) -> Value {
    let artifact = outputs
        .get(&format!("schema/{path}"))
        .unwrap_or_else(|| panic!("schema/{path} in {:?}", outputs.keys().collect::<Vec<_>>()));
    serde_json::from_str(&artifact.contents).expect("a schema is JSON")
}

fn yaml(outputs: &std::collections::BTreeMap<String, ess_gen::Artifact>, key: &str) -> Value {
    let artifact = outputs
        .get(key)
        .unwrap_or_else(|| panic!("{key} in {:?}", outputs.keys().collect::<Vec<_>>()));
    serde_yaml::from_str(&artifact.contents).expect("the document is YAML")
}

#[test]
fn the_schema_tree_opens_the_ignored_response_and_struct_and_keeps_the_siblings_closed() {
    let model = ir(&text(Some("ignored"), Some("ignored")));
    let outputs = outputs(&model);

    let placed = schema(&outputs, "responses/catalog.orders.PlaceOrder.schema.json");
    assert_eq!(placed["additionalProperties"], json!(true), "{placed}");
    assert_eq!(
        placed["$defs"]["catalog.orders.Extension"]["additionalProperties"],
        json!(true),
        "{placed}"
    );
    assert_eq!(
        placed["$defs"]["catalog.orders.Closed"]["additionalProperties"],
        json!(false),
        "{placed}"
    );

    let checked = schema(&outputs, "responses/catalog.orders.CheckOrder.schema.json");
    assert_eq!(checked["additionalProperties"], json!(false), "{checked}");

    // The input is never opened by the command's key: it governs the response only.
    let input = schema(&outputs, "commands/catalog.orders.PlaceOrder.schema.json");
    assert_eq!(input["additionalProperties"], json!(false), "{input}");

    // The struct is open wherever it is reached: an event payload too.
    let event = schema(&outputs, "events/catalog.orders.OrderPlaced.schema.json");
    assert_eq!(event["additionalProperties"], json!(false), "{event}");
    assert_eq!(
        event["$defs"]["catalog.orders.Extension"]["additionalProperties"],
        json!(true),
        "{event}"
    );
    assert_eq!(
        event["$defs"]["catalog.orders.Closed"]["additionalProperties"],
        json!(false),
        "{event}"
    );

    // The published schema admits an undeclared member at the open objects only.
    let validator = jsonschema::draft202012::new(&placed).expect("the schema compiles");
    let answer = json!({
        "order_ref": "o-1",
        "extension": {"vendor": "v", "vendor_note": "x"},
        "closed": {"note": "n"},
        "extra": 1,
    });
    assert!(
        validator.is_valid(&answer),
        "extras at the open objects validate"
    );
    let closed_extra = json!({
        "order_ref": "o-1",
        "extension": {"vendor": "v"},
        "closed": {"note": "n", "extra": 1},
    });
    assert!(
        !validator.is_valid(&closed_extra),
        "an extra member at the closed struct is still refused"
    );
    let missing = json!({"extension": {"vendor": "v"}, "closed": {"note": "n"}});
    assert!(
        !validator.is_valid(&missing),
        "a declared field is still required"
    );
}

#[test]
fn openapi_and_asyncapi_open_the_ignored_response_and_struct_and_keep_the_siblings_closed() {
    let model = ir(&text(Some("ignored"), Some("ignored")));
    let outputs = outputs(&model);

    let openapi = yaml(&outputs, "openapi/orders.yaml");
    let schemas = &openapi["components"]["schemas"];
    assert_eq!(
        schemas["catalog.orders.PlaceOrder.Result"]["additionalProperties"],
        json!(true),
        "{}",
        schemas["catalog.orders.PlaceOrder.Result"]
    );
    assert_eq!(
        schemas["catalog.orders.CheckOrder.Result"]["additionalProperties"],
        json!(false)
    );
    assert_eq!(
        schemas["catalog.orders.PlaceOrder.Input"]["additionalProperties"],
        json!(false)
    );
    assert_eq!(
        schemas["catalog.orders.Extension"]["additionalProperties"],
        json!(true)
    );
    assert_eq!(
        schemas["catalog.orders.Closed"]["additionalProperties"],
        json!(false)
    );

    let component = model.components().values().next().expect("one component");
    let json_document: Value =
        serde_json::from_str(&ess_gen::openapi::json(&model, component)).expect("JSON");
    assert_eq!(
        json_document["components"]["schemas"]["catalog.orders.PlaceOrder.Result"]
            ["additionalProperties"],
        json!(true)
    );

    let asyncapi = yaml(&outputs, "asyncapi/orders.yaml");
    let schemas = &asyncapi["components"]["schemas"];
    assert_eq!(
        schemas["type.catalog.orders.Extension"]["additionalProperties"],
        json!(true),
        "{schemas}"
    );
    assert_eq!(
        schemas["type.catalog.orders.Closed"]["additionalProperties"],
        json!(false),
        "{schemas}"
    );
}

#[test]
fn the_generated_documentation_states_which_records_ignore_undeclared_fields() {
    let model = ir(&text(Some("ignored"), Some("ignored")));
    let artifacts =
        ess_gen::artifact::run(ess_gen::generator("docs").unwrap().as_ref(), &model).unwrap();
    let pages: String = artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect();
    assert!(
        pages.contains("Its response ignores a field it does not declare"),
        "the command says its response is open"
    );
    assert!(
        pages.contains("A field it does not declare is ignored"),
        "the struct says it is open"
    );

    let closed = ir(&text(None, None));
    let artifacts =
        ess_gen::artifact::run(ess_gen::generator("docs").unwrap().as_ref(), &closed).unwrap();
    let pages: String = artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect();
    assert!(!pages.contains("does not declare is ignored"), "{pages}");
    assert!(
        !pages.contains("ignores a field it does not declare"),
        "{pages}"
    );
}

#[test]
fn writing_the_default_or_nothing_changes_no_byte_of_any_projection() {
    let unwritten = outputs(&ir(&text(None, None)));
    let refused = outputs(&ir(&text(Some("refused"), Some("refused"))));
    let differing: Vec<&String> = unwritten
        .iter()
        .filter(|(path, artifact)| {
            refused
                .get(*path)
                .is_none_or(|other| other.contents != artifact.contents)
        })
        .map(|(path, _)| path)
        .collect();
    assert!(differing.is_empty(), "{differing:?}");
    assert!(
        unwritten.values().all(|artifact| !artifact
            .contents
            .contains("\"additionalProperties\": true")
            && !artifact.contents.contains("additionalProperties: true")),
        "a closed model publishes no open object"
    );
}
