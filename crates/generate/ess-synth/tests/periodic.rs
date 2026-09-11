//! Native event transports disclose the required periodic host integration.
use ess_compiler::{resolve::compile_locating, source::SourceMap};
use ess_domain::{system::Source, RawSpecFile, Specification};
use ess_synth::{synthesize_for, Target};
#[test]
fn native_and_browser_plans_do_not_invent_event_transport_for_periodic_hosts() {
    let text = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    let spec = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("periodic.yaml", text);
    let ir = compile_locating(&spec, &sources, &["periodic.yaml"]).unwrap();
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        let output = synthesize_for(&ir, target).unwrap();
        let plan = output.plan.to_canonical_json();
        assert_eq!(
            plan.matches("periodic_host_required").count(),
            2,
            "{target:?}: {plan}"
        );
        assert!(plan.contains("authenticated-session-status"));
        let code = output
            .artifacts
            .values()
            .map(|artifact| artifact.contents.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!code.contains("event Periodic"));
    }
}
