//! A stored reference the specification keeps present (`ess/30`, beyond10x/ess#509,
//! `docs/design/generic-language-features.md` § G2, "Enforced references"): the command reading
//! the row it names declares no `exists: false` branch, so none of that command's scenarios sends
//! a reference to a missing row, and a target removing the row would pass them all. After each
//! accepting branch that writes rows of the referencing or the referenced entity, the row every
//! arranged reference names is read again and required present.
//!
//! Filed under the referencing entity's `<entity>/invariant/after/<command>/<outcome>` id: that
//! each of its rows names a present row is what must hold of that entity after the branch, an
//! invariant the specification derives rather than declares, so a suite gains no identity a
//! reader of its major refuses. Where the entity also declares invariants and the branch is about
//! one of its rows, [`super::invariants`] has filed a scenario under that id already, and the reads
//! are added to it.
//!
//! The arrangement, in order:
//!
//! 1. the branch's own arrangement;
//! 2. where the branch is about an existing row of the referenced entity, a referencing row
//!    pointed at it through the creating branch that stores the reference from its input; else,
//!    where the arrangement points no referencing row at any row, a fresh referenced row and a
//!    referencing row pointed at it;
//! 3. the branch, invoked and required as its own scenario requires it;
//! 4. a read of an unfiltered view of the referenced entity, requiring each row a referencing row
//!    of the scenario names.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedOutcome,
    ResolvedPayloadValue, ResolvedRelatedTest, ResolvedView,
};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_domain::system::FormatVersion;
use ess_primitives::node::Node;

use super::{
    caller, input_types, read_again, related_guard, run_as, view_reference, Arrangement, Focus,
    Refusal, RefusalCause, Run, ScenarioId, ScenarioPurpose, ScenarioStep, ScenarioValue,
    ViewExpectation, ViewRef, Witness,
};
use crate::scenario::{
    ActorRef, CommandRef, ConformanceScenario, ConformanceSuite, EntityRef, EssSemanticRef,
    OutcomeRef,
};
use crate::witness::{Distinction, WitnessGap};

/// The first further witness this module's rows are arranged at: past the block
/// [`view_reference`] numbers from, so the instance names and witness values are its own.
const BASE: usize = 1_010_000;

/// Why no scenario reads a kept reference's row again after a branch.
const NO_VIEW: &str = "no view of the referenced entity shows every row by its identity, without \
                       a filter, a parameter, paging or a default row";
const NO_CREATOR: &str = "no branch creating the referencing entity stores the reference from its \
                          input unchanged, so no arranged row can be pointed at a chosen one";
const UNARRANGED: &str = "the referenced row, or the referencing row pointed at it, could not be \
                          arranged";
const COUNTED: &str = "the branch's own scenario reads every row of a view or a selection, so a \
                       row pointed at the referenced one cannot be added beside it";
const UNMERGED: &str = "the scenario filed under this id does not begin with the branch's own \
                        arrangement and invocation, so the reads cannot be added to it";

/// A stored reference of a referencing entity that no reading command answers missing: its field
/// and the entity it names.
struct Kept<'ir> {
    field: &'ir str,
    far: &'ir EntityHandle,
}

/// Every integrity scenario the kept references of the model owe, or the refusal of each. Nothing
/// below `ess/30`, where every reading command declares its `exists: false` branch.
pub(super) fn references(
    models: &caller::InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let ir = models.arrangement;
    if ir.format().major() < FormatVersion::V30.major() {
        return;
    }
    let kept = kept(ir);
    if kept.is_empty() {
        return;
    }
    let projections = super::row_projections(ir);
    for (near, references) in &kept {
        let touched: BTreeSet<&EntityHandle> = std::iter::once(*near)
            .chain(references.iter().map(|reference| reference.far))
            .collect();
        for command in models.acting.commands().values() {
            if !focus.takes(&command.name) {
                continue;
            }
            for outcome in &command.outcomes {
                if !accepts(outcome) || !touches(outcome, &touched) {
                    continue;
                }
                let id = ScenarioId::Invariant {
                    entity: EntityRef::from(*near),
                    after: OutcomeRef::new(
                        CommandRef::new(command.name.clone()),
                        outcome.name.clone(),
                    ),
                };
                let built = witness(
                    models,
                    actors,
                    &projections,
                    (near, references),
                    (command, outcome),
                    suite.scenarios.get(&id),
                );
                match built {
                    Ok(scenario) => {
                        suite.scenarios.insert(id, scenario);
                    }
                    Err(cause) => refusals.push(Refusal::about(&id, cause)),
                }
            }
        }
    }
}

