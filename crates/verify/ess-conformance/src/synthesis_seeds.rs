//! The persisted provenance of explicit synthesis seeds (suite/42 and /43, beyond10x/ess#413,
//! `docs/design/synthesis-seeds.md`).
//!
//! A seed is an authored setup row an operator selected by file and instance so that synthesis can
//! build a generated obligation no bounded command arrangement reaches. The suite records which
//! sources were read, the exact rows admitted, and where each row was used. The record is closed
//! and concrete; it is not a generic facet. Every seed-free suite omits it and keeps its format.
use std::collections::{BTreeMap, BTreeSet};

use ess_domain::entity::StateName;
use ess_primitives::node::Node;

use crate::admission::AdmissionError;
use crate::coverage::{Inventory, OutsideReason, SourceIdentity};
use crate::scenario::{
    ConformanceSuite, EntityRef, InstanceName, ScenarioId, ScenarioStep, ScenarioValue,
};

/// The ordinary suite major that carries [`SynthesisSeeds`].
pub const ORDINARY: u32 = 42;
/// The declared-coverage suite major that carries [`SynthesisSeeds`].
pub const COVERAGE: u32 = 43;
/// The most selections one request admits.
pub const MAX_SELECTIONS: usize = 64;

/// Which explicit seeds a suite was synthesized with, and where each was used.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisSeeds {
    /// Every source read, by checked root-relative identity, with the `sha256:` digest of its
    /// original UTF-8 bytes. Nonempty.
    pub sources: BTreeMap<SourceIdentity, String>,
    /// Every admitted row, used or not, sorted by source and instance. Nonempty.
    pub selections: Vec<SeedRecord>,
    /// Every use in an emitted generated scenario, sorted by scenario and setup step.
    pub applications: Vec<SeedApplication>,
}

/// One admitted seed row: the exact initial setup of one authored arrangement.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedRecord {
    /// The source the row was selected from.
    pub source: SourceIdentity,
    /// The arrangement of that source.
    pub instance: InstanceName,
    /// The declared entity.
    pub entity: EntityRef,
    /// The literal identity.
    pub identity: Node,
    /// The literal fields, with Optional absence and present null as written.
    pub fields: BTreeMap<String, Node>,
    /// The declared lifecycle state.
    pub state: StateName,
}

/// One use of a selected row: the generated scenario, its `establish_entity` step and the real
/// command step addressing the row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedApplication {
    /// The selection's source.
    pub source: SourceIdentity,
    /// The selection's instance.
    pub instance: InstanceName,
    /// The generated scenario holding the use.
    pub scenario: ScenarioId,
    /// Index of the `establish_entity` step establishing exactly the selected row.
    pub establish_step: usize,
    /// Index of the command step sent to that row; the next step asserts its outcome.
    pub command_step: usize,
}

impl SeedRecord {
    fn key(&self) -> (&SourceIdentity, &InstanceName) {
        (&self.source, &self.instance)
    }
}

impl SeedApplication {
    fn key(&self) -> (&ScenarioId, usize) {
        (&self.scenario, self.establish_step)
    }
}

/// Whether a suite carries seed provenance.
pub(crate) fn used_by(suite: &ConformanceSuite) -> bool {
    suite.provenance.synthesis_seeds.is_some()
}

/// Whether a major is one of the seed-bearing pair.
pub(crate) fn seed_major(major: u32) -> bool {
    major == ORDINARY || major == COVERAGE
}

/// Whether a suite labelled `major` may carry seed provenance: the seed-bearing pair, which requires
/// it, and the counted event-claim pair above it, which is cumulative over it (beyond10x/ess#427).
pub(crate) fn admitted_in(major: u32) -> bool {
    seed_major(major) || crate::event_multiplicity::ADMITTED.contains(&major)
}

/// What refuses seed provenance under any other major.
pub(crate) const OUTSIDE: &str =
    "synthesis seeds require suite/42 or /43, or the counted event-claim pair /44 or /45";

fn refuse(path: &str, detail: impl Into<String>) -> AdmissionError {
    AdmissionError::new("InvalidSynthesisSeeds", path, detail)
}

