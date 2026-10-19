//! A view that reads through a reference of its source (`ess/28`, beyond10x/ess#564,
//! `docs/design/generic-language-features.md` § G2, navigation): one `<view>/reference/<field>`
//! scenario per view and reference, which reads the view before and after the row the reference
//! names changes.
//!
//! The arrangement, in order:
//!
//! 1. two rows of the referenced entity, `F1` and `F2`, where its lifecycle starts;
//! 2. a source row pointed at each, `N1` at `F1` and `N2` at `F2`, through the creating branch
//!    that stores the reference from its input — and, where the reference is `Optional` and that
//!    input may be left out, a third, `N3`, pointed at nothing;
//! 3. `F2` changed by the first branch of its entity that changes what the view reads of it (a
//!    move where `state` is read, else an update writing a field read), so the two named rows
//!    differ;
//! 4. a read of the view, each source row required as the filter and the referenced row decide:
//!    shown with what it reads of its referenced row, or not shown; `N3` reading nothing;
//! 5. `F1` changed the same way, and the view read again: `N1` shows what `F1` holds now.
//!
//! A target that copies the referenced value when the source row is written answers the old value
//! at step 5; one that reads a decoy or the latest referenced row answers `F2`'s at step 4. Where a
//! step cannot be built — no creating branch stores the reference from its input, the view does
//! not project the source's identity, the filter cannot be decided — the scenario is refused by
//! name rather than asserted partly.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedEffect, ResolvedPayloadValue, ResolvedView, ResolvedViewReference,
};
use ess_domain::command::TestStrategy;
use ess_domain::entity::{EntitySpec, StateName};
use ess_domain::name::QualifiedName;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;

use super::{
    created_by, insert, invoke, invoke_created_with, reach, related_guard, subject_fact,
    Arrangement, Determined, Driver, Refusal, RefusalCause, ScenarioId, ScenarioStep,
    ScenarioValue, ViewExpectation, ViewRef,
};
use crate::scenario::{ActorRef, ConformanceScenario, ConformanceSuite, EntityRef};
use crate::witness::{Distinction, WitnessGap};

/// The first further witness this module's rows are arranged at: past every block an arrangement
/// or a related read numbers, so the instance names and witness values are its own.
const BASE: usize = 1_000_000;

/// Every `<view>/reference/<field>` scenario the model's views owe, or the refusal of each.
pub(super) fn references(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for view in ir.views().values() {
        for reference in &view.references {
            let id = ScenarioId::ViewReference {
                view: ViewRef::new(view.name.clone()),
                via: reference.via.clone(),
            };
            match witness(ir, actors, view, reference) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(reason) => refusals.push(Refusal::about(
                    &id,
                    RefusalCause::NoWitness(WitnessGap {
                        path: format!("{}.{}", view.name, reference.via),
                        type_ref: ir.entity(&reference.entity).name.to_string(),
                        reason,
                    }),
                )),
            }
        }
    }
}

/// Why no scenario reads a view through a reference.
const NO_CREATOR: &str = "no branch creating the view's source stores the reference from its \
                          input unchanged, so no arranged row can be pointed at a chosen one";
const NO_IDENTITY: &str = "the view does not project its source's identity, so a read cannot say \
                           which source row it answers";
const UNARRANGED: &str = "the rows the reference names, or the source rows pointed at them, could \
                          not be arranged";
const UNDECIDED: &str = "the view's filter reads a value of the source row or the referenced row \
                         that the arrangement does not determine";
const UNBOUND: &str = "the view's parameters are not bound by the rows the arrangement made";

/// One source row the scenario made, and the referenced row it points at, as the steps so far
/// leave it.
struct Pointed<'a> {
    row: Arrangement,
    far: Option<&'a Arrangement>,
}

