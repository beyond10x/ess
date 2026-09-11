//! Original input vectors shared by real Go and Firefox admission tests.
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const ID: &str = "example.domain/authored/created";
#[derive(Debug, Serialize, Deserialize)]
pub struct Case {
    pub name: String,
    pub input: String,
    pub accepted: bool,
}
pub fn document() -> Value {
    document_at("ess-conformance/5")
}
/// The same original candidate under an exact suite version, so cases that need the corrected
/// structured operand reader (coverage/9) carry a document that admits them.
pub fn document_at(suite_version: &str) -> Value {
    json!({
        "provenance":{"suite_version":suite_version,"system":"example","specification_version":"v1",
            "spec_digest":"a".repeat(64),"contract_digest":"b".repeat(64)},
        "scenarios":{ID:{"purpose":"Original candidate","steps":[],"source":[]}},
        "coverage":{
            "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
            "knowledge":"complete_inventory","generated":[],"authored":[ID],"outside":[],"refused":[],
            "authored_sources":{"created.yaml":{"digest":format!("sha256:{}","c".repeat(64)),"scenario":ID,"disposition":"accepted"}},
            "counts":{"generated":0,"authored":1,"outside":0,"refused":0}
        }
    })
}
pub fn carrier(suite: &Value) -> Value {
    json!({"format":"ess-conformance-input/1","suite_json":suite.to_string(),"parent_suites":[]})
}
fn pair(left: &Value, right: &Value) -> Value {
    pair_at("ess-conformance/5", left, right)
}
fn pair_at(suite_version: &str, left: &Value, right: &Value) -> Value {
    let mut document = document_at(suite_version);
    document["scenarios"][ID]["steps"] = left.clone();
    let parent =
        AdmittedInput::from_suite(AdmittedSuite::from_json(&document.to_string()).unwrap())
            .unwrap();
    let child = parent.select(&[ID.parse().unwrap()]).unwrap();
    let mut input: Value =
        serde_json::from_str(&child.document().to_canonical_json().unwrap()).unwrap();
    let mut selected: Value = serde_json::from_str(input["suite_json"].as_str().unwrap()).unwrap();
    selected["scenarios"][ID]["steps"] = right.clone();
    input["suite_json"] = json!(selected.to_string());
    input
}
fn append(cases: &mut Vec<Case>, name: &str, input: &Value, accepted: bool) {
    cases.push(Case {
        name: name.into(),
        input: input.to_string(),
        accepted,
    });
}
fn equivalences() -> Vec<(&'static str, Value, Value)> {
    vec![
        (
            "command-defaults",
            json!([{"step":"execute_command","command":"example.Create"}]),
            json!([{"step":"execute_command","command":"example.Create","input":{},"actor":null}]),
        ),
        (
            "event-defaults",
            json!([{"step":"expect_event","event":"example.Created"}]),
            json!([{"step":"expect_event","event":"example.Created","payload":{},"shape":{}}]),
        ),
        (
            "eventual-event-defaults",
            json!([{"step":"eventually_event","event":"example.Created"}]),
            json!([{"step":"eventually_event","event":"example.Created","payload":{},"shape":{}}]),
        ),
        (
            "error-defaults",
            json!([{"step":"expect_error","error":"example.Refused"}]),
            json!([{"step":"expect_error","error":"example.Refused","fields":{}}]),
        ),
        (
            "query-defaults",
            json!([{"step":"query_view","view":"example.All"}]),
            json!([{"step":"query_view","view":"example.All","params":{}}]),
        ),
        (
            "invocation-defaults",
            json!([{"step":"expect_invocation","binding":"forward","command":"example.Create"}]),
            json!([{"step":"expect_invocation","binding":"forward","command":"example.Create","input":{}}]),
        ),
        (
            "halt-defaults",
            json!([{"step":"expect_halt","view":"example.All","after":18_446_744_073_709_551_615_u64}]),
            json!([{"step":"expect_halt","view":"example.All","after":18_446_744_073_709_551_615_u64,"params":{}}]),
        ),
        (
            "eventual-halt-defaults",
            json!([{"step":"eventually_halt","view":"example.All","after":9_007_199_254_740_993_u64}]),
            json!([{"step":"eventually_halt","view":"example.All","after":9_007_199_254_740_993_u64,"params":{}}]),
        ),
        (
            "shape-defaults",
            json!([{"step":"expect_event","event":"example.Created","shape":{"id":{"holds":"primitive","kind":"integer"}}}]),
            json!([{"step":"expect_event","event":"example.Created","shape":{"id":{"holds":"primitive","kind":"integer","optional":false}}}]),
        ),
        (
            "count-defaults",
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"counts"}}]),
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"counts","at_least":null,"at_most":null}}]),
        ),
        (
            "count-upper-wire",
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"counts","at_least":18_446_744_073_709_551_615_u64}}]),
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"counts","at_least":18_446_744_073_709_551_615_u64,"at_most":null}}]),
        ),
        (
            "position-upper-wire",
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"at","order_by":["id"],"position":{"row":"nth","index":18_446_744_073_709_551_615_u64}}}]),
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"at","order_by":["id asc"],"position":{"row":"nth","index":18_446_744_073_709_551_615_u64},"fields":{}}}]),
        ),
        (
            "at-defaults-and-ranking",
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"at","order_by":["some/field"],"position":{"row":"first"}}}]),
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"at","order_by":["some/field ascending"],"position":{"row":"first"},"fields":{}}}]),
        ),
        (
            "eventual-view-defaults",
            json!([{"step":"eventually_view","view":"example.All","expectation":{"expect":"contains","fields":{}}}]),
            json!([{"step":"eventually_view","view":"example.All","expectation":{"expect":"contains","fields":{}},"params":{}}]),
        ),
        (
            "ranking-descending",
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"ranked","order_by":["id desc"]}}]),
            json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"ranked","order_by":["id descending"]}}]),
        ),
    ]
}
fn predicate_pairs() -> Vec<(Value, Value)> {
    vec![
        (json!(true), json!({"all":null})),
        (json!(false), json!({"any":null})),
        (json!(true), json!({"none":null})),
        (json!({"none":["ready"]}), json!({"not":{"or":["ready"]}})),
        (json!({"all_of":["ready"]}), json!("ready")),
        (
            json!({"none_of_these":[false,true]}),
            json!({"not":{"any":[false,true]}}),
        ),
        (json!({"not":{"not":"ready"}}), json!("ready")),
        (json!({"ready":{"defined":false}}), json!("missing(ready)")),
        (json!({"x":{"equals":1}}), json!("x == 1")),
        (json!({"x":{"not_in":null}}), json!({"x":{"none_of":[]}})),
        (json!({"x":{"truthy":{"not":"an expression"}}}), json!("x")),
        // `1` and `1.0` are one predicate: binary64 carries both, so the two spellings name one
        // value in every lane. The pair that used to sit here — `9007199254740992` against
        // `9007199254740993` — is *not* an equivalence and now sits in `refusal_cases` under
        // `child-swaps-two-integers-binary64-collapses`; see story:review-primitive-semantics.
        (json!({"x":1}), json!({"x":1.0})),
        (json!({"x":"\u{feff}word"}), json!("x == '\u{feff}word'")),
        (json!({"x":1}), json!("x ==\u{85}1")),
        (json!(true), json!("\u{85}true")),
    ]
}
/// Equivalences whose structured spelling carries an operand an original reader would have read
/// as literal text. They are one predicate only under the corrected reader, so they are authored
/// at coverage/9 and refused by `quoted_predicate_format` below it.
fn lossless_predicate_pairs() -> Vec<(Value, Value)> {
    vec![(json!({"x":{"gte":"other.0"}}), json!("x >= other.0"))]
}
pub fn cases() -> Vec<Case> {
    let mut result = Vec::new();
    default_cases(&mut result);
    node_cases(&mut result);
    selection_cases(&mut result);
    parent_cases(&mut result);
    vocabulary_cases(&mut result);
    let incomplete = refusal_cases(&mut result);
    closed_cases(&mut result, &incomplete);
    result
}

