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
//! From `/44`, the `expect_event` and `expect_event_values` claims of one event after one command —
//! one act — are met by that command's occurrences of the event as a set, never by the order they
//! are written in: a maximum matching of claims to distinct occurrences carrying their values and
//! shape ([`assignment`]), the same in the Rust, Go and TypeScript runners. Among several maximum
//! matchings the one taken gives each claim, in order, the lowest occurrence it can have. A claim
//! the matching leaves out takes the first occurrence nothing took, whose values are then reported
//! (`ESS-CF-PAYLOAD`); with none left it fails `ESS-CF-EVENT`, naming how many were published and
//! how many the act claims.
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

/// The claim steps of the act at the head of `steps`, in order: every event step up to the next
/// command step.
pub fn act_claim_steps(steps: &[ScenarioStep]) -> Vec<ScenarioStep> {
    steps
        .iter()
        .take_while(|step| {
            !matches!(
                step,
                ScenarioStep::ExecuteCommand { .. }
                    | ScenarioStep::ExecuteCommandWithoutInput { .. }
            )
        })
        .filter(|step| {
            matches!(
                step,
                ScenarioStep::ExpectEvent { .. } | ScenarioStep::ExpectEventValues { .. }
            )
        })
        .cloned()
        .collect()
}

/// The occurrence each claim of one event in one act takes, by claim, from `carries[claim]
/// [occurrence]`: whether that occurrence carries the claim's values and shape.
///
/// A maximum matching of claims to distinct occurrences that carry them, so the order claims are
/// written in never decides how many are met. Where several maximum matchings exist, each claim in
/// turn takes the lowest occurrence some maximum matching gives it, given the choices before it.
/// Every claim left out then takes, in turn, the lowest occurrence nothing took — one that carries
/// none of them, whose values the claim reports — and `None` once none is left. The Go and
/// TypeScript runtimes implement the same function (`assignClaims`).
pub fn assignment(carries: &[Vec<bool>], occurrences: usize) -> Vec<Option<usize>> {
    let claims = carries.len();
    let edge =
        |claim: usize, occurrence: usize| carries[claim].get(occurrence).copied().unwrap_or(false);
    let everyone: Vec<usize> = (0..claims).collect();
    let best = maximum(&edge, &everyone, &vec![false; occurrences]);
    let mut used = vec![false; occurrences];
    let mut taken = vec![None; claims];
    let mut fixed = 0;
    for (claim, slot) in taken.iter_mut().enumerate() {
        let rest: Vec<usize> = (claim + 1..claims).collect();
        for occurrence in 0..occurrences {
            if used[occurrence] || !edge(claim, occurrence) {
                continue;
            }
            used[occurrence] = true;
            if fixed + 1 + maximum(&edge, &rest, &used) == best {
                *slot = Some(occurrence);
                fixed += 1;
                break;
            }
            used[occurrence] = false;
        }
    }
    for slot in &mut taken {
        if slot.is_none() {
            if let Some(free) = used.iter().position(|held| !held) {
                used[free] = true;
                *slot = Some(free);
            }
        }
    }
    taken
}

/// The size of a maximum matching of `claims` to occurrences not `blocked` (augmenting paths).
fn maximum(edge: &impl Fn(usize, usize) -> bool, claims: &[usize], blocked: &[bool]) -> usize {
    fn augment(
        claim: usize,
        edge: &impl Fn(usize, usize) -> bool,
        blocked: &[bool],
        seen: &mut [bool],
        owner: &mut [Option<usize>],
    ) -> bool {
        for occurrence in 0..blocked.len() {
            if blocked[occurrence] || seen[occurrence] || !edge(claim, occurrence) {
                continue;
            }
            seen[occurrence] = true;
            if owner[occurrence].is_none_or(|held| augment(held, edge, blocked, seen, owner)) {
                owner[occurrence] = Some(claim);
                return true;
            }
        }
        false
    }
    let mut owner = vec![None; blocked.len()];
    claims
        .iter()
        .filter(|claim| {
            let mut seen = vec![false; blocked.len()];
            augment(**claim, edge, blocked, &mut seen, &mut owner)
        })
        .count()
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

#[cfg(test)]
mod tests {
    use super::assignment;

    #[test]
    fn the_order_claims_are_written_in_does_not_decide_the_matching() {
        // Occurrences `x` then `y`; a claim naming no value carries both, one naming `x` only `x`.
        assert_eq!(
            assignment(&[vec![true, true], vec![true, false]], 2),
            vec![Some(1), Some(0)]
        );
        assert_eq!(
            assignment(&[vec![true, false], vec![true, true]], 2),
            vec![Some(0), Some(1)]
        );
    }

    #[test]
    fn a_claim_left_out_takes_the_first_occurrence_nothing_took_then_none() {
        // Two claims of `x` against `x` and `y`: the second reports `y`'s values.
        assert_eq!(
            assignment(&[vec![true, false], vec![true, false]], 2),
            vec![Some(0), Some(1)]
        );
        // Three claims against two occurrences: the third is met by none.
        assert_eq!(
            assignment(&[vec![true, true], vec![true, true], vec![true, true]], 2),
            vec![Some(0), Some(1), None]
        );
        assert_eq!(assignment(&[vec![]], 0), vec![None]);
    }

    #[test]
    fn among_maximum_matchings_each_claim_takes_its_lowest_occurrence() {
        assert_eq!(
            assignment(&[vec![true, true, true], vec![true, true, false]], 3),
            vec![Some(0), Some(1)]
        );
        assert_eq!(
            assignment(&[vec![false, true, true], vec![false, true, false]], 3),
            vec![Some(2), Some(1),]
        );
    }
}