/// The scenario reading `view` through `reference`.
#[allow(
    clippy::too_many_lines,
    reason = "one scenario: the rows arranged, the first read, the change and the second read"
)]
fn witness(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    view: &ResolvedView,
    reference: &ResolvedViewReference,
) -> Result<ConformanceScenario, &'static str> {
    let source_handle = &view.source;
    let source = ir.entity(source_handle);
    if view
        .field(&source.identity.name)
        .is_none_or(|field| field.type_ref != source.identity.type_ref)
    {
        return Err(NO_IDENTITY);
    }
    let drivers = ir.drivers();
    let (creator, input_field) = drivers
        .get(source_handle)
        .into_iter()
        .flatten()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .filter(|driver| driver.outcome.test_strategy != TestStrategy::InjectFault)
        .find_map(|driver| stored_from(driver, &reference.via).map(|input| (driver, input)))
        .ok_or(NO_CREATOR)?;
    let far_handle = &reference.entity;
    let far_entity = ir.entity(far_handle);
    let reads = read_members(ir, view, reference);
    let at = |offset: usize| Distinction::further(BASE + offset);
    let arrange_far = |offset: usize| {
        super::arrange_first(
            ir,
            far_handle,
            std::slice::from_ref(&far_entity.lifecycle.initial),
            actors,
            at(offset),
            &[],
        )
        .map_err(|_| UNARRANGED)
    };
    let first = arrange_far(0)?;
    let second = arrange_far(1)?;
    let near = |far: &Arrangement, offset: usize| {
        point_at(ir, actors, (creator, input_field), Some(far), at(offset))
    };
    let named = near(&first, 2)?;
    let decoy = near(&second, 3)?;
    let unpointed = (reference.via_type.is_optional()
        && creator
            .command
            .input
            .iter()
            .any(|field| field.name == input_field && field.type_ref.is_optional())
        && !related_guard::routes(creator.command, creator.outcome))
    .then(|| point_at(ir, actors, (creator, input_field), None, at(4)).ok())
    .flatten();

    let change = changer(ir, far_handle, &reads, &first.state);
    let mut steps = Vec::new();
    let mut sources: BTreeSet<_> = BTreeSet::new();
    for arranged in [&first, &second, &named, &decoy]
        .into_iter()
        .chain(unpointed.as_ref())
    {
        steps.extend(arranged.steps.iter().cloned());
        sources.extend(arranged.source.iter().cloned());
    }
    let changed_second = match &change {
        Some(driver) => Some(changed(ir, actors, driver, &second, at(5))?),
        None => None,
    };
    if let Some((_, invocation_steps, invocation_source)) = &changed_second {
        steps.extend(invocation_steps.iter().cloned());
        sources.extend(invocation_source.iter().cloned());
    }
    let second_now = changed_second
        .as_ref()
        .map_or(&second, |(changed, _, _)| changed);
    let name = ViewRef::new(view.name.clone());
    let possible = possible_values(ir, far_handle, reference, &[&first, &second, second_now]);
    let rows = [
        Pointed {
            row: named.clone(),
            far: Some(&first),
        },
        Pointed {
            row: decoy,
            far: Some(second_now),
        },
    ]
    .into_iter()
    .chain(unpointed.map(|row| Pointed { row, far: None }));
    for pointed in rows {
        expect(ir, view, reference, &name, &pointed, &possible, &mut steps)?;
    }
    if let Some(driver) = &change {
        let (first_now, invocation_steps, invocation_source) =
            changed(ir, actors, driver, &first, at(6))?;
        steps.extend(invocation_steps);
        sources.extend(invocation_source);
        let pointed = Pointed {
            row: named,
            far: Some(&first_now),
        };
        expect(ir, view, reference, &name, &pointed, &possible, &mut steps)?;
    }
    sources.insert(name.clone().into());
    sources.insert(EntityRef::from(source_handle).into());
    sources.insert(EntityRef::from(far_handle).into());
    let purpose = format!(
        "`{}` reads the `{}` row `{}` names as it is when read: a row whose referenced row \
         changes, a decoy, and an absent reference",
        view.name, far_entity.name, reference.via
    );
    Ok(ConformanceScenario::new(
        super::clipped(&purpose),
        steps,
        sources,
    ))
}

/// The input `driver` stores in the source field `via` unchanged, where it does.
fn stored_from<'a>(driver: &Driver<'a>, via: &str) -> Option<&'a str> {
    driver.outcome.sets.iter().find_map(|set| match &set.value {
        ResolvedPayloadValue::InputField { field, .. }
            if set.target == via
                && set.conversion.is_none()
                && !ess_domain::command::input_path::is_path(field) =>
        {
            Some(field.as_str())
        }
        _ => None,
    })
}

/// What `view` reads of the row `reference` names: the members its fields read and its filter's
/// paths through it read.
fn read_members(
    ir: &EssIr,
    view: &ResolvedView,
    reference: &ResolvedViewReference,
) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = reference
        .fields
        .iter()
        .map(|read| read.read.clone())
        .collect();
    for path in view.filter_reads() {
        if let Some((navigated, member)) = view.navigated(ir, path) {
            if navigated.via == reference.via {
                out.insert(member);
            }
        }
    }
    out
}

