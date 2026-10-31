//! Rows the deployment provides (`provided: true`, ess/33,
//! architecture-decision-record:a-deployment-may-provide-an-entitys-rows).
//!
//! Ordinary arrangement keeps precedence: a provided entity some outcome creates is arranged by its
//! creators as any other. Where none does, synthesis takes a row of it in one position only, the
//! related row a `when_related: {via: input.<field>}` guard reads ([`serves`], [`search`]): the
//! scenario resolves an entity fixture `{name, type, entity, state}` before execution and sends
//! its identity, and [`crate::fixtures::install`] lifts the resolution into the scenario's one
//! prelude. The `exists: false` branch is sent a fresh identity, as for any related row.
//!
//! Every other position is refused by name ([`Unreachable::Provided`]): a subject, an owner, a
//! view's population. So is a guard over the row's fields ([`Unreachable::ProvidedFields`]), whose
//! values the deployment holds. No decoy row is placed beside a provided one, since the deployment
//! would have to provide that too; the scenario says so in a [`Note::ProvidedRelatedRow`].

use super::{
    Arrangement, BTreeMap, BTreeSet, Distinction, EntityHandle, EntityRef, EssIr, InstanceName,
    InstanceNeed, Note, RefusalCause, ResolvedEffect, ScenarioStep, Unreachable,
};
use crate::fixtures::{Contract, FixtureField};
use crate::scenario::ConformanceSuite;
use ess_domain::command::fixture_inputs::FixtureName;

/// Why no row of `entity` can be arranged where no outcome creates one: the deployment provides
/// its rows, or nothing does.
pub(super) fn nothing_creates(ir: &EssIr, entity: &EntityHandle) -> Unreachable {
    if ir.entity(entity).provided {
        Unreachable::Provided
    } else {
        Unreachable::NothingCreates
    }
}

/// Whether a related row of `entity` is resolved as an entity fixture: the deployment provides its
/// rows, no outcome creates one, and it is filed under no owner.
pub(super) fn serves(ir: &EssIr, entity: &EntityHandle) -> bool {
    let created = ir.drivers().get(entity).is_some_and(|drivers| {
        drivers
            .iter()
            .any(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    });
    ir.entity(entity).provided && !created && ir.owner_of(entity).is_none()
}

/// The provided rows of `entity` at `distinction`, one per lifecycle state, the initial one first
/// and then the rest in name order, each offered to `goal`: the first it takes, with what it
/// found. Refused as [`Unreachable::ProvidedFields`] where it takes none, as the state is all a
/// fixture names of the row.
pub(super) fn search<T>(
    ir: &EssIr,
    entity: &EntityHandle,
    distinction: Distinction,
    mut goal: impl FnMut(&Arrangement) -> Result<Option<T>, RefusalCause>,
) -> Result<(Arrangement, T), RefusalCause> {
    let declared = ir.entity(entity);
    let lifecycle = &declared.lifecycle;
    let instance = super::instance_name(&declared.name, distinction);
    let refused = |reason| RefusalCause::InstanceRequired {
        entity: EntityRef::from(entity),
        need: InstanceNeed::Updates,
        reason,
    };
    let name = FixtureName::new(instance.as_str()).map_err(|_| refused(Unreachable::Provided))?;
    let states = std::iter::once(&lifecycle.initial).chain(
        lifecycle
            .states
            .iter()
            .filter(|state| **state != lifecycle.initial),
    );
    for state in states {
        let field = FixtureField::provided(ir, name.clone(), &declared.name, state.clone());
        let Ok(fixtures) = Contract::of_fields(ir, vec![field]) else {
            continue;
        };
        let row = Arrangement {
            instance: instance.clone(),
            state: state.clone(),
            steps: vec![ScenarioStep::ResolveFixtures { fixtures }],
            source: BTreeSet::from([EntityRef::from(entity).into()]),
            settled: BTreeMap::new(),
            unwritten: BTreeSet::new(),
            unobserved: BTreeSet::new(),
        };
        if let Some(found) = goal(&row)? {
            return Ok((row, found));
        }
    }
    Err(refused(Unreachable::ProvidedFields))
}

/// The instances a fixture prelude binds: one per row the deployment provides, under its
/// fixture's name.
pub(super) fn bound(fixtures: &Contract) -> Vec<InstanceName> {
    fixtures
        .fields
        .iter()
        .filter(|field| field.entity.is_some())
        .filter_map(|field| InstanceName::new(field.name.as_str()).ok())
        .collect()
}

/// One [`Note::ProvidedRelatedRow`] for each provided row each scenario of `suite` resolves.
pub(super) fn notes(suite: &ConformanceSuite) -> Vec<Note> {
    let mut notes = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            let ScenarioStep::ResolveFixtures { fixtures } = step else {
                continue;
            };
            for entity in fixtures
                .fields
                .iter()
                .filter_map(|field| field.entity.as_ref())
            {
                notes.push(Note::ProvidedRelatedRow {
                    scenario: id.clone(),
                    entity: entity.clone(),
                });
            }
        }
    }
    notes
}
