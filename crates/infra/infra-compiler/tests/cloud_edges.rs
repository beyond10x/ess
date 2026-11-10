//! A service account's cloud role and a load-balancer service's external hostnames in the IR:
//! typed fields read from named annotation keys and the service status, never an annotation bag,
//! and `infra-ir/4` only where one of them is present.

use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;
use infra_domain::InfraCode;
use serde_json::{json, Value};

const ROLE: &str = "arn:aws:iam::000000000000:role/shop-api";

/// A synthetic cluster: one plain service account, one plain service, one Secret key.
fn base_bundle() -> Value {
    json!({
        "format": "infra-observation/3",
        "context": "synthetic",
        "scanned_at": "2026-11-10T00:00:00Z",
        "scout_version": "0.92.0",
        "kinds": {
            "namespaces": { "items": [ { "metadata": { "name": "shop", "uid": "ns-1" } } ] },
            "nodes": { "items": [] },
            "deployments": { "items": [] },
            "statefulsets": { "items": [] },
            "daemonsets": { "items": [] },
            "pods": { "items": [] },
            "services": { "items": [
                { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "sv-1",
                                "annotations": { "unrelated.example.internal/note": "kept out" } },
                  "spec": { "ports": [ { "port": 8080 } ] } }
            ] },
            "ingresses": { "items": [] },
            "configmaps": { "items": [] },
            "secrets": { "items": [
                { "metadata": { "name": "creds", "namespace": "shop", "uid": "se-1" },
                  "type": "Opaque", "data": { "token": { "present": true } } }
            ] },
            "serviceaccounts": { "items": [
                { "metadata": { "name": "shop-api", "namespace": "shop", "uid": "sa-1" } }
            ] },
            "persistentvolumeclaims": { "items": [] }
        }
    })
}

/// The base cluster with an IRSA role on its account and its service behind a load balancer.
fn cloud_bundle() -> Value {
    let mut bundle = base_bundle();
    bundle["kinds"]["serviceaccounts"]["items"][0]["metadata"]["annotations"] =
        json!({ "eks.amazonaws.com/role-arn": ROLE, "unrelated.example.internal/note": "x" });
    let service = &mut bundle["kinds"]["services"]["items"][0];
    service["metadata"]["annotations"] = json!({
        "service.beta.kubernetes.io/aws-load-balancer-scheme": "internal",
        "unrelated.example.internal/note": "kept out"
    });
    service["spec"]["type"] = json!("LoadBalancer");
    service["status"] = json!({ "loadBalancer": { "ingress": [
        { "hostname": "shop-api.elb.example.internal" }
    ] } });
    bundle
}

fn document(bundle: Value) -> Value {
    let raw: RawBundle = serde_json::from_value(bundle).expect("the bundle parses");
    let ir = infra_compiler::compile(&Observation::try_from(raw).expect("the bundle is valid"));
    serde_json::to_value(ir.document()).expect("the document serializes")
}

#[test]
fn a_role_annotation_and_a_load_balancer_status_become_typed_fields_in_infra_ir_4() {
    let document = document(cloud_bundle());
    assert_eq!(document["format"], "infra-ir/4");
    let model = &document["model"];
    assert_eq!(
        model["service_accounts"]["shop/shop-api"]["cloud_identity"],
        json!({ "provider": "aws", "role_arn": ROLE })
    );
    assert_eq!(
        model["services"]["shop/shop-api"]["load_balancer"],
        json!({ "hostnames": ["shop-api.elb.example.internal"], "scheme": "internal" })
    );
    assert!(
        !document.to_string().contains("kept out"),
        "no other annotation enters the model"
    );
    let read = infra_compiler::read_document(&document).expect("IR/4 reads back");
    assert_eq!(
        serde_json::to_value(read.document()).expect("serializes"),
        document
    );
}

#[test]
fn without_either_field_the_model_keeps_infra_ir_3_and_its_digest() {
    let base = document(base_bundle());
    assert_eq!(base["format"], "infra-ir/3");
    assert!(!base.to_string().contains("cloud_identity"));
    assert!(!base.to_string().contains("load_balancer"));
    // A load-balancer service with nothing assigned yet records nothing new.
    let mut pending = base_bundle();
    pending["kinds"]["services"]["items"][0]["spec"]["type"] = json!("LoadBalancer");
    let pending = document(pending);
    assert_eq!(pending["format"], "infra-ir/3");
    assert!(pending["model"]["services"]["shop/shop-api"]
        .get("load_balancer")
        .is_none());
}

#[test]
fn a_document_claiming_infra_ir_3_with_a_cloud_field_is_refused() {
    let mut claimed = document(cloud_bundle());
    claimed["format"] = json!("infra-ir/3");
    let canonical = serde_json::to_vec(&claimed["model"]).expect("model");
    claimed["digest"] = json!(infra_compiler::digest_of_canonical(&canonical));
    let errors = infra_compiler::read_document(&claimed).expect_err("cloud fields are IR/4");
    assert!(errors.contains(InfraCode::IrMalformed), "{errors}");
}
