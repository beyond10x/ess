//! The `hold_now` step: a suite fixes the instant a target reads as `now` for one command
//! (beyond10x/ess#586, `docs/design/current-time-guards.md`).
//!
//! `{step: hold_now, seconds: <n>}` stands immediately before the `execute_command` it holds. `n`
//! is a `now_offset` number, resolved through the scenario's one table of them
//! ([`crate::now_offset`]), so a held step and the values compared with it name one instant. The
//! runner writes that instant into the command clock it handed out
//! ([`crate::Runner::command_clock`]), or asks the target through
//! [`hold_now`](crate::ConformanceTarget::hold_now); a target without the capability reports the
//! scenario `unsupported`, naming `hold_now`.
//!
//! A suite carrying one is written as ordinary suite/[`ORDINARY`], or coverage suite/[`COVERAGE`];
//! a `/49` reader refuses it by version, and a suite without one keeps its format and bytes.

use crate::admission::AdmissionError;
use crate::{ConformanceSuite, ScenarioStep};

/// The ordinary suite major that carries `hold_now` steps.
pub const ORDINARY: u32 = 50;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 51;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// Whether a suite carries a `hold_now` step.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::HoldNow { .. }))
    })
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuse a suite pinned below `/50` that carries a `hold_now` step, a `hold_now` that does not
/// stand immediately before an `execute_command`, and an offset no `now_offset` may carry.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "hold_now steps require suite/50 or /51",
        ));
    }
    for (id, scenario) in &suite.scenarios {
        for (index, step) in scenario.steps.iter().enumerate() {
            let ScenarioStep::HoldNow { seconds } = step else {
                continue;
            };
            let at = format!("$suite.scenarios.{id}.steps.{index}");
            if seconds.unsigned_abs() > crate::now_offset::MAX_SECONDS.unsigned_abs() {
                return Err(AdmissionError::new(
                    "InvalidSuite",
                    at,
                    format!(
                        "a hold_now offset lies within {} seconds of now",
                        crate::now_offset::MAX_SECONDS
                    ),
                ));
            }
            if !matches!(
                scenario.steps.get(index + 1),
                Some(ScenarioStep::ExecuteCommand { .. })
            ) {
                return Err(AdmissionError::new(
                    "InvalidSuite",
                    at,
                    "a hold_now step stands immediately before the execute_command it holds",
                ));
            }
        }
    }
    Ok(())
}
