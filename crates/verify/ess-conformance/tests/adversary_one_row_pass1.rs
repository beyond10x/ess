//! Pass 1 against `story:the-one-row-check-reads-the-instance-its-scenario-bound`.
//!
//! `one_row` now names the snapshot's row from the scenario's last call of the command, and still
//! decides which views may carry the snapshot from a second run of the updating branch. In
//! `create-or-update-stored-fields.yaml` the scenario of `reopened` calls `SignIn` on three rows,
//! each left with its own `call_ref`, while the second run decides the view filter on the plain
//! row alone.
//!
//! * The row the moved bytes-table row names: `reopened` snapshots `seat-3`, the row its last
//!   `SignIn` addresses.
//! * A filtered read-your-writes view decided to admit the second run's row must also admit the
//!   row the snapshot names, or a correct implementation fails the scenario.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml"
);

const REOPENED: &str = "demo.desk.SignIn/outcome/reopened";

/// A read-your-writes view of the seats no call holds.
const FREE_SEATS: &str = "  - name: demo.desk.FreeSeats
    source: demo.desk.Seat
    consistency: read_your_writes
    filter: call_ref == \"\"
    fields:
      - {name: user_id, type: String}
      - {name: call_ref, type: String}
      - {name: state, type: demo.desk.Seat.State}
";

/// A read-your-writes view of the seats some call holds.
const HELD_SEATS: &str = "  - name: demo.desk.HeldSeats
    source: demo.desk.Seat
    consistency: read_your_writes
    filter: call_ref != \"\"
    fields:
      - {name: user_id, type: String}
      - {name: call_ref, type: String}
      - {name: state, type: demo.desk.Seat.State}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, (Status, Vec<String>)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    results
        .into_iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnostics))| format!("{id}: {status:?} {diagnostics:?}"))
        .collect()
}

/// The steps of one scenario, one JSON line each.
fn steps(suite: &ConformanceSuite, id: &str) -> Vec<serde_json::Value> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("{id} is in the suite"));
    scenario
        .steps
        .iter()
        .map(|step| serde_json::to_value(step).expect("a step serializes"))
        .collect()
}

/// `(view, instance)` of every `snapshot_subject` step.
fn snapshots(steps: &[serde_json::Value]) -> Vec<(String, String)> {
    steps
        .iter()
        .filter(|step| step["step"] == "snapshot_subject")
        .map(|step| {
            (
                step["view"].as_str().unwrap_or("?").to_owned(),
                step["subject"]["user_id"]["instance"]
                    .as_str()
                    .unwrap_or("?")
                    .to_owned(),
            )
        })
        .collect()
}

fn listing(steps: &[serde_json::Value]) -> String {
    steps
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn adv_one_row_reopened_snapshots_the_row_the_changelog_names() {
    let model = ir(MODEL);
    let result = synthesize(&model);
    let reopened = steps(&result.suite, REOPENED);
    assert_eq!(
        snapshots(&reopened),
        vec![("demo.desk.Seats".to_owned(), "seat-3".to_owned())],
        "the changelog says `reopened` snapshots `seat-3`, the row its last call addresses; the scenario:\n{}",
        listing(&reopened)
    );
}

#[test]
fn adv_one_row_fixture_suite_passes_interpreted() {
    let model = ir(MODEL);
    let result = synthesize(&model);
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

#[test]
fn adv_one_row_filtered_view_admits_the_row_the_snapshot_names() {
    let text = format!("{MODEL}{FREE_SEATS}");
    let model = ir(&text);
    let result = synthesize(&model);
    let reopened = steps(&result.suite, REOPENED);
    assert_eq!(
        not_passed(model, &result.suite),
        Vec::<String>::new(),
        "the scenario of `reopened`:\n{}",
        listing(&reopened)
    );
}

/// The created seat holds a call reference, so the second run's row (created, then `reopened`
/// from `Open`) keeps it and `HeldSeats` is decided to admit that row. The scenario's last
/// `SignIn` is sent for the seat a `Take` left `Busy` with an empty reference, a row `HeldSeats`
/// does not hold: the one-row snapshot read from it selects no row on a correct implementation.
#[test]
fn adv_one_row_view_decided_on_the_second_run_admits_the_snapshot_row() {
    let created = MODEL.replace("sets: {call_ref: \"\"}", "sets: {call_ref: \"held\"}");
    assert_ne!(created, MODEL, "the creating branch sets `call_ref`");
    let text = format!("{created}{HELD_SEATS}");
    let model = ir(&text);
    let result = synthesize(&model);
    let reopened = steps(&result.suite, REOPENED);
    let failed: Vec<String> = not_passed(model, &result.suite)
        .into_iter()
        .filter(|line| line.starts_with(&format!("{REOPENED}:")))
        .collect();
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "the scenario of `reopened`:\n{}",
        listing(&reopened)
    );
}
