//! Omitted content cannot be turned into patches that claim to repair it.

mod support;

#[test]
fn namespace_topology_refuses_projection_and_preserves_its_owner() {
    let ir = support::compile(include_str!(
        "../../infra-compiler/tests/fixtures/namespace-topology.json"
    ));
    let before = serde_json::to_value(ir.document()).unwrap();
    let spec =
        infra_spec::read_spec(&support::read("examples/k3d-dev-cluster/expected.yaml")).unwrap();
    assert!(infra_project::project(&spec, &ir).is_err());
    assert_eq!(serde_json::to_value(ir.document()).unwrap(), before);
}
