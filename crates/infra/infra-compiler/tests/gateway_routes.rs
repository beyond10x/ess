//! Gateway API routes in the IR: references resolved or carried, and the `infra-ir/4` decision —
//! a model holding a gateway or route is `/4`, a model without one keeps its `/3` (or `/1`)
//! bytes and digest, and a reader older than `/4` refuses it.

use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;
use infra_domain::InfraCode;
use serde_json::{json, Value};

/// A synthetic cluster without any Gateway API kind: one Service, one Secret key.
fn base_bundle() -> Value {
    json!({
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
                  "spec": { "ports": [ { "port": 8080 } ] } }
            ] },
            "ingresses": { "items": [] },
            "configmaps": { "items": [] },
            "secrets": { "items": [
                { "metadata": { "name": "creds", "namespace": "shop", "uid": "se-1" },
                  "type": "Opaque", "data": { "token": { "present": true } } }
            ] },
            "serviceaccounts": { "items": [] },
            "persistentvolumeclaims": { "items": [] }
        }
    })
}

/// The base cluster with its HTTP edge on the Gateway API.
fn gateway_bundle() -> Value {
    let mut bundle = base_bundle();
    bundle["kinds"]["gateways"] = json!({ "items": [
        { "metadata": { "name": "edge", "namespace": "gateways", "uid": "g-1" },
          "spec": { "gatewayClassName": "shared", "listeners": [
              { "name": "https", "hostname": "*.example.internal", "port": 443, "protocol": "HTTPS" } ] },
          "status": { "addresses": [ { "type": "Hostname", "value": "edge.example.internal" } ] } }
    ] });
    bundle["kinds"]["httproutes"] = json!({ "items": [
        { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "r-1" },
          "spec": {
            "hostnames": ["shop.example.internal"],
            "parentRefs": [
                { "group": "gateway.networking.k8s.io", "kind": "Gateway", "name": "edge",
                  "namespace": "gateways", "sectionName": "https" },
                { "group": "gateway.networking.x-k8s.io", "kind": "ListenerSet", "name": "extra" },
                { "name": "missing-gateway" }
            ],
            "rules": [ {
                "matches": [ { "path": { "type": "PathPrefix", "value": "/" } } ],
                "backendRefs": [
                    { "group": "", "kind": "Service", "name": "shop-api", "port": 8080, "weight": 1 },
                    { "name": "shop-legacy", "port": 80 }
                ]
            } ]
          } }
    ] });
    bundle
}

fn compile(bundle: Value) -> infra_compiler::InfraIr {
    let raw: RawBundle = serde_json::from_value(bundle).expect("the bundle parses");
    infra_compiler::compile(&Observation::try_from(raw).expect("the bundle is valid"))
}

fn document(bundle: Value) -> Value {
    serde_json::to_value(compile(bundle).document()).expect("the document serializes")
}

fn restamp_digest(document: &mut Value) {
    let canonical = serde_json::to_vec(&document["model"]).expect("the model serializes");
    document["digest"] = json!(infra_compiler::digest_of_canonical(&canonical));
}

#[test]
fn a_route_resolves_its_service_and_gateway_and_carries_what_did_not_resolve() {
    let document = document(gateway_bundle());
    assert_eq!(document["format"], "infra-ir/4");
    let model = &document["model"];
    assert_eq!(
        model["gateways"]["gateways/edge"]["addresses"],
        json!([{ "address_type": "Hostname", "value": "edge.example.internal" }])
    );
    let route = &model["http_routes"]["shop/shop-api"];
    assert_eq!(route["hostnames"], json!(["shop.example.internal"]));
    assert_eq!(
        route["parents"][0]["gateway"],
        json!({ "state": "resolved", "key": "gateways/edge" })
    );
    assert_eq!(
        route["parents"][1]["gateway"],
        Value::Null,
        "a ListenerSet parent is carried by its kind, never guessed to be a gateway"
    );
    assert_eq!(route["parents"][1]["kind"], "ListenerSet");
    assert_eq!(
        route["parents"][2]["gateway"],
        json!({ "state": "unresolved", "name": "missing-gateway" })
    );
    let backends = &route["rules"][0]["backends"];
    assert_eq!(
        backends[0]["service"],
        json!({ "state": "resolved", "key": "shop/shop-api" })
    );
    assert_eq!(
        backends[1]["service"],
        json!({ "state": "unresolved", "name": "shop-legacy" })
    );
    let unresolved = model["unresolved"].as_array().expect("an array");
    assert!(unresolved.contains(&json!({
        "from": "http_routes/shop/shop-api",
        "site": "rules[0].backendRefs[1]",
        "target": { "kind": "service", "name": "shop-legacy" }
    })));
    assert!(unresolved.contains(&json!({
        "from": "http_routes/shop/shop-api",
        "site": "parentRefs[2]",
        "target": { "kind": "gateway", "name": "missing-gateway" }
    })));
    assert!(
        !unresolved
            .iter()
            .any(|fact| fact["site"] == "parentRefs[1]"),
        "an unmodelled parent kind is not an unobserved gateway"
    );
    let read = infra_compiler::read_document(&document).expect("IR/4 reads back");
    assert_eq!(read, compile(gateway_bundle()));
}

