//! Adversarial format-4 cases derived from the binding lexical-capture contract.

#[allow(dead_code)]
#[path = "fixtures/normalization_raw.rs"]
mod fixture;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    let mut good = |branch: &str, input: String, value: Value| {
        cases.push(json!({"branch":branch,"input":input,"value":value}));
    };
    for token in [
        r#"{"format":"ess-normalization/999","branches":{"root":[]},"raw_json_inputs":{"root":[[]]}}"#.to_owned(),
        r#"{"\u0000":"\"\\\/\b\f\n\r\t","\u0000":-0e+000}"#.to_owned(),
        format!("[0e{},-0e-{},1E+{}]", "9".repeat(1000), "9".repeat(1000), "9".repeat(1000)),
        r#"{ "é": "e\u0301", "é":"\u00e9" }"#.to_owned(),
    ] {
        let retained = STANDARD.encode(token.as_bytes());
        good("root", format!("\r\n\t {token} \r\n"), json!(retained));
        good("record", format!(r#"{{"\u0070ay\u006coad" : {token} ,"items":[{token},null]}}"#), json!({"payload":retained,"items":[retained,"bnVsbA=="]}));
    }
    let object64 = format!("{}null{}", r#"{"": "#.repeat(64), "}".repeat(64));
    good("root", object64.clone(), json!(STANDARD.encode(&object64)));
    let mut bad = |branch: &str, input: String, at: &str, rule: &str, detail: &str| {
        cases.push(json!({"branch":branch,"input":input,"errors":[{"pointer":at,"rule":rule,"detail":detail}]}));
    };
    for depth in [64, 65] {
        for leaf in [r#""\ud800""#, r#"{"\udc00":0}"#] {
            let token = format!("{}{leaf}{}", "[".repeat(depth), "]".repeat(depth));
            let (rule, detail) = if depth == 64 {
                (
                    "input_syntax",
                    "captured JSON token contains invalid Unicode",
                )
            } else {
                ("input_depth", "JSON input exceeds 64 levels")
            };
            bad("root", token, "/input", rule, detail);
        }
    }
    let deep = format!("{}null{}", r#"{"": "#.repeat(65), "}".repeat(65));
    bad(
        "root",
        deep.clone(),
        "/input",
        "input_depth",
        "JSON input exceeds 64 levels",
    );
    for input in [
        format!("{deep} []"),
        format!(r#"{{"payload":{deep},"last":"\uZZZZ"}}"#),
    ] {
        bad(
            "record",
            input,
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        );
    }
    bad(
        "record",
        format!(r#"{{"payload":{deep},"\u0070ayload":null}}"#),
        "/input",
        "input_syntax",
        "JSON object keys must be unique",
    );
    bad(
        "record",
        r#"{"payload":"\ud800","\ue000":1e999,"\ud800\udc00":"\ud800"}"#.to_owned(),
        "/input/payload",
        "input_syntax",
        "captured JSON token contains invalid Unicode",
    );
    bad(
        "plain",
        r#"{"\ud800\udc00":"\ud800","\ue000":1e999}"#.to_owned(),
        "/input/\u{e000}",
        "input_number",
        "JSON number is outside the supported representation",
    );
    for (input, rule, detail) in [
        (
            format!(r#"{{"z":{deep},"a":"\ud800"}}"#),
            "input_depth",
            "JSON input exceeds 64 levels",
        ),
        (
            format!(r#"{{"z":"\ud800","a":{deep}}}"#),
            "input_syntax",
            "captured JSON token contains invalid Unicode",
        ),
    ] {
        bad("root", input, "/input", rule, detail);
    }
    // Helpers must retain exactly the same error priority, including unknown dispatch.
    for case in cases.clone() {
        let mut encoded = case;
        encoded["input"] = json!(STANDARD.encode(encoded["input"].as_str().unwrap()));
        encoded["mode"] = json!("base64");
        cases.push(encoded);
    }
    cases
}

#[test]
fn captured_lexemes_and_mixed_error_order_obey_the_document() {
    let plan = fixture::plan();
    let cases = cases();
    for (index, case) in cases.iter().enumerate() {
        let branch = case["branch"].as_str().unwrap();
        let input = case["input"].as_str().unwrap();
        let result = if case["mode"] == "base64" {
            plan.run_base64_json(branch, input)
        } else {
            plan.run_json(branch, input)
        };
        match result {
            Ok(value) => assert_eq!(Some(&value), case.get("value"), "case {index}: {case}"),
            Err(error) => assert_eq!(
                Some(json!(error.0)),
                case.get("errors").cloned(),
                "case {index}: {case}"
            ),
        }
    }
    eprintln!("adversary lexical cases executed {}", cases.len());
}

#[test]
fn selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order() {
    let source = json!({"components":{"schemas":{"Record":{"type":"object","additionalProperties":{"type":"string"},"properties":{
        "":{"type":["string","null"]}, "a/b~":{"type":"array","items":{"type":"string"}}, "n":{"type":"number"}
    }}}}});
    let bundle = import(
        &source.to_string(),
        &["Record".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let root = Root::pin(&bundle, "Record").unwrap();
    let mut recipe = json!({"format":"ess-normalization/4","branches":{"a/b~":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]},"raw_json_inputs":{"a/b~":[[fixture::field("")],[fixture::field("a/b~"),{"kind":"items"}]]},"binary64_inputs":{"a/b~":[[fixture::field("n")]]}});
    let plan = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap();
    assert_eq!(
        plan.run_json(
            "a/b~",
            r#"{"":null,"a\/b\u007e":[0e999],"n":0.10000000000000001}"#
        )
        .unwrap(),
        json!({"":"bnVsbA==","a/b~":["MGU5OTk="],"n":0.1})
    );
    recipe["raw_json_inputs"]["a/b~"] = json!([[fixture::field("open")]]);
    let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
    assert_eq!(
        json!(errors.0),
        json!([{"pointer":"/raw_json_inputs/a~1b~0/0/0","rule":"unknown_field","detail":"\"open\" is not a declared object member"}])
    );
    recipe["raw_json_inputs"]["a/b~"] = json!([[fixture::field("a/b~"),{"kind":"items"}],[fixture::field("a/b~")],[fixture::field("n")],[fixture::field("n")]]);
    let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
    assert_eq!(
        json!(errors.0),
        json!([
            {"pointer":"/raw_json_inputs/a~1b~0/1","rule":"overlapping_capture_path","detail":"capture path overlaps /raw_json_inputs/a~1b~0/0"},
            {"pointer":"/raw_json_inputs/a~1b~0/2","rule":"input_policy_overlap","detail":"capture path overlaps /binary64_inputs/a~1b~0/0"},
            {"pointer":"/raw_json_inputs/a~1b~0/3","rule":"duplicate_capture_path","detail":"capture path duplicates /raw_json_inputs/a~1b~0/2"}
        ])
    );
    // Two wire spellings of the same decoded selector name must collide.
    let text = recipe
        .to_string()
        .replace("\"name\":\"n\"", "\"name\":\"\\u006e\"");
    assert_eq!(Plan::read(&text, &[bundle]).unwrap_err(), errors);
}

fn write_target(root: &Path, files: BTreeMap<String, String>) {
    for (path, source) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}

#[test]
fn generated_rust_runs_adversarial_lexical_cases() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-raw-adversary-rust");
    write_target(
        &root,
        fixture::plan().rust("normalization_adapter").unwrap().files,
    );
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("tests/cases.json"), json!(cases()).to_string()).unwrap();
    fs::write(
        root.join("tests/behavior.rs"),
        include_str!("fixtures/normalization_raw_rust_tests.rs.txt"),
    )
    .unwrap();
    for features in [None, Some("serde_json/arbitrary_precision")] {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
            );
        if let Some(features) = features {
            command.args(["--features", features]);
        }
        let output = command.output().unwrap();
        eprintln!(
            "native Rust {features:?}; cases {}; exit {}\n{}\n{}",
            cases().len(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success());
    }
}

#[test]
#[cfg(feature = "go-typecheck")]
fn generated_go_runs_adversarial_lexical_cases() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-raw-adversary-go");
    write_target(
        &root,
        fixture::plan()
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap()
            .files,
    );
    fs::write(root.join("cases.json"), json!(cases()).to_string()).unwrap();
    fs::write(
        root.join("behavior_test.go"),
        include_str!("fixtures/normalization_raw_go_tests.go.txt"),
    )
    .unwrap();
    let output = Command::new(
        std::env::var_os("ESS_GO_COMPILER").expect("set the leased native Go compiler"),
    )
    .args(["test", "-v", "-count=1", "-race", "-mod=readonly", "./..."])
    .current_dir(&root)
    .env("GOPROXY", "off")
    .env("GOSUMDB", "off")
    .env("GOTOOLCHAIN", "local")
    .env("GOFLAGS", "")
    .output()
    .unwrap();
    eprintln!(
        "native Go; cases {}; exit {}\n{}\n{}",
        cases().len(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success());
}
