//! Every row a view read returns carries the view's declared fields
//! (story:a-view-read-checks-every-rows-shape, beyond10x/ess#680, `docs/design/view-row-shape.md`).
//!
//! A view step compared only the fields a scenario named, so a target that dropped a declared field
//! from its rows, or answered it with another type, passed every step that did not name it. From
//! suite/[`ORDINARY`] an `expect_view` or `eventually_view` step carries `shape`: the view's
//! declared fields flattened as [`crate::scenario::PayloadShape`] flattens an event, checked on
//! every row of the read with the event's own rule. A declared `Optional` leaf admits absent and
//! `null`, and a key the view does not declare is not refused.
//!
//! An older reader does not know the key, so a suite that carries one is ordinary suite/[`ORDINARY`]
//! or coverage suite/[`COVERAGE`], cumulative over every major below, and every reader refuses it
//! under a lower major before target effects. A suite without one keeps its format and bytes.

use crate::admission::AdmissionError;
use crate::scenario::ScenarioStep;
use crate::ConformanceSuite;

/// The ordinary suite major whose view steps may carry a row shape.
pub const ORDINARY: u32 = crate::suite_pair_66::ORDINARY;

/// Its coverage counterpart.
pub const COVERAGE: u32 = crate::suite_pair_66::COVERAGE;

/// What a refusal under a lower major says.
pub const REQUIRES: &str = "a view row shape requires suite/66 or /67";

/// Whether a view step of the suite carries a row shape.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .any(|step| {
            matches!(
                step,
                ScenarioStep::ExpectView { shape, .. } | ScenarioStep::EventuallyView { shape, .. }
                    if !shape.is_empty()
            )
        })
}

/// Refuses a suite whose view steps carry a row shape under a major below [`ORDINARY`].
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
