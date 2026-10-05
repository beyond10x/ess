//! Original-byte and typed mutation cases at the nested observation boundary.
use super::{suite, MODEL};
use ess_conformance::{response::Observation, AdmittedSuite, ConformanceSuite};
use serde_json::{json, Value};

pub fn document() -> Value {
    serde_json::from_str(suite(MODEL).original_json()).unwrap()
}
pub fn observation(document: &mut Value) -> &mut Value {
    document["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .flat_map(|scenario| scenario["steps"].as_array_mut().unwrap())
        .find(|step| step["step"] == "expect_response_payload")
        .unwrap()
        .get_mut("response")
        .unwrap()
}
type Mutation = (&'static str, fn(&mut Value));
const MUTATIONS: &[Mutation] = &[
    ("null", |v| v["nested"] = Value::Null),
    ("empty", |v| v["nested"] = json!({})),
    ("unknown", |v| v["nested"]["extra"] = json!(true)),
    ("invalid-response-name", |v| {
        v["fields"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"invalid.name","type":"Integer"}));
    }),
    ("root-wire", |v| {
        v["nested"]["roots"][0]["wire"] = json!("other");
    }),
    ("member-presence", |v| {
        v["nested"]["declarations"]["demo.api.Packet"]["fields"][0]["presence"] =
            json!("null_when_absent");
    }),
    ("inactive-kind", |v| {
        v["nested"]["declarations"]["demo.api.Packet"]["of"] = json!("Integer");
    }),
    ("response-metadata", |v| {
        v["fields"][0]["naming"] = json!({});
    }),
    ("null-presence", |v| {
        v["fields"][0]["presence"] = Value::Null;
    }),
    ("one-segment", |v| {
        v["nested"]["mappings"][0]["target"] = json!(["packet"]);
    }),
    ("empty-segment", |v| {
        v["nested"]["mappings"][0]["target"] = json!(["packet", ""]);
    }),
    ("dotted-segment", |v| {
        v["nested"]["mappings"][0]["target"] = json!(["packet", "value.other"]);
    }),
    ("undeclared-member", |v| {
        v["nested"]["mappings"][0]["target"] = json!(["packet", "missing"]);
    }),
    ("undeclared-source", |v| {
        v["nested"]["mappings"][0]["source"] = json!("missing");
    }),
    ("terminal-type", |v| {
        v["nested"]["declarations"]["demo.api.Packet"]["fields"][0]["type"] = json!("String");
    }),
    ("conflicting-representation", |v| {
        v["fields"][0]["type"] = json!("demo.api.Value");
        v["declarations"]["demo.api.Value"] = json!({"kind":"newtype","of":"String"});
        v["nested"]["declarations"]["demo.api.Packet"]["fields"][0]["type"] =
            json!("demo.api.Value");
        v["nested"]["declarations"]["demo.api.Value"] = json!({"kind":"newtype","of":"Integer"});
    }),
    ("prefix-overlap", |v| {
        v["fields"] =
            json!([{"name":"aggregate","type":"demo.api.Value"},{"name":"value","type":"Integer"}]);
        let body = json!({"kind":"struct","fields":[{"name":"leaf","type":"Integer"}]});
        v["declarations"]["demo.api.Value"] = body.clone();
        v["nested"]["declarations"]["demo.api.Value"] = body;
        v["nested"]["declarations"]["demo.api.Packet"]["fields"][0]["type"] =
            json!("demo.api.Value");
        v["nested"]["mappings"] = json!([{"target":["packet","value"],"source":"aggregate"},{"target":["packet","value","leaf"],"source":"value"}]);
    }),
    ("duplicate-path", |v| {
        let path = v["nested"]["mappings"][0].clone();
        v["nested"]["mappings"].as_array_mut().unwrap().push(path);
    }),
    ("duplicate-root", |v| {
        let root = v["nested"]["roots"][0].clone();
        v["nested"]["roots"].as_array_mut().unwrap().push(root);
    }),
    ("unused-root", |v| {
        v["nested"]["roots"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":"unused","type":"demo.api.Packet"}));
    }),
    ("missing-declaration", |v| {
        v["nested"]["declarations"] = json!({});
    }),
    ("unrelated-declaration", |v| {
        v["nested"]["declarations"]["demo.api.Unused"] = json!({"kind":"newtype","of":"String"});
    }),
    ("duplicate-member", |v| {
        let field = v["nested"]["declarations"]["demo.api.Packet"]["fields"][0].clone();
        v["nested"]["declarations"]["demo.api.Packet"]["fields"]
            .as_array_mut()
            .unwrap()
            .push(field);
    }),
    ("list-ancestor", |v| {
        v["nested"]["roots"][0]["type"] = json!("List<demo.api.Packet>");
    }),
    ("map-ancestor", |v| {
        v["nested"]["roots"][0]["type"] = json!("Map<String, demo.api.Packet>");
    }),
    ("alias-cycle", |v| {
        v["nested"]["roots"][0]["type"] = json!("demo.api.Loop");
        v["nested"]["declarations"] =
            json!({"demo.api.Loop":{"kind":"newtype","of":"demo.api.Loop"}});
    }),
    ("flat-overlap", |v| {
        v["mappings"]["packet"] = json!("value");
        v["targets"] = json!([{"name":"packet","type":"Integer"}]);
    }),
    ("combined-limit", |v| {
        v["nested"]["mappings"] = json!(vec![v["nested"]["mappings"][0].clone(); 257]);
    }),
    ("34-segments", |v| {
        v["nested"]["mappings"][0]["target"] = json!(vec!["value"; 34]);
    }),
];
pub fn malformed() -> Vec<(&'static str, Value)> {
    let mut cases = Vec::new();
    let base = document();
    for (name, edit) in MUTATIONS {
        let mut value = base.clone();
        edit(observation(&mut value));
        cases.push((*name, value));
    }
    for (name, major) in [("legacy32", 32), ("legacy26", 26), ("legacy4", 4)] {
        let mut value = base.clone();
        value["provenance"]["suite_version"] = json!(format!("ess-conformance/{major}"));
        value["provenance"]
            .as_object_mut()
            .unwrap()
            .remove("scenario_initial_state");
        cases.push((name, value));
    }
    for (name, nested) in [("legacy-null", Value::Null), ("legacy-empty", json!({}))] {
        let mut value = base.clone();
        value["provenance"]["suite_version"] = json!("ess-conformance/32");
        value["provenance"]
            .as_object_mut()
            .unwrap()
            .remove("scenario_initial_state");
        observation(&mut value)["nested"] = nested;
        cases.push((name, value));
    }
    cases
}

#[test]
fn malformed_original_authority_is_refused_before_execution() {
    for (name, value) in malformed() {
        assert!(
            AdmittedSuite::from_json(&value.to_string()).is_err(),
            "{name}"
        );
    }
    let original = document().to_string();
    let duplicate = original.replacen("\"nested\":", "\"nested\":null,\"nested\":", 1);
    assert!(AdmittedSuite::from_json(&duplicate).is_err());
}

#[test]
fn typed_legacy_writer_cannot_smuggle_new_authority() {
    let mut typed: ConformanceSuite = serde_json::from_value(document()).unwrap();
    typed.provenance.suite_version = "ess-conformance/32".parse().unwrap();
    typed.provenance.scenario_initial_state = None;
    assert!(AdmittedSuite::from_suite(&typed).is_err());
    assert!(typed.to_compact_json().is_err());
}

#[test]
fn structural_siblings_and_finite_cycles_do_not_acquire_response_codec_restrictions() {
    let mut value = document();
    let nested = &mut observation(&mut value)["nested"];
    nested["declarations"]["demo.api.Packet"]["fields"]
        .as_array_mut()
        .unwrap()
        .extend([
            json!({"name":"number","type":"Binary64"}),
            json!({"name":"lookup","type":"Map<Integer, String>"}),
            json!({"name":"cycle","type":"Optional<demo.api.Packet>"}),
            json!({"name":"label","type":"demo.api.Label"}),
            json!({"name":"variant","type":"demo.api.Variant"}),
        ]);
    nested["declarations"]["demo.api.Label"] =
        json!({"kind":"enum","variants":["", "é<&\u{2028}", ""]});
    nested["declarations"]["demo.api.Variant"] =
        json!({"kind":"union","tag":"é<&\u{2028}","variants":{"":"String","other":"Binary64"}});
    let admitted = AdmittedSuite::from_json(&value.to_string()).unwrap();
    let (go, target) = super::foreign::go_run(
        &admitted,
        admitted.original_json(),
        super::Backend::new(super::Fault::None),
        "structural-siblings",
    );
    assert!(go.success, "{}", go.log);
    assert_eq!(target.calls.get(), 1);
    let (output, _, target) = super::foreign::typescript_run(
        &admitted,
        admitted.original_json(),
        super::Backend::new(super::Fault::None),
        "structural-siblings",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(target.calls.get(), 1);
}

#[test]
fn nested_canonical_accounting_uses_utf8_and_active_members_at_exact_byte_boundary() {
    let mut value = document();
    let observation = observation(&mut value);
    observation["nested"]["declarations"]["demo.api.Packet"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"kind","type":"demo.api.Label"}));
    observation["nested"]["declarations"]["demo.api.Label"] =
        json!({"kind":"enum","variants":["é<&\u{2028}\n\t\u{0001}"]});
    let parsed: Observation = serde_json::from_value(observation.clone()).unwrap();
    let initial = serde_json::to_vec(&parsed).unwrap().len();
    observation["nested"]["declarations"]["demo.api.Label"]["variants"][0]
        .as_str()
        .unwrap();
    let label = observation["nested"]["declarations"]["demo.api.Label"]["variants"][0]
        .as_str()
        .unwrap()
        .to_owned();
    observation["nested"]["declarations"]["demo.api.Label"]["variants"][0] =
        json!(format!("{label}{}", "x".repeat(1_048_576 - initial)));
    let parsed: Observation = serde_json::from_value(observation.clone()).unwrap();
    assert_eq!(serde_json::to_vec(&parsed).unwrap().len(), 1_048_576);
    let label = observation["nested"]["declarations"]["demo.api.Label"]["variants"][0]
        .as_str()
        .unwrap()
        .to_owned();
    observation["nested"]["declarations"]["demo.api.Label"]["variants"][0] =
        json!(format!("{label}x"));
    assert!(serde_json::from_value::<Observation>(observation.clone()).is_err());
}

