//! Format authority for counted event claims: suite `/44` ordinary and `/45` coverage
//! (beyond10x/ess#427).
//!
//! # What changes meaning
//!
//! One command answer can publish one declared event several times — a batch command publishes one
//! occurrence per record it changes — and an outcome says so by listing the event as often in its
//! `emits:`. Synthesis writes one `expect_event` per listed entry, and an author lists one `events:`
//! entry per occurrence. Through `/43` every runner met each such claim with the first occurrence of
//! the event by name, so one occurrence met them all.
//!
//! From `/44`, every `expect_event` (and `expect_event_values`) after one command claims a distinct
//! occurrence of its event that no earlier claim of that act took: the first unclaimed occurrence
//! carrying the claim's values and shape, else the first unclaimed one, whose values are then
//! reported (`ESS-CF-PAYLOAD`). An act whose claims outnumber the occurrences fails `ESS-CF-EVENT`
//! for each claim left over, naming how many were published and how many the act claims.
//!
//! # What selects it
//!
//! A suite one of whose acts — the steps between one command and the next — claims one event more
//! than once ([`used_by`]). Every other suite keeps its bytes and its format, and a suite labelled
//! `/43` or below keeps first-match semantics whatever it carries, so an older document means what
//! it always meant. A reader that admits only through `/43` refuses a `/44` suite by version rather
//! than under-checking it.
//!
//! # Cumulative
//!
//! `/44` and `/45` are cumulative over every major below them, including the seed-bearing pair:
//! a seeded suite whose act claims an event twice is `/44` (or `/45`) and carries its
//! `synthesis_seeds` record there.

use std::collections::BTreeMap;

use crate::scenario::{ConformanceScenario, EventRef, ScenarioStep};
use crate::ConformanceSuite;

/// The ordinary suite major that counts event claims.
pub const ORDINARY: u32 = 44;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 45;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// Whether a suite labelled `major` counts its event claims.
pub fn counts(major: u32) -> bool {
    major >= ORDINARY
}

/// How many times each event is claimed by the act starting at the head of `steps`: every event
/// step up to the next command step.
pub fn act_claims(steps: &[ScenarioStep]) -> BTreeMap<EventRef, usize> {
    let mut claims = BTreeMap::new();
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { .. }
            | ScenarioStep::ExecuteCommandWithoutInput { .. } => break,
            ScenarioStep::ExpectEvent { event, .. }
            | ScenarioStep::ExpectEventValues { event, .. } => {
                *claims.entry(event.clone()).or_insert(0) += 1;
            }
            _ => {}
        }
    }
    claims
}

/// Whether one act of `scenario` claims one event more than once.
pub fn repeats(scenario: &ConformanceScenario) -> bool {
    scenario.steps.iter().enumerate().any(|(at, step)| {
        matches!(
            step,
            ScenarioStep::ExecuteCommand { .. } | ScenarioStep::ExecuteCommandWithoutInput { .. }
        ) && act_claims(&scenario.steps[at + 1..])
            .values()
            .any(|claims| *claims > 1)
    })
}

/// Whether any act of the suite claims one event more than once.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(repeats)
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`], else none.
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}
