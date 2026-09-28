//! The suite format authority for field presence policies (beyond10x/ess#139).
//!
//! `docs/design/wire-presence-json-prefix.md` is the binding design. A payload leaf that carries
//! [`presence`](crate::LeafShape::presence) is written in ordinary suite/[`ORDINARY`] or coverage
//! suite/[`COVERAGE`], and each implies every major below it.
//!
//! The number moves because of what an older reader does with the key: the Go and TypeScript
//! runtimes unmarshal a leaf into a type that does not name it and drop it in silence, and an older
//! Rust reader ignores it the same way. Either would pass an implementation that sends `null`
//! where the specification says the key is omitted — a wrong verdict caused by the age of the
//! tool. A reader that checks this number first refuses the suite instead. Both emitted runtimes
//! execute these majors and read the policy (beyond10x/ess#188).

use crate::scenario::{ConformanceSuite, ScenarioStep};

/// The first ordinary suite major that carries presence policies.
pub const ORDINARY: u32 = 24;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = 25;

const REQUIRES: &str = "field presence policies require suite/24 or /25";

/// Whether any payload leaf, or any response observation field, of the suite carries a presence
/// policy.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectEvent { shape, .. }
            | ScenarioStep::EventuallyEvent { shape, .. }
            | ScenarioStep::ExpectEventValues { shape, .. } => {
                shape.leaves().values().any(|leaf| leaf.presence.is_some())
            }
            // A response field's policy travels on the observation's own field list.
            ScenarioStep::ExpectResponsePayload { response } => response
                .fields
                .iter()
                .any(|field| field.presence().is_some()),
            _ => false,
        })
    })
}

/// Refuse an explicitly pinned older format before serialization or target effects.
pub(crate) fn admit_format(
    suite: &ConformanceSuite,
) -> Result<(), crate::admission::AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