fn canonical_boundary(size: usize) -> Value {
    let mut value = document();
    let observation = observation(&mut value);
    observation["nested"]["declarations"]["demo.api.Packet"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"label","type":"demo.api.Label"}));
    observation["nested"]["declarations"]["demo.api.Label"] =
        json!({"kind":"enum","variants":["é<&\u{2028}\u{2029}\n\t\u{0001}"]});
    let parsed: Observation = serde_json::from_value(observation.clone()).unwrap();
    let initial = serde_json::to_vec(&parsed).unwrap().len();
    let label = observation["nested"]["declarations"]["demo.api.Label"]["variants"][0]
        .as_str()
        .unwrap()
        .to_owned();
    observation["nested"]["declarations"]["demo.api.Label"]["variants"][0] =
        json!(format!("{label}{}", "x".repeat(size - initial)));
    value
}

fn declarations_boundary(count: usize) -> Value {
    let mut value = document();
    let nested = &mut observation(&mut value)["nested"];
    nested["declarations"]["demo.api.Packet"]["fields"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"sibling","type":"demo.api.Alias0"}));
    for index in 0..count - 1 {
        let next = if index + 2 == count {
            "Integer".to_owned()
        } else {
            format!("demo.api.Alias{}", index + 1)
        };
        nested["declarations"][format!("demo.api.Alias{index}")] =
            json!({"kind":"newtype","of":next});
    }
    value
}
fn relationships_boundary(count: usize) -> Value {
    let mut value = document();
    let nested = &mut observation(&mut value)["nested"];
    nested["declarations"]["demo.api.Packet"]["fields"] = json!((0..count)
        .map(|index| json!({"name":format!("member{index}"),"type":"Optional<Integer>"}))
        .collect::<Vec<_>>());
    nested["mappings"] = json!((0..count)
        .map(|index| json!({"target":["packet",format!("member{index}")],"source":"value"}))
        .collect::<Vec<_>>());
    observation(&mut value)["fields"][0]["type"] = json!("Optional<Integer>");
    value
}

