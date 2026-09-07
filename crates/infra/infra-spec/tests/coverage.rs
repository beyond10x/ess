//! Native consumers retain uncertainty instead of manufacturing absence claims.

mod support;
use infra_spec::{drift, simulate, DriftRefusal};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../infra-compiler/tests/fixtures/namespace-topology.json"
    ))
    .unwrap()
}
fn compile(value: &Value) -> infra_compiler::InfraIr {
    support::compile(&value.to_string())
}

#[test]
fn graph_and_drift_carry_scope_and_only_compare_matching_coverage() {
    let from = compile(&fixture());
    let mut next = fixture();
    next["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]["containers"][0]
        ["image"] = json!("web:2");
    let to = compile(&next);
    let report = drift(&from, &to).unwrap();
    assert_eq!(report.format, "infra-drift/2");
    assert_eq!(report.coverage, from.model().coverage);
    assert_eq!(report.changes.len(), 1);
    assert!(matches!(
        report.changes[0],
        infra_spec::InfraChange::ImageChanged { .. }
    ));
    let equal = drift(&from, &from).unwrap();
    assert!(equal.is_empty());
    assert!(infra_spec::drift_to_text(&equal).contains("omitted content remains unobserved"));
    next["format"] = json!("infra-observation/1");
    next.as_object_mut().unwrap().remove("coverage");
    assert_eq!(
        drift(&from, &compile(&next)),
        Err(DriftRefusal::DifferentCoverage)
    );
    let elsewhere = fixture().to_string().replace("\"app\"", "\"elsewhere\"");
    assert_eq!(
        drift(&from, &support::compile(&elsewhere)),
        Err(DriftRefusal::DifferentCoverage)
    );
    let graph = infra_analyze::InfraGraph::of(&from);
    let document = infra_analyze::GraphDocument::of(&graph, &from, None);
    assert_eq!(document.format, "infra-graph/2");
    assert_eq!(document.coverage, from.model().coverage);
    assert!(!document.edges.is_empty());
    assert!(graph.mermaid().contains("namespace app;"));
    assert!(graph.restricted_to("app").mermaid().contains("unobserved"));
}

#[test]
fn limited_observation_withholds_diagnosis_invariants_and_intent_verdicts() {
    let ir = compile(&fixture());
    let diagnosis = infra_analyze::diagnose(&ir);
    assert_eq!(diagnosis.findings.len(), 1);
    assert_eq!(
        diagnosis.findings[0].code,
        infra_analyze::DiagCode::ObservationLimited
    );
    assert!(infra_analyze::candidates(&ir).is_empty());
    let simulation = simulate(&support::example_spec(), &ir);
    assert_eq!(simulation.format, "infra-simulation/2");
    assert_eq!(simulation.summary.holds, 0);
    assert_eq!(simulation.summary.gaps, 0);
    assert!(simulation.summary.expectations > 0);
    assert_eq!(
        simulation.summary.undecidable,
        simulation.summary.expectations
    );
    assert!(simulation.to_json().contains("collection_limited"));
    for facts in
        infra_spec::facts::workload_facts(&ir, &infra_analyze::InfraGraph::of(&ir)).values()
    {
        assert_eq!(
            facts.withheld.len(),
            infra_spec::facts::WORKLOAD_FACTS.len()
        );
    }
}

#[test]
fn referenced_nodes_are_not_reported_as_cluster_node_additions() {
    let before = compile(&fixture());
    let mut next = fixture();
    next["kinds"]["nodes"]["items"] = json!([{"metadata": {"name": "node-a", "uid": "node-a"}}]);
    next["kinds"]["pods"]["items"] = json!([{"metadata": {"name": "web-1", "namespace": "app", "uid": "pod-a"}, "spec": {"nodeName": "node-a"}}]);
    let after = compile(&next);
    let report = drift(&before, &after).unwrap();
    assert_eq!(
        report.changes,
        vec![infra_spec::InfraChange::TopologyDigestChanged]
    );
    assert!(!report.changes.iter().any(|change| matches!(
        change,
        infra_spec::InfraChange::Added {
            kind: infra_spec::drift::MemberKind::Node,
            ..
        } | infra_spec::InfraChange::Removed {
            kind: infra_spec::drift::MemberKind::Node,
            ..
        }
    )));
}
