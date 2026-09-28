//! The suite format authority for a binding's bounded retry (ess/16 `on_failure: {retry:
//! {attempts, final}}`, beyond10x/ess#165).
//!
//! `docs/design/binding-delivery-guarantees.md`, "A bound that is the component's behaviour", is
//! the binding design. A bounded retry is witnessed by two step fields and one binding aspect:
//!
//! * [`ConfigureExternalOutcome::times`](crate::ScenarioStep::ConfigureExternalOutcome) forces the
//!   outcome on the next `times` invocations rather than on the next one only;
//! * [`ExpectInvocation::count`](crate::ScenarioStep::ExpectInvocation) requires exactly that many
//!   invocations by the binding;
//! * [`crate::BindingAspect::FinalFailure`] files the scenario in which
//!   a `final` refusal is forced once and exactly one invocation is required.
//!
//! Such a suite is written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the
//! round-3 pair [`crate::leaf_payloads`] registered. The number moves because of what an older
//! reader does with the fields: it forces one failure instead of `times`, and reads a count as "at
//! least one", so it passes a sender that retries forever — a wrong verdict caused by the age of the
//! tool. The Go and TypeScript runtimes execute both fields
//! too (beyond10x/ess#188).

use crate::admission::AdmissionError;
use crate::scenario::{BindingAspect, ConformanceSuite, ScenarioId, ScenarioStep};

/// The first ordinary suite major that carries a repeated forced outcome or an invocation count.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

pub(crate) const REQUIRES: &str =
    "a repeated forced outcome, an invocation count or a final-failure scenario requires suite/26 \
     or /27";

/// Whether any scenario of the suite forces an outcome more than once, counts invocations, or is
/// filed under the final-failure aspect.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        matches!(
            id,
            ScenarioId::Binding {
                aspect: BindingAspect::FinalFailure,
                ..
            }
        ) || scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::ConfigureExternalOutcome { times: Some(_), .. }
                    | ScenarioStep::ExpectInvocation { count: Some(_), .. }
            )
        })
    })
}

/// Whether a coverage inventory refuses a scenario filed under the final-failure aspect, which a
/// reader older than [`COVERAGE`] cannot parse even where no scenario carries a count.
pub(crate) fn refused_in(inventory: &crate::coverage::Inventory) -> bool {
    inventory.refused.iter().any(|refusal| {
        matches!(
            refusal.scenario,
            Some(ScenarioId::Binding {
                aspect: BindingAspect::FinalFailure,
                ..
            })
        )
    })
}

/// Refuse an explicitly pinned older format carrying either field, before serialization or target
/// effects.
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

/// The keys a step document may carry at `major`, where this construct widens them: `None` for a
/// step it does not touch or a major older than [`ORDINARY`].
pub(crate) fn step_keys(
    tag: &str,
    major: u32,
) -> Option<(&'static [&'static str], &'static [&'static str])> {
    if major < ORDINARY {
        return None;
    }
    match tag {
        "configure_external_outcome" => Some((&["step", "force"], &["times"])),
        "expect_invocation" => Some((&["step", "binding", "command"], &["input", "count"])),
        _ => None,
    }
}

/// A `times` or `count` value: a whole number of at least one, as written.
pub(crate) fn admit_positive(raw: &str) -> Result<(), String> {
    serde_json::from_str::<std::num::NonZeroU32>(raw)
        .map(|_| ())
        .map_err(|error| format!("a repetition is a whole number of at least 1: {error}"))
}
