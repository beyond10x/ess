//! Closed reader, resolution and ownership acceptance cases.
use ess_cli_contract::{compile, Binding};
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/model.yaml");
const BINDING: &str = include_str!("fixtures/cli.yaml");
const SERVICE_MODEL: &str = include_str!("fixtures/service-model.yaml");
const SERVICE_BINDING: &str = include_str!("fixtures/service-cli.yaml");

fn model(text: &str) -> EssIr {
    let specification = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap()
}

fn service_model() -> EssIr {
    let specification = Specification::assemble(vec![
        (
            Source::new("service.yaml"),
            RawSpecFile::parse(SERVICE_MODEL).unwrap(),
        ),
        (
            Source::new("foreign.yaml"),
            RawSpecFile::parse("domain: demo.foreign\n").unwrap(),
        ),
    ])
    .unwrap();
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap()
}

#[test]
fn resolves_value_types_without_inventing_a_component_or_entity() {
    let model = model(MODEL);
    let before = model.to_canonical_json();
    let binding = Binding::from_yaml(BINDING).unwrap();
    let compiled = compile(&model, &binding).unwrap();
    let encoded = compiled.to_canonical_json();
    assert!(encoded.contains("demo.Stored"));
    assert!(encoded.contains("store-credential"));
    assert_eq!(
        encoded,
        compile(&model, &binding).unwrap().to_canonical_json()
    );
    assert_eq!(model.to_canonical_json(), before);
    assert!(model.components().is_empty());
    assert!(model.entities().is_empty());
}

#[test]
fn closes_unknown_format_and_fields_at_every_reader_layer() {
    for invalid in [
        BINDING.replace("ess-cli/1", "ess-cli/2"),
        format!("{BINDING}\nunknown: false\n"),
        BINDING.replace("kind: local", "kind: local, unknown: false"),
        BINDING.replace("kind: option", "kind: option, unknown: false"),
        BINDING.replace("state: state-dir", "state: state-dir, unknown: false"),
        BINDING.replace("input: demo.Input", "input: demo.Input\n    unknown: false"),
        BINDING.replace("callable: store", "callable: store\n    unknown: false"),
        BINDING.replace(
            "{field: profile, source:",
            "{unknown: false, field: profile, source:",
        ),
    ] {
        assert!(Binding::from_yaml(&invalid).is_err(), "accepted {invalid}");
    }
}

#[test]
fn service_forward_command_preserves_the_model_owner_and_input_identity() {
    use ess_cli_contract::wire::Target;
    let model = service_model();
    let before = model.to_canonical_json();
    let compiled = compile(&model, &Binding::from_yaml(SERVICE_BINDING).unwrap()).unwrap();
    let callable = &compiled.plan().callables["rename"];
    assert!(
        matches!(&callable.target, Target::ServiceForward { owner, operation }
        if owner == "owner-service" && operation == "demo.records.Rename")
    );
    assert_eq!(
        callable.input.as_ref().unwrap().type_ref,
        "demo.records.RenameInput"
    );
    assert!(callable
        .input
        .as_ref()
        .unwrap()
        .shape
        .accepts(&serde_json::json!({"name":"new"})));
    assert_eq!(model.to_canonical_json(), before);
}

#[test]
fn service_forward_parameterized_view_resolves_rows_without_changing_ownership() {
    let model = service_model();
    let before = model.to_canonical_json();
    let compiled = compile(&model, &Binding::from_yaml(SERVICE_BINDING).unwrap()).unwrap();
    let callable = &compiled.plan().callables["query"];
    assert!(callable
        .input
        .as_ref()
        .unwrap()
        .shape
        .accepts(&serde_json::json!({"prefix":"a"})));
    assert!(!callable
        .input
        .as_ref()
        .unwrap()
        .shape
        .accepts(&serde_json::json!({})));
    assert!(callable
        .result
        .shape
        .accepts(&serde_json::json!([{"name":"a"}])));
    assert!(!callable
        .result
        .shape
        .accepts(&serde_json::json!([{"name":3}])));
    assert_eq!(model.to_canonical_json(), before);
    let row_result =
        SERVICE_BINDING.replace("result: List<demo.records.Row>", "result: demo.records.Row");
    assert!(compile(&model, &Binding::from_yaml(&row_result).unwrap()).is_ok());
}

#[test]
fn service_forward_refuses_foreign_owners_and_mismatched_input_or_view_shapes() {
    let model = service_model();
    for (needle, replacement) in [
        (
            "owner: owner-service, operation: demo.records.Rename",
            "owner: foreign-service, operation: demo.records.Rename",
        ),
        (
            "owner: owner-service, operation: demo.records.NamedRecords",
            "owner: foreign-service, operation: demo.records.NamedRecords",
        ),
        ("owner: owner-service", "owner: missing-service"),
        (
            "operation: demo.records.Rename",
            "operation: demo.records.Missing",
        ),
        (
            "input: demo.records.RenameInput",
            "input: demo.records.WrongInput",
        ),
        (
            "input: demo.records.QueryInput",
            "input: demo.records.WrongQuery",
        ),
        (
            "result: List<demo.records.Row>",
            "result: List<demo.records.WrongRow>",
        ),
    ] {
        let binding = Binding::from_yaml(&SERVICE_BINDING.replace(needle, replacement)).unwrap();
        assert!(compile(&model, &binding).is_err(), "accepted {replacement}");
    }
}

