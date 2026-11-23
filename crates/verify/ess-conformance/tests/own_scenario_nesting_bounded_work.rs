//! A setup run arranged as its branch's own scenario nests at most one level deep.
//!
//! A creating branch reading several related rows whose strict search finds no run is arranged as
//! its own scenario is. In a chain of such creators each level's own scenario creates the rows of
//! the level below it, every arrangement at a distinction of its own: unbounded, the suite grew
//! about three times and the work about fifteen times per level, and a chain of five levels did
//! not finish in nine minutes. Synthesis of a generic chain is held to the nesting bound and to a
//! bound on the arrangements and strict searches it makes, a work bound rather than a clock, and
//! a branch past the bound is refused as it was before the fallback existed.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{own_scenario_work, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("relay.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// A chain of `depth` levels. `Hop0` is the root. At each level `i`, `Slot{i}` is owned by
/// `Hop{i-1}`, and `Hop{i}` is created reading both `Hop{i-1}` and `Slot{i}`, refused where the
/// slot's owner is not the hop the input names. `Tap` reads the last hop.
#[allow(clippy::too_many_lines)] // One model, written section by section in source order.
fn chain(depth: usize) -> String {
    let mut text = String::from(
        "format: ess/37\nsystem: relay\nversion: v1\ndomain: relay.hub\nsummary: A chain of \
         creators each reading two related rows.\ntypes:\n",
    );
    for i in 0..=depth {
        let _ = writeln!(
            text,
            "  - {{name: relay.hub.Hop{i}Id, kind: newtype, of: Uuid}}"
        );
        if i >= 1 {
            let _ = writeln!(
                text,
                "  - {{name: relay.hub.Slot{i}Id, kind: newtype, of: Uuid}}"
            );
        }
    }
    text.push_str("  - {name: relay.hub.TapId, kind: newtype, of: Uuid}\nentities:\n");
    for i in 0..=depth {
        let _ = write!(
            text,
            "  - name: relay.hub.Hop{i}\n    identity: {{name: hop{i}_id, type: \
             relay.hub.Hop{i}Id}}\n    fields:\n      - {{name: tag, type: String}}\n"
        );
        if i >= 1 {
            let p = i - 1;
            let _ = write!(
                text,
                "      - {{name: hop{p}_id, type: relay.hub.Hop{p}Id}}\n      - {{name: \
                 slot{i}_id, type: relay.hub.Slot{i}Id}}\n"
            );
        }
        text.push_str("    relations:\n");
        if i < depth {
            let c = i + 1;
            let _ = writeln!(
                text,
                "      - {{name: slots, kind: owns, target: relay.hub.Slot{c}, cardinality: \
                 many, via: hop{i}_id}}"
            );
        }
        if i >= 1 {
            let p = i - 1;
            let _ = writeln!(
                text,
                "      - {{name: up, kind: references, target: relay.hub.Hop{p}, cardinality: \
                 one, via: hop{p}_id}}"
            );
        }
        text.push_str("    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\n");
        if i >= 1 {
            let p = i - 1;
            let _ = write!(
                text,
                "  - name: relay.hub.Slot{i}\n    identity: {{name: slot{i}_id, type: \
                 relay.hub.Slot{i}Id}}\n    fields:\n      - {{name: hop{p}_id, type: \
                 relay.hub.Hop{p}Id}}\n    relations:\n      - {{name: hops, kind: owns, \
                 target: relay.hub.Hop{i}, cardinality: many, via: slot{i}_id}}\n    \
                 lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}\n"
            );
        }
    }
    let _ = write!(
        text,
        "  - name: relay.hub.Tap\n    identity: {{name: tap_id, type: relay.hub.TapId}}\n    \
         fields:\n      - {{name: hop{depth}_id, type: relay.hub.Hop{depth}Id}}\n    lifecycle: \
         {{initial: Open, states: [Open], terminal: [Open]}}\nerrors:\n  - {{name: \
         relay.hub.Absent, summary: No such row., fields: []}}\n  - {{name: relay.hub.Foreign, \
         summary: Another owner., fields: []}}\nevents:\n"
    );
    for i in 0..=depth {
        let _ = writeln!(
            text,
            "  - {{name: relay.hub.Hop{i}Made, fields: [{{name: hop{i}_id, type: \
             relay.hub.Hop{i}Id}}]}}"
        );
        if i >= 1 {
            let _ = writeln!(
                text,
                "  - {{name: relay.hub.Slot{i}Made, fields: [{{name: slot{i}_id, type: \
                 relay.hub.Slot{i}Id}}]}}"
            );
        }
    }
    text.push_str(
        "  - {name: relay.hub.TapMade, fields: [{name: tap_id, type: relay.hub.TapId}]}\nactors:\n  \
         - name: relay.hub.Operator\n    may: [relay.hub.MakeHop0",
    );
    for i in 1..=depth {
        let _ = write!(text, ", relay.hub.MakeSlot{i}, relay.hub.MakeHop{i}");
    }
    text.push_str(
        ", relay.hub.MakeTap]\ncommands:\n  - name: relay.hub.MakeHop0\n    input:\n      - {name: \
         tag, type: String}\n    outcomes:\n      - name: made\n        creates: relay.hub.Hop0\n",
    );
    text.push_str("        instance: hop0_id\n        emits: [relay.hub.Hop0Made]\n");
    text.push_str(
        "        payload: {relay.hub.Hop0Made: {hop0_id: {generated: true}}}\n        sets: {tag: \
         input.tag}\n",
    );
    for i in 1..=depth {
        let p = i - 1;
        let _ = write!(
            text,
            "  - name: relay.hub.MakeSlot{i}\n    input:\n      - {{name: hop{p}_id, type: \
             relay.hub.Hop{p}Id}}\n    outcomes:\n      - name: no-owner\n        when_related: \
             {{via: input.hop{p}_id, exists: false}}\n        error: relay.hub.Absent\n      - \
             name: made\n        creates: relay.hub.Slot{i}\n        instance: slot{i}_id\n        \
             emits: [relay.hub.Slot{i}Made]\n        payload: {{relay.hub.Slot{i}Made: \
             {{slot{i}_id: {{generated: true}}}}}}\n        sets: {{hop{p}_id: input.hop{p}_id}}\n"
        );
        let _ = write!(
            text,
            "  - name: relay.hub.MakeHop{i}\n    input:\n      - {{name: tag, type: String}}\n      \
             - {{name: hop{p}_id, type: relay.hub.Hop{p}Id}}\n      - {{name: slot{i}_id, type: \
             relay.hub.Slot{i}Id}}\n    outcomes:\n      - name: no-up\n        when_related: \
             {{via: input.hop{p}_id, exists: false}}\n        error: relay.hub.Absent\n      - \
             name: no-slot\n        when_related: {{via: input.slot{i}_id, exists: false}}\n        \
             error: relay.hub.Absent\n      - name: foreign\n        when_related: {{via: \
             input.slot{i}_id, predicate: hop{p}_id != input.hop{p}_id}}\n        error: \
             relay.hub.Foreign\n      - name: made\n        creates: relay.hub.Hop{i}\n        \
             instance: hop{i}_id\n        emits: [relay.hub.Hop{i}Made]\n        payload: \
             {{relay.hub.Hop{i}Made: {{hop{i}_id: {{generated: true}}}}}}\n        sets: {{tag: \
             input.tag, hop{p}_id: input.hop{p}_id, slot{i}_id: input.slot{i}_id}}\n"
        );
    }
    let _ = write!(
        text,
        "  - name: relay.hub.MakeTap\n    input:\n      - {{name: hop{depth}_id, type: \
         relay.hub.Hop{depth}Id}}\n    outcomes:\n      - name: no-hop\n        when_related: \
         {{via: input.hop{depth}_id, exists: false}}\n        error: relay.hub.Absent\n      - \
         name: made\n        creates: relay.hub.Tap\n        instance: tap_id\n        emits: \
         [relay.hub.TapMade]\n        payload: {{relay.hub.TapMade: {{tap_id: {{generated: \
         true}}}}}}\n        sets: {{hop{depth}_id: input.hop{depth}_id}}\n"
    );
    text
}

