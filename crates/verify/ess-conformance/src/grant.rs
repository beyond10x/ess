//! The suite format authority for the refusal an ungranted actor gets (beyond10x/ess#265).
//!
//! A specification that declares actors says who may invoke what, and the served contract answers
//! one standard refusal — `403` `{refused: "not granted", actor}` — to a caller no grant admits,
//! the same for every command and before the command runs. Synthesis witnesses it per command: a
//! [`Grant`](ScenarioId::Grant) scenario, `<command>/grant/denied`, sends the command as an actor
//! the specification does not grant it and requires
//! [`ExpectNotGranted`](ScenarioStep::ExpectNotGranted), in a model that serves a component. A
//! target answers that refusal as
//! [`TargetError::NotGranted`](crate::target::TargetError::NotGranted).
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the
//! round-3 pair [`crate::leaf_payloads`] registered. The number moves because of what an older
//! reader does with the step: a runner that does not know it skips it, so a target that runs the
//! command for anyone passes a scenario whose whole claim is that it must not — a wrong verdict
//! caused by the age of the tool. A reader that checks this number first refuses the suite
//! instead. The Go and TypeScript runtimes execute the step (beyond10x/ess#188).

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioId, ScenarioStep};

/// The first ordinary suite major that carries the refusal an ungranted actor gets.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// Why an older suite major refuses the refusal.
pub(crate) const REQUIRES: &str =
    "a scenario requiring the refusal an ungranted actor gets requires suite/26 or /27";

/// Whether any scenario of the suite is filed under a grant id or requires the refusal.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        matches!(
            id,
            ScenarioId::Grant { .. } | ScenarioId::GrantAdmitted { .. }
        ) || scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
    })
}

/// Refuse an explicitly pinned older format that requires the refusal, before serialization or
/// target effects.
pub(crate) fn admit_format(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
