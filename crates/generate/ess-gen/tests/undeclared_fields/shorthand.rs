//! `shorthand: <field>` in the projections (`ess/27`, beyond10x/ess#594).
//!
//! A struct with a shorthand is `oneOf` the field's own schema and the closed object, annotated
//! `x-ess-shorthand` with the field's wire name. The two branches differ by JSON type, so exactly
//! one matches. Every projection reaches the one definition, so `OpenAPI` shares it.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::{json, Value};

/// `{shorthand}` is the line written on `Operation`.
const MODEL: &str = "\
format: ess/27
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Operation
    kind: struct
{shorthand}    fields:
      - name: op
        type: String
        wire: operation
      - name: paging
        type: Optional<String>
actors:
  - name: tracker.ops.Operator
    may:
      - tracker.ops.Run
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
    response:
      - name: operation
        type: tracker.ops.Operation
    outcomes:
      - name: ran
        returns: true
components:
  - component: ops
    owns: {domains: [tracker.ops]}
    accepts: {commands: [tracker.ops.Run]}
    reached_by: network
";

fn outputs(shorthand: bool) -> std::collections::BTreeMap<String, ess_gen::Artifact> {
    let text = MODEL.replace(
        "{shorthand}",
        if shorthand { "    shorthand: op\n" } else { "" },
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("tracker.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let model: EssIr =
        compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"));
    ess_gen::generate_all(&model).unwrap_or_else(|error| panic!("{error}"))
}

fn json_artifact(
    outputs: &std::collections::BTreeMap<String, ess_gen::Artifact>,
    key: &str,
) -> Value {
    let artifact = outputs
        .get(key)
        .unwrap_or_else(|| panic!("{key} in {:?}", outputs.keys().collect::<Vec<_>>()));
    ess_yaml::from_str(&artifact.contents).expect("the document parses")
}

#[test]
fn shorthand_struct_definition_is_one_of_the_field_and_the_closed_object() {
    let outputs = outputs(true);
    let input = json_artifact(&outputs, "schema/commands/tracker.ops.Run.schema.json");
    let definition = &input["$defs"]["tracker.ops.Operation"];
    assert_eq!(
        definition["x-ess-shorthand"],
        json!("operation"),
        "{definition}"
    );
    assert_eq!(definition["x-ess-kind"], json!("struct"), "{definition}");
    let branches = definition["oneOf"].as_array().expect("a oneOf");
    assert_eq!(branches.len(), 2, "{definition}");
    assert_eq!(branches[0], json!({"type": "string"}), "{definition}");
    assert_eq!(branches[1]["type"], json!("object"), "{definition}");
    assert_eq!(
        branches[1]["additionalProperties"],
        json!(false),
        "{definition}"
    );
    assert_eq!(
        branches[1]["required"],
        json!(["operation"]),
        "{definition}"
    );
    assert!(definition.get("type").is_none(), "{definition}");

    let validator = jsonschema::draft202012::new(&input).expect("the schema compiles");
    for admitted in [
        json!({"operation": "getIssue"}),
        json!({"operation": {"operation": "getIssue", "paging": "token"}}),
    ] {
        assert!(validator.is_valid(&admitted), "{admitted}");
    }
    for refused in [
        json!({"operation": 7}),
        json!({"operation": ["getIssue"]}),
        json!({"operation": {"paging": "token"}}),
        json!({"operation": {"operation": "getIssue", "extra": 1}}),
    ] {
        assert!(!validator.is_valid(&refused), "{refused}");
    }

    // `OpenAPI` reads the same definition.
    let openapi = json_artifact(&outputs, "openapi/ops.yaml");
    assert_eq!(
        &openapi["components"]["schemas"]["tracker.ops.Operation"]["x-ess-shorthand"],
        &json!("operation")
    );

    // The generated documentation says so.
    assert!(
        outputs.values().any(|artifact| artifact
            .contents
            .contains("may also be written as its `op` alone")),
        "a generated page names the shorthand"
    );
}

#[test]
fn a_struct_without_shorthand_keeps_its_projection() {
    let outputs = outputs(false);
    let input = json_artifact(&outputs, "schema/commands/tracker.ops.Run.schema.json");
    let definition = &input["$defs"]["tracker.ops.Operation"];
    assert_eq!(definition["type"], json!("object"), "{definition}");
    assert!(definition.get("oneOf").is_none(), "{definition}");
    assert!(definition.get("x-ess-shorthand").is_none(), "{definition}");
}