fn default_cases(result: &mut Vec<Case>) {
    append(result, "all", &carrier(&document()), true);
    let mut oversized_parent = document();
    let other = "example.domain/authored/oversized";
    oversized_parent["scenarios"][other] = json!({"purpose":"Parent-only large reader limit",
        "steps":[{"step":"expect_halt","view":"example.All","after":18_446_744_073_709_551_615_u64}],"source":[]});
    oversized_parent["coverage"]["authored"] = json!([ID, other]);
    oversized_parent["coverage"]["counts"]["authored"] = json!(2);
    oversized_parent["coverage"]["authored_sources"]["oversized.yaml"] = json!({
        "digest":format!("sha256:{}","d".repeat(64)),"scenario":other,"disposition":"accepted"});
    let parent =
        AdmittedInput::from_suite(AdmittedSuite::from_json(&oversized_parent.to_string()).unwrap())
            .unwrap();
    let selected = parent.select(&[ID.parse().unwrap()]).unwrap();
    append(
        result,
        "oversized-parent-only",
        &serde_json::from_str(&selected.document().to_canonical_json().unwrap()).unwrap(),
        true,
    );
    for (name, left, right) in equivalences() {
        append(result, name, &pair(&left, &right), true);
    }
    let step = |predicate| json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"satisfies","predicate":predicate}}]);
    for (index, (left, right)) in predicate_pairs().into_iter().enumerate() {
        append(
            result,
            &format!("predicate-equivalent-{index}"),
            &pair(&step(left), &step(right)),
            true,
        );
    }
    for (index, (left, right)) in lossless_predicate_pairs().into_iter().enumerate() {
        append(
            result,
            &format!("predicate-lossless-{index}"),
            &pair_at("ess-conformance/9", &step(left), &step(right)),
            true,
        );
    }
}

