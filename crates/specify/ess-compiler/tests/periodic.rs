//! Periodic causes resolve without a publisher, preserving required host authority.
use ess_compiler::{resolve::compile_locating, source::SourceMap};
use ess_domain::{system::Source, RawSpecFile, Specification};

#[test]
fn periodic_cause_resolves_host_types_without_event_or_publisher() {
    let source = include_str!("../../ess-domain/tests/fixtures/periodic.yaml");
    let spec = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("periodic.yaml", source);
    let ir = compile_locating(&spec, &sources, &["periodic.yaml"]).unwrap();
    let binding = ir.bindings().values().next().unwrap();
    assert!(binding.cause.event().is_none());
    let periodic = binding.cause.periodic().unwrap();
    assert_eq!(periodic.contract.every.seconds(), 2);
    assert_eq!(periodic.context.len(), 1);
    assert_eq!(periodic.read.len(), 1);
    let graph = ess_compiler::graph::SemanticDependencyGraph::of(&ir);
    assert!(graph
        .edges()
        .any(|edge| edge.relation == ess_compiler::graph::DependencyRelation::HostedBy));
    assert!(!graph
        .edges()
        .any(|edge| edge.relation == ess_compiler::graph::DependencyRelation::ReactsTo));
    let value = serde_json::to_value(binding).unwrap();
    assert_eq!(value["mapping"][0]["value"]["kind"], "host_context");
    assert_eq!(value["mapping"][1]["value"]["kind"], "host_read");
}
