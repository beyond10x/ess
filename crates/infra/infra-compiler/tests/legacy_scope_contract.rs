//! Frozen version-1 envelope admission, before adding any persisted qualification field.

use serde::Deserialize;
use serde_json::{json, Value};

// The old reader checks the format before parsing its deny-unknown-fields mirrors. Keep this
// version claim independent of the current writer's constant so upgrading both cannot make
// the migration test silently accept data an installed old reader would refuse.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct LegacyEnvelope {
    format: String,
    provenance: Value,
    digest: String,
    model: Value,
}

fn old_reader_accepts(value: Value) -> bool {
    value["format"] == "infra-ir/1" && serde_json::from_value::<LegacyEnvelope>(value).is_ok()
}

#[test]
fn version_two_cannot_be_silently_admitted_by_the_version_one_envelope() {
    let mut old: Value = serde_json::from_str(include_str!(
        "../../../../examples/k3d-dev-cluster/cluster.ir.json"
    ))
    .expect("frozen legacy IR");
    assert!(old_reader_accepts(old.clone()));
    old["format"] = json!("infra-ir/2");
    assert!(!old_reader_accepts(old));
}

#[test]
fn version_one_cannot_gain_an_envelope_field() {
    let mut old: Value = serde_json::from_str(include_str!(
        "../../../../examples/k3d-dev-cluster/cluster.ir.json"
    ))
    .expect("frozen legacy IR");
    old["coverage"] = json!({"profile": "namespace_topology", "namespace": "app"});
    assert!(!old_reader_accepts(old.clone()));
    assert!(infra_compiler::read_document(&old).is_err());
}