/// The kept references, by the entity holding them: a command reads the one row a stored field of
/// its subject names and declares no branch answering it missing, which the domain admits from
/// `ess/30` only where the specification keeps that row present. Each field once.
fn kept(ir: &EssIr) -> BTreeMap<&EntityHandle, Vec<Kept<'_>>> {
    let mut out: BTreeMap<&EntityHandle, Vec<Kept<'_>>> = BTreeMap::new();
    for command in ir.commands().values() {
        if related_guard::several(command) {
            continue;
        }
        let Some((field, far)) = related_guard::stored::field(command) else {
            continue;
        };
        let answered = command.outcomes.iter().any(|outcome| {
            matches!(
                outcome.condition,
                ResolvedCondition::Related {
                    test: ResolvedRelatedTest::Absent,
                    ..
                }
            )
        });
        if answered {
            continue;
        }
        let Some(near) = command
            .outcomes
            .iter()
            .find_map(|outcome| outcome.subject.as_ref())
            .map(|subject| &subject.entity)
        else {
            continue;
        };
        let held = out.entry(near).or_default();
        if !held.iter().any(|reference| reference.field == field) {
            held.push(Kept { field, far });
        }
    }
    out
}

/// Whether `outcome` accepts and writes, the branch a target could remove a row in.
fn accepts(outcome: &ResolvedOutcome) -> bool {
    outcome.error.is_none()
        && outcome.replays.is_none()
        && outcome.test_strategy != TestStrategy::InjectFault
}

/// Whether `outcome` writes a row of an entity in `entities`: its subject, the rows its
/// `instances:` selection names, or an `affects:` entry.
fn touches(outcome: &ResolvedOutcome, entities: &BTreeSet<&EntityHandle>) -> bool {
    outcome
        .subject
        .as_ref()
        .is_some_and(|subject| entities.contains(&subject.entity))
        || outcome
            .instances
            .as_ref()
            .is_some_and(|instances| entities.contains(&instances.entity))
        || outcome
            .affects
            .iter()
            .any(|affect| entities.contains(&affect.entity))
}

/// The scenario that runs `outcome` and then reads each row a referencing row of it names, or the
/// first reason it cannot be built. `existing` is the scenario already filed under the id, which
/// the reads are added to.
fn witness(
    models: &caller::InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    projections: &BTreeMap<&EntityHandle, Vec<&ResolvedView>>,
    (near, references): (&EntityHandle, &[Kept<'_>]),
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    existing: Option<&ConformanceScenario>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = models.arrangement;
    let run = run_as(models, command, outcome, actors, Witness::Full)?;
    let arranged = run.steps();
    let mut between = Vec::new();
    let mut reads = Vec::new();
    let mut source: BTreeSet<EssSemanticRef> = run.source.clone();
    for (index, reference) in references.iter().enumerate() {
        let far_entity = ir.entity(reference.far);
        let view = reader(ir, projections, reference.far)
            .ok_or_else(|| gap(ir, near, reference, NO_VIEW))?;
        let mut rows = named(ir, &arranged, near, reference.field);
        if let Some(target) = pointed_at(&run, outcome, reference, &rows) {
            if caller::reads_every_row(ir, &run.invoke) {
                return Err(gap(ir, near, reference, COUNTED));
            }
            rows.push(point(
                ir,
                actors,
                (near, reference, index),
                target,
                (&mut between, &mut source),
            )?);
        }
        let name = ViewRef::new(view.name.clone());
        for row in rows {
            let fields = [(far_entity.identity.name.clone(), row)]
                .into_iter()
                .collect();
            read_again(
                view,
                &name,
                BTreeMap::new(),
                ViewExpectation::Contains { fields },
                &mut reads,
            );
        }
        source.insert(name.into());
        source.insert(EntityRef::from(reference.far).into());
    }
    models.mark(caller::InvocationPhase::Arrange, &mut between);
    let mut steps = run.setup.clone();
    steps.extend(between);
    steps.extend(run.invoke.iter().cloned());
    let command_ref = CommandRef::new(command.name.clone());
    source.insert(command_ref.clone().into());
    source.insert(OutcomeRef::new(command_ref, outcome.name.clone()).into());
    source.insert(EntityRef::from(near).into());
    if let Some(actor) = run.actor.clone() {
        source.insert(actor.into());
    }
    source.extend(
        input_types(ir, command)
            .into_iter()
            .map(EssSemanticRef::from),
    );
    let purpose = merged(
        ir,
        (near, references),
        (command, outcome),
        existing,
        &arranged,
        (&mut steps, &mut source),
    )?;
    steps.extend(reads);
    Ok(ConformanceScenario::new(purpose, steps, source))
}

/// Why no read of `near`'s `reference` is added after a branch.
fn gap(
    ir: &EssIr,
    near: &EntityHandle,
    reference: &Kept<'_>,
    reason: &'static str,
) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: format!("{}.{}", ir.entity(near).name, reference.field),
        type_ref: ir.entity(reference.far).name.to_string(),
        reason,
    })
}