#[test]
fn exact_byte_declaration_and_relationship_limits_match_all_actual_readers() {
    let admitted = suite(MODEL);
    for (label, value, accepted) in [
        ("bytes-limit", canonical_boundary(1_048_576), true),
        ("bytes-over", canonical_boundary(1_048_577), false),
        ("declarations-limit", declarations_boundary(4096), true),
        ("declarations-over", declarations_boundary(4097), false),
        ("relationships-limit", relationships_boundary(256), true),
        ("relationships-over", relationships_boundary(257), false),
    ] {
        let document = value.to_string();
        assert_eq!(
            AdmittedSuite::from_json(&document).is_ok(),
            accepted,
            "{label}"
        );
        let (go, target) = super::foreign::go_run(
            &admitted,
            &document,
            super::Backend::new(super::Fault::None),
            label,
        );
        assert_eq!(target.callbacks.get() > 0, accepted, "{label}: {}", go.log);
        let (output, _report, target) = super::foreign::typescript_run(
            &admitted,
            &document,
            super::Backend::new(super::Fault::None),
            label,
        );
        assert_eq!(
            target.callbacks.get() > 0,
            accepted,
            "{label}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.status.code() == Some(2), !accepted, "{label}");
    }
}

#[test]
fn legacy_flat_observation_bytes_remain_exact_and_nestedless() {
    let bytes = r#"{"command":"demo.api.Read","outcome":{"command":"demo.api.Read","outcome":"returned"},"event":"demo.api.Returned","fields":[{"name":"value","type":"Integer"}],"declarations":{},"mappings":{"flat":"value"},"targets":[{"name":"flat","type":"Integer"}]}"#;
    let observation: Observation = serde_json::from_str(bytes).unwrap();
    assert!(observation.nested.is_none());
    assert_eq!(serde_json::to_string(&observation).unwrap(), bytes);
}
