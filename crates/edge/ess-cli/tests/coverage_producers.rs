//! Actual producers against independently pinned source and semantic expectations.
#[path = "support/coverage_producer.rs"]
mod producer;

#[test]
fn actual_rust_coverage_exports_match_the_independent_plan() {
    producer::export_rust();
}

#[test]
fn actual_go_coverage_exports_match_the_independent_plan() {
    producer::export_go();
}

#[test]
fn actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output() {
    producer::export_controls();
}
