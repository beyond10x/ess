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

/// An alphabet constrains a type exactly as an invariant does, and the CLI cannot enforce either
/// (`docs/design/string-alphabet-and-length.md`, section 6): the same refusal for both.
#[test]
fn refuses_an_alphabet_only_newtype_as_it_refuses_an_invariant_one() {
    let binding = Binding::from_yaml(BINDING).unwrap();
    for constraint in [
        "    invariants: [value != \"\"]\n",
        "    alphabet: \"abc\"\n",
    ] {
        let text = format!(
            "{MODEL}\n  - name: demo.Secret\n    kind: newtype\n    of: String\n{constraint}"
        )
        .replace(
            "name: secret, type: String",
            "name: secret, type: demo.Secret",
        )
        .replacen("format: ess/1\n", "format: ess/11\n", 1);
        let error = compile(&model(&text), &binding).expect_err(constraint);
        assert!(
            error
                .to_string()
                .contains("`demo.Secret` has unsupported invariants or union semantics"),
            "{constraint}: {error}"
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

/// beyond10x/ess#274: `invalid_input` names a declared error whose type admits `{}`, because the
/// answer to invalid input carries no field and so no input text.
#[test]
fn invalid_input_names_a_declared_error_that_needs_no_field() {
    let model = model(&format!(
        "{MODEL}  - name: demo.Invalid\n    kind: struct\n    fields:\n      - {{name: detail, type: 'Optional<String>'}}\n"
    ));
    let declared = BINDING.replace(
        "errors: {store_failed: demo.Failure}",
        "errors: {store_failed: demo.Failure, invalid: demo.Invalid, listed: 'Map<String, String>'}\n    invalid_input: invalid",
    );
    let compiled = compile(&model, &Binding::from_yaml(&declared).unwrap()).unwrap();
    assert_eq!(
        compiled.plan().callables["store"].invalid_input.as_deref(),
        Some("invalid")
    );
    assert!(compiled
        .to_canonical_json()
        .contains("\"invalid_input\": \"invalid\""));
    let map = declared.replace("invalid_input: invalid", "invalid_input: listed");
    assert!(compile(&model, &Binding::from_yaml(&map).unwrap()).is_ok());
    // Absent, the plan bytes carry no key.
    let plain = compile(&model, &Binding::from_yaml(BINDING).unwrap()).unwrap();
    assert!(plain.plan().callables["store"].invalid_input.is_none());
    assert!(!plain.to_canonical_json().contains("invalid_input"));
    for (value, expected) in [
        ("missing", "undeclared error `missing`"),
        ("store_failed", "requires a field"),
        ("cli_parse", "fails before a callable is known"),
        ("cli_input", "undeclared error `cli_input`"),
    ] {
        let binding =
            declared.replace("invalid_input: invalid", &format!("invalid_input: {value}"));
        let error = compile(&model, &Binding::from_yaml(&binding).unwrap())
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{value}: {error}");
    }
    let inputless = "format: ess-cli/1\nbinary: demo\nabout: Inputless action\nglobals: {config: config, state: state-dir, output: output}\ncallables:\n  show:\n    target: {kind: local, owner: demo.cli, action: show}\n    input: null\n    result: demo.Stored\n    errors: {invalid: demo.Invalid}\n    invalid_input: invalid\ncommands:\n  - path: [show]\n    callable: show\n    about: Show\n    arguments: []\n";
    let error = compile(&model, &Binding::from_yaml(inputless).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("inputless"), "{error}");
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

/// beyond10x/ess#466: a launcher whose `args` field takes every argv word after `--`.
const LAUNCH_MODEL: &str = "format: ess/1
system: demo
version: v1
types:
  - name: demo.Word
    kind: newtype
    of: String
  - name: demo.LaunchInput
    kind: struct
    fields:
      - {name: connection, type: String}
      - {name: args, type: 'List<String>'}
  - name: demo.Launched
    kind: struct
    fields:
      - {name: status, type: String}
";

const LAUNCH_BINDING: &str = "format: ess-cli/1
binary: demo
about: Launch a pinned program
globals: {config: config, state: state-dir, output: output}
callables:
  launch:
    target: {kind: local, owner: demo.cli, action: launch}
    input: demo.LaunchInput
    result: demo.Launched
commands:
  - path: [launch]
    callable: launch
    about: Launch with the arguments after --
    arguments:
      - {field: connection, source: {kind: option, long: connection}}
      - {field: args, source: {kind: trailing}}
";

fn launch(
    model_text: &str,
    binding_text: &str,
) -> Result<ess_cli_contract::CompiledBinding, String> {
    compile(
        &model(model_text),
        &Binding::from_yaml(binding_text).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[test]
fn trailing_source_binds_a_required_list_of_strings() {
    use ess_cli_contract::wire::ArgumentSource;
    for model_text in [
        LAUNCH_MODEL.to_owned(),
        // An unconstrained String newtype is a String on the wire.
        LAUNCH_MODEL.replace("type: 'List<String>'", "type: 'List<demo.Word>'"),
    ] {
        let compiled = launch(&model_text, LAUNCH_BINDING).unwrap();
        let arguments = &compiled.plan().commands[0].arguments;
        assert_eq!(arguments[1].field, "args");
        assert!(matches!(arguments[1].source, ArgumentSource::Trailing {}));
        assert!(compiled
            .to_canonical_json()
            .contains("\"source\": {\n            \"kind\": \"trailing\"\n          }"));
    }
}

#[test]
fn binding_refuses_second_trailing_source() {
    let model_text = LAUNCH_MODEL.replace(
        "      - {name: args, type: 'List<String>'}\n",
        "      - {name: args, type: 'List<String>'}\n      - {name: more, type: 'List<String>'}\n",
    );
    let binding = format!("{LAUNCH_BINDING}      - {{field: more, source: {{kind: trailing}}}}\n");
    let error = launch(&model_text, &binding).unwrap_err();
    assert!(error.contains("at most one trailing source"), "{error}");
}

#[test]
fn binding_refuses_trailing_on_other_shapes() {
    for shape in [
        "String",
        "'List<Integer>'",
        "'Optional<List<String>>'",
        "'List<Optional<String>>'",
    ] {
        let model_text = LAUNCH_MODEL.replace("type: 'List<String>'", &format!("type: {shape}"));
        let error = launch(&model_text, LAUNCH_BINDING).unwrap_err();
        assert!(
            error.contains("a trailing source binds a required List<String> field"),
            "{shape}: {error}"
        );
    }
}

#[test]
fn older_kind_refusal_names_trailing() {
    let error = Binding::from_yaml(&LAUNCH_BINDING.replace("kind: trailing", "kind: variadic"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown variant `variadic`"), "{error}");
    assert!(error.contains("`trailing`"), "{error}");
    // The variant has no fields, like the closed reader's other objects.
    assert!(Binding::from_yaml(
        &LAUNCH_BINDING.replace("{kind: trailing}", "{kind: trailing, index: 2}")
    )
    .is_err());
}

/// The design page's launcher example is a complete model and binding that compiles.
#[test]
fn design_page_trailing_example_compiles() {
    let page = include_str!("../../../../docs/design/cli-presentation-binding.md");
    let block = |first: &str| {
        let start = page
            .find(&format!("```yaml\n{first}\n"))
            .unwrap_or_else(|| panic!("the design page has no `{first}` block"))
            + "```yaml\n".len();
        let end = start + page[start..].find("```").unwrap();
        page[start..end].to_owned()
    };
    let binding = block("# launch-cli.yaml");
    assert!(binding.contains("source: {kind: trailing}"), "{binding}");
    let compiled = launch(&block("# launch-system.yaml"), &binding).unwrap();
    assert!(compiled
        .to_canonical_json()
        .contains("\"kind\": \"trailing\""));
}

/// beyond10x/ess#468: a command that answers an arbitrary JSON document declares it `Json`.
/// `ess-cli-project`'s `json_results.rs` generates a package from the same two documents.
const JSON_MODEL: &str = include_str!("fixtures/json-model.yaml");
const JSON_BINDING: &str = include_str!("fixtures/json-cli.yaml");

#[test]
fn json_result_fields_compile_to_the_json_shape() {
    use ess_cli_contract::wire::Shape;
    use std::collections::BTreeMap;
    let json = || Box::new(Shape::Json);
    let compiled = launch(JSON_MODEL, JSON_BINDING).unwrap();
    assert_eq!(
        compiled.plan().callables["describe"].result.shape,
        Shape::Struct {
            fields: BTreeMap::from([
                ("payload".to_owned(), Shape::Json),
                ("maybe".to_owned(), Shape::Optional { of: json() }),
                ("items".to_owned(), Shape::List { of: json() }),
                ("keyed".to_owned(), Shape::Map { value: json() }),
                (
                    "nested".to_owned(),
                    Shape::Optional {
                        of: Box::new(Shape::Struct {
                            fields: BTreeMap::from([("document".to_owned(), Shape::Json)]),
                        }),
                    },
                ),
            ]),
        }
    );
    assert!(compiled.to_canonical_json().contains("\"kind\": \"json\""));
    // The result itself may be `Json`, alone or wrapped.
    for (result, shape) in [
        ("Json", Shape::Json),
        ("'Optional<Json>'", Shape::Optional { of: json() }),
        ("'List<Json>'", Shape::List { of: json() }),
        ("'Map<String, Json>'", Shape::Map { value: json() }),
    ] {
        let binding = JSON_BINDING.replace("result: demo.Described", &format!("result: {result}"));
        let compiled = launch(JSON_MODEL, &binding).unwrap();
        assert_eq!(
            compiled.plan().callables["describe"].result.shape,
            shape,
            "{result}"
        );
    }
}

#[test]
fn json_result_shape_accepts_any_json_value_and_refuses_the_rest() {
    use serde_json::json;
    let compiled = launch(JSON_MODEL, JSON_BINDING).unwrap();
    let shape = &compiled.plan().callables["describe"].result.shape;
    for document in [
        json!({"a": [1, true, null], "b": {"c": "d"}}),
        json!([1, "two", {"three": 3.5}]),
        json!("{\"not\": \"parsed\"}"),
        json!(-3),
        json!(1.5),
        json!(false),
        json!(null),
    ] {
        let value = json!({
            "payload": document,
            "maybe": document,
            "items": [document, {}],
            "keyed": {"k": document},
            "nested": {"document": document},
        });
        assert!(shape.accepts(&value), "rejected {value}");
    }
    assert!(shape.accepts(&json!({"payload": {}, "items": [], "keyed": {}})));
    for invalid in [
        json!({"items": [], "keyed": {}}),
        json!({"payload": {}, "keyed": {}}),
        json!({"payload": {}, "items": {}, "keyed": {}}),
        json!({"payload": {}, "items": [], "keyed": []}),
        json!({"payload": {}, "items": [], "keyed": "{}"}),
        json!({"payload": {}, "items": [], "keyed": {}, "nested": {}}),
        json!({"payload": {}, "items": [], "keyed": {}, "extra": 1}),
        json!([{"payload": {}, "items": [], "keyed": {}}]),
        json!("{\"payload\": {}, \"items\": [], \"keyed\": {}}"),
    ] {
        assert!(!shape.accepts(&invalid), "accepted {invalid}");
    }
}

#[test]
fn json_input_and_error_fields_are_refused_naming_result_fields() {
    for field in [
        "Json",
        "'Optional<Json>'",
        "'List<Json>'",
        "'Map<String, Json>'",
        "demo.Inner",
    ] {
        let model = JSON_MODEL.replacen(
            "{name: profile, type: String}",
            &format!("{{name: profile, type: {field}}}"),
            1,
        );
        assert_eq!(
            launch(&model, JSON_BINDING).unwrap_err(),
            "CLI input field `profile` carries `Json`, which has no argv spelling; \
             `Json` is admitted only in result fields",
            "{field}"
        );
    }
    let binding = JSON_BINDING.replace("{failed: demo.Failure}", "{failed: demo.Inner}");
    assert_eq!(
        launch(JSON_MODEL, &binding).unwrap_err(),
        "CLI error `failed` carries `Json`; `Json` is admitted only in result fields"
    );
}

/// The `Shape` every `ess` before 0.55.0 reads a compiled plan with: the closed tagged enum of
/// `wire.rs` at 0.54.0, with no `json` kind.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(dead_code)]
enum OlderShape {
    String,
    Boolean,
    Integer,
    Enum {
        variants: Vec<String>,
    },
    Optional {
        of: Box<OlderShape>,
    },
    List {
        of: Box<OlderShape>,
    },
    Map {
        value: Box<OlderShape>,
    },
    Struct {
        fields: std::collections::BTreeMap<String, OlderShape>,
    },
}

#[test]
fn an_older_plan_reader_refuses_the_json_shape_by_name() {
    use ess_cli_contract::wire::Plan;
    let text = launch(JSON_MODEL, JSON_BINDING)
        .unwrap()
        .to_canonical_json();
    let plan: serde_json::Value = serde_json::from_str(&text).unwrap();
    let shape = plan["callables"]["describe"]["result"]["shape"].clone();
    let error = serde_json::from_value::<OlderShape>(shape)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown variant `json`"), "{error}");
    // This reader round-trips it, and stays closed past it.
    let read: Plan = serde_json::from_str(&text).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&read).unwrap()),
        text
    );
    let later = text.replacen("\"kind\": \"json\"", "\"kind\": \"json5\"", 1);
    assert_ne!(later, text);
    let error = serde_json::from_str::<Plan>(&later)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown variant `json5`"), "{error}");
    assert!(error.contains("`json`"), "{error}");
}
