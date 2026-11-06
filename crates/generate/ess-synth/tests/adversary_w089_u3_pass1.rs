//! Adversary pass 1 on story:an-each-write-may-take-repeated-identities-in-list-order (ess/35,
//! beyond10x/ess#748, wave-2026-11-06-ess U3).
//!
//! The acceptance: "Generated Rust and Go: apply in order, or the command is an obligation naming
//! the key". `determined::affects` names the key only when it reaches the repeated entry: an
//! earlier entry of the branch that is owed for its own reason returns first, and the obligation
//! then reads as for any `affects:` entry, without the key.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
const FEED: &str = "components:\n  - component: feed-service\n    owns:\n      domains: [demo.feed]\n    accepts:\n      commands: [demo.feed.AddSource, demo.feed.RunSource]\n    publishes:\n      events: [demo.feed.SourceAdded, demo.feed.SourceRan]\n";
const AFFECTS: &str = "        affects:\n          - entity: demo.feed.SeenDocument\n            each: {in: input.applied, as: doc}\n";
/// An entry over the source's documents a filter selects, written before the `each:` entry:
/// every document of the source is reset, then the list is applied.
const RESET: &str = "        affects:\n          - entity: demo.feed.SeenDocument\n            where: source_id == input.source_id\n            sets: {revision: 0}\n          - entity: demo.feed.SeenDocument\n            each: {in: input.applied, as: doc, repeated: last_wins}\n";

fn model(text: &str) -> EssIr {
    let text = format!("{text}{FEED}");
    let spec = Specification::assemble([(
        Source::new("feed.yaml"),
        RawSpecFile::parse(&text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture holds:\n{from}");
    text.replacen(from, to, 1)
}

#[test]
fn the_obligation_names_the_key_when_an_earlier_entry_is_owed_too() {
    let text = edit(EACH, "format: ess/23", "format: ess/35");
    let text = edit(&text, DISTINCT, "");
    let text = edit(&text, AFFECTS, RESET);
    let ir = model(&text);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.feed.RunSource");
        let Some(SynthesisDisposition::Obligation(owed)) = disposition else {
            panic!("{target:?}: RunSource stays owed: {disposition:?}");
        };
        assert!(
            owed.reason.describes().contains("repeated: last_wins"),
            "{target:?}: defect: the obligation does not name `repeated: last_wins` when an \
             earlier `affects:` entry of the branch is owed first (determined.rs `affects` returns \
             on the first owed entry): {}",
            owed.reason.describes()
        );
    }
}