fn node_cases(result: &mut Vec<Case>) {
    for owner in ["input", "params", "payload", "fields"] {
        let (tag, required) = match owner {
            "input" => ("execute_command", json!({"command":"example.Create"})),
            "params" => ("query_view", json!({"view":"example.All"})),
            "payload" => ("expect_event", json!({"event":"example.Created"})),
            _ => ("expect_error", json!({"error":"example.Refused"})),
        };
        let mut step = required;
        step["step"] = json!(tag);
        let wrapped = |value| {
            if owner == "input" || owner == "params" {
                json!({"x":{"kind":"literal","value":value}})
            } else {
                json!({"x":value})
            }
        };
        step[owner] = wrapped(json!({"nested":[9_007_199_254_740_992_u64,0]}));
        let left = json!([step.clone()]);
        // The two spellings of zero are one value — `docs/design/review-primitive-semantics.md`
        // gives `units × 10^-scale` one zero, and every lane agrees. The integer beside it is the
        // *same* integer on both sides; it used to be `9007199254740993`, and that expectation was
        // the F08 defect (two integers binary64 collapses were one value), not a fact about
        // finiteness. The swap it used to assert is now asserted the other way, below.
        step[owner] = wrapped(json!({"nested":[9_007_199_254_740_992_u64,-0.0]}));
        append(
            result,
            &format!("finite-node-{owner}"),
            &pair(&left, &json!([step.clone()])),
            true,
        );
        // Two integers binary64 collapses are two nodes, in Rust and in the browser adapter alike.
        step[owner] = wrapped(json!({"nested":[9_007_199_254_740_993_u64,0]}));
        append(
            result,
            &format!("collapsed-node-{owner}"),
            &pair(&left, &json!([step.clone()])),
            false,
        );
        step[owner] = wrapped(json!({"nested":[9_007_199_254_740_994_u64,0]}));
        append(
            result,
            &format!("changed-node-{owner}"),
            &pair(&left, &json!([step.clone()])),
            false,
        );

        // The other direction of the same rule: the exact-integer path applies to a JSON *number*
        // token, and a JSON string is never reinterpreted however numeric it looks. The keys are
        // astral and BMP so the string arm carries the arbitrary-key check the number arm has.
        let lookalike = |first: &str, second: Value| {
            wrapped(json!({
                "nested": [first, "2.0"],
                "\u{1f600}": "9007199254740993",
                "\u{e000}": second,
            }))
        };
        step[owner] = lookalike("9007199254740992", json!("2.0"));
        let strings = json!([step.clone()]);
        append(
            result,
            &format!("string-lookalike-node-{owner}"),
            &pair(&strings, &strings),
            true,
        );
        // Two numeric-looking strings are two strings, exactly as two integers are two integers.
        step[owner] = lookalike("9007199254740993", json!("2.0"));
        append(
            result,
            &format!("changed-string-lookalike-node-{owner}"),
            &pair(&strings, &json!([step.clone()])),
            false,
        );
        // And a string is never the number it looks like: swapping `"2.0"` for `2.0` is a change.
        step[owner] = lookalike("9007199254740992", json!(2.0));
        append(
            result,
            &format!("string-is-not-the-number-it-looks-like-node-{owner}"),
            &pair(&strings, &json!([step])),
            false,
        );
    }
}

