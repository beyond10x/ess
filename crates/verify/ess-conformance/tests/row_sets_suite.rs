//! The suites the two row-set fixtures synthesize, run by the reference runner against the healthy
//! interpreter and against each faulty implementation (`docs/design/filtered-related-reads.md`,
//! "Decisive acceptance"; `docs/design/expression-family-source22.md`, final decision 13;
//! beyond10x/ess#228, #299).
//!
//! Every branch a row set decides gets a synthesized scenario arranging its rows through the
//! declared creating commands: the rows the selector selects, and one decoy per conjunct that
//! refutes that conjunct alone. A selected value differs from every decoy's and from zero. The
//! healthy interpreter passes every scenario; each fault fails at least the scenarios
//! `support_row_sets::FAULT_KILLS` names, by an outcome, event or row the scenario reads back.
//! `tests/row_sets_runtimes.rs` holds the Go and TypeScript runtimes to the same verdicts.

mod support_row_sets;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
};
use ess_primitives::node::Node;
use support_row_sets::{model, Fault, Faulty, FAULT_KILLS, READS, UNIQUE};

/// The scenarios a row set decides in each fixture, every one required.
const REQUIRED_READS: [&str; 8] = [
    "demo.jobs.Retry/outcome/ambiguous",
    "demo.jobs.Retry/outcome/started",
    "demo.jobs.Retry/outcome/retried",
    "demo.jobs.CheckLimit/outcome/within-limit",
    "demo.jobs.CheckLimit/outcome/over-limit",
    "demo.jobs.Close/outcome/crowded",
    "demo.jobs.Close/outcome/closed",
    "demo.jobs.Record/outcome/recorded",
];
const REQUIRED_UNIQUE: [&str; 3] = [
    "demo.binding.BindIdentity/outcome/claims-taken",
    "demo.binding.BindIdentity/outcome/bound",
    "demo.binding.BindIdentity/outcome/already-bound",
];

fn suite(text: &str, required: &[&str]) -> ConformanceSuite {
    let synthesis = synthesize(&model(text));
    let ids: BTreeSet<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for id in required {
        assert!(
            ids.contains(*id),
            "{id} is synthesized: {ids:#?}\nrefusals: {:#?}",
            synthesis.refusals
        );
    }
    synthesis.suite
}

fn run(suite: &ConformanceSuite, target: &Faulty) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, target)
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

fn failed(statuses: &BTreeMap<String, Status>) -> BTreeSet<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

fn steps<'s>(suite: &'s ConformanceSuite, id: &str) -> &'s [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1
        .steps
}

