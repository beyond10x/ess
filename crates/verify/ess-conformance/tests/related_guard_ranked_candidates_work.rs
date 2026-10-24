//! Rows that reach a state past a list-guarded move, asked for again and again by the arrangements
//! above them, are made once per synthesis (beyond10x/ess#633).
//!
//! `Propose` needs a running run, a patch and a graph; `Start` and `Build` each need the graph in
//! `Checked`, which it reaches only on a row created toward the list guard of `Check`; `Select`
//! refuses a candidate outranked by a lower-scored candidate of its own run. Each arrangement of a
//! candidate, a run or a patch arranges the same checked graph, and the arrangements of the
//! sibling candidate arrange the same run and patch again. Made anew each time, an adopter's
//! specification of this shape ran for more than thirteen minutes; this one made the creating
//! branches' rows 574 times where 54 distinct arrangements were asked for. Synthesis is held to a
//! bound on that count, a work bound rather than a clock.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{creations_made, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const FIT: &str = include_str!("fixtures/related-guard-ranked-candidates.yaml");

/// The creating branches' rows made for the whole synthesis: 54 when each is made once, 574 when
/// each request makes them anew.
const BOUND: u64 = 120;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("fit.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn rows_reached_through_a_guarded_move_are_made_within_a_bound() {
    let ir = ir(FIT);
    let (synthesis, made) = creations_made(|| synthesize(&ir));
    assert!(
        made <= BOUND,
        "the creating branches' rows were made {made} times, more than {BOUND}\nscenarios: \
         {:#?}\nrefusals: {:#?}",
        ids(&synthesis),
        refusals(&synthesis)
    );
}

#[test]
fn the_branches_on_a_checked_graph_keep_their_scenarios() {
    let synthesis = synthesize(&ir(FIT));
    let ids = ids(&synthesis);
    for id in [
        "demo.fit.Start/outcome/started",
        "demo.fit.Build/outcome/built",
        "demo.fit.Propose/outcome/proposed",
        "demo.fit.Propose/outcome/not-running",
    ] {
        assert!(
            ids.iter().any(|it| it == id),
            "{id} is synthesized: {ids:#?}\nrefusals: {:#?}",
            refusals(&synthesis)
        );
    }
}

#[test]
fn the_synthesis_is_deterministic() {
    let ir = ir(FIT);
    let (first, made) = creations_made(|| synthesize(&ir));
    let (second, again) = creations_made(|| synthesize(&ir));
    assert_eq!(made, again, "each synthesis makes the same rows");
    assert_eq!(
        serde_json::to_string(&first.suite).expect("the suite serializes"),
        serde_json::to_string(&second.suite).expect("the suite serializes"),
    );
    assert_eq!(refusals(&first), refusals(&second));
}