fn selection_cases(result: &mut Vec<Case>) {
    let base = pair(&json!([]), &json!([]));
    let mut optional = base.clone();
    let mut selected: Value =
        serde_json::from_str(optional["suite_json"].as_str().unwrap()).unwrap();
    selected["provenance"]["component"] = Value::Null;
    optional["suite_json"] = json!(selected.to_string());
    append(result, "provenance-omitted-null", &optional, true);
    let mut unknown = document();
    unknown["coverage"]["knowledge"] = json!("unknown");
    append(result, "unknown-is-diagnostic", &carrier(&unknown), true);
    let all = AdmittedInput::from_suite(AdmittedSuite::from_json(&document().to_string()).unwrap())
        .unwrap();
    let empty = all.select(&[]).unwrap();
    append(
        result,
        "explicit-empty",
        &serde_json::from_str(&empty.document().to_canonical_json().unwrap()).unwrap(),
        true,
    );
    let mut deep = all;
    for _ in 0..70 {
        deep = deep.select(&[ID.parse().unwrap()]).unwrap();
    }
    append(
        result,
        "seventy-generations",
        &serde_json::from_str(&deep.document().to_canonical_json().unwrap()).unwrap(),
        true,
    );
}

fn parent_cases(result: &mut Vec<Case>) {
    let base = pair(&json!([]), &json!([]));
    for (name, path, replacement) in [
        (
            "changed-purpose",
            "/scenarios/example.domain~1authored~1created/purpose",
            json!("Different"),
        ),
        (
            "changed-source",
            "/scenarios/example.domain~1authored~1created/source",
            json!([{"kind":"view","name":"example.Other"}]),
        ),
        ("changed-knowledge", "/coverage/knowledge", json!("unknown")),
        (
            "changed-source-digest",
            "/coverage/authored_sources/created.yaml/digest",
            json!(format!("sha256:{}", "d".repeat(64))),
        ),
        (
            "changed-outside",
            "/coverage/outside",
            json!([{"scenario":"example.domain/authored/other","origin":"generated","reason":"origin_selection","needs":[]}]),
        ),
    ] {
        let mut input = base.clone();
        let mut selected: Value =
            serde_json::from_str(input["suite_json"].as_str().unwrap()).unwrap();
        *selected.pointer_mut(path).unwrap() = replacement;
        input["suite_json"] = json!(selected.to_string());
        append(result, name, &input, false);
    }
    for (name, parents) in [
        ("missing-parent", json!([])),
        ("wrong-parent-type", json!([{}])),
        (
            "duplicate-parent",
            json!([base["parent_suites"][0], base["parent_suites"][0]]),
        ),
    ] {
        let mut input = base.clone();
        input["parent_suites"] = parents;
        append(result, name, &input, false);
    }
    let mut changed = base.clone();
    changed["parent_suites"][0] =
        json!(format!("{}\n", base["parent_suites"][0].as_str().unwrap()));
    append(result, "changed-parent-original", &changed, false);
    let mut unused = carrier(&document());
    unused["parent_suites"] = base["parent_suites"].clone();
    append(result, "unused-parent", &unused, false);
}