#[test]
fn supported_nested_value_shapes_are_closed_and_validate_every_element() {
    let specification = format!("{MODEL}\n  - name: demo.Label\n    kind: newtype\n    of: String\n  - name: demo.Composite\n    kind: struct\n    fields:\n      - {{name: label, type: demo.Label}}\n      - {{name: enabled, type: Boolean}}\n      - {{name: notes, type: 'Optional<List<String>>'}}\n      - {{name: counts, type: 'Map<String, Integer>'}}\n");
    let binding =
        Binding::from_yaml(&BINDING.replace("result: demo.Stored", "result: demo.Composite"))
            .unwrap();
    let compiled = compile(&model(&specification), &binding).unwrap();
    let shape = &compiled.plan().callables["store"].result.shape;
    for valid in [
        serde_json::json!({"label":"ok", "enabled":false, "counts":{"first":3}}),
        serde_json::json!({"label":"ok", "enabled":true, "notes":null, "counts":{}}),
        serde_json::json!({"label":"ok", "enabled":true, "notes":["x"], "counts":{}}),
    ] {
        assert!(shape.accepts(&valid));
    }
    for invalid in [
        serde_json::json!({"label":"ok", "enabled":true, "counts":{}, "unknown":false}),
        serde_json::json!({"enabled":true, "counts":{}}),
        serde_json::json!({"label":false, "enabled":true, "counts":{}}),
        serde_json::json!({"label":"ok", "enabled":"true", "counts":{}}),
        serde_json::json!({"label":"ok", "enabled":true, "notes":[1], "counts":{}}),
        serde_json::json!({"label":"ok", "enabled":true, "counts":{"first":"3"}}),
    ] {
        assert!(!shape.accepts(&invalid), "accepted {invalid}");
    }
}

#[test]
fn inputless_local_calls_require_explicit_null_and_no_fictional_struct() {
    let binding = "format: ess-cli/1\nbinary: demo\nabout: Inputless action\nglobals: {config: config, state: state-dir, output: output}\ncallables:\n  show:\n    target: {kind: local, owner: demo.cli, action: show}\n    input: null\n    result: demo.Stored\ncommands:\n  - path: [show]\n    callable: show\n    about: Show\n    arguments: []\n";
    let compiled = compile(&model(MODEL), &Binding::from_yaml(binding).unwrap()).unwrap();
    assert!(compiled.plan().callables["show"].input.is_none());
    assert!(Binding::from_yaml(&binding.replace("    input: null\n", "")).is_err());
    assert!(compile(
        &model(MODEL),
        &Binding::from_yaml(&binding.replace("input: null", "input: demo.Missing")).unwrap()
    )
    .is_err());
}

#[test]
fn integer_values_use_the_models_exact_signed_64_bit_range() {
    use ess_cli_contract::wire::Shape;
    for text in [
        "0",
        "-1",
        "9007199254740993",
        "9223372036854775807",
        "-9223372036854775808",
    ] {
        assert!(
            Shape::Integer.accepts(&serde_json::from_str(text).unwrap()),
            "rejected {text}"
        );
    }
    for text in [
        "9223372036854775808",
        "18446744073709551615",
        "-9223372036854775809",
        "1.5",
        "true",
        "\"1\"",
    ] {
        assert!(
            !Shape::Integer.accepts(&serde_json::from_str(text).unwrap()),
            "accepted {text}"
        );
    }
}

#[test]
fn refuses_recursive_nested_unresolved_and_constrained_types() {
    let binding = Binding::from_yaml(BINDING).unwrap();
    for invalid in [
        MODEL.replace("name: secret, type: String", "name: secret, type: 'Optional<demo.Input>'"),
        format!("{MODEL}\n  - name: demo.Secret\n    kind: newtype\n    of: String\n    invariants: [value != \"\"]\n").replace("name: secret, type: String", "name: secret, type: demo.Secret"),
    ] {
        assert!(compile(&model(&invalid), &binding).is_err());
    }
    for invalid in [
        "List<demo.Missing>",
        "Map<String, demo.Missing>",
        "Optional<demo.Missing>",
        "Map<Integer, String>",
    ] {
        let binding = Binding::from_yaml(
            &BINDING.replace("result: demo.Stored", &format!("result: '{invalid}'")),
        )
        .unwrap();
        assert!(
            compile(&model(MODEL), &binding).is_err(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn refuses_unresolved_types_fields_and_ambiguous_cli_sources() {
    let model = model(MODEL);
    for (needle, replacement) in [
        ("result: demo.Stored", "result: demo.Missing"),
        ("input: demo.Input", "input: String"),
        ("field: profile", "field: missing"),
        ("long: profile", "long: output"),
        ("stdin: secret-stdin", "stdin: secret-file"),
        ("[credential, set]", "[credential, store]"),
        ("[credential, store]", "[a, b, c]"),
        ("owner: demo.cli", "owner: bad owner"),
    ] {
        let binding = Binding::from_yaml(&BINDING.replace(needle, replacement));
        assert!(
            binding.is_err() || compile(&model, &binding.unwrap()).is_err(),
            "accepted {replacement}"
        );
    }
}

#[test]
fn refuses_unsupported_constraints_and_primitive_promises() {
    let binding = Binding::from_yaml(BINDING).unwrap();
    for invalid in [
        MODEL.replace("name: secret, type: String", "name: secret, type: Uuid"),
        MODEL.replace(
            "name: demo.Stored\n    kind: struct",
            "name: demo.Stored\n    kind: struct\n    invariants: [profile != \"\"]",
        ),
    ] {
        assert!(compile(&model(&invalid), &binding).is_err());
    }
}
