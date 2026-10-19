//! Format authority for a command the system issues itself: suite `/52` ordinary and `/53`
//! coverage (beyond10x/ess#542, `docs/design/system-issued-commands.md`).
//!
//! # What changes meaning
//!
//! From `ess/27` an actor may be the system itself, and a command every grant of which is such an
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
//! format. A reader that admits only through `/51` refuses a `/52` suite by version, before any
//! target callback; every reader refuses the vocabulary under a lower major.
//!
//! # Cumulative, in every runner
//!
//! `/52` and `/53` are cumulative over every major below them. The Rust runner and the generated
//! Go and TypeScript runtimes execute them (stage 2: `AwaitTarget` in Go, `awaitOutcome?` in
//! TypeScript). Browser replay replays only `/5`, `/9` and `/35`, and refuses both by version.

use std::collections::BTreeSet;
use std::fmt;

use ess_compiler::ir::{ResolvedCommand, ResolvedInstance};
use ess_domain::command::OutcomeName;

use crate::{ConformanceSuite, ScenarioStep, ScenarioValue};

/// Why a system-issued command names no one existing record to await its outcome on. Stage 1
/// awaits only a step on an existing record, named by its outcomes' `instance:`; synthesis refuses
/// the rest as `ESS-SYNTH-023` and the interpreter answers them unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unaddressed {
    /// An outcome creates the record it addresses, so there is none before the step.
    Creates {
        /// The creating outcome.
        outcome: OutcomeName,
    },
    /// No outcome addresses an existing record through `instance:` (only `instances:` or
    /// `affects:`, or nothing).
    NoInstance,
    /// The outcomes address records through different inputs, so no one input is the instance.
    SeveralInputs {
        /// The inputs, in name order.
        fields: Vec<String>,
    },
}

impl fmt::Display for Unaddressed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Creates { outcome } => write!(
                f,
                "its outcome `{outcome}` creates the record it addresses, so no record exists \
                 before the step"
            ),
            Self::NoInstance => f.write_str(
                "no outcome addresses an existing record through `instance:`, so there is no \
                 instance to await",
            ),
            Self::SeveralInputs { fields } => write!(
                f,
                "its outcomes address records through different inputs ({}), so the awaited \
                 instance names no one input",
                fields
                    .iter()
                    .map(|field| format!("`{field}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// The input every outcome of a system-issued command that addresses an existing record names as
/// its `instance:`: the identity of the record its outcome is awaited on. One rule for synthesis,
/// which writes the await, and the interpreter, which answers it.
///
/// # Errors
///
/// [`Unaddressed`], where the command names no one existing record.
pub fn identity_input(command: &ResolvedCommand) -> Result<&str, Unaddressed> {
    let mut named = BTreeSet::new();
    for outcome in &command.outcomes {
        match outcome.subject.as_ref().map(|subject| &subject.instance) {
            Some(ResolvedInstance::Supplied { field }) => {
                named.insert(field.name.as_str());
            }
            Some(ResolvedInstance::Observed { .. }) => {
                return Err(Unaddressed::Creates {
                    outcome: outcome.name.clone(),
                });
            }
            None => {}
        }
    }
    let mut fields = named.into_iter();
    match (fields.next(), fields.next()) {
        (Some(field), None) => Ok(field),
        (None, _) => Err(Unaddressed::NoInstance),
        (Some(first), Some(second)) => Err(Unaddressed::SeveralInputs {
            fields: [first, second]
                .into_iter()
                .chain(fields)
                .map(str::to_owned)
                .collect(),
        }),
    }
}

/// The ordinary suite major that carries the await vocabulary.
pub const ORDINARY: u32 = 52;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 53;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str =
    "awaiting a system-issued command requires suite/52 or /53 (await_outcome, expect_not_issued, \
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

/// Whether a suite carries an await step or an awaited value (suite/52 and /53).
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

/// The synthesis refusals this vocabulary adds (`ESS-SYNTH-023`, `-024`): a coverage suite
/// carries them only from [`COVERAGE`], which no older reader admits.
pub const REFUSALS: [u16; 2] = [
    crate::synthesize::SYSTEM_ISSUED_UNADDRESSED,
    crate::synthesize::SYSTEM_DECIDED_REFUSAL,
];

/// Whether `code` is one of [`REFUSALS`].
pub(crate) fn is_refusal(code: &str) -> bool {
    REFUSALS
        .iter()
        .any(|number| code == format!("ESS-SYNTH-{number:03}"))
}

/// Whether a coverage inventory carries one of [`REFUSALS`]: it then needs coverage [`COVERAGE`]
/// even where every awaited scenario was refused.
pub(crate) fn refused_in(inventory: &crate::coverage::Inventory) -> bool {
    inventory
        .refused
        .iter()
        .any(|refusal| is_refusal(&refusal.code))
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

/// Refuse an explicitly pinned suite below `/52` that carries the vocabulary.
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
