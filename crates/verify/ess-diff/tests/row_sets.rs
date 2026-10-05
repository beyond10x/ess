//! A row set's selector, test or read field moving is a behaviour change, and the diff renders the
//! row set as what it selects and tests (`docs/design/filtered-related-reads.md`, "Compatibility and
//! targets": diff classifies a changed selector or source field explicitly, never as an unknown
//! expression; beyond10x/ess#228, #299).

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const READS: &str =
    include_str!("../../ess-conformance/tests/fixtures/filtered-related-reads.yaml");

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("jobs.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn a_changed_row_set_test_or_selector_is_a_condition_change() {
    let before = ir(READS);
    for (from, to, rendered) in [
        (
            "count: {gt: 1}",
            "count: {gt: 2}",
            "more than 2 rows are selected",
        ),
        (
            "forall: delay <= input.limit",
            "forall: delay < input.limit",
            "every selected row satisfies `delay < input.limit`",
        ),
    ] {
        let after = ir(&READS.replacen(from, to, 1));
        let json = diff(&before, &after).unwrap().to_canonical_json();
        assert!(json.contains("outcome-condition-changed"), "{to}: {json}");
        assert!(json.contains(rendered), "{to}: {json}");
    }
    // The selector of one guard: one conjunct dropped.
    let (head, tail) = READS.split_once("      - name: started").unwrap();
    let narrowed = format!(
        "{head}      - name: started{}",
        tail.replacen(
            "where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}",
            "where: worker_id == input.worker_id",
            1
        )
    );
    let json = diff(&before, &ir(&narrowed)).unwrap().to_canonical_json();
    assert!(
        json.contains("outcome-condition-changed") && json.contains("rows satisfying"),
        "{json}"
    );
}

#[test]
fn a_changed_filtered_read_is_a_sets_and_payload_change() {
    let before = ir(READS);
    let after = ir(&READS.replacen(
        "                where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}\n                field: delay",
        "                where: {all: [worker_id == input.worker_id, batch_id != input.batch_id]}\n                field: delay",
        1,
    ));
    let json = diff(&before, &after).unwrap().to_canonical_json();
    assert!(json.contains("outcome-response-payload-changed"), "{json}");
    assert!(json.contains("batch_id != input.batch_id"), "{json}");
    // The selector of the read in `sets:`, moved.
    let moved = ir(&READS.replacen(
        "          delay:\n            related:\n              entity: demo.jobs.Attempt\n              where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}",
        "          delay:\n            related:\n              entity: demo.jobs.Attempt\n              where: worker_id == input.worker_id",
        1,
    ));
    let json = diff(&before, &moved).unwrap().to_canonical_json();
    assert!(json.contains("outcome-sets-changed"), "{json}");
}

#[test]
fn an_unchanged_row_set_is_no_change() {
    let before = ir(READS);
    let json = diff(&before, &ir(READS)).unwrap().to_canonical_json();
    assert!(!json.contains("changed"), "{json}");
}