/// The setup steps of `suite`'s generated scenarios, read off the emitted steps, bound to the
/// selections they establish: what a writer records as applications.
pub(crate) fn applications(
    suite: &ConformanceSuite,
    selections: &[SeedRecord],
) -> Vec<SeedApplication> {
    let mut out = Vec::new();
    for (id, scenario) in &suite.scenarios {
        if matches!(id, ScenarioId::Authored { .. }) {
            continue;
        }
        for (at, step) in scenario.steps.iter().enumerate() {
            let ScenarioStep::EstablishEntity { instance, .. } = step else {
                continue;
            };
            let Some(selection) = selections.iter().find(|record| establishes(step, record)) else {
                continue;
            };
            let Some(command) = addressed(&scenario.steps, at, instance) else {
                continue;
            };
            out.push(SeedApplication {
                source: selection.source.clone(),
                instance: selection.instance.clone(),
                scenario: id.clone(),
                establish_step: at,
                command_step: command,
            });
        }
    }
    out.sort_by(|left, right| left.key().cmp(&right.key()));
    out
}

/// Whether `step` establishes exactly `record`'s row.
fn establishes(step: &ScenarioStep, record: &SeedRecord) -> bool {
    matches!(step, ScenarioStep::EstablishEntity { entity, identity, fields, state, .. }
        if *entity == record.entity
            && *identity == record.identity
            && *fields == record.fields
            && *state == record.state)
}

/// The first command step after `at` sending `instance`, where its next step asserts the outcome
/// of that command.
fn addressed(steps: &[ScenarioStep], at: usize, instance: &InstanceName) -> Option<usize> {
    let reference = ScenarioValue::instance(instance.clone());
    let position = steps
        .iter()
        .enumerate()
        .skip(at + 1)
        .find_map(|(n, step)| {
            matches!(step, ScenarioStep::ExecuteCommand { input, .. }
            if input.values().any(|value| *value == reference))
            .then_some(n)
        })?;
    let ScenarioStep::ExecuteCommand { command, .. } = &steps[position] else {
        return None;
    };
    matches!(steps.get(position + 1), Some(ScenarioStep::ExpectOutcome { outcome })
        if outcome.command == *command)
    .then_some(position)
}

/// Refuse seed provenance outside suite/42 through /45, its absence in /42 and /43, and any record
/// that is not bound to the suite's own steps. `coverage` is the admitted inventory, where there
/// is one: a selected coverage suite may name an application whose scenario its selection filter
/// moved outside, and its retained parent proves that use.
pub(crate) fn admit(
    suite: &ConformanceSuite,
    coverage: Option<&Inventory>,
) -> Result<(), AdmissionError> {
    let major = suite.provenance.suite_version.major();
    let Some(seeds) = &suite.provenance.synthesis_seeds else {
        if seed_major(major) {
            return Err(AdmissionError::new(
                "UnsupportedVocabulary",
                "/provenance/synthesis_seeds",
                "suite/42 and /43 require synthesis_seeds",
            ));
        }
        return Ok(());
    };
    if !admitted_in(major) {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "/provenance/synthesis_seeds",
            OUTSIDE,
        ));
    }
    admit_selections(seeds)?;
    let bound = admit_applications(suite, coverage, seeds)?;
    for (id, scenario) in &suite.scenarios {
        if matches!(id, ScenarioId::Authored { .. }) {
            continue;
        }
        for (at, step) in scenario.steps.iter().enumerate() {
            if matches!(step, ScenarioStep::EstablishEntity { .. }) && !bound.contains(&(id, at)) {
                return Err(refuse(
                    PATH,
                    format!("generated scenario `{id}` establishes a row no application binds"),
                ));
            }
        }
    }
    Ok(())
}

/// Where a reader names the seed record in a refusal.
const PATH: &str = "/provenance/synthesis_seeds";

