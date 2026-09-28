//! Adversary, story:defined-over-optional-aggregates (beyond10x/ess#176), pass 1: admission.
//!
//! `defined()` over an `Optional` aggregate is admitted under `ess/16` and refused below it as
//! `unsupported_format_version` in *every* predicate position, not only the entity invariant and
//! the command `when` the unit's own tests exercise: a `when_subject` stored-field guard, a view
//! `filter:`, `missing(x)` and the structured `{x: {exists: true}}` form.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("queue.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The fixture without its entity invariant, so that only the construct under test can be refused.
fn without_invariant() -> String {
    let text = QUEUE.replace(
        "    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "",
    );
    assert_ne!(text, QUEUE);
    text
}

/// A `when_subject` stored-field guard over the queue's `Optional<Metrics>`.
fn when_subject(predicate: &str) -> String {
    without_invariant()
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.queue.QueueHoldsMetrics\n    summary: The queue still holds metrics.\n    fields: []\n",
        )
        .replace(
            "events:\n",
            &format!(
                "  - name: demo.queue.DrainQueue\n    input:\n      - {{name: queue_id, type: demo.queue.QueueId}}\n    outcomes:\n      - name: refused-holding\n        when_subject: {{predicate: '{predicate}'}}\n        error: demo.queue.QueueHoldsMetrics\n      - name: drained\n        moves: demo.queue.Queue.resume\n        instance: queue_id\n        emits: [demo.queue.QueueResumed]\n        payload: {{demo.queue.QueueResumed: {{queue_id: input.queue_id}}}}\n      - {{name: wrong-state, wrong_state: true, error: demo.queue.QueueStateConflict}}\nevents:\n"
            ),
        )
        .replace(
            "may: [demo.queue.OpenQueue,",
            "may: [demo.queue.DrainQueue, demo.queue.OpenQueue,",
        )
}

/// A view whose `filter:` reads the queue's `Optional<Metrics>`.
fn view_filter(filter: &str) -> String {
    format!(
        "{}  - name: demo.queue.HoldingQueues\n    source: demo.queue.Queue\n    consistency: read_your_writes\n    filter: '{filter}'\n    fields:\n      - {{name: queue_id, type: demo.queue.QueueId}}\n      - {{name: metrics, type: Optional<demo.queue.Metrics>}}\n",
        without_invariant()
    )
}

fn invariant(predicate: &str) -> String {
    QUEUE.replace(
        "      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        &format!("      - {predicate}\n"),
    )
}

fn at_ess_15(text: &str) -> String {
    text.replace("format: ess/16", "format: ess/15")
}

fn admitted(name: &str, text: &str) {
    if let Err(errors) = assemble(text) {
        panic!(
            "{name} must validate under ess/16:\n{}\n{text}",
            listed(&errors)
        );
    }
}

fn refused_as_format(name: &str, text: &str) {
    let Err(errors) = assemble(&at_ess_15(text)) else {
        panic!("{name} must be refused under ess/15, and validated");
    };
    assert!(
        errors.as_slice().iter().any(|error| error.code
            == ValidationCode::UnsupportedFormatVersion
            && error.message.contains("ess/16")),
        "{name} under ess/15 is not refused as unsupported_format_version naming ess/16:\n{}",
        listed(&errors)
    );
}

#[test]
fn adversary_defined_a_when_subject_guard_over_an_optional_struct_is_admitted_and_gated() {
    for predicate in ["defined(metrics)", "missing(metrics)"] {
        let text = when_subject(predicate);
        admitted(&format!("when_subject `{predicate}`"), &text);
        refused_as_format(&format!("when_subject `{predicate}`"), &text);
    }
}

#[test]
fn adversary_defined_a_view_filter_over_an_optional_struct_is_admitted_and_gated() {
    for filter in ["defined(metrics)", "missing(metrics)"] {
        let text = view_filter(filter);
        admitted(&format!("view filter `{filter}`"), &text);
        refused_as_format(&format!("view filter `{filter}`"), &text);
    }
}

#[test]
fn adversary_defined_missing_and_the_structured_form_are_admitted_and_gated_in_an_invariant() {
    for predicate in [
        "{any: [state == Paused, \"missing(metrics)\"]}",
        "{any: [state == Paused, {metrics: {exists: false}}]}",
        "{any: [state == Paused, {not: {metrics: {defined: true}}}]}",
    ] {
        let text = invariant(predicate);
        assert!(
            text.contains(predicate),
            "fixture edit did not land: {predicate}"
        );
        admitted(&format!("invariant `{predicate}`"), &text);
        refused_as_format(&format!("invariant `{predicate}`"), &text);
    }
}
