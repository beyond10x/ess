//! The same serialized one-time contract will be consumed by every runner.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::AdmittedSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const CONSTRAINED_SOURCE: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ntypes:\n  - name: credentials.api.Secret\n    kind: newtype\n    of: String\n    alphabet: abcdef\n    prefix: ab\n    invariants: [value.count >= 4]\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: credentials.api.Secret}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";

fn document() -> serde_json::Value {
    document_with("")
}

fn document_with(extra: &str) -> serde_json::Value {
    let source = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";
    document_from_source(&format!("{source}{extra}"))
}

fn document_from_source(source: &str) -> serde_json::Value {
    let spec = Specification::assemble([(
        Source::new("contract.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let response = ess_conformance::one_time_response::Response::of(
        &ir,
        ir.commands().values().next().unwrap(),
    )
    .unwrap();
    let suite = ess_conformance::synthesize(&ir).suite;
    let mut value = serde_json::to_value(suite).unwrap();
    if value["scenarios"].as_object().unwrap().is_empty() {
        // Admission fixtures author the same invocation explicitly when the legacy
        // synthesizer cannot yet witness constrained returns. This is no synthesis claim.
        let mut authored = document();
        authored["provenance"] = value["provenance"].clone();
        value = authored;
    }
    value["provenance"]["suite_version"] = "ess-conformance/34".into();
    for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "expect_direct_response" {
                step["response"]["fields"] = serde_json::to_value(&response.fields).unwrap();
                step["response"]["declarations"] =
                    serde_json::to_value(&response.declarations).unwrap();
            }
        }
        scenario["one_time_response"] = serde_json::json!({
            "origins": [{
                "command": "credentials.api.Issue",
                "outcome": {"command": "credentials.api.Issue", "outcome": "issued"},
                "response": response,
                "fields": ["secret"]
            }],
            "required_origins": [{"command": "credentials.api.Issue", "outcome": "issued"}],
            "events": [],
            "event_windows": []
        });
    }
    assert!(!value["scenarios"].as_object().unwrap().is_empty());
    value
}

#[test]
fn one_time_closed_trace_contract_is_admitted() {
    let bytes = serde_json::to_string(&document()).unwrap();
    AdmittedSuite::from_json(&bytes).expect("suite34 carries closed one-time authority");
}

#[test]
fn one_time_old_envelope_refuses_new_authority() {
    let mut value = document();
    value["provenance"]["suite_version"] = "ess-conformance/28".into();
    assert!(AdmittedSuite::from_json(&serde_json::to_string(&value).unwrap()).is_err());
}

#[test]
fn one_time_forged_exemption_is_refused() {
    let mut value = document();
    for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
        scenario["one_time_response"]["origins"][0]["fields"] = serde_json::json!(["absent"]);
    }
    assert!(AdmittedSuite::from_json(&serde_json::to_string(&value).unwrap()).is_err());
}

#[test]
fn shared_admission_vectors_are_closed_and_reproducible() {
    let valid = document();
    let mut vectors = vec![("valid-string", true, valid.clone())];
    for (name, key, replacement) in [
        ("empty-origins", "origins", serde_json::json!([])),
        (
            "empty-required-origins",
            "required_origins",
            serde_json::json!([]),
        ),
        (
            "unknown-required-origin",
            "required_origins",
            serde_json::json!([{"command":"credentials.api.Issue","outcome":"missing"}]),
        ),
        ("unknown-policy-key", "exempt_all", serde_json::json!(true)),
        (
            "invalid-window",
            "event_windows",
            serde_json::json!([{"event":"credentials.api.Issued", "after_step":99999, "within_ms":10}]),
        ),
    ] {
        let mut value = valid.clone();
        for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
            scenario["one_time_response"][key] = replacement.clone();
        }
        vectors.push((name, false, value));
    }
    for (name, key, replacement) in [
        ("empty-fields", "fields", serde_json::json!([])),
        (
            "duplicate-fields",
            "fields",
            serde_json::json!(["secret", "secret"]),
        ),
        ("unknown-field", "fields", serde_json::json!(["missing"])),
        (
            "wrong-command",
            "command",
            serde_json::json!("credentials.api.Other"),
        ),
        (
            "unknown-origin-key",
            "captured_value",
            serde_json::json!("untrusted"),
        ),
    ] {
        let mut value = valid.clone();
        for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
            scenario["one_time_response"]["origins"][0][key] = replacement.clone();
        }
        vectors.push((name, false, value));
    }
    vectors.extend(event_vectors());
    let mut missing_source = valid.clone();
    for scenario in missing_source["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        scenario["source"] = serde_json::json!([]);
    }
    vectors.push(("missing-source-authority", false, missing_source));
    for (name, admitted, value) in vectors {
        assert_vector(name, admitted, &value);
    }
}

