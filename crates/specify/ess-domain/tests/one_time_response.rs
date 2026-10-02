//! Source authority for the one-time disclosure contract, independent of a runner.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../../../docs/design/one-time-response-values.example.yaml");

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("one-time.yaml"), raw)])
        .map_err(|error| error.to_string())
}

#[test]
fn one_time_string_origin_is_admitted_and_round_trips() {
    let spec = assemble(MODEL).expect("source21 admits a one-time String origin");
    let command = spec.commands().values().next().unwrap();
    let serialized = serde_json::to_value(&command.outcomes[0]).unwrap();
    assert_eq!(
        serialized["one_time_response"],
        serde_json::json!(["secret"])
    );
}

#[test]
fn one_time_transparent_constrained_string_is_admitted() {
    let text = MODEL.replace("type: String", "type: credentials.api.Secret")
        + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    prefix: tok_\n    invariants: [value.count >= 8]\n";
    assemble(&text).expect("a constrained String wrapper retains its authority");
}

#[test]
fn one_time_invalid_contracts_are_named_refusals() {
    for text in [
        MODEL.replace("ess/21", "ess/20"),
        MODEL.replace("[secret]", "[]"),
        MODEL.replace("[secret]", "[secret, secret]"),
        MODEL.replace("[secret]", "[missing]"),
        MODEL.replace("returns: true", "returns: false"),
        MODEL.replace("type: String", "type: 'Optional<String>'"),
        MODEL.replace("type: String", "type: Integer"),
        MODEL.replace("type: String", "type: 'List<String>'"),
    ] {
        let error = assemble(&text).expect_err("invalid policy must be refused");
        assert!(error.contains("one_time_response"), "{error}");
    }
}

#[test]
fn one_time_explicit_null_is_not_an_absent_policy() {
    assemble(&MODEL.replace("[secret]", "null"))
        .expect_err("explicit null must not erase the disclosure authority");
}

#[test]
fn one_time_nested_event_flow_is_refused_by_its_origin_only() {
    let text = MODEL.replace("        returns: true", "        returns: true\n        emits: [credentials.api.Issued]\n        payload:\n          credentials.api.Issued:\n            value: {nested: {response: secret}}")
        + "\ntypes:\n  - name: credentials.api.Envelope\n    kind: struct\n    fields: [{name: nested, type: String}]\nevents:\n  - name: credentials.api.Issued\n    fields: [{name: value, type: credentials.api.Envelope}]\n";
    let error = assemble(&text).expect_err("nested event flow discloses the origin");
    assert!(error.contains("one_time_response"), "{error}");
    assemble(&text.replace("        one_time_response: [secret]\n", ""))
        .expect("the same mapping from an unmarked origin remains valid");
}

#[test]
fn one_time_replay_refusal_preserves_the_unmarked_replay_contract() {
    let text = include_str!("../../../verify/ess-conformance/tests/fixtures/retained-replay.yaml")
        .replace("ess/7", "ess/21")
        .replace(
            "      - {name: revision_id, type: Uuid}",
            "      - {name: secret, type: String}\n      - {name: revision_id, type: Uuid}",
        )
        .replace(
            "      - name: seeded",
            "      - name: seeded\n        returns: true",
        );
    assemble(&text).expect("unmarked full-result replay remains valid");
    let marked = text.replace(
        "        returns: true",
        "        returns: true\n        one_time_response: [secret]",
    );
    let error = assemble(&marked).expect_err("retained replay would redisclose the origin");
    assert!(
        error.contains("one_time_response") && error.contains("replays"),
        "{error}"
    );
}

#[test]
fn one_time_opaque_reading_is_a_named_refusal() {
    let text = MODEL.replace("type: String", "type: credentials.api.Secret")
        + "\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n";
    let error = assemble(&text).expect_err("opaque reading is not a plaintext String contract");
    assert!(error.contains("one_time_response"), "{error}");
    assemble(&text.replace("        one_time_response: [secret]\n", ""))
        .expect("the reading itself is valid without a one-time policy");
}

#[test]
fn one_time_direct_event_flow_is_refused_but_unmarked_flow_is_admitted() {
    let text = MODEL.replace("        returns: true", "        returns: true\n        emits: [credentials.api.Issued]\n        payload:\n          credentials.api.Issued:\n            value: {response: secret}")
        + "\nevents:\n  - name: credentials.api.Issued\n    fields:\n      - {name: value, type: String}\n";
    let error = assemble(&text).expect_err("event flow discloses the origin");
    assert!(error.contains("one_time_response"), "{error}");
    assemble(&text.replace("        one_time_response: [secret]\n", ""))
        .expect("an unmarked outcome keeps existing response flow semantics");
}

#[test]
fn legacy_outcome_bytes_omit_one_time_policy() {
    let text = MODEL
        .replace("ess/21", "ess/20")
        .replace("        one_time_response: [secret]\n", "");
    let spec = assemble(&text).unwrap();
    let serialized = serde_json::to_string(spec.commands().values().next().unwrap()).unwrap();
    assert!(!serialized.contains("one_time_response"));
}

#[test]
fn generated_schema_preserves_the_required_array_shape() {
    let schema = schemars::schema_for!(RawSpecFile);
    let value = serde_json::to_value(&schema).unwrap();
    let policy = &value["definitions"]["RawOutcome"]["properties"]["one_time_response"];
    assert_eq!(policy["type"], "array");
    assert_eq!(policy["items"]["type"], "string");
    let written = format!("{}\n", serde_json::to_string_pretty(&schema).unwrap());
    if let Some(output) = std::env::var_os("ESS_ONE_TIME_SCHEMA_OUT") {
        std::fs::write(output, written).unwrap();
    } else {
        assert_eq!(
            std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../schemas/generated/ess.schema.json")
            )
            .unwrap(),
            written
        );
    }
}
