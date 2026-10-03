//! Relabel a fresh suite for a legacy vocabulary test without carrying new provenance authority.
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
