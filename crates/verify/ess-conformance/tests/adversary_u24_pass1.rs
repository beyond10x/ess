//! Adversary pass 1 on the `present: true` help of a broken invariant (story
//! `a-minted-optional-an-invariant-requires-names-present-true`).
//!
//! The help is chosen from the model alone: the first outcome, in command-name order, that stores
//! a field the invariant reads as `{generated: true}`. It is not chosen from the row that broke.
//! Two consequences, each a case here:
//!
//! 1. A row whose minted field is present, and which breaks the invariant *because* it is present,
//!    is told to declare the mint `present: true`, which it already is.
//! 2. When an outcome off the scenario's path also mints the field absent and its command sorts
//!    first, the help names that outcome, not the one that left this row's field absent.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::ConformanceTarget,
    AdmittedSuite, ConformanceSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/minted-value-sources.yaml");
const BRIDGED: &str = "demo.routing.BridgeEntry/outcome/bridged";

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// `model` with a `ParkEntry` command, off the bridge's path, whose outcome stores `connect_seq`
/// as `{generated: true}` alone. `demo.routing.ParkEntry` sorts before `demo.routing.SelectAgent`.
fn with_park(model: &str) -> String {
    let model = edit(
        model,
        "      states: [Waiting, Selected, Bridged]\n      terminal: [Bridged]\n",
        "      states: [Waiting, Selected, Bridged, Parked]\n      terminal: [Bridged, Parked]\n",
    );
    let model = edit(
        &model,
        "        - {name: bridge, from: [Selected], to: Bridged}\n",
        "        - {name: bridge, from: [Selected], to: Bridged}\n        - {name: park, from: [Waiting], to: Parked}\n",
    );
    let model = edit(
        &model,
        "demo.routing.BridgeEntry]\n",
        "demo.routing.BridgeEntry, demo.routing.ParkEntry]\n",
    );
    edit(
        &model,
        "views:\n",
        "  - name: demo.routing.ParkEntry\n    input:\n      - {name: entry_id, type: demo.routing.EntryId}\n    outcomes:\n      - name: parked\n        moves: demo.routing.Entry.park\n        instance: entry_id\n        emits: [demo.routing.EntryQueued]\n        payload:\n          demo.routing.EntryQueued: {entry_id: input.entry_id}\n        sets:\n          connect_seq: {generated: true}\n      - {name: not-waiting, wrong_state: true, error: demo.routing.EntryStateConflict}\nviews:\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let passed = matches!(run.status, Status::Passed);
            (
                run.scenario.to_string(),
                if passed {
                    "passed".to_owned()
                } else {
                    format!("{:?} {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

/// The bridge stores `connect_seq` present (`SelectAgent` declares `present: true`), and the
/// invariant forbids it present once bridged. The row breaks because the value is present, so a
/// line telling the author to declare it `present: true` is the opposite of the repair.
#[test]
fn a_row_broken_by_a_present_mint_is_not_told_to_declare_present_true() {
    let text = edit(
        &with_park(MODEL),
        "      - any: [{not: \"state == Bridged\"}, \"defined(connect_seq)\"]\n",
        "      - any: [{not: \"state == Bridged\"}, {not: \"defined(connect_seq)\"}]\n",
    );
    let model = ir(&text);
    let statuses = run(&synthesize(&model).suite, &Interpreted::for_model(model));
    let bridged = statuses.get(BRIDGED).expect("the bridge has its scenario");
    assert!(
        bridged.contains("violating its invariant"),
        "the bridge breaks the invariant: {bridged}"
    );
    assert!(
        !bridged.contains("present: true}`"),
        "the row's connect_seq is present, yet the refusal says to declare it present: {bridged}"
    );
}

/// `SelectAgent/selected` is the outcome on the bridge's path that left `connect_seq` absent;
/// `ParkEntry/parked` is never run in that scenario. The help names the outcome to repair.
#[test]
fn the_help_names_the_outcome_that_left_this_row_absent() {
    let text = edit(
        &with_park(MODEL),
        "          connect_seq: {generated: true, present: true}\n",
        "          connect_seq: {generated: true}\n",
    );
    let model = ir(&text);
    let statuses = run(&synthesize(&model).suite, &Interpreted::for_model(model));
    let bridged = statuses.get(BRIDGED).expect("the bridge has its scenario");
    let Some((_, help)) = bridged.split_once("help:") else {
        panic!("the refusal carries a help line: {bridged}");
    };
    assert!(
        help.contains("demo.routing.SelectAgent/selected"),
        "the help names the outcome on this scenario's path: {bridged}"
    );
}

/// Mutant guard: the only mint of `connect_seq` is `{generated: true, present: true}`, and the
/// broken invariant reads `connect_seq`. No help. The unit's own negative test pairs a present
/// mint only with an invariant that reads no field, so a help that fired on any
/// `Generated { .. }` whatever its `present` would pass it.
#[test]
fn a_present_mint_read_by_a_broken_invariant_gets_no_help() {
    let text = edit(
        MODEL,
        "      - any: [{not: \"state == Bridged\"}, \"defined(connect_seq)\"]\n",
        "      - any: [{not: \"state == Bridged\"}, {not: \"defined(connect_seq)\"}]\n",
    );
    let model = ir(&text);
    let statuses = run(&synthesize(&model).suite, &Interpreted::for_model(model));
    let bridged = statuses.get(BRIDGED).expect("the bridge has its scenario");
    assert!(
        bridged.contains("violating its invariant"),
        "the bridge breaks the invariant: {bridged}"
    );
    assert!(!bridged.contains("help:"), "no help line: {bridged}");
}
