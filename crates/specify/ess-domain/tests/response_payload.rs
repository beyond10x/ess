//! Actual response sources and explicit ownership preserve legacy payload semantics.
fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("response.yaml"), raw)])
        .map_err(|e| e.to_string())
}
fn model(format: &str, response: &str, payload: &str) -> String {
    format!("format: {format}\nsystem: demo\nversion: v1\ndomain: demo.api\ntypes:\n  - name: demo.api.Item\n    kind: struct\n    fields:\n      - {{name: remaining, type: Integer}}\nevents:\n  - name: demo.api.Returned\n    fields:\n      - {{name: item, type: demo.api.Item}}\n      - {{name: receipt, type: String}}\n  - name: demo.api.External\n    fields:\n      - {{name: body, type: String}}\ncommands:\n  - name: demo.api.Cancel\n{response}    outcomes:\n      - name: cancelled\n        emits: [demo.api.Returned]\n{payload}")
}
const RESPONSE: &str = "    response:\n      - {name: item, type: demo.api.Item}\n";
const PAYLOAD: &str = "        payload:\n          demo.api.Returned:\n            item: {response: item}\n            receipt: {generated: true}\n";
#[test]
fn response_sources_and_generated_fields_admit_and_round_trip() {
    let source = model("ess/4", RESPONSE, PAYLOAD);
    let parsed = spec(&source).unwrap();
    let command = parsed.commands().values().next().unwrap();
    let encoded = serde_yaml::to_string(command).unwrap();
    assert!(encoded.contains("response:"));
    assert!(encoded.contains("generated: true"));
    let raw: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&encoded).unwrap();
    let roundtrip = ess_domain::command::CommandSpec::try_from(raw).unwrap();
    assert_eq!(&roundtrip, command);
}
#[test]
fn incomplete_emitted_payload_is_refused_only_in_new_format() {
    for format in ["ess/1", "ess/2", "ess/3"] {
        spec(&model(format, "", "")).unwrap();
    }
    let error = spec(&model("ess/4", "", "")).unwrap_err();
    assert!(error.contains("no source"), "{error}");
    assert!(error.contains("receipt"), "{error}");
    assert!(!error.contains("External.body"), "{error}");
}
#[test]
fn response_sources_refuse_unknown_types_fields_and_old_formats() {
    for bad in [
        PAYLOAD.replace("response: item", "response: absent"),
        PAYLOAD.replace("response: item", "response: item, generated: true"),
        PAYLOAD.replace("generated: true", "generated: false"),
    ] {
        assert!(spec(&model("ess/4", RESPONSE, &bad)).is_err());
    }
    for format in ["ess/1", "ess/2", "ess/3"] {
        assert!(spec(&model(format, RESPONSE, PAYLOAD)).is_err());
    }
    assert!(spec(&model(
        "ess/4",
        &RESPONSE.replace("demo.api.Item", "Missing"),
        PAYLOAD
    ))
    .is_err());
}
#[test]
fn response_prefixed_string_remains_a_literal() {
    let payload = PAYLOAD.replace("receipt: {generated: true}", "receipt: response.receipt");
    let parsed = spec(&model("ess/4", RESPONSE, &payload)).unwrap();
    let command = parsed.commands().values().next().unwrap();
    let source = command.outcomes[0]
        .payload
        .values()
        .next()
        .unwrap()
        .get("receipt")
        .unwrap();
    assert!(
        matches!(source, ess_domain::command::PayloadSource::Literal { value } if value == "response.receipt")
    );
}