fn assert_vector(name: &str, admitted: bool, value: &serde_json::Value) {
    let fixture_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/one-time-response");
    let bytes = format!("{}\n", serde_json::to_string_pretty(value).unwrap());
    assert_eq!(AdmittedSuite::from_json(&bytes).is_ok(), admitted, "{name}");
    if let Some(output) = std::env::var_os("ESS_ONE_TIME_VECTOR_OUT") {
        std::fs::create_dir_all(&output).unwrap();
        std::fs::write(
            std::path::Path::new(&output).join(format!("{name}.json")),
            &bytes,
        )
        .unwrap();
    } else {
        assert_eq!(
            std::fs::read_to_string(fixture_root.join(format!("{name}.json"))).unwrap(),
            bytes,
            "{name} vector drift"
        );
    }
}

#[test]
fn shared_constrained_vectors_preserve_and_bound_the_authority() {
    let valid = document_from_source(CONSTRAINED_SOURCE);
    assert_vector("valid-constrained-string", true, &valid);
    for (name, path, replacement) in [
        (
            "constraint-unknown-name",
            "constraints",
            serde_json::json!({"credentials.api.Other":{"alphabet":"abc","prefix":null,"invariants":[]}}),
        ),
        (
            "constraint-non-string",
            "declarations/credentials.api.Secret/of",
            serde_json::json!("Integer"),
        ),
        (
            "constraint-unknown-key",
            "constraints/credentials.api.Secret/exempt",
            serde_json::json!(true),
        ),
        (
            "constraint-invalid-alphabet",
            "constraints/credentials.api.Secret/alphabet",
            serde_json::json!(42),
        ),
        (
            "constraint-conflicting-prefix",
            "constraints/credentials.api.Secret/prefix",
            serde_json::json!("z"),
        ),
        (
            "constraint-invalid-predicate",
            "constraints/credentials.api.Secret/invariants",
            serde_json::json!([{"op":"untrusted"}]),
        ),
        (
            "constraint-unknown-value-path",
            "constraints/credentials.api.Secret/invariants",
            serde_json::json!(["missing.count >= 4"]),
        ),
    ] {
        let mut value = valid.clone();
        for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
            let mut target = &mut scenario["one_time_response"]["origins"][0]["response"];
            for part in path.split('/') {
                target = &mut target[part];
            }
            *target = replacement.clone();
        }
        assert_vector(name, false, &value);
    }
}

fn event_vectors() -> Vec<(&'static str, bool, serde_json::Value)> {
    let mut with_event = document_with(
        "events:\n  - name: credentials.api.Issued\n    fields: [{name: audit, type: String}]\n",
    );
    for scenario in with_event["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        scenario["source"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({"kind":"event","name":"credentials.api.Issued"}));
        scenario["one_time_response"]["events"] =
            serde_json::json!([{"event":"credentials.api.Issued","within_ms":10}]);
        scenario["one_time_response"]["event_windows"] = serde_json::json!([
            {"event":"credentials.api.Issued","after_step":0,"within_ms":0},
            {"event":"credentials.api.Issued","after_step":0,"within_ms":10}
        ]);
    }
    let mut vectors = vec![("valid-event-window", true, with_event.clone())];
    for (name, windows) in [
        ("omitted-event-windows", serde_json::json!([])),
        (
            "omitted-delayed-window",
            serde_json::json!([{"event":"credentials.api.Issued","after_step":0,"within_ms":0}]),
        ),
    ] {
        let mut value = with_event.clone();
        for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
            scenario["one_time_response"]["event_windows"] = windows.clone();
        }
        vectors.push((name, false, value));
    }
    vectors
}

#[test]
fn marked_constrained_newtype_authority_is_carried_from_ir() {
    let spec = Specification::assemble([(
        Source::new("constrained.yaml"),
        RawSpecFile::parse(CONSTRAINED_SOURCE).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let response = ess_conformance::one_time_response::Response::of(
        &ir,
        ir.commands().values().next().unwrap(),
    )
    .unwrap();
    assert_eq!(response.constraints.len(), 1);
    let rules = response.constraints.values().next().unwrap();
    assert_eq!(rules.alphabet.as_deref(), Some("abcdef"));
    assert_eq!(rules.prefix.as_deref(), Some("ab"));
    assert_eq!(rules.invariants.len(), 1);
    response.validate().unwrap();
}