/// One synthesis of the chain of `depth` levels, with the arrangements of a setup run as its own
/// scenario it made, the strict searches over several rows it ran, and the most such arrangements
/// open at once.
fn work(depth: usize) -> (Synthesis, (u64, u64, usize)) {
    let model = ir(&chain(depth));
    let (synthesis, work) = own_scenario_work(|| synthesize(&model));
    eprintln!(
        "depth {depth}: {} own-scenario arrangements, {} strict searches, {} open at most, {} \
         scenarios, {} refusals",
        work.0,
        work.1,
        work.2,
        synthesis.suite.scenarios.len(),
        synthesis.refusals.len()
    );
    (synthesis, work)
}

/// At most this many arrangements as an own scenario per level of the chain.
const ARRANGED_PER_LEVEL: u64 = 21;

/// At most this many strict searches per level of the chain, times the depth: every level's
/// creation runs the strict search of each level below it, as it did before the fallback.
const STRICT_PER_LEVEL: u64 = 8;

/// Unbounded, the fallback nested one level per level of the chain and made 3 arrangements and 3
/// strict searches at depth 1, 34 and 34 at depth 2, about fifteen times more per level.
#[test]
fn the_own_scenario_fallback_nests_one_level_at_every_depth() {
    for depth in 1..=6 {
        let (_, (arranged, strict, open)) = work(depth);
        let level = depth as u64;
        assert!(
            open <= 1
                && arranged <= ARRANGED_PER_LEVEL * level
                && strict <= STRICT_PER_LEVEL * level * level,
            "depth {depth}: {arranged} own-scenario arrangements (at most {}), {strict} strict \
             searches (at most {}), {open} open at once (at most 1)",
            ARRANGED_PER_LEVEL * level,
            STRICT_PER_LEVEL * level * level,
        );
    }
}

