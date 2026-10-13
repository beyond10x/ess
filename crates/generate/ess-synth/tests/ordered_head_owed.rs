//! The head of an ordered selection stays owed by generated Rust and Go, named by the plan
//! (`docs/design/ordered-related-head.md`, "Targets"): generated storage enumerates no rows by a
//! selector, for guards or values, so a command whose value reads a head is an obligation whose
//! reason names the construct, beside the filtered read's.
//!
//! The command here reads the head with no row-set guard beside it, so the guard's own obligation
//! cannot be what keeps it owed.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const HEAD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/ordered-related-head.yaml");

/// The fixture with `Pick`'s `exists: false` refusal taken away: the head read alone.
fn unguarded() -> String {
    let guard = "      - name: empty
        when_related:
          entity: demo.ranked.Membership
          where: {all: [slot_id == input.slot_id, state == Present]}
          exists: false
        error: demo.ranked.Empty
";
    assert!(HEAD.contains(guard), "the fixture's guard is where it was");
    HEAD.replacen(guard, "", 1)
}

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("ranked.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn generated_behaviour_names_the_ranked_head_owed() {
    let ir = model(&unguarded());
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.ranked.Pick");
        assert!(
            matches!(disposition, Some(SynthesisDisposition::Obligation(owed))
                if owed.reason.describes().contains("first row of an ordered selection")
                    && owed.reason.describes().contains("order_by")),
            "{target:?}: the head read is left to the implementor, named: {disposition:?}"
        );
        // The commands that read no head are still generated.
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.ranked.Join"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
