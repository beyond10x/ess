//! A `when_related:` guard needing the related row in a later state, where the move to that state
//! is refused by a guard over the row's stored lists (story seeded-state-reaches-related-rows).
//!
//! `Start` runs only on a graph in `Checked`, and `Check` refuses a graph with an edge naming a
//! node the graph does not declare. The creating branch guards its `nodes` input, so the plain
//! witness keeps `nodes` empty, while it copies `edges` whole, so the plain witness holds one
//! edge. That row cannot be checked; the arrangement creates the row again with the inputs taken
//! toward the guards of the move, and the related row reaches `Checked`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::Synthesis, target::ConformanceTarget,
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const GRAPH: &str = include_str!("fixtures/related-guard-checked-graph.yaml");

const STARTED: &str = "demo.graph.Start/outcome/started";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("graph.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(GRAPH))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?} {}", refusal.cause))
        .collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id}; refused: {:#?}", refusals(synthesis)),
            |(_, scenario)| scenario,
        )
}

/// The commands the scenario sends, in order.
fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

#[test]
fn a_related_row_is_arranged_through_a_move_its_stored_lists_guard() {
    let synthesis = synthesis();
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        refusals(&synthesis)
    );
    let sent = sent(scenario(&synthesis, STARTED));
    let check = sent
        .iter()
        .position(|command| command == "demo.graph.Check")
        .unwrap_or_else(|| panic!("the related graph is checked first: {sent:?}"));
    let start = sent
        .iter()
        .position(|command| command == "demo.graph.Start")
        .unwrap_or_else(|| panic!("the scenario starts a run: {sent:?}"));
    assert!(check < start, "checked before the run starts: {sent:?}");
}

#[test]
fn the_arranged_related_row_passes_the_interpreted_model() {
    let synthesis = synthesis();
    let target = Interpreted::for_model(ir(GRAPH));
    let statuses = run(&synthesis.suite, &target);
    assert_eq!(
        statuses.get(STARTED),
        Some(&Status::Passed),
        "the started scenario passes: {statuses:#?}"
    );
    let failed: Vec<&String> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id)
        .collect();
    assert!(failed.is_empty(), "every scenario passes: {failed:?}");
}
