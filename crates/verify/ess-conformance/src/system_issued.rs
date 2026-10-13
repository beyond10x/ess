//! Format authority for a command the system issues itself: suite `/50` ordinary and `/51`
//! coverage (beyond10x/ess#542, `docs/design/system-issued-commands.md`).
//!
//! # What changes meaning
//!
//! From `ess/25` an actor may be the system itself, and a command every grant of which is such an
//! actor is system-issued. Conformance never asks a target to invoke it. Three constructs carry
//! that:
//!
//! | construct | what it asks |
//! |---|---|
//! | [`AwaitOutcome`](ScenarioStep::AwaitOutcome) | an outcome of the command on an instance within the bound; the answer is the scenario's current command result |
//! | [`ExpectNotIssued`](ScenarioStep::ExpectNotIssued) | no outcome of the command on the instance within the bound, and none of its events published |
//! | [`ScenarioValue::Awaited`] | the input the most recent await's answer carried in one field |
//!
//! # What selects it
//!
//! A suite carrying any of the three ([`used_by`]). Every other suite keeps its bytes and its
//! format. A reader that admits only through `/49` refuses a `/50` suite by version, before any
//! target callback; every reader refuses the vocabulary under a lower major.
//!
//! # Cumulative, and Rust only in stage 1
//!
//! `/50` and `/51` are cumulative over every major below them. The Rust runner executes them; the
//! generated Go and TypeScript runtimes do not yet (stage 2), so their emitters keep refusing both
//! majors by version, and so does browser replay.

use crate::{ConformanceSuite, ScenarioStep, ScenarioValue};

/// The ordinary suite major that carries the await vocabulary.
pub const ORDINARY: u32 = 50;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 51;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str =
    "awaiting a system-issued command requires suite/50 or /51 (await_outcome, expect_not_issued, \
     {kind: awaited})";

/// The step tags this vocabulary adds.
const AWAIT: &str = "await_outcome";
const NOT_ISSUED: &str = "expect_not_issued";

/// The value tag this vocabulary adds.
pub(crate) const AWAITED: &str = "awaited";

/// Whether `value`, or any value nested in it, reads an awaited input.
fn reads_awaited(value: &ScenarioValue) -> bool {
    match value {
        ScenarioValue::Awaited { .. } => true,
        ScenarioValue::List { items } => items.iter().any(reads_awaited),
        ScenarioValue::Members { members } => members.values().any(reads_awaited),
        _ => false,
    }
}

/// Whether one step carries the vocabulary.
fn carries(step: &ScenarioStep) -> bool {
    match step {
        ScenarioStep::AwaitOutcome { .. } | ScenarioStep::ExpectNotIssued { .. } => true,
        ScenarioStep::ExpectEventValues { payload, .. } => payload.values().any(reads_awaited),
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => input.values().any(reads_awaited),
        ScenarioStep::ExpectEveryInvocation {
            selecting, input, ..
        } => selecting.values().chain(input.values()).any(reads_awaited),
        ScenarioStep::SnapshotSubject { subject, .. }
        | ScenarioStep::SnapshotCompleteSubject { subject, .. }
        | ScenarioStep::ExpectSubjectAbsent { subject, .. } => subject.values().any(reads_awaited),
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::EventuallyView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => params.values().any(reads_awaited),
        ScenarioStep::ExpectView { expectation, .. } => expectation_reads(expectation),
        _ => false,
    }
}

fn expectation_reads(expectation: &crate::scenario::ViewExpectation) -> bool {
    use crate::scenario::ViewExpectation;
    match expectation {
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => fields.values().any(reads_awaited),
        _ => false,
    }
}

/// Whether a suite carries an await step or an awaited value (suite/50 and /51).
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(carries))
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Whether the Rust reader admits `major`: every major the generated runtimes admit, and this
/// pair, which they refuse by version until the stage-2 runners land.
pub(crate) fn rust_admits(major: u32) -> bool {
    crate::go::admitted_major(major) || ADMITTED.contains(&major)
}

/// Whether `tag` is one of this vocabulary's steps, under a major that does not carry it.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY && matches!(tag, AWAIT | NOT_ISSUED)
}

/// The required and optional keys of this vocabulary's steps, or `None` for another tag.
pub(crate) fn step_keys(tag: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    match tag {
        AWAIT => Some((
            &["step", "command", "instance", "identity"],
            &["within", "input", "shape"],
        )),
        NOT_ISSUED => Some((&["step", "command", "instance"], &["within", "unpublished"])),
        _ => None,
    }
}

/// Refuse an explicitly pinned suite below `/50` that carries the vocabulary.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), crate::AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(crate::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
