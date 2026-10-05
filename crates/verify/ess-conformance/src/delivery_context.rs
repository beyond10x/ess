//! Delivering an event from an external channel with its delivery context (suite/30 and /31,
//! ess/18, beyond10x/ess#195).
//!
//! A binding that declares `when.context_fields` reacts to an event nothing in the specification
//! publishes, and reads part of its command input from the context the channel binds. Two steps
//! carry that into a suite:
//!
//! | step | what it asks | of whom |
//! |---|---|---|
//! | [`DeliverEvent`](crate::ScenarioStep::DeliverEvent) | deliver one occurrence, with these fields and this context | [`deliver_event`](crate::target::ConformanceTarget::deliver_event), optional |
//! | [`ExpectEveryInvocation`](crate::ScenarioStep::ExpectEveryInvocation) | every invocation made for one occurrence carries these values | [`observe_invocations`](crate::target::ConformanceTarget::observe_invocations) |
//!
//! A new step is a new closed-enum variant an older reader fails on with `unknown variant`, so a
//! suite carrying either one says so in its first line: suite/[`ORDINARY`], and
//! suite/[`COVERAGE`] where it also carries a declared coverage inventory. Each implies every
//! major below it. The generated Go and TypeScript runners do not execute these steps yet, and
//! generating either for such a suite is refused rather than written with scenarios that could
//! only be skipped.

/// First ordinary suite that delivers events from an external channel.
pub const ORDINARY: u32 = 30;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = 31;

/// What an older envelope reports for this vocabulary.
pub const REQUIRES: &str = "delivery with context requires suite/30 or /31";

/// The two step tags this vocabulary adds.
const STEPS: [&str; 2] = ["deliver_event", "expect_every_invocation"];

/// Whether this suite delivers an event with its delivery context, or asks of every invocation.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(
                step,
                crate::ScenarioStep::DeliverEvent { .. }
                    | crate::ScenarioStep::ExpectEveryInvocation { .. }
            )
        })
    })
}

/// Whether `tag` is this vocabulary and `major` is too old to carry it.
pub(crate) fn needs_newer(tag: &str, major: u32) -> bool {
    major < ORDINARY && STEPS.contains(&tag)
}

/// The required and optional keys of this vocabulary's steps, or `None` for another tag.
pub(crate) fn step_keys(tag: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    match tag {
        "deliver_event" => Some((&["step", "event", "authority", "context"], &["payload"])),
        "expect_every_invocation" => {
            Some((&["step", "binding", "command", "input"], &["selecting"]))
        }
        _ => None,
    }
}

/// The same version rule for a suite built in memory rather than read from bytes.
pub(crate) fn admit(suite: &crate::ConformanceSuite) -> Result<(), crate::AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(crate::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
