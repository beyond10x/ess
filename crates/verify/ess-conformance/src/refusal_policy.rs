//! The suite format authority for a binding's refusal-selected failure policy (ess/22,
//! beyond10x/ess#269).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Required policy controls", is the
//! binding design. Each declared refusal the policy selects is witnessed on a scenario of its own
//! with the existing repeated-force, invocation-count and event steps, plus one step the existing
//! ones cannot say:
//!
//! | construct | what it asks |
//! |---|---|
//! | [`ScenarioId::BindingRefusal`] | `<binding>/binding/refusal/<outcome>`, the scenario for one refusal |
//! | [`ExpectNoPublication`](crate::ScenarioStep::ExpectNoPublication) | the event was not published under this scenario's correlation, through the deadline |
//! | [`ExpectPublicationCount`](crate::ScenarioStep::ExpectPublicationCount) | the event was published exactly `count` times under it, through the deadline: one escalation per escalating attempt |
//!
//! The negative is a step because [`ExpectNoEvent`](crate::ScenarioStep::ExpectNoEvent) reads
//! the last command's direct events only, and an escalation is published by the binding after
//! that command returned: without it, a sender that escalates a refusal it should drop passes.
//!
//! Both are vocabulary an older reader fails on, so a suite carrying either says so in its first
//! line: suite/[`ORDINARY`], and suite/[`COVERAGE`] where it carries a coverage inventory. They are
//! the binding pair this bundle allocated for zero-invocation observation
//! ([`crate::no_invocation`]); unaffected suites keep their majors and bytes.

use crate::scenario::{ConformanceSuite, ScenarioId, ScenarioStep};

/// First ordinary suite that files a scenario per selected refusal.
pub const ORDINARY: u32 = crate::no_invocation::ORDINARY;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = crate::no_invocation::COVERAGE;

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str = "a scenario per selected refusal requires suite/36 or /37";

/// The step tags this vocabulary adds.
const STEP: &str = "expect_no_publication";
const COUNT_STEP: &str = "expect_publication_count";

fn selected(id: &ScenarioId) -> bool {
    matches!(id, ScenarioId::BindingRefusal { .. })
}

/// Whether any scenario is filed under a selected refusal, or asks that an event go unpublished.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        selected(id)
            || scenario.steps.iter().any(|step| {
                matches!(
                    step,
                    ScenarioStep::ExpectNoPublication { .. }
                        | ScenarioStep::ExpectPublicationCount { .. }
                )
            })
    })
}

/// Whether a coverage inventory refuses a scenario filed under a selected refusal, which a reader
/// older than [`COVERAGE`] cannot parse even where no scenario carries one.
pub(crate) fn refused_in(inventory: &crate::coverage::Inventory) -> bool {
    inventory
        .refused
        .iter()
        .any(|refusal| refusal.scenario.as_ref().is_some_and(selected))
}

/// Whether `tag` is this vocabulary's step and `major` is too old to carry it.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY && (tag == STEP || tag == COUNT_STEP)
}

/// The required and optional keys of this vocabulary's step, or `None` for another tag.
pub(crate) fn step_keys(tag: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    match tag {
        STEP => Some((&["step", "event"], &[])),
        COUNT_STEP => Some((&["step", "event", "count"], &[])),
        _ => None,
    }
}

/// The same version rule for a suite built in memory rather than read from bytes.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), crate::AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(crate::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
