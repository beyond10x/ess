//! Independent pass-one attacks against the public format-six boundary.

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

fn tuple() -> Value {
    json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2})
}

fn read(path: &[&str]) -> Value {
    json!({"op":"read","scope":"input","path":path})
}

fn policy(path: Value) -> Value {
    let mut policy = json!({"kind":"fixed_string_array","length":2,"missing":"preserve","null":"zero","short":"zero_pad","extra":"discard","null_element":"zero"});
    policy["path"] = path;
    policy
}

fn fixture(input: Value, value: Value, policies: Vec<Value>) -> (Bundle, Value) {
    let mut source = json!({"components":{"schemas":{"Output":true}}});
    source["components"]["schemas"]["Input"] = input;
    let bundle = import(
        &source.to_string(),
        &["Input".to_owned(), "Output".to_owned()]
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let mut recipe = json!({"format":"ess-normalization/6","branches":{"run/~":[{"input":Root::pin(&bundle,"Input").unwrap(),"output":Root::pin(&bundle,"Output").unwrap(),"requires":[]}]},"positional_inputs":{}});
    recipe["branches"]["run/~"][0]["value"] = value;
    recipe["positional_inputs"]["run/~"] = Value::Array(policies);
    (bundle, recipe)
}

fn checked(input: Value, value: Value, policies: Vec<Value>) -> Plan {
    let (bundle, recipe) = fixture(input, value, policies);
    Plan::read(&recipe.to_string(), &[bundle]).unwrap()
}

fn finding(pointer: &str, rule: &str, detail: &str) -> Value {
    json!([{"pointer":pointer,"rule":rule,"detail":detail}])
}

fn position(value: Value, index: u64) -> Value {
    let mut expression = json!({"op":"position","index":index});
    expression["value"] = value;
    expression
}

fn adversarial_plan() -> Plan {
    let field = |name| json!({"kind":"field","name":name});
    let record = json!({"type":"object","additionalProperties":false,"properties":{
        "a/~":tuple(),"doc":{"type":"string"},"number":{"type":"number"},
        "rows":{"type":["array","null"],"items":{"type":["object","null"],"additionalProperties":false,"properties":{"a/~":tuple()}}}
    }});
    let source = json!({"components":{"schemas":{"Input":record,"Pair":tuple(),"Output":true}}});
    let bundle = import(
        &source.to_string(),
        &["Input".to_owned(), "Pair".to_owned(), "Output".to_owned()]
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let stage = |root, value| json!({"input":Root::pin(&bundle,root).unwrap(),"output":Root::pin(&bundle,"Output").unwrap(),"requires":[],"value":value});
    let recipe = json!({"format":"ess-normalization/6","branches":{
        "run/~":[stage("Pair", read(&[]))],
        "pick/~":[stage("Pair", position(read(&[]),1))],
        "record":[stage("Input",read(&[]))],
        "optional":[stage("Input",json!({"op":"fallback","value":position(read(&["a/~"]),0),"fallback":{"op":"string","value":"absent"},"on_null":false}))]
    },"positional_inputs":{
        "run/~":[policy(json!([]))],"pick/~":[policy(json!([]))],
        "record":[policy(json!([field("a/~")])),policy(json!([field("rows"),{"kind":"items"},field("a/~")]))],
        "optional":[policy(json!([field("a/~")]))]
    },"raw_json_inputs":{"record":[[field("doc")]]},"binary64_inputs":{"record":[[field("number")]]}});
    Plan::read(&recipe.to_string(), &[bundle]).unwrap()
}

fn independent_cases() -> Vec<Value> {
    let failure = |branch: &str, input: &str, pointer: &str, rule: &str, detail: &str| json!({"branch":branch,"input":input,"errors":finding(pointer,rule,detail)});
    let mut cases = vec![
        json!({"branch":"record","input":"{}","value":{}}),
        json!({"branch":"record","input":r#"{"rows":null}"#,"value":{"rows":null}}),
        json!({"branch":"record","input":r#"{"rows":[null,{}, {"a\/\u007e":[null,"\u0000",{"x":1e999,"\u0078":false}]}]}"#,"value":{"rows":[null,{}, {"a/~":["","\0"]}]}}),
        json!({"branch":"optional","input":"{}","value":"absent"}),
        json!({"branch":"optional","input":r#"{"a/~":null}"#,"value":""}),
        json!({"branch":"pick/~","input":r#"["first","second",1e99999999999999999999999]"#,"value":"second"}),
        json!({"branch":"pick/~","input":"[]","value":""}),
        json!({"branch":"record","input":r#"{"number":-0,"a/~":["a",null,1e999],"doc":{"x":1e999,"x":0}}"#,"value":{"number":-0.0,"a/~":["a",""],"doc":"eyJ4IjoxZTk5OSwieCI6MH0="}}),
        failure(
            "run/~",
            r#"[1e999,null,{"bad":"\ud800"}]"#,
            "/input/0",
            "positional_element_type",
            "fixed_string_array element must be a string or null",
        ),
        failure(
            "run/~",
            r#"["ok",{"bad":"\ud800"}]"#,
            "/input/1",
            "positional_element_type",
            "fixed_string_array element must be a string or null",
        ),
        failure(
            "run/~",
            r#"{"bad":"\ud800"}"#,
            "/input",
            "positional_input_type",
            "fixed_string_array requires an array or null",
        ),
        failure(
            "run/~",
            r#"[false,null,"unterminated]"#,
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ),
        failure(
            "run/~",
            "[false,null] []",
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ),
        failure(
            "unknown",
            "[false,null,]",
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ),
        failure(
            "record",
            r#"{"number":1e999,"a/~":[false]}"#,
            "/input/a~1~0/0",
            "positional_element_type",
            "fixed_string_array element must be a string or null",
        ),
        failure(
            "record",
            r#"{"a/~":[false],"a\/\u007e":null}"#,
            "/input",
            "input_syntax",
            "JSON object keys must be unique",
        ),
        failure(
            "record",
            r#"{"a/~":["ok",null],"rows":[{}, {"a/~":["ok","\ud800"]}]}"#,
            "/input/rows/1/a~1~0/1",
            "input_syntax",
            "positional string contains invalid Unicode",
        ),
    ];
    let deep = format!("{}0{}", "[".repeat(63), "]".repeat(63));
    cases.push(failure(
        "run/~",
        &format!(r#"[null,null,{{"z":{deep},"\ud800":0}}]"#),
        "/input/2",
        "input_depth",
        "JSON input exceeds 64 levels",
    ));
    cases.push(failure(
        "run/~",
        &format!(r#"[null,null,{{"\ud800":0,"z":{deep}}}]"#),
        "/input/2",
        "input_syntax",
        "discarded positional token contains invalid Unicode",
    ));
    cases
}

#[test]
fn literal_cross_boundary_cases_and_repeated_calls_are_atomic() {
    let plan = adversarial_plan();
    let before = plan.to_json();
    let cases = independent_cases();
    for (index, case) in cases.iter().enumerate() {
        let result = plan.run_json(
            case["branch"].as_str().unwrap(),
            case["input"].as_str().unwrap(),
        );
        match result {
            Ok(value) => assert_eq!(case.get("value"), Some(&value), "case {index}: {case}"),
            Err(error) => assert_eq!(
                case.get("errors"),
                Some(&json!(error.0)),
                "case {index}: {case}"
            ),
        }
        assert_eq!(plan.run_json("run/~", "null").unwrap(), json!(["", ""]));
        assert_eq!(plan.to_json(), before);
    }
    let input = json!({});
    assert_eq!(
        json!(plan.run("optional", &input).unwrap_err().0),
        finding(
            "/input",
            "input_positional_provenance",
            "positional input decoding requires original JSON text; use a text input entrypoint"
        )
    );
    assert_eq!(
        json!(plan.run("record", &input).unwrap_err().0),
        finding(
            "/input",
            "input_capture_provenance",
            "raw JSON capture requires original token bytes; use a text input entrypoint"
        )
    );
    assert_eq!(input, json!({}));
    let negative = plan.run_json("record", r#"{"number":-0}"#).unwrap();
    assert_eq!(negative["number"].as_f64().unwrap().to_bits(), 1_u64 << 63);
    eprintln!("independent reference: {} literal vectors, repeated fresh-state controls, two value-API refusals, signed-zero bits",cases.len());
}

#[test]
fn source_union_proofs_survive_computed_collection_and_nested_position() {
    let alternative = json!({"anyOf":[tuple(),tuple()]});
    let schemas_and_values = [
        (
            json!({"type":"array","items":alternative}),
            json!({"op":"find","list":read(&[]),"condition":{"op":"all","conditions":[]},"value":position(json!({"op":"read","scope":"item","path":[]}),0),"otherwise":{"op":"string","value":""}}),
        ),
        (
            json!({"type":"array","prefixItems":[alternative, {"type":"boolean"}],"items":false,"minItems":2,"maxItems":2}),
            position(position(read(&[]), 0), 0),
        ),
        (
            json!({"anyOf":[{"type":"null"},{"type":"object","properties":{"pair":tuple()}},{"type":"object","properties":{"pair":tuple()}}]}),
            json!({"op":"null"}),
        ),
    ];
    for (index, (schema, value)) in schemas_and_values.into_iter().enumerate() {
        let policies = if index == 2 {
            vec![policy(json!([{"kind":"field","name":"pair"}]))]
        } else {
            vec![]
        };
        let (bundle, recipe) = fixture(schema, value, policies);
        let refused = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
        let rule = if index == 2 {
            "positional_schema"
        } else {
            "position_type"
        };
        assert!(refused.0.iter().any(|e| e.rule == rule), "{refused:?}");
    }
    // A tuple inside one exact heterogeneous tuple has a distinct statically
    // proved arity at each position expression and remains an admitted value read.
    let schema = json!({"type":"array","prefixItems":[tuple(), {"type":"boolean"}],"items":false,"minItems":2,"maxItems":2});
    let plan = checked(schema, position(position(read(&[]), 0), 1), vec![]);
    assert_eq!(
        plan.run("run/~", &json!([["x", "y"], true])).unwrap(),
        json!("y")
    );
}

#[test]
fn source_refinements_and_stage_identity_remain_checked() {
    let mut refined = tuple();
    refined["uniqueItems"] = json!(true);
    let plan = checked(refined, read(&[]), vec![policy(json!([]))]);
    assert_eq!(
        plan.run_json("run/~", "[]").unwrap_err().0[0].rule,
        "schema_validation"
    );
    assert_eq!(
        plan.run_json("run/~", r#"["a","b","a"]"#).unwrap(),
        json!(["a", "b"])
    );
    let (bundle, mut recipe) = fixture(
        tuple(),
        json!({"op":"list","items":[{"op":"string","value":"one"}]}),
        vec![policy(json!([]))],
    );
    recipe["branches"]["run/~"].as_array_mut().unwrap().push(json!({"input":Root::pin(&bundle,"Input").unwrap(),"output":Root::pin(&bundle,"Output").unwrap(),"requires":[],"value":position(read(&[]),0)}));
    let error = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
    assert_eq!(
        json!(error.0),
        finding(
            "/branches/run~1~0/1/input",
            "stage_identity",
            "input identity differs from the previous output"
        )
    );
}

fn write_generated(root: &std::path::Path, files: std::collections::BTreeMap<String, String>) {
    for (name, source) in files {
        let path = root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
}

fn command_output(root: &std::path::Path, label: &str, command: &mut std::process::Command) {
    let output = command.output().unwrap();
    let log = format!(
        "command: {command:?}\nexit: {}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join(format!("{label}.log")), &log).unwrap();
    assert!(output.status.success(), "{log}");
    eprintln!("{log}");
}

#[test]
fn native_rust_preserves_independent_token_cases_and_checked_metadata() {
    use std::{fs, path::Path, process::Command};
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("positional-adversary-rust-{}", std::process::id()));
    let target = adversarial_plan().rust("positional_adversary").unwrap();
    assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
    write_generated(&root, target.files);
    fs::write(
        root.join("cases.json"),
        json!(independent_cases()).to_string(),
    )
    .unwrap();
    let path = root.join("src/lib.rs");
    let mut source = fs::read_to_string(&path).unwrap();
    source.push_str(include_str!("fixtures/positional_adversary_rust.rs.txt"));
    fs::write(path, source).unwrap();
    command_output(
        &root,
        "lock",
        Command::new(env!("CARGO"))
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(root.join("Cargo.toml")),
    );
    for feature in [None, Some("serde_json/arbitrary_precision")] {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(["test", "--offline", "--locked", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
            );
        if let Some(feature) = feature {
            command.args(["--features", feature]);
        }
        command.args(["--", "--nocapture", "--test-threads=1"]);
        command_output(
            &root,
            if feature.is_some() {
                "arbitrary"
            } else {
                "default"
            },
            &mut command,
        );
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_preserves_independent_token_cases_and_checked_metadata() {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
    };
    let compiler = PathBuf::from(
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER"),
    );
    assert!(compiler.is_absolute() && compiler.is_file());
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("positional-adversary-go-{}", std::process::id()));
    let target = adversarial_plan()
        .go(
            "positional_adversary",
            "example.invalid/positional-adversary",
        )
        .unwrap();
    assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
    write_generated(&root, target.files);
    fs::write(
        root.join("cases.json"),
        json!(independent_cases()).to_string(),
    )
    .unwrap();
    fs::write(
        root.join("adversary_test.go"),
        include_str!("fixtures/positional_adversary_go.go.txt"),
    )
    .unwrap();
    let mut command = Command::new(compiler);
    command
        .args([
            "test",
            "-v",
            "-count=1",
            "-race",
            "-mod=readonly",
            "-p",
            "1",
            "./...",
        ])
        .current_dir(&root)
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOTOOLCHAIN", "local")
        .env("GOFLAGS", "")
        .env("GOMAXPROCS", "4")
        .env(
            "GOCACHE",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("positional-go-cache"),
        );
    command_output(&root, "native-go", &mut command);
}

#[test]
fn discarded_source_order_depth_precedes_later_invalid_key() {
    let plan = checked(tuple(), read(&[]), vec![policy(json!([]))]);
    // Root tuple depth 0, discarded object depth 1, arrays at depths 2..64,
    // then its numeric leaf at depth 65. A later malformed key must not win.
    let deep = format!("{}0{}", "[".repeat(63), "]".repeat(63));
    let input = format!(r#"[null,null,{{"z":{deep},"\ud800":0}}]"#);
    assert_eq!(
        json!(plan.run_json("run/~", &input).unwrap_err().0),
        finding("/input/2", "input_depth", "JSON input exceeds 64 levels")
    );
    let reversed = format!(r#"[null,null,{{"\ud800":0,"z":{deep}}}]"#);
    assert_eq!(
        json!(plan.run_json("run/~", &reversed).unwrap_err().0),
        finding(
            "/input/2",
            "input_syntax",
            "discarded positional token contains invalid Unicode"
        )
    );
    // Exactly one fewer array admits the depth-64 leaf, including duplicate
    // escape-equivalent keys and an exponent that no numeric decoder can use.
    let limit = format!(
        "{}1e999999999999999999999999{}",
        "[".repeat(62),
        "]".repeat(62)
    );
    let input = format!(r#"["\u0000",null,{{"z":{limit},"z":0,"\u007a":true}}]"#);
    assert_eq!(plan.run_json("run/~", &input).unwrap(), json!(["\0", ""]));
}
