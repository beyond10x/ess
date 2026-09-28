//! Format authority for the ess/15 outcome shapes (`docs/design/outcome-shapes.md`): ordinary
//! suite/22 and coverage suite/23.
//!
//! Two constructs need an expectation no earlier suite could state. A `deletes:` branch is
//! witnessed by the removed subject's **absence** from every immediate view of its entity
//! ([`ExpectSubjectAbsent`](ScenarioStep::ExpectSubjectAbsent)); an `accepts: nothing` branch by
//! every immediate view holding exactly the rows it held before the command
//! ([`SnapshotView`](ScenarioStep::SnapshotView) and
//! [`ExpectViewUnchanged`](ScenarioStep::ExpectViewUnchanged)). A reader older than the construct
//! would refuse either as an unknown step — which blames the document for the age of the tool — so
//! the number turns that into "upgrade the tool". Each major implies the ones below it, so a suite
//! that also needs a lower major's vocabulary still takes this one
//! ([`ConformanceSuite::select_fresh_format`]).
//!
//! The other three constructs need no vocabulary of their own: an unknown-instance branch is an
//! outcome and an error, a creation `into:` a state is a view expectation on `state`, and a
//! precondition is an ordinary command at the head of every scenario. A suite using only those
//! keeps its format and its bytes.
//!
//! The Go and TypeScript readers execute suite/22 and /23 as the Rust runner does
//! (beyond10x/ess#188).

use crate::admission::AdmissionError;
use crate::{ConformanceSuite, ScenarioStep};

/// The first ordinary suite major that carries the absence and whole-view preservation steps.
pub const ORDINARY: u32 = 22;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = 23;

const REQUIRES: &str = "subject absence and whole-view preservation require suite/22 or /23";

/// Whether a suite carries any of the three steps this format owns.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::ExpectSubjectAbsent { .. }
                    | ScenarioStep::SnapshotView { .. }
                    | ScenarioStep::ExpectViewUnchanged { .. }
            )
        })
    })
}

/// Whether an original step `tag` needs a newer major than `major` because this format owns it.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY
        && matches!(
            tag,
            "expect_subject_absent" | "snapshot_view" | "expect_view_unchanged"
        )
}

/// Reject an explicitly pinned older format before serialization or target effects.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