/// A new source row created by `creator`, its input `input` pointed at `far` — or, with `far`
/// `None`, left out.
fn point_at(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (creator, input): (&Driver<'_>, &str),
    far: Option<&Arrangement>,
    distinction: Distinction,
) -> Result<Arrangement, &'static str> {
    let entity = &creator.outcome.subject.as_ref().ok_or(UNARRANGED)?.entity;
    created_by(ir, entity, creator, distinction, None, |bound, _| {
        let mut bound = bound.clone();
        match far {
            Some(far) => {
                bound.insert(input.to_owned(), far.instance.clone());
            }
            None => {
                bound.remove(input);
            }
        }
        if related_guard::routes(creator.command, creator.outcome) {
            let far = far.ok_or(UNARRANGED)?;
            if related_guard::reads(creator.command).map(|(read, _)| read) != Some(input) {
                return Err(UNARRANGED);
            }
            related_guard::drive_held(ir, creator, actors, distinction, &bound, far)
                .map_err(|_| UNARRANGED)
        } else {
            let mut sent =
                reach(ir, creator.command, creator.outcome, distinction).map_err(|_| UNARRANGED)?;
            if far.is_none() {
                sent.remove(input);
            }
            invoke_created_with(ir, creator, actors, distinction, &bound, &sent, &[])
                .map_err(|_| UNARRANGED)
        }
    })
}

/// The first branch of the referenced entity that changes what the view reads of a row resting in
/// `state`: a move out of it where `state` is read, else an update or move writing a field read
/// from its input. `None` where no branch does, or where its input is not this module's to choose.
fn changer<'ir>(
    ir: &'ir EssIr,
    entity: &EntityHandle,
    reads: &BTreeSet<String>,
    state: &StateName,
) -> Option<Driver<'ir>> {
    let drivers = ir.drivers();
    let candidates: Vec<Driver<'ir>> = drivers
        .get(entity)
        .into_iter()
        .flatten()
        .filter(|driver| {
            driver.outcome.test_strategy != TestStrategy::InjectFault
                && !subject_fact::uses(driver.command)
                && !related_guard::routes(driver.command, driver.outcome)
        })
        .copied()
        .collect();
    let moves_out = |driver: &&Driver<'_>| {
        driver
            .effect
            .transition()
            .is_some_and(|transition| transition.from.contains(state) && transition.to != *state)
    };
    if reads.contains(EntitySpec::STATE) {
        if let Some(driver) = candidates.iter().find(moves_out) {
            return Some(*driver);
        }
    }
    candidates
        .iter()
        .find(|driver| {
            (matches!(driver.effect, ResolvedEffect::Updates) || moves_out(driver))
                && driver.outcome.sets.iter().any(|set| {
                    reads.contains(&set.target)
                        && matches!(set.value, ResolvedPayloadValue::InputField { .. })
                })
        })
        .copied()
}

/// `far` after `driver` ran on it, with the steps that ran it and what they depend on.
fn changed(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    driver: &Driver<'_>,
    far: &Arrangement,
    distinction: Distinction,
) -> Result<
    (
        Arrangement,
        Vec<ScenarioStep>,
        BTreeSet<crate::scenario::EssSemanticRef>,
    ),
    &'static str,
> {
    let invoked = invoke(
        ir,
        driver,
        Some(&far.instance),
        Some(&far.state),
        actors,
        distinction,
        &BTreeMap::new(),
        &[],
    )
    .map_err(|_| UNARRANGED)?;
    let mut next = far.clone();
    next.absorb(driver.outcome, invoked.settled);
    if let Some(transition) = driver.effect.transition() {
        next.state = transition.to.clone();
    }
    Ok((next, invoked.steps, invoked.source))
}

/// Every value a field read through `reference` may hold on a row the scenario arranged, by view
/// field: each state of the referenced entity for a `state` read, else what the arranged rows
/// hold. A row pointed at nothing holds none of them.
fn possible_values(
    ir: &EssIr,
    entity: &EntityHandle,
    reference: &ResolvedViewReference,
    rows: &[&Arrangement],
) -> BTreeMap<String, BTreeSet<Node>> {
    reference
        .fields
        .iter()
        .map(|read| {
            let values: BTreeSet<Node> = if read.read == EntitySpec::STATE {
                ir.entity(entity)
                    .lifecycle
                    .states
                    .iter()
                    .map(|state| Node::Text(state.to_string()))
                    .collect()
            } else {
                rows.iter()
                    .filter_map(|row| match held(ir, entity, row, &read.read) {
                        Some(ScenarioValue::Literal { value }) if value != Node::Null => {
                            Some(value)
                        }
                        _ => None,
                    })
                    .collect()
            };
            (read.field.clone(), values)
        })
        .collect()
}

