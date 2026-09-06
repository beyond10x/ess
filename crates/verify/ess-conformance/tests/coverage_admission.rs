//! Coverage admission keeps exact source identity separate from execution and inventory.
use ess_conformance::coverage::{AdmittedInput, SuiteInputDocument};
use ess_conformance::AdmittedSuite;
use serde_json::{json, Value};

fn document() -> Value {
    json!({
        "provenance": {
            "suite_version": "ess-conformance/5", "system": "example",
            "specification_version": "v1",
            "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "contract_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "scenarios": {
            "example.domain/authored/created": {"purpose":"Known candidate", "steps":[], "source":[]}
        },
        "coverage": {
            "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
            "knowledge":"complete_inventory", "generated":[],
            "authored":["example.domain/authored/created"], "outside":[], "refused":[],
            "authored_sources":{"created.yaml":{
                "digest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "scenario":"example.domain/authored/created","disposition":"accepted"
            }},
            "counts":{"generated":0,"authored":1,"outside":0,"refused":0}
        }
    })
}

#[test]
fn complete_inventory_is_admitted_from_its_original_bytes() {
    let text = serde_json::to_string_pretty(&document()).unwrap() + "\n";
    let admitted = AdmittedSuite::from_json(&text).expect("suite5 inventory is admitted");
    assert_eq!(admitted.original_json(), text);
    assert_eq!(admitted.suite().len(), 1);
    let compact = serde_json::to_string(&document()).unwrap();
    assert_ne!(
        admitted.digest(),
        AdmittedSuite::from_json(&compact).unwrap().digest()
    );
}

#[test]
fn inventory_corruption_is_refused_at_admission() {
    let mutations = [
        ("/coverage/counts/authored", json!(0)),
        ("/coverage/knowledge", json!("complete")),
        ("/coverage/authored", json!([])),
        (
            "/coverage/generated",
            json!(["example.domain/authored/created"]),
        ),
        (
            "/coverage/selection/scope",
            json!({"kind":"component","component":"worker"}),
        ),
        (
            "/coverage/authored_sources/created.yaml/disposition",
            json!("refused"),
        ),
        (
            "/coverage/authored_sources/created.yaml/scenario",
            Value::Null,
        ),
        (
            "/coverage/authored_sources/created.yaml/digest",
            json!("sha256:no"),
        ),
    ];
    for (path, replacement) in mutations {
        let mut value = document();
        *value.pointer_mut(path).unwrap() = replacement;
        assert!(
            AdmittedSuite::from_json(&value.to_string()).is_err(),
            "{path}"
        );
    }
}

#[test]
fn coverage_integer_tokens_and_closed_fields_are_checked_before_serde() {
    let original = document().to_string();
    for number in ["1.0", "1e0", "-0", "18446744073709551616"] {
        let changed = original.replace("\"authored\":1", &format!("\"authored\":{number}"));
        assert_ne!(original, changed);
        assert!(AdmittedSuite::from_json(&changed).is_err(), "{number}");
    }
    for path in [
        "/coverage",
        "/coverage/selection",
        "/coverage/counts",
        "/coverage/authored_sources/created.yaml",
        "/scenarios/example.domain~1authored~1created",
    ] {
        let mut value = document();
        value
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("alien".into(), json!(true));
        assert!(
            AdmittedSuite::from_json(&value.to_string()).is_err(),
            "{path}"
        );
    }
}

#[test]
fn coverage_source_identity_is_checked_without_normalizing_it() {
    for name in [
        "/a.yaml",
        "../a.yaml",
        "a//b.yaml",
        "./a.yaml",
        "a\\b.yaml",
        "a:b.yaml",
        "a/\nb.yaml",
    ] {
        let mut value = document();
        let sources = value["coverage"]["authored_sources"]
            .as_object_mut()
            .unwrap();
        let source = sources.remove("created.yaml").unwrap();
        sources.insert(name.into(), source);
        assert!(
            AdmittedSuite::from_json(&value.to_string()).is_err(),
            "{name:?}"
        );
    }
}

#[test]
fn explicit_selection_retains_original_parents_and_refuses_direct_admission() {
    let original = document().to_string() + "\n";
    let all = AdmittedInput::from_suite(AdmittedSuite::from_json(&original).unwrap()).unwrap();
    let child = all.select(&[]).expect("empty explicit selection is valid");
    assert_eq!(child.selected().suite().len(), 0);
    assert_eq!(child.parents()[0].original_json(), original);
    assert!(AdmittedSuite::from_json(child.selected().original_json()).is_err());
    let text = child.document().to_canonical_json().unwrap();
    let admitted = AdmittedInput::from_json(&text).unwrap();
    assert_eq!(admitted.selected().digest(), child.selected().digest());
    assert_eq!(admitted.selected().coverage().unwrap().counts.outside, 1);
    assert_eq!(
        admitted
            .selected()
            .coverage()
            .unwrap()
            .authored_sources
            .len(),
        1
    );
    let repeated = admitted.select(&[]).unwrap();
    assert_eq!(repeated.parents().len(), 2);
    assert_eq!(repeated.parents()[1].original_json(), original);
}

#[test]
fn input_carrier_is_structural_and_cannot_mint_admission() {
    let text =
        json!({"format":"ess-conformance-input/1","suite_json":"not a suite","parent_suites":[]})
            .to_string();
    assert!(SuiteInputDocument::from_json(&text).is_ok());
    assert!(AdmittedInput::from_json(&text).is_err());
    for text in [
        r#"{"format":"ess-conformance-input/1","suite_json":"x","suite_json":"y","parent_suites":[]}"#,
        r#"{"format":"ess-conformance-input/1","suite_json":"x","parent_suites":[],"extra":0}"#,
        r#"{"format":"ess-conformance-input/1","suite_json":"\uD800","parent_suites":[]}"#,
        r#"{"format":"ess-conformance-input/1","suite_json":{},"parent_suites":[]}"#,
    ] {
        assert!(SuiteInputDocument::from_json(text).is_err(), "{text}");
        assert!(
            serde_json::from_str::<SuiteInputDocument>(text).is_err(),
            "{text}"
        );
    }
}

#[test]
fn all_input_is_admitted_only_without_unused_parents() {
    let original = document().to_string();
    let text = json!({"format":"ess-conformance-input/1","suite_json":original,"parent_suites":[]});
    assert!(AdmittedInput::from_json(&text.to_string()).is_ok());
    let mut changed = text;
    changed["parent_suites"] = json!([original]);
    assert!(AdmittedInput::from_json(&changed.to_string()).is_err());
}
