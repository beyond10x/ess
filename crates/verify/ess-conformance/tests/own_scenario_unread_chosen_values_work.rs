//! A creation arranged as its own scenario is built once however many values of a field its
//! command does not read the callers choose.
//!
//! A creating branch reading several related rows, whose strict search finds no run with the
//! values a caller chose, is arranged as its own scenario is, and kept per model. A creator
//! searched for a row that another command reads tries inputs that differ in fields the creating
//! command only copies into the row. With the chosen values in the key, each one was arranged
//! afresh: a creation reading a grant and the class it names, beside commands deciding on the
//! copied fields of the row it creates, took 585 s to synthesize. Synthesis is held to building the
//! arrangement as often when the copied field has eight values as when it has two: a work bound
//! rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{own_overlay_work, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/a-creator-chosen-values-beside-several-rows.yaml");

/// The tiers a grant may be issued in: `IssueGrant` copies the tier it is sent into the grant and
/// reads it nowhere.
const TIERS: [&str; 8] = ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `text` with `from` replaced once, which it must hold.
fn once(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replacen(from, to, 1)
}

/// The model with a tier of the first `tiers` of [`TIERS`] on the grant, sent to `IssueGrant`
/// and set from it, and `UseGrant` refusing a grant of any tier but the first.
fn desk(tiers: usize) -> String {
    let modes = "  - {name: shop.desk.Mode, kind: enum, variants: [free, asked, denied]}\n";
    let mut text = once(
        MODEL,
        modes,
        &format!(
            "{modes}  - {{name: shop.desk.Tier, kind: enum, variants: [{}]}}\n",
            TIERS[..tiers].join(", ")
        ),
    );
    // The grant's field, then `IssueGrant`'s input: the first two declarations of a mode.
    let mode = "      - {name: mode, type: shop.desk.Mode}\n";
    assert!(
        text.matches(mode).count() >= 2,
        "the grant and its issue carry a mode"
    );
    text = text.replacen(
        mode,
        &format!("{mode}      - {{name: tier, type: shop.desk.Tier}}\n"),
        2,
    );
    text = once(
        &text,
        "mode: input.mode}",
        "mode: input.mode, tier: input.tier}",
    );
    let denied = "              - mode == denied\n";
    once(
        &text,
        denied,
        &format!(
            "{denied}              - {{tier: {{none_of: [{}]}}}}\n",
            TIERS[0]
        ),
    )
}

/// `(built, stopped, asked)`: the own-scenario arrangements carrying chosen values built, the
/// chosen-values runs the nesting bound stopped, and the arrangements carrying chosen values
/// asked for.
fn work(tiers: usize) -> (Synthesis, (u64, u64, u64)) {
    let model = ir(&desk(tiers));
    own_overlay_work(|| synthesize(&model))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// Whether the reader's accepting branch is witnessed.
fn used(synthesis: &Synthesis) -> bool {
    synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "shop.desk.UseGrant/outcome/used")
}

#[test]
fn values_of_a_field_the_creator_does_not_read_build_the_arrangement_once() {
    let (one, (built_one, _, asked_one)) = work(2);
    let (eight, (built_eight, stopped, asked_eight)) = work(TIERS.len());
    assert!(refusals(&one).is_empty(), "refusals: {:#?}", refusals(&one));
    assert!(
        refusals(&eight).is_empty(),
        "refusals: {:#?}",
        refusals(&eight)
    );
    assert!(
        used(&one) && used(&eight),
        "the accepting branch is witnessed"
    );
    assert_eq!(stopped, 0, "chosen-values runs stopped by the bound");
    assert!(
        asked_eight > asked_one,
        "eight tiers ask for the arrangement more often than one: {asked_one} then {asked_eight}"
    );
    assert!(built_one >= 1, "the arrangement is built: {built_one}");
    assert_eq!(
        built_eight, built_one,
        "the arrangements built with one tier and with eight (asked {asked_one} and {asked_eight} \
         times)"
    );
}