/// Every `Record` the scenario sends, as (worker, batch, delay).
fn recorded(suite: &ConformanceSuite, id: &str) -> Vec<(String, String, Node)> {
    steps(suite, id)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.jobs.Record" =>
            {
                let text = |name: &str| {
                    input
                        .get(name)
                        .and_then(|value| value.as_literal())
                        .and_then(Node::as_text)
                        .unwrap_or_default()
                        .to_owned()
                };
                Some((
                    text("worker_id"),
                    text("batch_id"),
                    input
                        .get("delay")
                        .and_then(|value| value.as_literal())
                        .cloned()
                        .unwrap_or(Node::Null),
                ))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn row_sets_synthesize_every_branch_a_row_set_decides() {
    suite(READS, &REQUIRED_READS);
    suite(UNIQUE, &REQUIRED_UNIQUE);
}

#[test]
fn the_retried_scenario_arranges_one_match_and_a_decoy_per_conjunct() {
    let suite = suite(READS, &REQUIRED_READS);
    let rows = recorded(&suite, "demo.jobs.Retry/outcome/retried");
    let sent = steps(&suite, "demo.jobs.Retry/outcome/retried")
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.jobs.Retry" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the command under test is sent");
    let worker = sent["worker_id"]
        .as_literal()
        .and_then(Node::as_text)
        .unwrap()
        .to_owned();
    let batch = sent["batch_id"]
        .as_literal()
        .and_then(Node::as_text)
        .unwrap()
        .to_owned();
    let matching: Vec<_> = rows
        .iter()
        .filter(|(w, b, _)| *w == worker && *b == batch)
        .collect();
    assert_eq!(matching.len(), 1, "one match: {rows:#?}");
    let decoy_worker = rows
        .iter()
        .filter(|(w, b, _)| *w != worker && *b == batch)
        .count();
    let decoy_batch = rows
        .iter()
        .filter(|(w, b, _)| *w == worker && *b != batch)
        .count();
    assert!(
        decoy_worker >= 1 && decoy_batch >= 1,
        "a decoy per conjunct: {rows:#?}"
    );
    // The copied value differs from every decoy's and from zero.
    let selected = &matching[0].2;
    assert_ne!(
        *selected,
        Node::Number(ess_primitives::facts::Number::from(0_i64))
    );
    for (w, b, delay) in &rows {
        if (w, b) != (&worker, &batch) {
            assert_ne!(
                delay, selected,
                "a decoy holds the selected value: {rows:#?}"
            );
        }
    }
}

/// A target that keeps one record per value of a selector's field, the newest or the oldest
/// replacing the rest, answers from that record and not from the set. So among the rows sharing
/// each sent value, the selected row is neither the first arranged nor the last: a decoy holding
/// that value is arranged on each side of it.
#[test]
fn the_selected_row_is_neither_the_first_nor_the_last_of_any_value_it_shares() {
    let suite = suite(READS, &REQUIRED_READS);
    let id = "demo.jobs.Retry/outcome/retried";
    let rows = recorded(&suite, id);
    let sent = steps(&suite, id)
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.jobs.Retry" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the command under test is sent");
    let text = |name: &str| {
        sent[name]
            .as_literal()
            .and_then(Node::as_text)
            .unwrap()
            .to_owned()
    };
    let (worker, batch) = (text("worker_id"), text("batch_id"));
    let selected = rows
        .iter()
        .position(|(w, b, _)| *w == worker && *b == batch)
        .expect("one row is selected");
    for (field, sharing) in [
        (
            "worker_id",
            rows.iter()
                .enumerate()
                .filter(|(_, (w, _, _))| *w == worker)
                .map(|(at, _)| at)
                .collect::<Vec<_>>(),
        ),
        (
            "batch_id",
            rows.iter()
                .enumerate()
                .filter(|(_, (_, b, _))| *b == batch)
                .map(|(at, _)| at)
                .collect::<Vec<_>>(),
        ),
    ] {
        assert!(
            sharing.first() != Some(&selected) && sharing.last() != Some(&selected),
            "the selected row is the first or the last of the rows sharing its {field}: \
             {rows:#?}"
        );
    }
}

#[test]
fn the_healthy_interpreter_passes_every_row_set_scenario() {
    for (text, required, target) in [
        (READS, &REQUIRED_READS[..], Faulty::new(None)),
        (UNIQUE, &REQUIRED_UNIQUE[..], Faulty::unique(None)),
    ] {
        let suite = suite(text, required);
        let statuses = run(&suite, &target);
        assert_eq!(failed(&statuses), BTreeSet::new(), "{statuses:#?}");
    }
}

#[test]
fn each_fault_fails_a_synthesized_scenario() {
    let reads = suite(READS, &REQUIRED_READS);
    let unique = suite(UNIQUE, &REQUIRED_UNIQUE);
    for (fault, killed) in FAULT_KILLS {
        let (suite, target) = if Fault::READS.contains(fault) {
            (&reads, Faulty::new(Some(*fault)))
        } else {
            (&unique, Faulty::unique(Some(*fault)))
        };
        let statuses = run(suite, &target);
        let failed = failed(&statuses);
        for id in *killed {
            assert!(failed.contains(id), "{fault:?}: {id} passed: {statuses:#?}");
        }
    }
}
