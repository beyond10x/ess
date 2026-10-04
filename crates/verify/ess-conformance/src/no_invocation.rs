//! Zero invocations of a binding, observed over the whole eventual window (suite/36 and /37,
//! ess/22, beyond10x/ess#268 and beyond10x/ess#194).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Zero invocation is observable", is the
//! binding design. A binding whose event-payload condition does not hold owes no attempt, and an
//! absent event is no evidence of that: a binding may invoke a command that publishes nothing.
//! So one step and two binding aspects carry it into a suite:
//!
//! | construct | what it asks |
//! |---|---|
//! | [`ExpectNoInvocation`](crate::ScenarioStep::ExpectNoInvocation) | the binding made no invocation of its command under this scenario's correlation, through the deadline |
//! | [`ConditionFalse`](crate::BindingAspect::ConditionFalse) | the scenario where the condition is false with every Optional member it reads present |
//! | [`ConditionAbsent`](crate::BindingAspect::ConditionAbsent) | the scenario where a member the condition proves present is absent |
//!
//! A new step and new aspect words are closed-enum variants an older reader fails on, so a suite
//! carrying either says so in its first line: suite/[`ORDINARY`], and suite/[`COVERAGE`] where it
//! also carries a declared coverage inventory. Each implies every major below it, including the
//! empty scenario initial state of suite/34 and /35. Unaffected suites keep their majors and bytes.

use crate::scenario::{BindingAspect, ConformanceSuite, ScenarioId, ScenarioStep};

/// First ordinary suite that observes zero invocations of a binding.
pub const ORDINARY: u32 = 36;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = 37;

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str = "zero-invocation observation requires suite/36 or /37";

/// The step tag this vocabulary adds.
const STEP: &str = "expect_no_invocation";

fn conditional(id: &ScenarioId) -> bool {
    matches!(
        id,
        ScenarioId::Binding {
            aspect: BindingAspect::ConditionFalse | BindingAspect::ConditionAbsent,
            ..
        }
    )
}

/// Whether any scenario asks for zero invocations, or is filed under a condition aspect.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        conditional(id)
            || scenario
                .steps
                .iter()
                .any(|step| matches!(step, ScenarioStep::ExpectNoInvocation { .. }))
    })
}

/// Whether a coverage inventory refuses a scenario filed under a condition aspect, which a reader
/// older than [`COVERAGE`] cannot parse even where no scenario carries the step.
pub(crate) fn refused_in(inventory: &crate::coverage::Inventory) -> bool {
    inventory
        .refused
        .iter()
        .any(|refusal| refusal.scenario.as_ref().is_some_and(conditional))
}

/// Whether `tag` is this vocabulary and `major` is too old to carry it.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY && tag == STEP
}

/// The required and optional keys of this vocabulary's step, or `None` for another tag.
pub(crate) fn step_keys(tag: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    (tag == STEP).then_some((&["step", "binding", "command"], &[]))
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
