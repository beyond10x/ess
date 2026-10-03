//! Generated disclosure cells have source-owned identities, distinct from authored claims.
use ess_conformance::{AdmittedSuite, ScenarioId};

const PREFIX: &str = "credentials.api.Issue/disclosure/issued/secret";

#[test]
fn an_actor_named_anonymous_is_distinct_from_no_actor() {
    let none =
        ess_conformance::one_time_response::Cell::parse(&format!("{PREFIX}/origin/as/anonymous"))
            .unwrap();
    let mut named = none.clone();
    named.actor = Some(ess_conformance::scenario::ActorRef::new(
        ess_domain::QualifiedName::new("anonymous").unwrap(),
    ));
    assert_ne!(none.to_string(), named.to_string());
    assert_eq!(
        ess_conformance::one_time_response::Cell::parse(&named.to_string()).unwrap(),
        named
    );
}

#[test]
fn marked_field_segment_uses_the_source_field_grammar() {
    for field in ["secret/value", "secret-value", "", "123"] {
        assert!(ess_conformance::one_time_response::Cell::parse(&format!(
            "credentials.api.Issue/disclosure/issued/{field}/origin/as/anonymous"
        ))
        .is_err());
    }
}

#[test]
fn generated_disclosure_id_grammar_round_trips() {
    let vectors: Vec<(bool, String)> = serde_json::from_str(include_str!(
        "fixtures/one-time-response-identifier-grammar.json"
    ))
    .unwrap();
    for (accepted, written) in vectors {
        let parsed = ScenarioId::parse(&written);
        assert_eq!(parsed.is_ok(), accepted, "{written}");
        if let Ok(parsed) = parsed {
            assert_eq!(parsed.to_string(), written);
        }
    }
    for suffix in [
        "origin/as/anonymous",
        "retry/as/actor/credentials.api.Alice",
        "rotation/as/actor/credentials.api.Bob",
        "read/credentials.api.Credentials/as/anonymous",
        "command/credentials.api.Read/accepted/as/actor/credentials.api.Alice",
        "denied/credentials.api.Read/as/actor/credentials.api.Bob",
    ] {
        let written = format!("{PREFIX}/{suffix}");
        assert_eq!(ScenarioId::parse(&written).unwrap().to_string(), written);
    }
    for suffix in [
        "unknown/as/anonymous",
        "retry",
        "retry/as/",
        "read//as/anonymous",
    ] {
        assert!(ScenarioId::parse(&format!("{PREFIX}/{suffix}")).is_err());
    }
}

#[test]
fn disclosure_identity_is_bound_to_its_policy() {
    let base = include_str!("fixtures/one-time-response/valid-string.json");
    let mut value: serde_json::Value = serde_json::from_str(base).unwrap();
    let scenarios = value["scenarios"].as_object_mut().unwrap();
    let old = scenarios.keys().next().unwrap().clone();
    let scenario = scenarios.remove(&old).unwrap();
    scenarios.insert(format!("{PREFIX}/origin/as/anonymous"), scenario);
    AdmittedSuite::from_json(&value.to_string()).unwrap();
    let mut unprotected = value.clone();
    for scenario in unprotected["scenarios"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        scenario
            .as_object_mut()
            .unwrap()
            .remove("one_time_response");
    }
    assert!(AdmittedSuite::from_json(&unprotected.to_string()).is_err());
    let scenarios = value["scenarios"].as_object_mut().unwrap();
    let scenario = scenarios
        .remove(&format!("{PREFIX}/origin/as/anonymous"))
        .unwrap();
    scenarios.insert(
        "credentials.api.Issue/disclosure/issued/missing/origin/as/anonymous".into(),
        scenario,
    );
    assert!(AdmittedSuite::from_json(&value.to_string()).is_err());
}

#[test]
fn an_origin_alone_cannot_stand_in_for_a_followup_cell() {
    let base = include_str!("fixtures/one-time-response/valid-string.json");
    for aspect in ["command/credentials.api.Issue/issued", "rotation", "retry"] {
        let mut value: serde_json::Value = serde_json::from_str(base).unwrap();
        let scenarios = value["scenarios"].as_object_mut().unwrap();
        let old = scenarios.keys().next().unwrap().clone();
        let scenario = scenarios.remove(&old).unwrap();
        scenarios.insert(format!("{PREFIX}/{aspect}/as/anonymous"), scenario);
        assert!(
            AdmittedSuite::from_json(&value.to_string()).is_err(),
            "{aspect}"
        );
    }
}

#[test]
fn immutable_disclosure_identity_admission_vectors() {
    let base: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/one-time-response/valid-string.json")).unwrap();
    for (name, aspect, repeated, accepted) in [
        ("valid-origin", "origin", false, true),
        ("valid-retry", "retry", true, true),
        ("valid-rotation", "rotation", true, true),
        (
            "valid-command",
            "command/credentials.api.Issue/issued",
            true,
            true,
        ),
        ("missing-retry", "retry", false, false),
        ("missing-rotation", "rotation", false, false),
        (
            "missing-command",
            "command/credentials.api.Issue/issued",
            false,
            false,
        ),
        (
            "missing-read",
            "read/credentials.api.Credentials",
            false,
            false,
        ),
        (
            "missing-denied",
            "denied/credentials.api.Issue",
            false,
            false,
        ),
    ] {
        let mut value = base.clone();
        let scenarios = value["scenarios"].as_object_mut().unwrap();
        let old = scenarios.keys().next().unwrap().clone();
        let mut scenario = scenarios.remove(&old).unwrap();
        if repeated {
            let steps = scenario["steps"].as_array_mut().unwrap();
            steps.extend(steps.clone());
        }
        scenarios.insert(format!("{PREFIX}/{aspect}/as/anonymous"), scenario);
        vector(name, accepted, &value);
        if name == "valid-origin" {
            let mut legacy = value.clone();
            legacy["provenance"]["suite_version"] = "ess-conformance/28".into();
            legacy["provenance"]
                .as_object_mut()
                .unwrap()
                .remove("scenario_initial_state");
            vector("old-envelope", false, &legacy);
            let mut absent = value;
            for scenario in absent["scenarios"].as_object_mut().unwrap().values_mut() {
                scenario
                    .as_object_mut()
                    .unwrap()
                    .remove("one_time_response");
            }
            vector("missing-policy", false, &absent);
        }
    }
}

fn vector(name: &str, admitted: bool, value: &serde_json::Value) {
    let bytes = format!("{}\n", serde_json::to_string_pretty(value).unwrap());
    assert_eq!(AdmittedSuite::from_json(&bytes).is_ok(), admitted, "{name}");
    if let Some(output) = std::env::var_os("ESS_ONE_TIME_ID_VECTOR_OUT") {
        std::fs::create_dir_all(&output).unwrap();
        std::fs::write(
            std::path::Path::new(&output).join(format!("{name}.json")),
            bytes,
        )
        .unwrap();
    } else {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/one-time-response-identifiers");
        assert_eq!(
            std::fs::read_to_string(root.join(format!("{name}.json"))).unwrap(),
            bytes,
            "{name} vector drift"
        );
    }
}