/// The refusals of the chain of three levels, as release 0.97.4 gives them: the creations of the
/// third level would nest the fallback a second time.
const REFUSED: [&str; 6] = [
    "refusal[ESS-SYNTH-003]: outcome relay.hub.MakeHop3/no-up has no scenario \
     `relay.hub.MakeHop3/outcome/no-up`\n  no candidate of the 1 tried satisfies `a row of \
     `relay.hub.Slot3` for `input.slot3_id` of `relay.hub.MakeHop3` on which no branch answering \
     before `no-up` is selected, beside the row the scenario is arranged around`\n  help: write \
     the branch's condition over values a candidate can carry, or supply a fixture for it",
    "refusal[ESS-SYNTH-003]: outcome relay.hub.MakeHop3/no-slot has no scenario \
     `relay.hub.MakeHop3/outcome/no-slot`\n  no candidate of the 1 tried satisfies `a row of \
     `relay.hub.Hop2` for `input.hop2_id` of `relay.hub.MakeHop3` on which no branch answering \
     before `no-slot` is selected, beside the row the scenario is arranged around`\n  help: write \
     the branch's condition over values a candidate can carry, or supply a fixture for it",
    "refusal[ESS-SYNTH-003]: outcome relay.hub.MakeHop3/foreign has no scenario \
     `relay.hub.MakeHop3/outcome/foreign`\n  no candidate of the 1 tried satisfies `a row of \
     `relay.hub.Hop2` for `input.hop2_id` of `relay.hub.MakeHop3` on which no branch answering \
     before `foreign` is selected, beside the row the scenario is arranged around`\n  help: write \
     the branch's condition over values a candidate can carry, or supply a fixture for it",
    "refusal[ESS-SYNTH-003]: outcome relay.hub.MakeHop3/made has no scenario \
     `relay.hub.MakeHop3/outcome/made`\n  no candidate of the 1 tried satisfies `a row of \
     `relay.hub.Hop2` for `input.hop2_id` of `relay.hub.MakeHop3` on which no branch answering \
     before `made` is selected, beside the row the scenario is arranged around`\n  help: write \
     the branch's condition over values a candidate can carry, or supply a fixture for it",
    "refusal[ESS-SYNTH-004]: outcome relay.hub.MakeSlot3/made has no scenario \
     `relay.hub.MakeSlot3/outcome/made`\n  it needs an instance of `relay.hub.Hop2` to change \
     without moving, and the route runs through `relay.hub.MakeHop2/made`, which no input \
     reaches\n  help: the route to this state runs through a branch no input reaches; see that \
     outcome's own refusal",
    "refusal[ESS-SYNTH-004]: outcome relay.hub.MakeTap/made has no scenario \
     `relay.hub.MakeTap/outcome/made`\n  it needs an instance of `relay.hub.Hop3` to change \
     without moving, and the route runs through `relay.hub.MakeHop3/made`, which no input \
     reaches\n  help: the route to this state runs through a branch no input reaches; see that \
     outcome's own refusal",
];

/// The chain of three levels: every branch of the first two levels has its scenario, and past the
/// nesting bound a branch is refused with the code and text release 0.97.4 gives it, which refused
/// the second level too.
#[test]
fn past_the_nesting_bound_a_branch_is_refused_as_before_the_fallback() {
    let (synthesis, _) = work(3);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, REFUSED, "{refusals:#?}");
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let mut within = vec!["relay.hub.MakeHop0/outcome/made".to_owned()];
    for level in 1..=2 {
        for outcome in ["foreign", "made", "no-slot", "no-up"] {
            within.push(format!("relay.hub.MakeHop{level}/outcome/{outcome}"));
        }
        for outcome in ["made", "no-owner"] {
            within.push(format!("relay.hub.MakeSlot{level}/outcome/{outcome}"));
        }
    }
    within.push("relay.hub.MakeSlot3/outcome/no-owner".to_owned());
    within.push("relay.hub.MakeTap/outcome/no-hop".to_owned());
    within.sort();
    assert_eq!(ids, within);
}
