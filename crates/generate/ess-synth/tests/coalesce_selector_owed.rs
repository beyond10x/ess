//! A selector comparing `coalesce(<field>, <literal>)` with a literal (`ess/30`,
//! beyond10x/ess#627) stays owed by generated Rust and Go: generated storage enumerates no rows by
//! a selector, so the command is an obligation whose reason names the selector it reads, as for
//! any other `where:`. `Join`, which reads none, is still generated.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/coalesce-selector.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("picking.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn generated_behaviour_names_the_coalesced_selector_owed() {
    let ir = model(MODEL);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.picking.Pick");
        assert!(
            matches!(disposition, Some(SynthesisDisposition::Obligation(owed))
                if owed.reason.describes().contains("rows a selector selects")
                    && owed.reason.describes().contains("when_related: {entity, where")),
            "{target:?}: the selector is left to the implementor, named: {disposition:?}"
        );
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.picking.Join"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
