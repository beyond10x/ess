//! A fixture that is a row the deployment provides
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds,
//! `docs/design/pre-execution-fixture-values.md` § Provided rows).
//!
//! From `ess/33` an entity may declare `provided: true`: the deployment holds its rows, and no
//! command of the specification need create one. Where synthesis needs such a row and no declared
//! outcome creates it, the scenario's fixture prelude names an entity fixture, `{name, type,
//! entity, state}`, and the scenario sends the fixture wherever it names the row. The target's
//! provider makes or finds a row of `entity` resting in `state` and answers its identity, a value
//! of `type`, the entity's identity type.
//!
//! A suite carrying one is ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], cumulative
//! over every pair below; every reader refuses one under a lower major, and a reader of an older
//! release refuses the fixture's `entity` key as an unknown member. A suite without one, a
//! fixture-free suite and one with type-only fixtures included, keeps its earlier format and bytes.

use crate::admission::AdmissionError;
use crate::scenario::ScenarioStep;
use crate::ConformanceSuite;

/// The ordinary suite major whose fixture prelude may name a row the deployment provides.
pub const ORDINARY: u32 = 60;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 61;

/// The suite majors this module introduces.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal under a lower major says.
pub const REQUIRES: &str =
    "an entity fixture (a row the deployment provides) requires suite/60 or /61";

/// Whether a suite's fixture prelude names a row the deployment provides.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ResolveFixtures { fixtures }
                if fixtures.fields.iter().any(|field| field.entity.is_some()))
        })
    })
}

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`].
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

/// Refuses an entity fixture under a major below [`ORDINARY`].
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if used_by(suite) && suite.provenance.suite_version.major() < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
