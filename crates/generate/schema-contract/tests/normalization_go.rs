//! Accounted Go target generation and an explicit native execution lane.

#[path = "fixtures/normalization_base64.rs"]
mod fixture_base64;
#[path = "fixtures/normalization_model.rs"]
mod fixture_model;
#[path = "fixtures/normalization_numeric.rs"]
mod fixture_numeric;
#[path = "fixtures/normalization_v1.rs"]
mod fixture_v1;
#[path = "fixtures/normalization_v2.rs"]
mod fixture_v2;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn digest(text: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for byte in Sha256::digest(text) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn plans() -> Vec<(Plan, Value)> {
    let first = fixture_v1::plan();
    let mut old_cases = fixture_v1::cases(&first).as_array().unwrap().clone();
    for input in [
        r#""\ud800""#,
        r#"{"key":"\ud800"}"#,
        r#"{"\ud800":0}"#,
        r#"{"a":0.10000000000000001,"z":1,"z":2}"#,
    ] {
        let errors = first.run_json("copy", input).unwrap_err().0;
        old_cases.push(json!({"branch":"copy","input":input,"errors":errors}));
    }
    for input in [
        "{}",
        r#"{"seconds":"bad","address":false,"items":[{}, {"key":3,"enabled":null}],"nested":{},"extra":1}"#,
        r#"{"seconds":1,"address":"x","items":[],"nested":{"other":1}}"#,
    ] {
        let errors = first.run_json("primary", input).unwrap_err().0;
        old_cases.push(json!({"branch":"primary","input":input,"errors":errors}));
    }
    let (bundle, recipe) = fixture_v2::fixture();
    let second = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut ordered = fixture_v2::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    let mut selected: Value = serde_json::from_str(fixture_v2::INPUT).unwrap();
    selected["items"][1]["enabled"] = json!(true);
    let input = selected.to_string();
    assert_eq!(second.run_json("find", &input).unwrap(), json!(3));
    ordered.push(json!({"branch":"find","input":input,"value":3}));
    let errors = second.run_json("select", &input).unwrap_err().0;
    ordered.push(json!({"branch":"select","input":input,"errors":errors}));
    let (bundle, recipe) = fixture_numeric::fixture();
    let third = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut numeric = fixture_numeric::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    for (branch, input) in [
        ("truncate", "9223372036854775807"),
        ("truncate", "9223372036854775808"),
        ("truncate", "-9223372036854777856"),
        ("truncate", "1e999"),
        ("overflow", "2"),
        ("copy", r#"{"exact":0.10000000000000001}"#),
        ("copy", r#"{"value":1,"value":2}"#),
    ] {
        let errors = third.run_json(branch, input).unwrap_err().0;
        numeric.push(json!({"branch":branch,"input":input,"errors":errors}));
    }
    vec![
        (first, Value::Array(old_cases)),
        (second, Value::Array(ordered)),
        (third, Value::Array(numeric)),
        model_aliases(),
        fixture_base64::fixture(),
    ]
}

fn model_aliases() -> (Plan, Value) {
    let (model, mut recipe) = fixture_model::fixture();
    let expanded = fixture_model::selection(
        fixture_model::SOURCE,
        &[
            "sample.settings.Decoded",
            "sample.settings.Runtime",
            "sample.settings.Id",
        ],
    );
    assert_eq!(model.to_json(), expanded.to_json());
    let mut alias = recipe["branches"]["primary"].clone();
    alias[0]["input"] = json!(schema_contract::realize::normalize::Root::pin_model(
        &expanded,
        "sample.settings.Decoded"
    )
    .unwrap());
    alias[0]["output"] = json!(schema_contract::realize::normalize::Root::pin_model(
        &expanded,
        "sample.settings.Runtime"
    )
    .unwrap());
    recipe["branches"]["alias"] = alias;
    let model_plan = Plan::check_with_models(
        serde_json::from_value(recipe).unwrap(),
        &[],
        &[model, expanded],
    )
    .unwrap();
    let mut model_cases = fixture_model::cases().as_array().unwrap().clone();
    let alias_cases = model_cases
        .iter()
        .cloned()
        .map(|mut case| {
            case["branch"] = json!("alias");
            if case.get("errors").is_some() {
                case["errors"] = json!(
                    model_plan
                        .run_json("alias", case["input"].as_str().unwrap())
                        .unwrap_err()
                        .0
                );
            }
            case
        })
        .collect::<Vec<_>>();
    model_cases.extend(alias_cases);
    (model_plan, json!(model_cases))
}

#[test]
fn go_target_retains_source_identity_and_accounts_for_every_file() {
    for (plan, cases) in plans() {
        assert!(!cases.as_array().unwrap().is_empty());
        let result = plan
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap();
        assert_eq!(
            result,
            plan.go(
                "normalization_adapter",
                "example.invalid/normalization-adapter"
            )
            .unwrap()
        );
        let report = serde_json::to_value(&result.report).unwrap();
        let recipe: Value = serde_json::from_str(&plan.to_json()).unwrap();
        assert_eq!(
            report["format"],
            if recipe["format"] == "ess-normalization/3" {
                "ess-normalization-target/2"
            } else {
                "ess-normalization-target/1"
            }
        );
        assert_eq!(report["configuration"]["package"], "normalization_adapter");
        assert_eq!(report["recipe_digest"], digest(&plan.to_json()));
        for (path, expected_digest) in report["files"].as_object().unwrap() {
            assert_eq!(expected_digest, &digest(&result.files[path]));
        }
        assert_eq!(
            report["files"].as_object().unwrap().len() + 1,
            result.files.len()
        );
        assert_eq!(result.files["source.recipe.json"], plan.to_json());
        assert!(result.files["go.mod"].contains("jsonschema/v6 v6.0.2"));
        assert!(result.files["go.sum"].contains("nxP4pPoyqOAgX8lYDFCfl3DyKeXErCvSvhcyzwGV9CE="));
    }
}

#[test]
fn go_target_refuses_invalid_library_identity() {
    let plan = fixture_v1::plan();
    for (package, module) in [
        ("main", "example.invalid/test"),
        ("type", "example.invalid/test"),
        ("valid", "../escape"),
        ("quote\"", "example.invalid/test"),
    ] {
        assert_eq!(
            plan.go(package, module).unwrap_err().0[0].rule,
            "go_configuration"
        );
    }
}

#[test]
fn go_target_refuses_referenced_pattern_semantics_before_emitting_files() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let source = json!({"components":{"schemas":{
        "Input":{"type":"object","properties":{"value":{"$ref":"#/components/schemas/Text"}}},
        "Text":{"type":"string","pattern":"^(?=a)a$"},
        "Plain":{"type":"string"}
    }}});
    let roots = ["Input", "Plain"].map(str::to_owned).into_iter().collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let make = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        Plan::read(&json!({"format":"ess-normalization/1","branches":{"copy":[{
            "input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}
        }]}}).to_string(), std::slice::from_ref(&bundle)).unwrap()
    };
    let plan = make("Input");
    let errors = plan.go("adapter", "example.invalid/adapter").unwrap_err().0;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "go_schema_pattern");
    assert!(errors[0].pointer.starts_with("/bundles/"));
    assert!(errors[0].pointer.ends_with("/Text/pattern"));
    assert!(plan.run_json("copy", r#"{"value":"a"}"#).is_ok());
    assert!(plan.run_json("copy", r#"{"value":"b"}"#).is_err());
    // An unselected schema in the same retained bundle cannot widen the refusal.
    assert!(make("Plain")
        .go("adapter", "example.invalid/adapter")
        .is_ok());
}

#[test]
fn unqualified_syntax_refuses_even_when_the_reference_matches() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let equivalent_base64 = format!("(?:{})", fixture_base64::PATTERN);
    for (pattern, input) in [
        ("^a$", "a"),
        ("", "anything"),
        ("^(?=a)a$", "a"),
        ("(?<=a)b", "ab"),
        (r"^(a)\1$", "aa"),
        (r"^\w+$", "abc"),
        (r"^\p{L}+$", "letters"),
        ("^.$", "x"),
        ("^(a+)+$", "aaaa"),
        ("[A-Za-z0-9+/]{4}", "AAAA"),
        (equivalent_base64.as_str(), "AAAA"),
    ] {
        let source = json!({"components":{"schemas":{"Text":{"type":"string","pattern":pattern}}}});
        let bundle = import(
            &source.to_string(),
            &["Text".to_owned()].into_iter().collect(),
            Dialect::Draft202012,
        )
        .unwrap();
        let root = Root::pin(&bundle, "Text").unwrap();
        let plan = Plan::read(&json!({"format":"ess-normalization/1","branches":{"copy":[{
            "input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}
        }]}}).to_string(), &[bundle]).unwrap();
        assert_eq!(plan.run("copy", &json!(input)).unwrap(), input, "{pattern}");
        let errors = plan.go("adapter", "example.invalid/adapter").unwrap_err().0;
        assert_eq!(errors.len(), 1, "{pattern}");
        assert_eq!(errors[0].rule, "go_schema_pattern", "{pattern}");
        assert!(errors[0].pointer.ends_with("/Text/pattern"));
    }
}

#[test]
fn qualified_patterns_do_not_hide_other_obligations_or_inspect_annotation_data() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let source = json!({"components":{"schemas":{
        "Encoded":{"type":"string","pattern":fixture_base64::PATTERN},
        "Input":{"type":"object","properties":{"data":{"$ref":"#/components/schemas/Encoded"},"pattern":{"type":"string"}},
            "default":{"pattern":"^(a+)+$"},"examples":[{"pattern":"(?=a)"}]},
        "Mixed":{"type":"object","properties":{"data":{"$ref":"#/components/schemas/Encoded"},"other":{"type":"string","pattern":"^a$"}}}
    }}});
    let bundle = import(
        &source.to_string(),
        &["Input".to_owned(), "Mixed".to_owned()]
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let make = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        Plan::read(&json!({"format":"ess-normalization/1","branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}}).to_string(), std::slice::from_ref(&bundle)).unwrap()
    };
    assert!(make("Input")
        .go("adapter", "example.invalid/adapter")
        .is_ok());
    let errors = make("Mixed")
        .go("adapter", "example.invalid/adapter")
        .unwrap_err()
        .0;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "go_schema_pattern");
    assert!(errors[0]
        .pointer
        .ends_with("/Mixed/properties/other/pattern"));
}

#[test]
fn every_primitive_map_key_pattern_is_qualified_at_its_nested_source_pointer() {
    use ess_domain::types::Primitive;
    use schema_contract::realize::normalize::Root;

    for key in Primitive::ALL {
        if *key == Primitive::Binary64 {
            assert!(ess_domain::TypeRef::parse("Map<Binary64, Bytes>").is_err());
            assert!(!ess_domain::TypeRegistry::new()
                .resolve(
                    &ess_domain::TypeRef::Map(
                        *key,
                        Box::new(ess_domain::TypeRef::Primitive(Primitive::Bytes))
                    ),
                    "map"
                )
                .is_empty());
            continue;
        }
        let source = format!(
            "format: ess/1\nsystem: sample\nversion: v1\ndomains: [sample.maps]\ndomain: sample.maps\ntypes:\n  - name: sample.maps.Encoded\n    kind: struct\n    fields:\n      - name: values\n        wire: encoded/~\n        type: List<Optional<Map<{key}, Bytes>>>\n"
        );
        let model = fixture_model::selection(&source, &["sample.maps.Encoded"]);
        let map_pointer = "/properties/encoded~1~0/items/anyOf/0";
        let map = model.definitions()["sample.maps.Encoded"]
            .pointer(map_pointer)
            .unwrap();
        let expected_key_keywords: &[&str] = match key {
            Primitive::Binary64 => unreachable!("map-key refusal checked above"),
            Primitive::String => &[],
            Primitive::Boolean => &["enum", "type"],
            Primitive::Integer => &["pattern", "type"],
            Primitive::Decimal | Primitive::Uuid => &["format", "pattern", "type"],
            Primitive::Timestamp | Primitive::Duration => &["format", "type"],
            Primitive::Bytes => &["contentEncoding", "pattern", "type"],
        };
        assert_eq!(
            map.get("propertyNames")
                .and_then(Value::as_object)
                .into_iter()
                .flat_map(|object| object.keys().map(String::as_str))
                .collect::<Vec<_>>(),
            expected_key_keywords,
            "{key}"
        );
        // Pattern accounting must stay private: structural reports already account for
        // key validation with model_map_keys, without a second nested obligation.
        let structural = schema_contract::realize::Plan::from_model(&model).unwrap();
        let report = serde_json::to_value(structural.typescript().report).unwrap();
        let obligations = report["obligations"].as_array().unwrap();
        // TypeScript also reports its compiler options and this root's closed object.
        assert_eq!(
            obligations.len(),
            if *key == Primitive::String { 3 } else { 4 },
            "{key}: {report}"
        );
        assert!(obligations.iter().all(|finding| {
            finding["rule"] == "typescript_compiler_options"
                || finding["rule"] == "closed_object"
                || finding["rule"] == "model_map_keys"
                || (finding["rule"] == "pattern"
                    && finding["pointer"]
                        .as_str()
                        .unwrap()
                        .ends_with("/additionalProperties/pattern"))
        }));

        let root = Root::pin_model(&model, "sample.maps.Encoded").unwrap();
        let recipe = json!({"format":"ess-normalization/3","branches":{"copy":[{
            "input":root,"output":root,"requires":[],
            "value":{"op":"read","scope":"input","path":[]}
        }]}});
        let prefix = format!(
            "/models/{}/$defs/sample.maps.Encoded{map_pointer}",
            digest(&model.to_json())
        );
        let plan = Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model])
            .unwrap();
        let result = plan.go("adapter", "example.invalid/adapter");
        match key {
            Primitive::Binary64 => unreachable!("map-key refusal checked above"),
            Primitive::Integer | Primitive::Decimal | Primitive::Uuid => {
                let refused = result.expect_err("unqualified key pattern must refuse");
                assert_eq!(refused.0.len(), 1, "{key}");
                assert_eq!(refused.0[0].rule, "go_schema_pattern", "{key}");
                assert_eq!(
                    refused.0[0].pointer,
                    format!("{prefix}/propertyNames/pattern"),
                    "{key}"
                );
            }
            Primitive::String
            | Primitive::Boolean
            | Primitive::Timestamp
            | Primitive::Duration
            | Primitive::Bytes => {
                assert!(result.is_ok(), "{key}: {result:?}");
            }
        }
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_executes_old_ordered_and_numeric_recipes() {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let compiler = PathBuf::from(
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER"),
    );
    assert!(compiler.is_absolute() && compiler.is_file());
    let version = Command::new(&compiler)
        .arg("version")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout)
            .split_whitespace()
            .nth(2)
            .and_then(|version| version.split('-').next()),
        Some("go1.26.5")
    );
    for (index, (plan, cases)) in plans().into_iter().enumerate() {
        let generated = plan
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap();
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("normalization-go-{}-{index}", std::process::id()));
        for (path, source) in generated.files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        fs::write(root.join("cases.json"), cases.to_string()).unwrap();
        fs::write(
            root.join("behavior_test.go"),
            include_str!("fixtures/normalization_go_tests.go.txt"),
        )
        .unwrap();
        let output = Command::new(&compiler)
            .args(["test", "-count=1", "-race", "-mod=readonly", "./..."])
            .current_dir(&root)
            .env("GOPROXY", "off")
            .env("GOSUMDB", "off")
            .env("GOTOOLCHAIN", "local")
            .env("GOFLAGS", "")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fixture {index}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
