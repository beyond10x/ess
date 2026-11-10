//! The cluster's edge on a synthetic cluster: every edge relation minted, the renderings carry
//! it, and a route to an unobserved service is diagnosed like any other missing reference. Its
//! cloud too: an account's IRSA role and a load-balancer service's assigned hostname.

use infra_analyze::{diagnose, DiagCode, EdgeRelation, GraphDocument, InfraGraph, NodeKind};
use infra_compiler::InfraIr;
use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;
use serde_json::json;

/// A synthetic cluster whose HTTP edge is the Gateway API. No name here is a real one.
fn edge_ir() -> InfraIr {
    let bundle = json!({
        "format": "infra-observation/3",
        "context": "synthetic",
        "scanned_at": "2026-11-10T00:00:00Z",
        "scout_version": "0.92.0",
        "kinds": {
            "namespaces": { "items": [
                { "metadata": { "name": "shop", "uid": "ns-1" } },
                { "metadata": { "name": "gateways", "uid": "ns-2" } }
            ] },
            "nodes": { "items": [] },
            "deployments": { "items": [] },
            "statefulsets": { "items": [] },
            "daemonsets": { "items": [] },
            "pods": { "items": [] },
            "services": { "items": [
                { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "sv-1" },
                  "spec": { "ports": [ { "port": 8080 } ] } },
                { "metadata": { "name": "shop-public", "namespace": "shop", "uid": "sv-2",
                    "annotations": {
                      "service.beta.kubernetes.io/aws-load-balancer-scheme": "internet-facing" } },
                  "spec": { "type": "LoadBalancer", "ports": [ { "port": 443 } ] },
                  "status": { "loadBalancer": { "ingress": [
                      { "hostname": "shop-public.elb.example.internal" } ] } } }
            ] },
            "ingresses": { "items": [] },
            "configmaps": { "items": [] },
            "secrets": { "items": [] },
            "serviceaccounts": { "items": [
                { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "sa-1",
                    "annotations": {
                      "eks.amazonaws.com/role-arn": "arn:aws:iam::000000000000:role/shop-api" } } }
            ] },
            "persistentvolumeclaims": { "items": [] },
            "gateways": { "items": [
                { "metadata": { "name": "edge", "namespace": "gateways", "uid": "g-1" },
                  "spec": { "gatewayClassName": "shared", "listeners": [
                      { "name": "https", "port": 443, "protocol": "HTTPS" } ] },
                  "status": { "addresses": [
                      { "type": "Hostname", "value": "edge.example.internal" } ] } }
            ] },
            "httproutes": { "items": [
                { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "r-1" },
                  "spec": {
                    "hostnames": ["shop.example.internal"],
                    "parentRefs": [ { "name": "edge", "namespace": "gateways" } ],
                    "rules": [ { "backendRefs": [
                        { "name": "shop-api", "port": 8080 },
                        { "name": "shop-legacy", "port": 80 }
                    ] } ]
                  } }
            ] }
        }
    });
    let raw: RawBundle = serde_json::from_value(bundle).expect("the bundle parses");
    infra_compiler::compile(&Observation::try_from(raw).expect("the bundle is valid"))
}

#[test]
fn every_edge_relation_is_minted_from_the_synthetic_edge_cluster() {
    let graph = InfraGraph::of(&edge_ir());
    let minted: std::collections::BTreeSet<EdgeRelation> =
        graph.edges().map(|edge| edge.relation).collect();
    for relation in EdgeRelation::EDGE {
        assert!(
            minted.contains(&relation),
            "no edge carries `{}` ({relation:?})",
            relation.verb()
        );
    }
}

#[test]
fn a_route_attaches_to_its_gateway_routes_to_its_service_and_the_gateway_has_its_hostname() {
    let graph = InfraGraph::of(&edge_ir());
    let edges: Vec<String> = graph.edges().map(|edge| edge.to_string()).collect();
    for expected in [
        "route shop/shop-api is attached to gateway gateways/edge",
        "route shop/shop-api routes to service shop/shop-api",
        "gateway gateways/edge is exposed at load balancer edge.example.internal",
        "service account shop/shop-api assumes cloud role arn:aws:iam::000000000000:role/shop-api",
        "service shop/shop-public is exposed at load balancer shop-public.elb.example.internal",
    ] {
        assert!(
            edges.iter().any(|edge| edge == expected),
            "{expected}: {edges:?}"
        );
    }
    assert!(
        !graph
            .nodes()
            .iter()
            .any(|node| node.kind == NodeKind::Service && node.key == "shop/shop-legacy"),
        "an unobserved backend is a fact of the IR, not a node"
    );

    let mermaid = graph.mermaid();
    assert!(mermaid.contains("[\"gateway edge\"]"), "{mermaid}");
    assert!(mermaid.contains("[\"route shop-api\"]"), "{mermaid}");
    assert!(
        mermaid.contains("    lb0[\"load balancer edge.example.internal\"]"),
        "a hostname is drawn whole, outside every namespace: {mermaid}"
    );
    assert!(mermaid.contains("|\"is attached to\"|"), "{mermaid}");
    assert!(
        mermaid.contains("    role0[\"cloud role arn:aws:iam::000000000000:role/shop-api\"]"),
        "a role is drawn whole, outside every namespace: {mermaid}"
    );
    assert!(mermaid.contains("|\"assumes\"|"), "{mermaid}");

    let ir = edge_ir();
    let document: serde_json::Value =
        serde_json::from_str(&GraphDocument::of(&graph, &ir, None).to_json()).expect("JSON");
    assert!(document["nodes"]
        .as_array()
        .expect("nodes")
        .contains(&json!({ "kind": "load_balancer", "key": "edge.example.internal" })));
    assert!(document["nodes"].as_array().expect("nodes").contains(
        &json!({ "kind": "cloud_role", "key": "arn:aws:iam::000000000000:role/shop-api" })
    ));
    assert!(document["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .any(|edge| edge["relation"] == "assumes"
            && edge["sites"] == json!(["metadata.annotations[eks.amazonaws.com/role-arn]"])));
    assert!(document["edges"]
        .as_array()
        .expect("edges")
        .iter()
        .any(|edge| edge["relation"] == "attached-to"));
}

#[test]
fn a_route_to_an_unobserved_service_is_a_missing_reference() {
    let diagnosis = diagnose(&edge_ir());
    let finding = diagnosis
        .findings
        .iter()
        .find(|finding| {
            finding.code == DiagCode::MissingReference
                && finding.subject == "http_routes/shop/shop-api"
        })
        .expect("INFRA-DIAG-002 on the route");
    assert_eq!(finding.site.as_deref(), Some("rules[0].backendRefs[1]"));
    assert_eq!(finding.evidence["name"], "shop-legacy");
}
