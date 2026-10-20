//! The `<view>/reference/<field>` scenario: a view that reads through a reference of its source,
//! read before and after the row the reference names changes (`ess/28`, beyond10x/ess#564,
//! `docs/design/generic-language-features.md` § G2, navigation).
//!
//! The scenario uses no step or value an older suite lacks; only its identity is new. A suite
//! carrying one is ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the pair `ess/28`'s
//! other vocabulary rides, and every reader refuses the identity under a lower major.

use crate::admission::AdmissionError;
use crate::{ConformanceSuite, ScenarioId};

/// The ordinary suite major that carries the scenario.
pub const ORDINARY: u32 = 54;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 55;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "a view read through a reference (`<view>/reference/<field>`) requires \
                            suite/54 or /55";

/// Whether a suite files a scenario reading a view through a reference.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .keys()
        .any(|id| matches!(id, ScenarioId::ViewReference { .. }))
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuses the scenario under a major below [`ORDINARY`].
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
