//! The suite format authority for invoking a command with no input at all (`input_absent:`,
//! beyond10x/ess#170).
//!
//! A suite carrying [`ExecuteCommandWithoutInput`](ScenarioStep::ExecuteCommandWithoutInput) is
//! written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`] — the round-3 pair
//! [`leaf_payloads`](crate::leaf_payloads) registers, which every round-3 construct shares. A reader
//! older than that pair does not know the step and must refuse the suite rather than send `{}` in
//! its place, which the implementation #170 describes answers differently. The Go and TypeScript
//! runtimes execute the step too (beyond10x/ess#188).

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep};

/// The first ordinary suite major that carries the step.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// Whether any scenario of the suite invokes a command with no input.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExecuteCommandWithoutInput { .. }))
    })
}

/// The step tag this module owns.
const STEP: &str = "execute_command_without_input";

/// Whether a suite document's step `tag` needs a newer major than `major` — the admission check a
/// reader runs on the JSON before it deserializes a step.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    tag == STEP && major < ORDINARY
}

/// Refuse an explicitly pinned older format carrying the step, before serialization or target
/// effects.
pub(crate) fn admit_format(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "a command invoked with no input requires suite/26 or /27",
        ));
    }
    Ok(())
}
