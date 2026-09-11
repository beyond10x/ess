//! Every relevant projection retains declared paths and typed presence.
use ess_compiler::{resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

#[test]
fn descriptions_wire_projections_and_graph_retain_the_shared_accessor_contract() {
    let text = include_str!("../../ess-synth/tests/fixtures/bounded-accessor.yaml");
    let spec = Specification::assemble([(
        Source::new("accessor.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("accessor.yaml", text);
    let ir = compile_locating(&spec, &sources, &["accessor.yaml"]).unwrap();
    let artifacts = ess_gen::generate_all(&ir).unwrap();
    for prefix in ["docs/", "asyncapi/", "openapi/"] {
        let content = artifacts
            .iter()
            .filter(|(path, _)| path.starts_with(prefix))
            .map(|(_, artifact)| artifact.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        if prefix == "asyncapi/" {
            assert!(content.contains("event_accessor"), "{prefix}");
            assert!(content.contains("may_miss: true"));
            assert!(content.contains("upstream_status"));
        } else {
            assert!(content.contains("event.wrapped.body.status"), "{prefix}");
        }
        if prefix == "openapi/" {
            assert!(content.contains("x-ess-binding-accessors"));
        }
    }
    assert!(ess_gen::SystemGraph::of(&ir)
        .dot()
        .contains("event.wrapped.body.status"));
}
