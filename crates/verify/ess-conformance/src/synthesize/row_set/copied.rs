//! Row-set rows whose selected field is copied from another row by `{related: {via, field}}`.
//!
//! A creating branch writing `qualifies: {related: {via: input.tier_id, field: qualifies}}` stores
//! what the tier row it is sent holds, not anything in its own input. So a selector reading
//! `qualifies` is decided by the tier row, and both halves of the row-set arrangement follow it:
//!
//! * [`values`] reads, for one send in a scenario's steps, the row each such `via` names among the
//!   rows the steps left, and carries its value under [`related::key`], where
//!   [`super::super::expression_value`] finds it; the walk over the steps then knows what the new
//!   row holds instead of leaving it unknown;
//! * [`steered`] arranges a row toward a goal over such a field by arranging the related row toward
//!   the same goal over the field it copies, and sending the creating command that row.
//!
//! A creator copying nothing a goal reads is not this module's: [`steered`] returns `None` for it
//! and the row is arranged as it always was.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedEffect, ResolvedOutcome, ResolvedPayloadField,
    ResolvedPayloadValue, ResolvedRelatedVia, ResolvedTypeRef,
};
use ess_domain::name::QualifiedName;
use ess_primitives::facts::FactPath;
use ess_primitives::predicate::{Predicate, Truth};

use super::super::{
    advance, arrange_toward_owned, candidates, created_bound, creator_bindings, map_paths, related,
    route_from, selects_branch, subject_fact, under_owner, ActorRef, Arrangement, Determined,
    Distinction, InstanceName, ScenarioValue,
};
use super::{Row, Rows};

/// Every `{related: …}` source among `fields`, at any depth of a struct.
fn sources<'a>(fields: &'a [ResolvedPayloadField], out: &mut Vec<&'a ResolvedPayloadField>) {
    for field in fields {
        match &field.value {
            ResolvedPayloadValue::RelatedField { .. } if field.conversion.is_none() => {
                out.push(field);
            }
            ResolvedPayloadValue::Struct { fields } => sources(fields, out),
            _ => {}
        }
    }
}

/// The row of `entity` that `named` names among `rows`: an arranged instance, or a literal identity.
fn named<'r>(rows: &'r Rows, entity: &EntityHandle, named: &ScenarioValue) -> Option<&'r Row> {
    rows.held.iter().find(|row| {
        row.entity == *entity
            && match named {
                ScenarioValue::Instance { instance } => row.instance.as_ref() == Some(instance),
                other => row.identity.as_ref() == Some(other),
            }
    })
}

/// What `row` holds for `field`: its identity, the value the steps settled, or — for an `Optional`
/// field no step wrote — nothing, carried as a null.
fn held(ir: &EssIr, row: &Row, field: &str) -> Option<ScenarioValue> {
    let identity = &ir.entity(&row.entity).identity.name;
    if field == identity {
        return row
            .identity
            .clone()
            .or_else(|| row.instance.clone().map(ScenarioValue::instance));
    }
    if let Some(held) = row.settled.get(field) {
        return Some(held.value.clone());
    }
    row.unwritten
        .contains(field)
        .then(|| ScenarioValue::literal(ess_primitives::node::Node::Null))
}

/// The values every `{related: …}` source of `outcome` reads on the rows the steps left, for the
/// send that supplied `supplied` to a row whose fields were `before`: each `via` and each further
/// reference it follows read on the row it names, under [`related::key`]. A reference naming no row
/// the steps left, or a row of an entity whose rows the steps changed in a way not followed, carries
/// nothing, and the value it fills stays undetermined as before.
pub(super) fn values(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    before: &BTreeMap<String, Determined>,
    rows: &Rows,
) -> BTreeMap<String, Determined> {
    let mut found = Vec::new();
    sources(&outcome.sets, &mut found);
    let mut out = BTreeMap::new();
    for source in found {
        let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity,
            field,
            ..
        } = &source.value
        else {
            continue;
        };
        let start = match via {
            ResolvedRelatedVia::Input { field, .. } => supplied.get(field).cloned(),
            ResolvedRelatedVia::Subject { field, .. } => {
                before.get(field).map(|held| held.value.clone())
            }
        };
        let Some(value) =
            start.and_then(|start| followed(ir, rows, start, through, (entity, field)))
        else {
            continue;
        };
        out.insert(
            related::key(via, through, field),
            Determined {
                value,
                type_ref: source.target_type.clone(),
            },
        );
    }
    out
}

