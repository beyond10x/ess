//! An `each:` entry admitting repeated identities (`repeated: last_wins`, ess/35,
//! story:an-each-write-may-take-repeated-identities-in-list-order) stays owed by generated Rust
//! and Go, as every `each:` entry is: generated storage writes no row per element in this cut. The
//! obligation names the key, so an implementor knows the list applies in order, each
//! element to the row as the earlier ones left it.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
const FEED: &str = "components:\n  - component: feed-service\n    owns:\n      domains: [demo.feed]\n    accepts:\n      commands: [demo.feed.AddSource, demo.feed.RunSource]\n    publishes:\n      events: [demo.feed.SourceAdded, demo.feed.SourceRan]\n";

fn model(text: &str) -> EssIr {
    let text = format!("{text}{FEED}");
    let spec = Specification::assemble([(
        Source::new("feed.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn generated_behaviour_names_the_repeated_key_owed() {
    let repeated = EACH
        .replacen("format: ess/23", "format: ess/35", 1)
        .replacen(DISTINCT, "", 1)
        .replacen(
            "each: {in: input.applied, as: doc}",
            "each: {in: input.applied, as: doc, repeated: last_wins}",
            1,
        );
    let ir = model(&repeated);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.feed.RunSource");
        assert!(
            matches!(disposition, Some(SynthesisDisposition::Obligation(owed))
                if owed.reason.describes().contains("repeated: last_wins")),
            "{target:?}: the repeated entry is left to the implementor, named: {disposition:?}"
        );
    }
    // Without the key the obligation reads as it always did.
    let ir = model(EACH);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.feed.RunSource");
        assert!(
            matches!(disposition, Some(SynthesisDisposition::Obligation(owed))
                if !owed.reason.describes().contains("repeated")),
            "{target:?}: {disposition:?}"
        );
    }
}