/// The sources and selections alone: nonempty, bounded, sorted, distinct, each source used and
/// each row a structurally valid setup row with its own qualified identity.
fn admit_selections(seeds: &SynthesisSeeds) -> Result<(), AdmissionError> {
    let path = PATH;
    if seeds.sources.is_empty() || seeds.selections.is_empty() {
        return Err(refuse(path, "sources and selections must be nonempty"));
    }
    if seeds.selections.len() > MAX_SELECTIONS {
        return Err(refuse(path, "more than 64 selections"));
    }
    for digest in seeds.sources.values() {
        if !crate::coverage::valid_digest(digest) {
            return Err(refuse(path, "invalid source digest"));
        }
    }
    if !seeds
        .selections
        .windows(2)
        .all(|pair| pair[0].key() < pair[1].key())
    {
        return Err(refuse(path, "selections must be sorted and distinct"));
    }
    let mut used = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for record in &seeds.selections {
        if !seeds.sources.contains_key(&record.source) {
            return Err(refuse(path, "a selection names an unknown source"));
        }
        used.insert(&record.source);
        let rendered = serde_json::to_string(&record.identity)
            .map_err(|error| refuse(path, error.to_string()))?;
        if !identities.insert((record.entity.to_string(), rendered)) {
            return Err(refuse(path, "two selections share one qualified identity"));
        }
        crate::admission::setup_row(&record.identity, &record.fields, path)?;
    }
    if used.len() != seeds.sources.len() {
        return Err(refuse(path, "a source is selected by no selection"));
    }
    Ok(())
}

/// Every application bound to a known selection and to its scenario's own setup and asserted
/// command steps; returns the setup steps they bind.
fn admit_applications<'a>(
    suite: &'a ConformanceSuite,
    coverage: Option<&Inventory>,
    seeds: &'a SynthesisSeeds,
) -> Result<BTreeSet<(&'a ScenarioId, usize)>, AdmissionError> {
    let path = PATH;
    let major = suite.provenance.suite_version.major();
    if !seeds
        .applications
        .windows(2)
        .all(|pair| pair[0].key() < pair[1].key())
    {
        return Err(refuse(path, "applications must be sorted and distinct"));
    }
    let mut bound = BTreeSet::new();
    for application in &seeds.applications {
        let Some(record) = seeds
            .selections
            .iter()
            .find(|record| record.key() == (&application.source, &application.instance))
        else {
            return Err(refuse(path, "an application names an unknown selection"));
        };
        if matches!(application.scenario, ScenarioId::Authored { .. }) {
            return Err(refuse(path, "an application names an authored scenario"));
        }
        let Some(scenario) = suite.scenarios.get(&application.scenario) else {
            let outside = coverage.is_some_and(|inventory| {
                inventory.outside.iter().any(|outside| {
                    outside.scenario == application.scenario
                        && outside.reason == OutsideReason::SelectionFilter
                })
            });
            // An in-memory coverage suite is checked against its inventory when it is admitted.
            if outside || (coverage.is_none() && major == COVERAGE) {
                continue;
            }
            return Err(refuse(
                path,
                "an application names a scenario the suite does not hold",
            ));
        };
        let steps = &scenario.steps;
        let Some(step @ ScenarioStep::EstablishEntity { instance, .. }) =
            steps.get(application.establish_step)
        else {
            return Err(refuse(
                path,
                "establish_step is not an establish_entity step",
            ));
        };
        if !establishes(step, record) {
            return Err(refuse(
                path,
                "the established row differs from the selection",
            ));
        }
        if application.command_step <= application.establish_step
            || addressed(steps, application.establish_step, instance)
                != Some(application.command_step)
        {
            return Err(refuse(
                path,
                "command_step is not the asserted command sent to the established row",
            ));
        }
        bound.insert((&application.scenario, application.establish_step));
    }
    Ok(bound)
}

/// Structural JSON admission of `synthesis_seeds` before typed decoding: closed members and
/// unsigned step indices.
pub(crate) fn admit_json(value: &crate::count_json::Json) -> Result<(), AdmissionError> {
    let fields = value.closed(&["sources", "selections", "applications"], &[])?;
    for digest in fields["sources"].object()?.values() {
        digest.text()?;
    }
    for selection in fields["selections"].array()? {
        let record = selection.closed(
            &[
                "source", "instance", "entity", "identity", "fields", "state",
            ],
            &[],
        )?;
        record["identity"].payload()?;
        record["fields"].object()?;
        record["fields"].payload()?;
    }
    for application in fields["applications"].array()? {
        let record = application.closed(
            &[
                "source",
                "instance",
                "scenario",
                "establish_step",
                "command_step",
            ],
            &[],
        )?;
        record["establish_step"].unsigned()?;
        record["command_step"].unsigned()?;
    }
    Ok(())
}