/// What `field` holds on the row of `entity` reached from `reference` through every further
/// reference `through` follows, among the rows the steps left; `None` where a reference names no
/// such row, or rows of an entity it reads were changed in a way the walk does not follow.
fn followed(
    ir: &EssIr,
    rows: &Rows,
    mut reference: ScenarioValue,
    through: &[ess_compiler::ir::ResolvedRelatedHop],
    (entity, field): (&EntityHandle, &str),
) -> Option<ScenarioValue> {
    for hop in through {
        if rows.opaque.contains(&hop.entity) {
            return None;
        }
        reference =
            named(rows, &hop.entity, &reference).and_then(|row| held(ir, row, &hop.field))?;
    }
    if rows.opaque.contains(entity) {
        return None;
    }
    named(rows, entity, &reference).and_then(|row| held(ir, row, field))
}

/// One field a creating branch copies from the row an input names: the field written, the input,
/// the entity of the row it names, the field read there and the type written.
struct Copied<'a> {
    target: &'a str,
    input: &'a str,
    entity: &'a EntityHandle,
    field: &'a str,
    type_ref: &'a ResolvedTypeRef,
}

/// Every top-level `sets:` entry of `creator` copying a field of the row a top-level input names,
/// in one hop and with no conversion.
fn copies(creator: &ResolvedOutcome) -> Vec<Copied<'_>> {
    creator
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::RelatedField {
                via: ResolvedRelatedVia::Input { field: input, .. },
                through,
                entity,
                field,
                ..
            } if through.is_empty() && !ess_domain::command::input_path::is_path(input) => {
                Some(Copied {
                    target: &set.target,
                    input,
                    entity,
                    field,
                    type_ref: &set.target_type,
                })
            }
            _ => None,
        })
        .collect()
}

/// The conjuncts of `predicate`, through nested `all:`.
fn flattened<'p>(predicate: &'p Predicate, out: &mut Vec<&'p Predicate>) {
    match predicate {
        Predicate::All(children) => {
            for child in children {
                flattened(child, out);
            }
        }
        other => out.push(other),
    }
}

/// The first field name each fact path of `predicate` reads on the row.
fn heads(predicate: &Predicate) -> BTreeSet<&str> {
    predicate
        .fact_paths()
        .into_iter()
        .filter_map(|path| path.segments().first().map(String::as_str))
        .collect()
}

/// The conjuncts of a goal over fields `copies` fill, by the input naming the row they are read on,
/// each read as the field it copies there; and every other conjunct.
type Split<'c> = (
    BTreeMap<&'c str, (&'c EntityHandle, Vec<Predicate>)>,
    Vec<Predicate>,
);

/// `conjuncts` split by [`Split`]: a conjunct goes to the related row only where every field it
/// reads is copied, all through one input.
fn split<'c>(copies: &[Copied<'c>], conjuncts: &[&Predicate]) -> Split<'c> {
    let mut toward: BTreeMap<&str, (&EntityHandle, Vec<Predicate>)> = BTreeMap::new();
    let mut rest: Vec<Predicate> = Vec::new();
    let onto = |path: &FactPath| match path.segments().split_first() {
        Some((head, tail)) => copies
            .iter()
            .find(|copy| copy.target == head.as_str())
            .map_or_else(
                || path.clone(),
                |copy| {
                    FactPath::from_segments(
                        std::iter::once(copy.field.to_owned()).chain(tail.iter().cloned()),
                    )
                },
            ),
        None => path.clone(),
    };
    for conjunct in conjuncts {
        let read = heads(conjunct);
        let copy_of = |head: &str| copies.iter().find(|copy| copy.target == head);
        let through: BTreeSet<&str> = read
            .iter()
            .filter_map(|head| copy_of(head))
            .map(|copy| copy.input)
            .collect();
        let all_copied = !read.is_empty() && read.iter().all(|head| copy_of(head).is_some());
        let copy = match through.iter().collect::<Vec<_>>()[..] {
            [input] if all_copied => copies.iter().find(|copy| copy.input == *input),
            _ => None,
        };
        match copy {
            Some(copy) => toward
                .entry(copy.input)
                .or_insert_with(|| (copy.entity, Vec::new()))
                .1
                .push(map_paths(conjunct, &onto)),
            None => rest.push((*conjunct).clone()),
        }
    }
    (toward, rest)
}

/// `rest` read onto `creator`'s input through its direct, unconverted `sets:` entries.
fn onto_input(creator: &ResolvedOutcome, rest: Vec<Predicate>) -> Predicate {
    let mapped: BTreeMap<&str, &str> = creator
        .sets
        .iter()
        .filter(|set| set.conversion.is_none())
        .filter_map(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. } => {
                Some((set.target.as_str(), field.as_str()))
            }
            _ => None,
        })
        .collect();
    let onto = |path: &FactPath| match path.segments().split_first() {
        Some((head, tail)) => mapped.get(head.as_str()).map_or_else(
            || path.clone(),
            |field| {
                FactPath::from_segments(
                    field
                        .split('.')
                        .map(str::to_owned)
                        .chain(tail.iter().cloned()),
                )
            },
        ),
        None => path.clone(),
    };
    map_paths(&Predicate::All(rest), &onto)
}