/// What `row`, a row of `entity`, holds in `member`: its state, its identity, or a field its
/// arrangement determined.
fn held(
    ir: &EssIr,
    entity: &EntityHandle,
    row: &Arrangement,
    member: &str,
) -> Option<ScenarioValue> {
    if member == EntitySpec::STATE {
        return Some(ScenarioValue::literal(Node::Text(row.state.to_string())));
    }
    if ir.entity(entity).identity.name == member {
        return Some(row.identity());
    }
    row.settled
        .get(member)
        .map(|held: &Determined| held.value.clone())
}

/// The read of `view` requiring `pointed`'s row: shown with what it reads of its referenced row
/// where the filter holds, not shown where it does not.
fn expect(
    ir: &EssIr,
    view: &ResolvedView,
    reference: &ResolvedViewReference,
    name: &ViewRef,
    pointed: &Pointed<'_>,
    possible: &BTreeMap<String, BTreeSet<Node>>,
    steps: &mut Vec<ScenarioStep>,
) -> Result<(), &'static str> {
    let source = ir.entity(&view.source);
    let identity = pointed.row.identity();
    let named: BTreeMap<String, ScenarioValue> = [(source.identity.name.clone(), identity.clone())]
        .into_iter()
        .collect();
    let params = super::bound(ir, view, &pointed.row.settled, Some(&identity));
    if view
        .params
        .iter()
        .any(|param| !param.type_ref.is_optional() && !params.contains_key(&param.name))
    {
        return Err(UNBOUND);
    }
    let shown = match &view.filter {
        None => true,
        Some(filter) => shows(ir, view, reference, filter, pointed, &identity, &params)?,
    };
    if !shown {
        super::read_again(
            view,
            name,
            params,
            ViewExpectation::Excludes { fields: named },
            steps,
        );
        return Ok(());
    }
    let mut row = named.clone();
    let mut absent = Vec::new();
    for read in &reference.fields {
        match pointed.far {
            Some(far) => {
                if let Some(value) = held(ir, &reference.entity, far, &read.read) {
                    row.insert(read.field.clone(), value);
                }
            }
            None => absent.push(&read.field),
        }
    }
    super::read_again(
        view,
        name,
        params.clone(),
        ViewExpectation::Contains { fields: row },
        steps,
    );
    // Absent: no value any arranged referenced row holds, whichever spelling — left out or
    // `null` — the target publishes it in.
    for field in absent {
        for value in possible.get(field).into_iter().flatten() {
            let mut fields = named.clone();
            fields.insert(field.clone(), ScenarioValue::literal(value.clone()));
            super::read_again(
                view,
                name,
                params.clone(),
                ViewExpectation::Excludes { fields },
                steps,
            );
        }
    }
    Ok(())
}

/// Whether `view`'s filter shows `pointed`'s row: read over the row's state, what its arrangement
/// determined, and what its referenced row holds; a path through an absent reference reads nothing,
/// and a filter left unknown by that alone shows no row.
fn shows(
    ir: &EssIr,
    view: &ResolvedView,
    reference: &ResolvedViewReference,
    filter: &ess_primitives::predicate::Predicate,
    pointed: &Pointed<'_>,
    identity: &ScenarioValue,
    params: &BTreeMap<String, ScenarioValue>,
) -> Result<bool, &'static str> {
    let mut navigated: Vec<(FactPath, FactValue)> = Vec::new();
    for path in view.filter_reads() {
        let Some((through, member)) = view.navigated(ir, path) else {
            continue;
        };
        if through.via != reference.via || path.segments().len() != 2 {
            continue;
        }
        let Some(far) = pointed.far else {
            continue;
        };
        if let Some(ScenarioValue::Literal { value }) = held(ir, &reference.entity, far, &member) {
            if let Some(fact) = super::fact_value(&value) {
                navigated.push((path.clone(), fact));
            }
        }
    }
    match super::row_truth_with(
        ir,
        view,
        filter,
        (&pointed.row.state, &pointed.row.settled),
        Some(identity),
        params,
        &navigated,
    ) {
        Ok(shown) => Ok(shown),
        Err(unbound)
            if pointed.far.is_none()
                && !unbound.is_empty()
                && unbound.iter().all(|path| {
                    view.navigated(ir, path)
                        .is_some_and(|(through, _)| through.via == reference.via)
                }) =>
        {
            Ok(false)
        }
        Err(_) => Err(UNDECIDED),
    }
}