#[test]
fn a_backend_in_another_namespace_resolves_there() {
    let mut bundle = gateway_bundle();
    bundle["kinds"]["httproutes"]["items"][0]["spec"]["rules"][0]["backendRefs"] = json!([
        { "name": "shop-api", "namespace": "shop", "port": 8080 },
        { "name": "shop-api", "namespace": "gateways", "port": 8080 }
    ]);
    let document = document(bundle);
    let backends = &document["model"]["http_routes"]["shop/shop-api"]["rules"][0]["backends"];
    assert_eq!(backends[0]["service"]["state"], "resolved");
    assert_eq!(backends[1]["service"]["state"], "unresolved");
    assert_eq!(backends[1]["namespace"], "gateways");
}

#[test]
fn a_model_without_gateway_kinds_keeps_its_format_and_its_bytes() {
    let base = document(base_bundle());
    assert_eq!(base["format"], "infra-ir/3");
    let model = base["model"].as_object().expect("an object");
    assert!(!model.contains_key("gateways") && !model.contains_key("http_routes"));
    // Pinned: the digest of this model as the build before the Gateway API kinds wrote it.
    assert_eq!(
        base["digest"],
        "1caa2fd10a42e9167648ee0aa46a5ca8322b589006c14259e69820d7bc8d98f1"
    );
}

#[test]
fn a_reader_older_than_version_four_refuses_it_and_this_reader_holds_the_format_to_the_model() {
    // The installed `/3` reader checks the format before anything else; keep its list
    // independent of this build's constants so a later change cannot make the test agree.
    let old_reader_admits = |document: &Value| {
        ["infra-ir/1", "infra-ir/2", "infra-ir/3"]
            .contains(&document["format"].as_str().unwrap_or(""))
    };
    let four = document(gateway_bundle());
    assert!(
        !old_reader_admits(&four),
        "an old reader must not drop the routes it cannot read"
    );
    assert!(old_reader_admits(&document(base_bundle())));

    let mut claimed_three = four.clone();
    claimed_three["format"] = json!("infra-ir/3");
    restamp_digest(&mut claimed_three);
    let errors = infra_compiler::read_document(&claimed_three).expect_err("routes are IR/4");
    assert!(errors.contains(InfraCode::IrMalformed), "{errors}");

    let mut claimed_four = document(base_bundle());
    claimed_four["format"] = json!("infra-ir/4");
    let errors = infra_compiler::read_document(&claimed_four).expect_err("nothing here is IR/4");
    assert!(errors.contains(InfraCode::IrMalformed), "{errors}");
}

#[test]
fn a_dangling_gateway_or_backend_claim_is_refused() {
    let mut forged = document(gateway_bundle());
    forged["model"]["http_routes"]["shop/shop-api"]["parents"][2]["gateway"] =
        json!({ "state": "resolved", "key": "gateways/missing-gateway" });
    forged["model"]["http_routes"]["shop/shop-api"]["rules"][0]["backends"][1]["service"] =
        json!({ "state": "resolved", "key": "shop/shop-legacy" });
    restamp_digest(&mut forged);
    let errors = infra_compiler::read_document(&forged).expect_err("dangling handles");
    assert!(errors.contains(InfraCode::IrDanglingHandle), "{errors}");
}

#[test]
fn the_namespace_topology_profile_carries_no_gateway_kind() {
    let topology: Value =
        serde_json::from_str(include_str!("fixtures/namespace-topology.json")).expect("JSON");
    let raw: RawBundle = serde_json::from_value(topology.clone()).expect("parses");
    let ir = infra_compiler::compile(&Observation::try_from(raw).expect("the profile is valid"));
    let mut document = serde_json::to_value(ir.document()).expect("serializes");
    for kind in ["gateways", "httproutes"] {
        let mut bundle = topology.clone();
        bundle["kinds"][kind] = json!({ "items": [] });
        let raw: RawBundle = serde_json::from_value(bundle).expect("parses");
        let errors = Observation::try_from(raw).expect_err("not in the profile");
        assert!(
            errors.contains(InfraCode::UnsupportedFormat),
            "{kind}: {errors}"
        );
    }
    document["model"]["gateways"] = json!({});
    restamp_digest(&mut document);
    let errors = infra_compiler::read_document(&document).expect_err("IR/2 holds no gateway");
    assert!(errors.contains(InfraCode::IrMalformed), "{errors}");
}
