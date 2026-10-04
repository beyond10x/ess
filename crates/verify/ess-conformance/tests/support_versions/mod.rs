//! Relabel a fresh suite for a legacy vocabulary test without carrying new provenance authority.
#[allow(dead_code)]
pub fn legacy_json(json: &str, major: u32) -> String {
    assert!(major < 34);
    let mut document: serde_json::Value = serde_json::from_str(json).unwrap();
    let provenance = document["provenance"].as_object_mut().unwrap();
    provenance.remove("scenario_initial_state");
    provenance.insert(
        "suite_version".into(),
        format!("ess-conformance/{major}").into(),
    );
    serde_json::to_string(&document).unwrap()
}

/// A fresh suite as synthesis wrote it before beyond10x/ess#273, for a test about another construct.
///
/// An event expectation comparing a captured identity is `expect_event_values`, suite/18 vocabulary;
/// before #273 synthesis wrote the same step as `expect_event` carrying its literal values alone.
/// A suite relabelled below /18 would otherwise be refused for that step before the construct the
/// test is about, and a pin of pre-#273 bytes would move for it. Only steps whose values are all
/// literals and instances, at least one an instance, are rewritten: a fixture or an observed value
/// is older vocabulary and stays.
#[allow(dead_code)]
pub fn without_captured_identities(json: &str) -> String {
    let mut document: serde_json::Value = serde_json::from_str(json).unwrap();
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] != "expect_event_values" {
                continue;
            }
            let payload = step["payload"].as_object().unwrap();
            let kind = |value: &serde_json::Value, name: &str| value["kind"] == name;
            if !payload
                .values()
                .all(|value| kind(value, "literal") || kind(value, "instance"))
                || !payload.values().any(|value| kind(value, "instance"))
            {
                continue;
            }
            let literals: serde_json::Map<String, serde_json::Value> = payload
                .iter()
                .filter(|(_, value)| kind(value, "literal"))
                .map(|(key, value)| (key.clone(), value["value"].clone()))
                .collect();
            let object = step.as_object_mut().unwrap();
            object.insert("step".into(), "expect_event".into());
            if literals.is_empty() {
                object.remove("payload");
            } else {
                object.insert("payload".into(), serde_json::Value::Object(literals));
            }
        }
    }
    serde_json::to_string(&document).unwrap()
}
