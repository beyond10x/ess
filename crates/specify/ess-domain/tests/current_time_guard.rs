//! A `Timestamp` guard compared with the current time and a tolerance (beyond10x/ess#171, source
//! format `ess/16`, `docs/design/current-time-guards.md`).
//!
//! `starts_at < now - 60s` used to be refused as an ordering against a text that is not an RFC 3339
//! instant. From `ess/16` the right-hand side of an ordering over a `Timestamp` in a command's
//! `when:` may be `now`, optionally with a signed whole-unit offset. Below `ess/16` it is refused
//! as `unsupported_format_version`; anywhere but a command's `when:` it stays refused.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const JOBS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/current-time-guard.yaml");

const GUARD: &str = "when: starts_at < now - 60s";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("jobs.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn edited(base: &str, before: &str, after: &str) -> String {
    assert!(base.contains(before), "fixture holds {before:?}");
    base.replacen(before, after, 1)
}

fn validates(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("{}\n---\n{text}", listed(&errors));
    }
}

fn refused(text: &str, code: ValidationCode, needle: &str) -> String {
    let errors = assemble(text).expect_err("refused");
    let all = listed(&errors);
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.message.contains(needle)),
        "expected {code:?} containing {needle:?}:\n{all}"
    );
    all
}

#[test]
fn issue_171_the_repro_validates_under_ess_16() {
    validates(JOBS);
}

#[test]
fn every_spelling_of_now_orders_a_timestamp_input() {
    for guard in [
        "when: starts_at < now",
        "when: starts_at >= now + 5m",
        "when: starts_at <= now - 1h",
        "when: starts_at > now + 90s",
        "when: {starts_at: {lt: now - 60s}}",
        "when: {all: [starts_at > now - 1h, starts_at < now + 1h]}",
    ] {
        validates(&edited(JOBS, GUARD, guard));
    }
}

#[test]
fn below_ess_16_now_is_refused_as_a_format_it_needs() {
    for format in ["ess/15", "ess/14"] {
        let text = edited(JOBS, "format: ess/16", &format!("format: {format}"));
        let all = refused(&text, ValidationCode::UnsupportedFormatVersion, "ess/16");
        assert!(
            all.contains("start-in-past"),
            "the refusal is placed at the guard: {all}"
        );
        assert!(
            !all.contains("not an RFC 3339 instant"),
            "the refusal says what the author needs, not that `now` is no instant: {all}"
        );
    }
}

#[test]
fn a_malformed_offset_is_refused_naming_the_spellings() {
    for guard in [
        "when: starts_at < now - 60",
        "when: starts_at < now - 1d",
        "when: starts_at < now - 060s",
    ] {
        refused(
            &edited(JOBS, GUARD, guard),
            ValidationCode::TypeMismatch,
            "now - 60s",
        );
    }
}

#[test]
fn now_is_ordered_against_never_equated() {
    refused(
        &edited(JOBS, GUARD, "when: starts_at == now"),
        ValidationCode::TypeMismatch,
        "order",
    );
}

#[test]
fn now_outside_a_command_guard_is_refused() {
    // An entity invariant is checked at rest, when no request is being handled.
    let invariant = edited(
        JOBS,
        "    lifecycle: {initial: Scheduled",
        "    invariants: [starts_at > now]\n    lifecycle: {initial: Scheduled",
    );
    refused(&invariant, ValidationCode::TypeMismatch, "`when:`");
    // A view filter is read by a query, not a command.
    let filter = edited(
        JOBS,
        "    consistency: read_your_writes\n",
        "    consistency: read_your_writes\n    filter: starts_at > now\n",
    );
    refused(&filter, ValidationCode::TypeMismatch, "`when:`");
}

#[test]
fn a_text_that_is_no_timestamp_still_compares_with_the_word() {
    let text = edited(
        JOBS,
        "      - {name: starts_at, type: Timestamp}\n    outcomes:",
        "      - {name: starts_at, type: Timestamp}\n      - {name: label, type: String}\n    outcomes:",
    );
    validates(&edited(&text, GUARD, "when: label == now"));
}
