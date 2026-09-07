//! Qualification survives admission, content addressing and checked transformations.

use infra_compiler::{compile, read_document, InfraIr, UnresolvedTarget};
use infra_domain::Observation;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/namespace-topology.json")).unwrap()
}

#[test]
fn every_workload_kind_round_trips_with_its_canonical_key_spelling() {
    let mut source = fixture();
    let workload = source["kinds"]["deployments"]["items"][0].clone();
    for kind in ["statefulsets", "daemonsets"] {
        source["kinds"][kind]["items"] = json!([workload.clone()]);
    }
    let model = ir(source);
    let document = serde_json::to_value(model.document()).unwrap();
    let read = read_document(&document).unwrap();
    assert_eq!(read.model().workloads.len(), 3);
    for kind in ["deployment", "statefulset", "daemonset"] {
        assert!(read
            .model()
            .workloads
            .contains_key(&format!("app/{kind}/web")));
    }
    assert_eq!(serde_json::to_value(read.document()).unwrap(), document);
}
fn ir(value: Value) -> InfraIr {
    compile(
        &Observation::try_from(serde_json::from_value::<infra_domain::RawBundle>(value).unwrap())
            .unwrap(),
    )
}

#[test]
fn omitted_keys_are_unknown_and_coverage_round_trips_without_changing_legacy_bytes() {
    let model = ir(fixture());
    assert_eq!(model.document().format, "infra-ir/2");
    assert!(!model.model().unresolved.iter().any(|r| matches!(
        r.target,
        UnresolvedTarget::ConfigMapKey { .. } | UnresolvedTarget::SecretKey { .. }
    )));
    let value = serde_json::to_value(model.document()).unwrap();
    let read = read_document(&value).unwrap();
    assert_eq!(serde_json::to_value(read.document()).unwrap(), value);
    assert_eq!(
        model.try_transform(|_| {}).unwrap().digest(),
        model.digest()
    );
    assert!(model.try_transform(|m| m.coverage = None).is_err());
    let legacy: Value = serde_json::from_str(include_str!(
        "../../../../examples/k3d-dev-cluster/cluster.ir.json"
    ))
    .unwrap();
    assert_eq!(
        serde_json::to_value(read_document(&legacy).unwrap().document()).unwrap(),
        legacy
    );
}

#[test]
fn coverage_is_digested_and_cannot_be_downgraded_or_contradicted() {
    let model = ir(fixture());
    let value = serde_json::to_value(model.document()).unwrap();
    for (pointer, replacement) in [
        ("/format", json!("infra-ir/1")),
        ("/model/coverage", Value::Null),
        ("/model/coverage/namespace", json!("elsewhere")),
        ("/model/pods", Value::Null),
    ] {
        let mut candidate = value.clone();
        *candidate.pointer_mut(pointer).unwrap() = replacement;
        assert!(read_document(&candidate).is_err(), "{pointer}");
    }
    assert!(model.try_transform(|m| m.namespaces.clear()).is_err());
    assert!(model
        .try_transform(|m| {
            let object = m.workloads.remove("app/deployment/web").unwrap();
            m.workloads
                .insert("elsewhere/deployment/web".to_owned(), object);
        })
        .is_err());
    assert!(model
        .try_transform(|m| m
            .workloads
            .get_mut("app/deployment/web")
            .unwrap()
            .identity
            .namespace = Some("elsewhere".to_owned()))
        .is_err());
    let changed = model
        .try_transform(|m| m.workloads.get_mut("app/deployment/web").unwrap().replicas = Some(3))
        .unwrap();
    assert_ne!(changed.digest(), model.digest());
    assert_eq!(changed.model().coverage, model.model().coverage);
}

#[test]
fn incomplete_scopes_and_omitted_payloads_are_refused_on_import() {
    for (pointer, replacement) in [
        ("/coverage", Value::Null),
        ("/coverage/namespace", json!("elsewhere")),
        ("/kinds/pods", json!({})),
        ("/kinds/horizontalpodautoscalers", Value::Null),
        (
            "/kinds/configmaps/items/0",
            json!({"metadata": {"name": "settings", "namespace": "app"}, "data": {"private": "synthetic"}}),
        ),
    ] {
        let mut candidate = fixture();
        *candidate.pointer_mut(pointer).unwrap() = replacement;
        let raw = serde_json::from_value::<infra_domain::RawBundle>(candidate).unwrap();
        assert!(Observation::try_from(raw).is_err(), "{pointer}");
    }
}

#[test]
fn unsupported_selector_terms_never_become_match_all_in_either_format() {
    for format in ["infra-observation/1", "infra-observation/2"] {
        for selector in [
            json!({"matchExpressions": [{"key": "app", "operator": "In", "values": ["web"]}]}),
            json!({"matchLabels": {"app": "web"}, "matchExpressions": []}),
            json!({"futureSelectorTerm": {"key": "app"}}),
        ] {
            for kind in ["deployments", "poddisruptionbudgets"] {
                let mut candidate = fixture();
                candidate["format"] = json!(format);
                if format.ends_with("/1") {
                    candidate.as_object_mut().unwrap().remove("coverage");
                }
                if kind == "poddisruptionbudgets" {
                    candidate["kinds"][kind]["items"] =
                        json!([{"metadata": {"name": "guard", "namespace": "app"}, "spec": {}}]);
                }
                candidate["kinds"][kind]["items"][0]["spec"]["selector"] = selector.clone();
                assert!(
                    Observation::try_from(
                        serde_json::from_value::<infra_domain::RawBundle>(candidate).unwrap()
                    )
                    .is_err(),
                    "{format} {kind}"
                );
            }
        }
    }
}

#[test]
fn unsupported_environment_sources_never_become_invented_empty_literals() {
    for source in [
        json!({}),
        json!({"futureSource": {"name": "unknown"}}),
        json!({
            "configMapKeyRef": {"name": "settings", "key": "mode"}, "secretKeyRef": {"name": "creds", "key": "token"}
        }),
    ] {
        let mut candidate = fixture();
        candidate["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]["containers"]
            [0]["env"][0]["valueFrom"] = source;
        let raw: infra_domain::RawBundle = serde_json::from_value(candidate).unwrap();
        assert!(Observation::try_from(raw).is_err());
    }
}
