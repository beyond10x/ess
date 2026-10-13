//! Synthesis of a command that reads several selectors (`docs/design/ordered-related-head.md`,
//! "Several selectors in one command", "Decisive acceptance"; beyond10x/ess#543).
//!
//! `tests/fixtures/two-sided-pick.yaml`: `Join` adds a membership to one side of a slot; `Pick`
//! refuses with `LeftEmpty` while no membership of the slot's left side is `Present`, then with
//! `RightEmpty` for the right side, and otherwise pairs the one membership waiting on each side,
//! publishing and storing both keys. The two sides are two selectors that differ by `side == Left`
//! and `side == Right`, so a decoy refuting one side's `side` conjunct is a row the other selects.
//!
//! Every branch gets a synthesized scenario, the healthy interpreter passes each, and a target
//! copying a side's key from the wrong side, or ignoring a selector's `side`, fails `picked`.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/two-sided-pick.yaml");

const LEFT: &str = "{all: [slot_id == input.slot_id, side == Left, state == Present]}";

const BRANCHES: [&str; 3] = [
    "demo.sides.Pick/outcome/left-empty",
    "demo.sides.Pick/outcome/right-empty",
    "demo.sides.Pick/outcome/picked",
];

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("sides.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

/// The ranked form at `ess/25`: each membership carries a score, and each side's key is read from
/// the first membership by `[element_score desc, membership_id]`, not from the one membership.
fn ranked() -> String {
    let mut text = MODEL.replacen("format: ess/22", "format: ess/25", 1);
    text = text.replacen(
        "      - {name: element_key, type: String}\n    lifecycle:",
        "      - {name: element_key, type: String}\n      - {name: element_score, type: Integer}\n    lifecycle:",
        1,
    );
    text = text.replacen(
        "      - {name: element_key, type: String}\n    outcomes:",
        "      - {name: element_key, type: String}\n      - {name: element_score, type: Integer}\n    outcomes:",
        1,
    );
    text = text.replacen(
        "          element_key: input.element_key\n",
        "          element_key: input.element_key\n          element_score: input.element_score\n",
        1,
    );
    let mut out = String::new();
    for line in text.lines() {
        if line.trim() == "field: element_key" {
            let indent = &line[..line.len() - line.trim_start().len()];
            out.push_str(indent);
            out.push_str("order_by: [element_score desc, membership_id]\n");
            out.push_str(indent);
            out.push_str("first: element_key\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    assert!(!out.contains("field: element_key"), "every read is ranked");
    assert_eq!(out.matches("first: element_key").count(), 4, "{out}");
    out
}

/// `text` with only `picked`'s reads of `side` changed from `from` to `to`, never the guards.
fn reads_changed(text: &str, from: &str, to: &str) -> String {
    let (head, tail) = text
        .split_once("      - name: picked")
        .expect("the picked branch");
    let (reads, rest) = tail
        .split_once("  - name: demo.sides.Leave")
        .expect("the next command");
    assert!(reads.contains(from), "the reads hold {from}");
    format!(
        "{head}      - name: picked{}  - name: demo.sides.Leave{rest}",
        reads.replace(from, to)
    )
}

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
    let refusals = format!("{:?}", synthesis.refusals);
    assert!(
        !refusals.contains("reads more than one row set"),
        "no branch is refused for reading two selectors: {refusals}"
    );
    synthesis.suite
}

fn run(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
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

/// Every `Join` the scenario sends, as (side, element key), in order.
fn joined(suite: &ConformanceSuite, id: &str) -> Vec<(String, String)> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.sides.Join" =>
            {
                let text = |name: &str| {
                    input
                        .get(name)
                        .and_then(|value| value.as_literal())
                        .and_then(Node::as_text)
                        .unwrap_or_default()
                        .to_owned()
                };
                Some((text("side"), text("element_key")))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_two_sided_pick_is_witnessed_on_each_branch() {
    let suite = suite(MODEL, &BRANCHES);
    let rows = joined(&suite, "demo.sides.Pick/outcome/picked");
    for side in ["Left", "Right"] {
        assert!(
            rows.iter().any(|(held, _)| held == side),
            "a {side} membership is arranged: {rows:#?}"
        );
    }
    let statuses = run(&suite, &Interpreted::for_model(model(MODEL)));
    assert_eq!(failed(&statuses), BTreeSet::new(), "{statuses:#?}");

    // A target that reads the right side's key from the left side, or that reads the left side
    // without its `side` conjunct, fails the scenario of the branch that reads them.
    for (fault, text) in [
        (
            "right key copied from the left side",
            reads_changed(
                MODEL,
                "side == Right, state == Present]}\n              field: element_key",
                "side == Left, state == Present]}\n              field: element_key",
            ),
        ),
        (
            "left read ignoring its side",
            reads_changed(
                MODEL,
                LEFT,
                "{all: [slot_id == input.slot_id, state == Present]}",
            ),
        ),
    ] {
        let statuses = run(&suite, &Interpreted::for_model(model(&text)));
        assert!(
            failed(&statuses).contains("demo.sides.Pick/outcome/picked"),
            "{fault}: picked passed: {statuses:#?}"
        );
    }
}

#[test]
fn a_two_sided_ranked_pick_is_witnessed() {
    let text = ranked();
    let suite = suite(&text, &BRANCHES);
    let statuses = run(&suite, &Interpreted::for_model(model(&text)));
    assert_eq!(failed(&statuses), BTreeSet::new(), "{statuses:#?}");
    // Each side arranges more than one selected row, so its head is a choice.
    let rows = joined(&suite, "demo.sides.Pick/outcome/picked");
    for side in ["Left", "Right"] {
        assert!(
            rows.iter().filter(|(held, _)| held == side).count() >= 2,
            "several {side} memberships are arranged: {rows:#?}"
        );
    }
    // A target ranking either side by the reversed score fails `picked`.
    for (fault, text) in [
        (
            "left side ranked by the reversed score",
            text.replacen(
                "order_by: [element_score desc, membership_id]",
                "order_by: [element_score asc, membership_id]",
                1,
            ),
        ),
        (
            "every side ranked by the reversed score",
            text.replace(
                "order_by: [element_score desc, membership_id]",
                "order_by: [element_score asc, membership_id]",
            ),
        ),
    ] {
        let statuses = run(&suite, &Interpreted::for_model(model(&text)));
        assert!(
            failed(&statuses).contains("demo.sides.Pick/outcome/picked"),
            "{fault}: picked passed: {statuses:#?}"
        );
    }
}