/// The purpose of the scenario: that of `existing`, the scenario already filed under the id, whose
/// requirements after the branch `steps` and `source` gain, or else one naming the references.
fn merged(
    ir: &EssIr,
    (near, references): (&EntityHandle, &[Kept<'_>]),
    (command, outcome): (&ResolvedCommand, &ResolvedOutcome),
    existing: Option<&ConformanceScenario>,
    arranged: &[ScenarioStep],
    (steps, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<ScenarioPurpose, RefusalCause> {
    let Some(existing) = existing else {
        let fields: Vec<String> = references
            .iter()
            .map(|reference| format!("`{}`", reference.field))
            .collect();
        return Ok(super::clipped(&format!(
            "every `{}` row's {} still names a present row after `{}` on `{}`",
            ir.entity(near).name,
            fields.join(" and "),
            command.name,
            outcome.name
        )));
    };
    let declared = existing.steps.strip_prefix(arranged).ok_or_else(|| {
        RefusalCause::NoWitness(WitnessGap {
            path: ir.entity(near).name.to_string(),
            type_ref: ir.entity(near).name.to_string(),
            reason: UNMERGED,
        })
    })?;
    steps.extend(declared.iter().cloned());
    source.extend(existing.source.iter().cloned());
    Ok(existing.purpose.clone())
}

/// The row of the referenced entity a new referencing row is pointed at before the branch runs.
enum Pointed {
    /// The branch's own existing row of it, which no arranged reference names yet.
    Own(Arrangement),
    /// A fresh one, where the arrangement points no referencing row at any row.
    Fresh,
}

/// Points a new row of `near` at `target` through the branch creating it that stores `reference`
/// from its input, arranging a fresh referenced row first for [`Pointed::Fresh`]. The steps go to
/// `between`, their sources to `source`; the row pointed at is returned, to be read.
fn point(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (near, reference, index): (&EntityHandle, &Kept<'_>, usize),
    target: Pointed,
    (between, source): (&mut Vec<ScenarioStep>, &mut BTreeSet<EssSemanticRef>),
) -> Result<ScenarioValue, RefusalCause> {
    let at = |offset: usize| Distinction::further(BASE + 2 * index + offset);
    let drivers = ir.drivers();
    let creator = drivers
        .get(near)
        .into_iter()
        .flatten()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .filter(|driver| driver.outcome.test_strategy != TestStrategy::InjectFault)
        .find_map(|driver| {
            stored_from(driver.outcome, reference.field).map(|input| (driver, input))
        })
        .ok_or_else(|| gap(ir, near, reference, NO_CREATOR))?;
    let far = match target {
        Pointed::Own(far) => far,
        Pointed::Fresh => {
            let fresh = super::arrange_first(
                ir,
                reference.far,
                std::slice::from_ref(&ir.entity(reference.far).lifecycle.initial),
                actors,
                at(0),
                &[],
            )
            .map_err(|_| gap(ir, near, reference, UNARRANGED))?;
            between.extend(fresh.steps.iter().cloned());
            source.extend(fresh.source.iter().cloned());
            fresh
        }
    };
    let pointing = view_reference::point_at(ir, actors, creator, Some(&far), at(1))
        .map_err(|_| gap(ir, near, reference, UNARRANGED))?;
    between.extend(pointing.steps.iter().cloned());
    source.extend(pointing.source.iter().cloned());
    Ok(ScenarioValue::instance(far.instance.clone()))
}

/// The row a new referencing row is pointed at before the branch runs, or `None` where the rows
/// the arrangement names are the ones to read.
fn pointed_at(
    run: &Run,
    outcome: &ResolvedOutcome,
    reference: &Kept<'_>,
    rows: &[ScenarioValue],
) -> Option<Pointed> {
    let own = outcome
        .subject
        .as_ref()
        .filter(|subject| {
            &subject.entity == reference.far && !matches!(subject.effect, ResolvedEffect::Creates)
        })
        .and(run.instance.as_ref())
        .zip(run.before.as_ref());
    if let Some((instance, state)) = own {
        if !rows.contains(&ScenarioValue::instance(instance.clone())) {
            // The row as the arrangement leaves it, which is what the creating branch reads.
            return Some(Pointed::Own(Arrangement {
                instance: instance.clone(),
                state: state.clone(),
                steps: Vec::new(),
                source: BTreeSet::new(),
                settled: run.before_settled.clone(),
                unwritten: BTreeSet::new(),
                unobserved: BTreeSet::new(),
            }));
        }
    }
    rows.is_empty().then_some(Pointed::Fresh)
}

/// The view a row of `far` is read on by its identity alone: one row per instance, no filter, no
/// parameter, no paging and no default row for an identity no row carries, the first by name.
fn reader<'ir>(
    ir: &EssIr,
    projections: &BTreeMap<&EntityHandle, Vec<&'ir ResolvedView>>,
    far: &EntityHandle,
) -> Option<&'ir ResolvedView> {
    let identity = &ir.entity(far).identity;
    projections.get(far)?.iter().copied().find(|view| {
        view.filter.is_none()
            && view.params.is_empty()
            && view.paging.is_none()
            && view.absent.is_none()
            && view
                .field(&identity.name)
                .is_some_and(|field| field.type_ref == identity.type_ref)
    })
}

/// The rows the referencing rows `steps` write through `field` name: the value of the input each
/// accepted write stores there unchanged, and the field of each row established directly, in step
/// order, each once.
fn named(
    ir: &EssIr,
    steps: &[ScenarioStep],
    near: &EntityHandle,
    field: &str,
) -> Vec<ScenarioValue> {
    let near_ref = EntityRef::from(near);
    let mut out: Vec<ScenarioValue> = Vec::new();
    let mut sent: Option<(&CommandRef, &BTreeMap<String, ScenarioValue>)> = None;
    for step in steps {
        let value = match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                sent = Some((command, input));
                None
            }
            ScenarioStep::ExecuteCommandWithoutInput { .. } => {
                sent = None;
                None
            }
            ScenarioStep::ExpectOutcome { outcome } => sent
                .take()
                .filter(|(command, _)| **command == outcome.command)
                .and_then(|(command, input)| {
                    let branch = ir
                        .commands()
                        .get(command.name())?
                        .outcomes
                        .iter()
                        .find(|branch| branch.name == outcome.outcome)?;
                    if branch
                        .subject
                        .as_ref()
                        .is_none_or(|subject| &subject.entity != near)
                    {
                        return None;
                    }
                    input.get(stored_from(branch, field)?).cloned()
                }),
            ScenarioStep::EstablishEntity { entity, fields, .. } if *entity == near_ref => fields
                .get(field)
                .filter(|value| !matches!(value, Node::Null))
                .cloned()
                .map(ScenarioValue::literal),
            _ => None,
        };
        if let Some(value) = value {
            if !matches!(&value, ScenarioValue::Literal { value: Node::Null })
                && !out.contains(&value)
            {
                out.push(value);
            }
        }
    }
    out
}

/// The input `outcome` stores in the subject field `field` unchanged, where it does.
fn stored_from<'ir>(outcome: &'ir ResolvedOutcome, field: &str) -> Option<&'ir str> {
    outcome.sets.iter().find_map(|set| match &set.value {
        ResolvedPayloadValue::InputField { field: input, .. }
            if set.target == field
                && set.conversion.is_none()
                && !ess_domain::command::input_path::is_path(input) =>
        {
            Some(input.as_str())
        }
        _ => None,
    })
}