fn vocabulary_cases(result: &mut Vec<Case>) {
    for token in ["1.0", "1e0", "-0", "18446744073709551616"] {
        let mut input = carrier(&document());
        input["suite_json"] = json!(document()
            .to_string()
            .replace("\"authored\":1", &format!("\"authored\":{token}")));
        append(result, &format!("count-token-{token}"), &input, false);
    }
    for (name, path, replacement) in [
        (
            "shape-binary64",
            "/scenarios/example.domain~1authored~1created/steps",
            json!([{"step":"expect_event","event":"example.Created","shape":{"x":{"holds":"primitive","kind":"binary64"}}}]),
        ),
        (
            "unknown-step",
            "/scenarios/example.domain~1authored~1created/steps",
            json!([{"step":"pretend"}]),
        ),
        (
            "null-map",
            "/scenarios/example.domain~1authored~1created/steps",
            json!([{"step":"execute_command","command":"example.Create","input":null}]),
        ),
        (
            "suite4-with-coverage",
            "/provenance/suite_version",
            json!("ess-conformance/4"),
        ),
    ] {
        let mut suite = document();
        *suite.pointer_mut(path).unwrap() = replacement;
        append(result, name, &carrier(&suite), false);
    }
    for (left, right) in [
        (9_007_199_254_740_992_u64, 9_007_199_254_740_993_u64),
        (0, u64::MAX),
    ] {
        let step = |after| json!([{"step":"expect_halt","view":"example.All","after":after}]);
        append(
            result,
            &format!("exact-integer-{left}-{right}"),
            &pair(&step(left), &step(right)),
            false,
        );
    }
}

