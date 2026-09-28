//! Adversary, pass 2, story:current-time-guard-operand (beyond10x/ess#171).
//!
//! Contract drift in the refusal texts. Where the operand is not admitted (a view filter, an
//! invariant), a malformed offset or an equality is refused with a hint that tells the author to
//! write the operand again — `now - 60s`, or `starts_at >= now` — which that same site then refuses
//! (`expression.rs` `current_time_literal` matches those two arms on `format` alone, not `site`).
//! The refusal must say the operand is admitted only in a command's `when:`, as the well-formed
//! ordering at the same site is told.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

const JOBS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/current-time-guard.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("jobs.yaml"), raw)])
}

fn with_filter(filter: &str) -> String {
    let before = "    consistency: read_your_writes\n";
    assert!(JOBS.contains(before));
    JOBS.replacen(
        before,
        &format!("    consistency: read_your_writes\n    filter: {filter}\n"),
        1,
    )
}

fn messages(text: &str) -> String {
    let errors = assemble(text).expect_err("the operand is refused in a view filter");
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {}", error.code, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn adversary_now_pass2_a_malformed_offset_in_a_filter_is_not_told_to_write_now() {
    let all = messages(&with_filter("starts_at > now - 1d"));
    assert!(
        all.contains("`when:`"),
        "the refusal offers the operand at a site that refuses it:\n{all}"
    );
}

#[test]
fn adversary_now_pass2_an_equality_in_a_filter_is_not_told_to_order_against_now() {
    let all = messages(&with_filter("starts_at == now"));
    assert!(
        all.contains("`when:`"),
        "the refusal offers the operand at a site that refuses it:\n{all}"
    );
}
