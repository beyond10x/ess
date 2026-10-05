//! The suite format authority for reads of a read-granted view (beyond10x/ess#286).
//!
//! From source `ess/22` an actor's `may:` may name a view: only the actors naming it may read it,
//! and a served surface answers anyone else the standard refusal a command answers an ungranted
//! actor, `403` `{refused: "not granted", actor}`. A view no actor names stays open. Synthesis
//! sends every read of a read-granted view as an actor, with
//! [`ReadAs`](ScenarioStep::ReadAs), and witnesses the grant per view: a
//! [`ReadGrant`](ScenarioId::ReadGrant) scenario, `<view>/grant/read/denied`, reads the view as an
//! actor the grant does not name and requires
//! [`ExpectNotGranted`](ScenarioStep::ExpectNotGranted) of the read, and a
//! [`ReadGrantAdmitted`](ScenarioId::ReadGrantAdmitted) scenario per granted actor reads it served.
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the pair
//! of the release that admits the source. The number moves because of what an older reader does
//! with the step: it cannot send a read as an actor, so it would read a read-granted view as no
//! actor and blame the target for the refusal it owes. A reader that checks this number first
//! refuses the suite instead. The Go and TypeScript runtimes send the actor on
//! each read and read the refusal from the view result.

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioId, ScenarioStep};

/// The first ordinary suite major that carries reads of a read-granted view.
pub const ORDINARY: u32 = crate::one_time_response::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::one_time_response::COVERAGE;

/// Why an older suite major refuses a read sent as an actor.
pub(crate) const REQUIRES: &str =
    "a read sent as an actor or as no actor (`read_as`, a read-granted view) requires suite/34 or /35";

/// Whether any scenario of the suite is filed under a read-grant id or sends a read as an actor.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        matches!(
            id,
            ScenarioId::ReadGrant { .. } | ScenarioId::ReadGrantAdmitted { .. }
        ) || scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::ReadAs { .. } | ScenarioStep::ExpectNotGranted { actor: None, .. }
            )
        })
    })
}

/// Whether `tag`, a step's wire name, needs a newer suite major than `major`.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY && tag == "read_as"
}

/// Refuse an explicitly pinned older format that sends a read as an actor, before serialization
/// or target effects.
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
