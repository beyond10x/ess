//! Complete shared text/helper corpus, with original constructors kept read-only.
#![allow(dead_code, clippy::duplicate_mod)]

#[path = "normalization_base64.rs"]
mod fixture_base64;
#[path = "normalization_model.rs"]
mod fixture_model;
#[path = "normalization_numeric.rs"]
mod fixture_numeric;
#[path = "normalization_v1.rs"]
mod fixture_v1;
#[path = "normalization_v2.rs"]
mod fixture_v2;

#[path = "normalization_binary64.rs"]
pub mod binary64;
#[path = "normalization_positional.rs"]
pub mod positional;
#[path = "normalization_raw.rs"]
pub mod raw;
use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
fn legacy_plans() -> Vec<(Plan, Value)> {
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

mod raw_adversary {
    // Copied unchanged from normalization_raw_adversary.rs at a2f02fc; SHA256 4d9ef5c61c5c0c99de478d8b6dd05cfad82c4673e3b3c1cd7b2cae42bace1b3c.
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde_json::{json, Value};
    pub(super) fn cases() -> Vec<Value> {
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
}

mod positional_adversary {
    // Pinned copy from normalization_positional_adversary.rs at a2f02fc; SHA256 5f58f52bfbfe2783ab60acbaeaa2218eec8d48e12a89bbbe4710b21a8b5adf2a.
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

    pub(super) fn adversarial_plan() -> Plan {
        let field = |name| json!({"kind":"field","name":name});
        let record = json!({"type":"object","additionalProperties":false,"properties":{
            "a/~":tuple(),"doc":{"type":"string"},"number":{"type":"number"},
            "rows":{"type":["array","null"],"items":{"type":["object","null"],"additionalProperties":false,"properties":{"a/~":tuple()}}}
        }});
        let source =
            json!({"components":{"schemas":{"Input":record,"Pair":tuple(),"Output":true}}});
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

    pub(super) fn independent_cases() -> Vec<Value> {
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
}

pub fn corpora() -> Vec<(&'static str, Plan, Vec<Value>)> {
    let mut all = legacy_plans()
        .into_iter()
        .zip(["v1", "v2", "numeric", "model", "base64"])
        .map(|((plan, cases), name)| (name, plan, cases.as_array().unwrap().clone()))
        .collect::<Vec<_>>();
    // Explicitly inapplicable: raw's six capture-provenance value calls and its
    // plain value control; positional's four position-provenance value calls,
    // record capture-provenance value call and plain value control. No other filter.
    let text_cases = |cases: Vec<Value>, expected_value_cases| {
        assert_eq!(
            cases.iter().filter(|case| case["mode"] == "value").count(),
            expected_value_cases
        );
        cases
            .into_iter()
            .filter(|case| case["mode"] != "value")
            .collect()
    };
    all.push(("raw", raw::plan(), text_cases(raw::cases(), 7)));
    all.push(("raw-adversary", raw::plan(), raw_adversary::cases()));
    all.push(("binary64-v5", binary64::plan(), binary64::cases()));
    all.push((
        "positional",
        positional::plan(),
        text_cases(positional::cases(), 6),
    ));
    all.push(("mixed", positional::mixed_plan(), positional::mixed_cases()));
    let (model, mut recipe) = binary64::fixture();
    recipe["format"] = json!("ess-normalization/6");
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap();
    all.push(("binary64-v6", plan, binary64::cases()));
    all.push((
        "positional-adversary",
        positional_adversary::adversarial_plan(),
        positional_adversary::independent_cases(),
    ));
    assert_eq!(
        all.iter()
            .map(|(_, _, cases)| cases.len())
            .collect::<Vec<_>>(),
        [40, 14, 35, 66, 2490, 149, 42, 70, 93, 3, 70, 19]
    );
    assert_eq!(
        all.iter().map(|(_, _, cases)| cases.len()).sum::<usize>(),
        3091
    );
    all
}

/// Independent keyword and multiplicity expectations, outside the shared 3,091.
pub fn schema_controls() -> (Plan, Vec<Value>) {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let schemas = json!({
        "Any":true,
        "minimum":{"type":"number","minimum":9_007_199_254_740_993_u64},
        "maximum":{"type":"number","maximum":9_007_199_254_740_992.0},
        "exclusive":{"type":"number","exclusiveMinimum":-1,"exclusiveMaximum":9_007_199_254_740_993_u64},
        "constant":{"const":9_007_199_254_740_993_u64},
        "enum":{"enum":[0,9_007_199_254_740_993_u64]},
        "deep":{"enum":[{"a/~":[9_007_199_254_740_993_u64,-0.0]}]},
        "required":{"type":"object","required":["a","b"],"properties":{"a":{"type":"integer"},"b":{"type":"string"}},"additionalProperties":false},
        "any":{"anyOf":[{"type":"number","minimum":1},{"type":"string","minLength":2}]},
        "one":{"oneOf":[{"type":"number"},{"type":"integer"}]},
        "tuple":{"type":"array","prefixItems":[{"type":"string","minLength":1},{"type":"integer"}],"items":false,"minItems":2,"maxItems":2},
        "list":{"type":"array","items":{"type":"string"},"minItems":2,"maxItems":2},
        "unicode":{"type":"string","minLength":2,"maxLength":2}
    });
    let roots = schemas.as_object().unwrap().keys().cloned().collect();
    let bundle = import(
        &json!({"components":{"schemas":schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    let output = Root::pin(&bundle, "Any").unwrap();
    let mut branches = serde_json::Map::new();
    for name in schemas.as_object().unwrap().keys() {
        if name == "Any" {
            continue;
        }
        branches.insert(name.clone(), json!([{"input":Root::pin(&bundle,name).unwrap(),"output":output,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]));
    }
    let plan = Plan::read(
        &json!({"format":"ess-normalization/6","branches":branches}).to_string(),
        &[bundle],
    )
    .unwrap();
    let mut cases = Vec::new();
    for (branch, input, pointers) in [
        ("minimum", "9007199254740992.0", vec![""]),
        ("minimum", "9007199254740993", vec![]),
        ("maximum", "9007199254740993", vec![""]),
        ("maximum", "9007199254740992.0", vec![]),
        ("exclusive", "-1.0", vec![""]),
        ("exclusive", "9007199254740993", vec![""]),
        ("exclusive", "9007199254740992.0", vec![]),
        ("constant", "9007199254740992.0", vec![""]),
        ("constant", "9007199254740993", vec![]),
        ("enum", "-0.0", vec![]),
        ("enum", "9007199254740992.0", vec![""]),
        ("enum", "9007199254740993", vec![]),
        ("deep", r#"{"a/~":[9007199254740993,0]}"#, vec![]),
        ("deep", r#"{"a/~":[9007199254740992.0,0]}"#, vec![""]),
        ("required", "{}", vec!["", ""]),
        ("required", r#"{"x":0,"y":1}"#, vec!["", "", ""]),
        ("required", r#"{"a":"bad","x":0,"y":1}"#, vec!["", "", "/a"]),
        ("any", "null", vec![""]),
        ("any", "0", vec![""]),
        ("any", r#""ok""#, vec![]),
        ("one", "1.0", vec![""]),
        ("one", "0.5", vec![]),
        ("one", "null", vec![""]),
        ("tuple", "[]", vec![""]),
        (
            "tuple",
            r#"["",true,0,1]"#,
            vec!["", "/0", "/1", "/2", "/3"],
        ),
        ("tuple", r#"["a",1.0]"#, vec![]),
        ("list", "[0]", vec!["", "/0"]),
        ("list", r#"["a",0,1]"#, vec!["", "/1", "/2"]),
        ("unicode", r#""😀é""#, vec![]),
        ("unicode", r#""😀""#, vec![""]),
        ("unicode", r#""a\u0301b""#, vec![""]),
    ] {
        // Observe the actual pinned validator independently of the new target.
        let actual = plan.run_json(branch, input);
        let case = if pointers.is_empty() {
            json!({"branch":branch,"input":input,"value":actual.unwrap()})
        } else {
            let mut findings = actual.unwrap_err().0;
            findings.sort_by(|a, b| a.pointer.cmp(&b.pointer));
            assert_eq!(
                findings
                    .iter()
                    .map(|f| f.pointer.clone())
                    .collect::<Vec<_>>(),
                pointers
                    .iter()
                    .map(|p| format!("/branches/{branch}/0/input{p}"))
                    .collect::<Vec<_>>(),
                "{branch} {input}"
            );
            assert!(findings.iter().all(|f| f.rule == "schema_validation"));
            json!({"branch":branch,"input":input,"errors":findings})
        };
        cases.push(case);
    }
    (plan, cases)
}

pub fn boundary_plan() -> Plan {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let bundle = import(&json!({"components":{"schemas":{"Any":true,"Number":{"type":"number"},"Text":{"type":"string"}}}}).to_string(),
        &["Any".to_owned(),"Number".to_owned(),"Text".to_owned()].into_iter().collect(), Dialect::Draft202012).unwrap();
    let stage = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        json!([{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}])
    };
    let recipe = json!({"format":"ess-normalization/6","branches":{"plain":stage("Any"),"rounded":stage("Number"),"raw":stage("Text"),"__proto__":stage("Any"),"constructor":stage("Any")},"binary64_inputs":{"rounded":[[]]},"raw_json_inputs":{"raw":[[]]}});
    Plan::read(&recipe.to_string(), &[bundle]).unwrap()
}

/// Preserve the frozen recipe-wide equality flag without broadening static types.
pub fn integer_equality_controls() -> Vec<(Plan, Vec<Value>)> {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let bundle = import(&json!({"components":{"schemas":{
        "Input":{"type":"object","required":["a","b"],"properties":{"a":{"type":"integer"},"b":{"type":"integer"}}},
        "Output":{"type":"boolean"}
    }}}).to_string(), &["Input".to_owned(),"Output".to_owned()].into_iter().collect(), Dialect::Draft202012).unwrap();
    let read = |name| json!({"op":"read","scope":"input","path":[name]});
    let mut out = Vec::new();
    for version in [1, 5, 6] {
        let recipe = json!({"format":format!("ess-normalization/{version}"),"branches":{"eq":[{
            "input":Root::pin(&bundle,"Input").unwrap(),"output":Root::pin(&bundle,"Output").unwrap(),"requires":[],
            "value":{"op":"choose","condition":{"op":"equal","left":read("a"),"right":read("b")},
                "then_value":{"op":"boolean","value":true},"else_value":{"op":"boolean","value":false}}
        }]}});
        let plan = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap();
        let cases = [
            (r#"{"a":1.0,"b":1.0}"#, (version != 1).then_some(true)),
            (r#"{"a":1.0,"b":2.0}"#, (version != 1).then_some(false)),
            (r#"{"a":-0.0,"b":0.0}"#, (version != 1).then_some(true)),
            (r#"{"a":1,"b":1.0}"#, None),
            (r#"{"a":1.0,"b":1}"#, None),
            (r#"{"a":1,"b":1}"#, Some(true)),
            (r#"{"a":18446744073709551615,"b":18446744073709551615}"#, None),
        ].into_iter().map(|(input,value)| match value {
            Some(value) => json!({"branch":"eq","input":input,"value":value}),
            None => json!({"branch":"eq","input":input,"errors":[{"pointer":"/branches/eq/0/value/condition","rule":"integer_representation","detail":"operation requires an exact integral token representable as signed 64-bit"}]}),
        }).collect();
        out.push((plan, cases));
    }
    out
}