fn refusal_cases(result: &mut Vec<Case>) -> Value {
    let mut incomplete = document();
    incomplete["coverage"]["authored_sources"]["rejected.yaml"] = json!({
        "digest":format!("sha256:{}","d".repeat(64)),"scenario":ID,"disposition":"refused"});
    let refusal = json!({"origin":"authored","scenario":ID,"subject":null,"source":"rejected.yaml",
        "code":"ESS-AUTHOR-001","message":"An original refused candidate","effect":"candidate_not_emitted",
        "retained":{"origin":"authored","source":"created.yaml"},"scope":"in_scope","needs":[]});
    incomplete["coverage"]["refused"] = json!([refusal.clone(), refusal]);
    incomplete["coverage"]["counts"]["refused"] = json!(2);
    append(
        result,
        "repeated-in-scope-refusals",
        &carrier(&incomplete),
        true,
    );
    let parent =
        AdmittedInput::from_suite(AdmittedSuite::from_json(&incomplete.to_string()).unwrap())
            .unwrap();
    let child = parent.select(&[ID.parse().unwrap()]).unwrap();
    let paired: Value =
        serde_json::from_str(&child.document().to_canonical_json().unwrap()).unwrap();
    append(result, "repeated-refusal-parent", &paired.clone(), true);
    for mutation in ["coalesced", "ordinal-message", "scope", "source-map"] {
        let mut input = paired.clone();
        let mut selected: Value =
            serde_json::from_str(input["suite_json"].as_str().unwrap()).unwrap();
        match mutation {
            "coalesced" => {
                selected["coverage"]["refused"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
                selected["coverage"]["counts"]["refused"] = json!(1);
            }
            "ordinal-message" => {
                selected["coverage"]["refused"][1]["message"] =
                    json!("An original refused candidate (2)");
            }
            "scope" => selected["coverage"]["refused"][0]["scope"] = json!("outside_origin"),
            "source-map" => {
                selected["coverage"]["authored_sources"]
                    .as_object_mut()
                    .unwrap()
                    .remove("rejected.yaml");
            }
            _ => unreachable!(),
        }
        input["suite_json"] = json!(selected.to_string());
        append(result, &format!("refusal-parent-{mutation}"), &input, false);
    }

    // Two integers binary64 collapses are two predicates, so a child that swaps one for the other
    // has changed the scenario and lineage refuses it.
    //
    // This pair was listed as an *equivalence* until story:review-primitive-semantics, and the
    // expectation was the F08 defect itself: `ess_primitives::facts::Number` was an `f64`, both
    // literals were one value, and a child that replaced one with the other appeared to change
    // nothing. `Number` is exact now, so `x == 9007199254740992` and `x == 9007199254740993` are
    // two guards over the same fact and a suite may not silently become the other one. The browser
    // adapter agrees because `coverage-admission.js` keeps an integer token JS `Number` cannot hold
    // — see *One rule for comparing integers* in `docs/design/review-primitive-semantics.md`.
    let step = |predicate| json!([{"step":"expect_view","view":"example.All","expectation":{"expect":"satisfies","predicate":predicate}}]);
    append(
        result,
        "child-swaps-two-integers-binary64-collapses",
        &pair(
            &step(json!({"x":9_007_199_254_740_992_u64})),
            &step(json!({"x":9_007_199_254_740_993_u64})),
        ),
        false,
    );
    incomplete
}

fn closed_cases(result: &mut Vec<Case>, incomplete: &Value) {
    for path in [
        "",
        "/provenance",
        "/coverage",
        "/coverage/selection",
        "/coverage/selection/scope",
        "/coverage/selection/filter",
        "/coverage/counts",
        "/coverage/authored_sources/created.yaml",
        "/coverage/refused/0",
        "/coverage/refused/0/retained",
        "/scenarios/example.domain~1authored~1created",
    ] {
        let mut changed = incomplete.clone();
        changed
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("alien".into(), json!(true));
        append(result, &format!("closed{path}"), &carrier(&changed), false);
    }
    for field in ["scenario", "subject", "source", "retained"] {
        let mut changed = incomplete.clone();
        changed["coverage"]["refused"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        append(
            result,
            &format!("missing-required-nullable-{field}"),
            &carrier(&changed),
            false,
        );
    }
    for identity in [
        "/absolute.yaml",
        "../escape.yaml",
        "a//b.yaml",
        "a/./b.yaml",
        "C:file.yaml",
        "a\\b.yaml",
    ] {
        let mut changed = document();
        let source = changed["coverage"]["authored_sources"]
            .as_object_mut()
            .unwrap()
            .remove("created.yaml")
            .unwrap();
        changed["coverage"]["authored_sources"][identity] = source;
        append(
            result,
            &format!("source-identity-{identity}"),
            &carrier(&changed),
            false,
        );
    }
    let mut duplicate = carrier(&document());
    duplicate["suite_json"] = json!(document().to_string().replacen('{', "{\"coverage\":{},", 1));
    append(result, "duplicate-inner-key", &duplicate, false);
    for (name, raw) in [
        (
            "duplicate-carrier-key",
            carrier(&document()).to_string().replacen(
                '{',
                "{\"format\":\"ess-conformance-input/1\",",
                1,
            ),
        ),
        (
            "unpaired-surrogate",
            carrier(&document()).to_string().replace("v1", "\\uD800"),
        ),
    ] {
        result.push(Case {
            name: name.into(),
            input: raw,
            accepted: false,
        });
    }
}