/// A row of each related entity in `toward`, arranged toward what the goal needs of it, by the
/// input that names it; `None` where one cannot be.
fn referenced<'c>(
    ir: &EssIr,
    toward: &BTreeMap<&'c str, (&'c EntityHandle, Vec<Predicate>)>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Option<BTreeMap<&'c str, Arrangement>> {
    let mut out = BTreeMap::new();
    for (input, (related, wanted)) in toward {
        let wanted = Predicate::All(wanted.clone());
        let holds = |row: &Arrangement| {
            subject_fact::row_truth_with(
                ir,
                related,
                &row.settled,
                &row.unwritten,
                Some(&row.state),
                &wanted,
                None,
            ) == Truth::True
        };
        let row = arrange_toward_owned(ir, related, &wanted, actors, distinction, None, &holds)?;
        out.insert(*input, row);
    }
    Some(out)
}

/// `start` with the steps of every row in `referenced` before its own, and the value each copy
/// reads there settled under the field it fills.
fn after_referenced(
    ir: &EssIr,
    copies: &[Copied<'_>],
    referenced: &BTreeMap<&str, Arrangement>,
    mut start: Arrangement,
) -> Arrangement {
    let mut steps = Vec::new();
    for (input, row) in referenced {
        steps.extend(row.steps.iter().cloned());
        start.source.extend(row.source.iter().cloned());
        for copy in copies.iter().filter(|copy| copy.input == *input) {
            let identity = &ir.entity(copy.entity).identity.name;
            let value = if copy.field == identity {
                Some(ScenarioValue::instance(row.instance.clone()))
            } else {
                row.settled.get(copy.field).map(|held| held.value.clone())
            };
            if let Some(value) = value {
                start.settled.insert(
                    copy.target.to_owned(),
                    Determined {
                        value,
                        type_ref: copy.type_ref.clone(),
                    },
                );
            }
        }
    }
    steps.append(&mut start.steps);
    start.steps = steps;
    start
}

/// One row of `entity` `accept` takes, where a creating branch copies a field `goal` reads from the
/// row an input names: that row arranged first, toward the conjuncts of `goal` over the copied
/// fields read as the fields they copy, then the creating command sent it, its input steered by the
/// rest of `goal`, and the row driven to the cheapest state `accept` takes. `None` where no creator
/// copies a field `goal` reads, or none lands a row `accept` takes.
pub(super) fn steered(
    ir: &EssIr,
    entity: &EntityHandle,
    goal: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (distinction, owner): (Distinction, Option<&InstanceName>),
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    let drivers = &ir.drivers_of(entity);
    let states = &ir.entity(entity).lifecycle.states;
    let mut conjuncts: Vec<&Predicate> = Vec::new();
    flattened(goal, &mut conjuncts);
    for creator in drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        let copies = copies(creator.outcome);
        if copies.is_empty() {
            continue;
        }
        let (toward, rest) = split(&copies, &conjuncts);
        if toward.is_empty() {
            continue;
        }
        let Some(referenced) = referenced(ir, &toward, actors, distinction) else {
            continue;
        };
        let targets: BTreeMap<&str, &str> = copies
            .iter()
            .filter(|copy| referenced.contains_key(copy.input))
            .map(|copy| (copy.target, copy.input))
            .collect();
        let fields: BTreeMap<String, InstanceName> = copies
            .iter()
            .filter_map(|copy| {
                referenced
                    .get(copy.input)
                    .map(|row| (copy.target.to_owned(), row.instance.clone()))
            })
            .collect();
        let Some(bound) =
            creator_bindings(ir, creator, &targets, (&fields, &BTreeMap::new()), false)
        else {
            continue;
        };
        let rest = onto_input(creator.outcome, rest);
        let mut guards = vec![&rest];
        guards.extend(crate::decision::when(creator.outcome));
        let Ok(inputs) = candidates(ir, creator.command, &guards, distinction) else {
            continue;
        };
        for input in inputs {
            if selects_branch(ir, creator.command, creator.outcome, None, &input) != Ok(true) {
                continue;
            }
            let under = owner.and_then(|owner| under_owner(ir, entity, creator, owner));
            let Ok(start) = created_bound(
                ir,
                entity,
                creator,
                actors,
                distinction,
                (under, false),
                &bound,
                &input,
            ) else {
                continue;
            };
            let start = after_referenced(ir, &copies, &referenced, start);
            let best = states
                .iter()
                .filter_map(|target| {
                    let route = route_from(ir, entity, drivers, &start.state, target)?;
                    advance(
                        ir,
                        entity,
                        start.clone(),
                        route,
                        target,
                        actors,
                        distinction,
                        &[],
                    )
                    .ok()
                })
                .filter(|reached| accept(reached))
                .min_by_key(|reached| reached.steps.len());
            if best.is_some() {
                return best;
            }
        }
    }
    None
}
