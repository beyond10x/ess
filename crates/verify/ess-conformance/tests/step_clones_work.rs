//! Steps a suite already holds are not copied again where nothing changes them.
//!
//! Fixture setup copied every step of every scenario even where no fixture applied, and the
//! preconditions' prelude was copied once to assemble it and once more to prepend it. Both are held
//! to a count of copied steps, a work bound rather than a clock. Unfixed code copied every step of
//! every scenario in fixture setup (all of the suite's steps, 0 fixtures), twice the emitted
//! precondition steps, and scanned the steps for counted claims twice.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::fixtures::{fixture_step_clones, precondition_step_clones};
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::synthesize::synthesize;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const PRECONDITIONS: &str = include_str!("fixtures/explore-preconditions.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn fixture_setup_copies_no_step_where_no_fixture_applies() {
    let ir = ir(PRECONDITIONS);
    let (synthesis, copied) = fixture_step_clones(|| synthesize(&ir));
    let steps: usize = synthesis
        .suite
        .scenarios
        .values()
        .map(|scenario| scenario.steps.len())
        .sum();
    assert!(steps > 0);
    assert!(
        synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| &scenario.steps)
            .all(|step| !matches!(step, ScenarioStep::ResolveFixtures { .. })),
        "the model must use no fixture"
    );
    assert_eq!(
        copied, 0,
        "fixture setup copied {copied} of {steps} steps with no fixture applying"
    );
}

#[test]
fn selecting_a_fresh_format_scans_the_unchanged_steps_once() {
    let ir = ir(PRECONDITIONS);
    let mut suite = synthesize(&ir).suite;
    let ((), scans) = ess_conformance::scenario::format_feature_scans(|| {
        suite.select_fresh_format_for(&ir);
    });
    assert_eq!(scans, 1, "the steps were scanned {scans} times");
    let kept = suite.provenance.suite_version.clone();
    let mut again = synthesize(&ir).suite;
    again.select_fresh_format_for(&ir);
    assert_eq!(again.provenance.suite_version, kept);
}

#[test]
fn a_precondition_prelude_is_copied_once_per_emitted_step() {
    let ir = ir(PRECONDITIONS);
    let (synthesis, copied) = precondition_step_clones(|| synthesize(&ir));
    let preconditions = 2 * ir.preconditions().len();
    let emitted: usize = synthesis
        .suite
        .scenarios
        .values()
        .map(|scenario| {
            scenario
                .steps
                .iter()
                .take_while(|step| {
                    matches!(
                        step,
                        ScenarioStep::ExecuteCommand { .. }
                            | ScenarioStep::ExpectOutcome { .. }
                            | ScenarioStep::QueryView { .. }
                            | ScenarioStep::SnapshotView { .. }
                    )
                })
                .count()
                .min(preconditions)
        })
        .sum();
    assert!(emitted > 0);
    assert!(
        copied as usize <= emitted,
        "{copied} step copies for {emitted} emitted precondition steps"
    );
}
