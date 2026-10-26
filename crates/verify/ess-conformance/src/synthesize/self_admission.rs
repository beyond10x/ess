//! A synthesized suite is held to its own admission before it is returned.
//!
//! Every writer of a suite admits it first ([`ConformanceSuite::to_canonical_json`]), and a suite
//! its own admission refuses stops the writer whole, for the sake of one scenario: an adopter's
//! synthesis at `ess/26` stopped with `InvalidReplay: complete subject snapshot has no comparison`
//! because one disclosure cell kept a read-back and lost its comparison. A scenario admission
//! refuses on its own is withdrawn here instead, and refused with the scenario and the step.

use super::{withdraw, RefusalCause, ScenarioStep, Synthesis, WitnessGap};
use crate::scenario::{ConformanceScenario, ConformanceSuite, ScenarioId};

/// The type the refusal of a withdrawn scenario names, which [`RefusalCause::hint`] reads it by.
pub(crate) const SELF_ADMISSION: &str = "synthesized suite admission";

/// What became of a withdrawn scenario, as its refusal's line ends.
const WITHDRAWN: &str = "the scenario is withdrawn rather than written";

/// The line a withdrawn scenario's refusal reads: the suite's own admission refuses, then where,
/// as a step's path and kind or the scenario's path, and the admission issue, then
/// [`WITHDRAWN`]. `refused` is the part between, which the cause carries.
pub(crate) fn line(refused: &str) -> String {
    format!("the suite's own admission refuses {refused}; {WITHDRAWN}")
}

/// Withdraws and refuses each scenario of `synthesis` its own admission refuses alone.
///
/// Nothing happens to a suite that is admitted, which is every suite synthesis wrote before, so
/// their bytes stand. A refusal no single scenario carries — one the suite refuses with no
/// scenario in it — is not attributed to any, and is left for the writer to report as it was. The
/// suite keeps the version it was selected at: what the rest uses is vocabulary of that version.
pub(super) fn withdraw_inadmissible(synthesis: &mut Synthesis) {
    if crate::admission::suite(&synthesis.suite).is_ok() {
        return;
    }
    let mut alone = ConformanceSuite::new(synthesis.suite.provenance.clone());
    if crate::admission::suite(&alone).is_err() {
        return;
    }
    let mut refused = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        alone.scenarios.insert(id.clone(), scenario.clone());
        if let Err(error) = crate::admission::suite(&alone) {
            refused.push((id.clone(), cause(id, scenario, &error)));
        }
        alone.scenarios.clear();
    }
    for (id, cause) in refused {
        withdraw(synthesis, &id, cause);
    }
}

/// The refusal of `scenario`, naming the step where admission locates one.
fn cause(
    id: &ScenarioId,
    scenario: &ConformanceScenario,
    error: &crate::admission::AdmissionError,
) -> RefusalCause {
    let scenario_path = format!("/scenarios/{}", crate::admission::escape(&id.to_string()));
    let refused = if let Err(refusal) = crate::replay::validate_steps(&scenario.steps) {
        format!(
            "step `{scenario_path}/steps/{}` (`{}`) with InvalidReplay: {}",
            refusal.step,
            tag(&scenario.steps[refusal.step]),
            refusal.reason
        )
    } else {
        let issues: Vec<_> = error
            .issues
            .iter()
            .map(|issue| format!("{}: {}", issue.reason, issue.detail))
            .collect();
        format!(
            "scenario `{scenario_path}`, at no one step, with {}",
            issues.join("; ")
        )
    };
    RefusalCause::NoWitness(WitnessGap {
        path: refused,
        type_ref: SELF_ADMISSION.into(),
        reason: WITHDRAWN,
    })
}

/// A step's name as the suite spells it.
fn tag(step: &ScenarioStep) -> String {
    serde_json::to_value(step)
        .ok()
        .and_then(|value| value.get("step")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unnamed step".into())
}
