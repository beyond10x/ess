//! Adversary pass 3 on M8: the digest-stability claim for producers before 0.32.0.
//!
//! `docs/design/observed-component-bindings.md` says documents without native sidecars from
//! producers before 0.32.0 keep their bytes and digests, and that ESS 0.31.0 never collected init
//! containers. ESS 0.31.0's full `ess import kubernetes --context` scan writes each API object
//! verbatim, so a template with a plain init container carries the `initContainers` key under
//! `scout_version: 0.31.0`. The digest below is what ESS 0.31.0 (`f7de9f82a`) printed for the
//! exact observation in this file, with and without the plain init container.

use infra_domain::observation::Observation;
use infra_domain::raw::RawBundle;

const DIGEST_BY_ESS_0_31_0: &str =
    "a2bd8673d3b7175c66e636cf570b8096f67a475945597041e4c055a01a70f724";

const OBSERVATION: &str = r#"{
  "format": "infra-observation/1",
  "context": "probe",
  "scanned_at": "2026-09-25T08:00:00Z",
  "scout_version": "0.31.0",
  "kinds": {
    "namespaces": { "items": [ { "metadata": { "name": "ns", "uid": "n1" } } ] },
    "nodes": { "items": [] }, "statefulsets": { "items": [] }, "daemonsets": { "items": [] },
    "pods": { "items": [] }, "services": { "items": [] }, "ingresses": { "items": [] },
    "configmaps": { "items": [] }, "secrets": { "items": [] }, "serviceaccounts": { "items": [] },
    "persistentvolumeclaims": { "items": [] },
    "deployments": { "items": [
      { "metadata": { "name": "web", "namespace": "ns", "uid": "u1" },
        "spec": {
          "replicas": 1,
          "selector": { "matchLabels": { "app": "web" } },
          "template": {
            "metadata": { "labels": { "app": "web" } },
            "spec": {
              "initContainers": [ { "name": "migrate", "image": "example/migrate:1" } ],
              "containers": [ { "name": "web", "image": "example/web:1" } ]
            }
          }
        }
      }
    ] }
  }
}"#;

#[test]
fn an_ess_0_31_0_scan_with_only_a_plain_init_container_keeps_its_0_31_0_ir_digest() {
    let raw: RawBundle = serde_json::from_str(OBSERVATION).expect("the bundle parses");
    let observation = Observation::try_from(raw).expect("the observation is valid");
    let ir = infra_compiler::compile(&observation);
    let document = serde_json::to_value(ir.document()).expect("the document serializes");
    let workload = document["model"]["workloads"]
        .as_object()
        .and_then(|all| all.values().next())
        .expect("one workload");
    assert!(
        workload.get("native_sidecars").is_none(),
        "a 0.31.0 document with no native sidecar grew the field: {}",
        workload["native_sidecars"]
    );
    assert_eq!(
        document["digest"], DIGEST_BY_ESS_0_31_0,
        "the IR digest of a 0.31.0 observation without native sidecars moved"
    );
}
