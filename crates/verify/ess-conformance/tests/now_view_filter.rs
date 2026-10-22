//! A view filter ordering a field against `now` (`ess/30`, beyond10x/ess#560,
//! `story:expression-arithmetic-and-time`, `now_relative_view_filter`).
//!
//! `RecentIncidents` shows an incident whose `started_at` is within 720 hours of the read. The row's
//! instant is arranged through the creator's input as a `now_offset`, so it travels with the run;
//! synthesis decides the filter at its fixed reference instant, and only where the row reads alike
//! for an hour after it was sent, so a target reading its clock at the view read decides it as the
//! suite requires. The view is witnessed either side of the bound: the subject's row is shown, and a
//! further row sent a second outside the bound is not. The interpreted target, reading its clock
//! at the read, passes every scenario; one filtering otherwise, or reading no clock, does not.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, AdmittedSuite,
    ConformanceSuite, ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/now-view-filter.yaml");
const VIEW: &str = "demo.ledger.RecentIncidents";
const COMMAND: &str = "demo.ledger.RecordIncident";
const FILTER: &str = "filter: \"started_at >= now - 720h\"";
/// The bound the filter writes, in seconds from the read: `now - 720h`.
const BOUND: i64 = -720 * 3600;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&ir(MODEL));
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .filter(|refusal| refusal.contains(VIEW))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    synthesis.suite
}

/// Each scenario's status against the interpreted `target` model, run at the scenario clock: the
/// interpreter reads the runner's step instant at every command decision and every view read, or no
/// clock at all where `clocked` is false.
fn run(suite: &ConformanceSuite, target: &EssIr, clocked: bool) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut runner = ess_conformance::now_offset::scenario_clock_runner(suite);
    let interpreted = Interpreted::for_model(target.clone());
    let interpreted = if clocked {
        interpreted.with_command_clock(runner.command_clock())
    } else {
        interpreted
    };
    runner
        .run_admitted(&admitted, &interpreted)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// The scenarios asserting something of `RecentIncidents`.
fn viewing(suite: &ConformanceSuite) -> Vec<String> {
    suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExpectView { view, .. } if view.to_string() == VIEW)
            })
        })
        .map(|(id, _)| id.to_string())
        .collect()
}

#[test]
fn the_view_is_witnessed_either_side_of_its_now_bound() {
    let suite = suite();
    let mut contains = 0;
    let mut excludes = 0;
    let mut sent = Vec::new();
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExpectView { view, expectation } if view.to_string() == VIEW => {
                    match expectation {
                        ViewExpectation::Contains { .. } => contains += 1,
                        ViewExpectation::Excludes { .. } => excludes += 1,
                        _ => {}
                    }
                }
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == COMMAND =>
                {
                    sent.push(input.get("started_at").cloned());
                }
                _ => {}
            }
        }
    }
    assert!(contains > 0 && excludes > 0, "{contains} {excludes}");
    // Every row's instant travels with the run, and rows are sent either side of the bound.
    let offsets: Vec<i64> = sent
        .iter()
        .map(|value| match value {
            Some(ScenarioValue::NowOffset { seconds }) => *seconds,
            other => panic!("`started_at` is sent as a now_offset, not {other:?}"),
        })
        .collect();
    assert!(
        offsets.iter().any(|seconds| *seconds >= BOUND)
            && offsets.iter().any(|seconds| *seconds < BOUND),
        "{offsets:?}"
    );
    // No row is claimed inside the hour a read may follow its write by: a row a second inside the
    // bound is shown at the write and not an hour later, so it is left to the boundary.
    assert!(
        offsets
            .iter()
            .all(|seconds| *seconds < BOUND || *seconds >= BOUND + 3600),
        "{offsets:?}"
    );
}

#[test]
fn the_interpreted_target_reading_its_clock_at_the_read_passes_every_scenario() {
    let suite = suite();
    assert_ne!(viewing(&suite), Vec::<String>::new());
    let statuses = run(&suite, &ir(MODEL), true);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
}

#[test]
fn a_target_filtering_otherwise_or_reading_no_clock_fails_the_view_claims() {
    let suite = suite();
    let viewing = viewing(&suite);
    for (label, target, clocked) in [
        (
            "no filter",
            MODEL.replacen(&format!("    {FILTER}\n"), "", 1),
            true,
        ),
        (
            "the bound a second outward",
            MODEL.replacen(FILTER, "filter: \"started_at >= now - 2592001s\"", 1),
            true,
        ),
        ("no clock", MODEL.to_owned(), false),
    ] {
        let statuses = run(&suite, &ir(&target), clocked);
        assert!(
            viewing.iter().any(|id| statuses
                .get(id)
                .is_some_and(|status| *status != Status::Passed)),
            "{label}: {statuses:#?}"
        );
    }
}
