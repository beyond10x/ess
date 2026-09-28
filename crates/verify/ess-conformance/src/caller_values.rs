//! The suite format authority for the caller a command is sent as (beyond10x/ess#168, source
//! format `ess/16`, `docs/design/caller-values.md`).
//!
//! An actor may declare `attributes:` its credential carries, and a command may read one as a value
//! (`{caller: account_id}`) or compare one in a guard (`caller.agent_id`). What the command does then
//! depends on **who sends it**, which the suite has to say: every
//! [`ExecuteCommand`](ScenarioStep::ExecuteCommand) of a command whose actor declares attributes
//! carries `caller`, the attribute values of the caller it is sent as, and the target sends the
//! command authenticated as that caller
//! ([`SemanticCommandRequest::caller`](crate::target::SemanticCommandRequest::caller)). A target
//! that cannot authenticate as a given caller answers `unsupported`.
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the
//! round-3 pair [`crate::leaf_payloads`] registered. The number moves because of what an older
//! reader does with the field: it ignores it and sends every command as whoever it is configured to
//! be, so the refusal a caller who is not the record's agent must meet, and the account a created
//! record must carry, are asserted against the wrong caller — a wrong verdict caused by the age of
//! the tool. A reader that checks this number first refuses the suite instead. The Go and
//! TypeScript runtimes refuse these majors by version.

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep};

/// The first ordinary suite major that carries callers.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// Why an older suite major refuses a caller.
pub(crate) const REQUIRES: &str = "caller attribute values require suite/26 or /27";

/// Whether any command step of the suite is sent as a caller.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExecuteCommand { caller, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { caller, .. } => !caller.is_empty(),
            _ => false,
        })
    })
}

/// Refuse an explicitly pinned older format that sends a command as a caller, before
/// serialization or target effects.
///
/// Admission from a document runs this too, after reading each `caller` as a mapping of values, so
/// this one gate refuses the field in a suite of either origin.
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
