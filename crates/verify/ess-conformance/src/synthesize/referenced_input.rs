//! The rows an input written into a `references` field names
//! (`docs/design/referenced-input-arrangement.md`).
//!
//! A branch whose `sets:` copies an input unchanged into a field of its subject that carries a
//! `references` relation (`cardinality: one` for a plain field, `many` for a `List`) is sent an
//! input naming rows the scenario arranged first, each created through the referenced entity's
//! creating route, so a target resolving the references it stores accepts the branch:
//!
//! * a `many` relation with a route is sent exactly as many rows as its witness list holds: the
//!   witness already satisfies every guard and invariant over the list, so its length is not
//!   chosen again, and an empty list stays empty;
//! * a `many` relation with no route is sent the empty list, which names no missing row, where
//!   the empty list still selects the branch and keeps the entity's invariants; where it does not
//!   (a sibling guard or an invariant holds the list non-empty) the scenario is refused by name;
//! * a `one` relation with no route refuses the scenario by naming the relation, as a plain
//!   reference has no empty value to fall back to;
//! * a list of more than [`MOST`] rows refuses the scenario by naming the relation.
//!
//! A referenced row rests where a creating branch leaves it (`into:`), and when that branch
//! writes a reference of its own from its input the row it names is arranged first: the row a
//! command names and the [`DEPTH`] rows chained below it, four rows in all; the fourth is created
//! plain, so its own references stay what the witness chose, and so do those of a creating branch
//! another family arranges (a related guard, a held-state guard, an owner).
//!
//! Left alone: a refusal, a replay, a branch whose own `when:` guard reads the field itself (its
//! witness list is what selects it; a guard reading only the list's `count` is not left alone, as
//! the arrangement keeps the count), a field another arrangement already named, and a relation
//! the subject's entity holds to itself. The caller leaves out every command a related guard, a list
//! of references or a row set arranges: they name their own rows.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    Driver, EntityHandle, EssIr, ResolvedCommand, ResolvedEffect, ResolvedOutcome, ResolvedRelation,
};
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::{
    arrange_first, binding_effects, born, created_by, has_subject_guards, invoke, related,
    related_guard, ActorRef, Arrangement, RefusalCause, ScenarioStep, ScenarioValue, Setup,
    Unreachable, Witness,
};
use crate::scenario::EntityRef;
use crate::witness::{Distinction, WitnessGap};

/// The first distinction the referenced rows are arranged at: past every block the other
/// arrangements number from, so no row is another arrangement's.
const BASE: usize = 27_720 * 60;

/// The distinctions one run reserves, so two runs of one scenario arrange different rows: more
/// than [`MOST`] rows at each of [`DEPTH`] levels can take.
const RUN: usize = 4096;

/// The most rows one list is arranged with.
const MOST: usize = 8;

/// How many rows are chained below the row a command names: that row and the [`DEPTH`] rows
/// beneath it make four, the last created plain.
const DEPTH: usize = 3;

/// Which run of its branch a witness is, so each run arranges rows of its own.
fn slot(witness: Witness) -> usize {
    match witness {
        Witness::Full => 0,
        Witness::LiteralFallbacks(nth) => 1 + nth,
        Witness::Listed(nth) => 20 + nth,
        Witness::RelatedBoundary { of, goal } => 40 + of * 4 + goal,
        Witness::RelatedAbsent(nth) => 100 + nth,
        Witness::RelatedValueAbsent(nth) => 120 + nth,
        Witness::UnionVariant(nth) => 140 + nth,
        Witness::ExpressionPoint(nth) => 160 + nth,
        Witness::Unstored => 180,
        Witness::InsideLaterRefusal(nth) => 181 + nth,
    }
}

/// The distinctions of one run, handed out in order.
struct Distinctions {
    base: usize,
    taken: usize,
}

impl Distinctions {
    fn next(&mut self) -> Distinction {
        let next = Distinction::further(self.base + self.taken);
        self.taken += 1;
        next
    }
}

/// Whether `outcome`'s own `when:` guard reads the input `field` other than through its length:
/// a read of `field.count` alone is decided by how many rows the witness list holds, and the
/// arrangement keeps that number.
fn guards_on(outcome: &ResolvedOutcome, field: &str) -> bool {
    crate::decision::when(outcome).is_some_and(|guard| {
        guard.fact_paths().iter().any(|path| {
            let segments = path.segments();
            segments.first().is_some_and(|root| root == field)
                && !(segments.len() == 2 && segments[1] == "count")
        })
    })
}

/// The references `outcome` writes unchanged from its command's input into its entity, as the
/// input field and the relation, leaving out a relation to the entity itself and one to an entity
/// already being arranged.
fn written<'ir>(
    ir: &'ir EssIr,
    entity: &EntityHandle,
    command: &ResolvedCommand,
    outcome: &'ir ResolvedOutcome,
    chain: &[&EntityHandle],
) -> Vec<(&'ir str, &'ir ResolvedRelation)> {
    let resolved = ir.entity(entity);
    resolved
        .relations
        .iter()
        .filter(|relation| {
            relation.kind == RelationKind::References
                && relation.via != resolved.identity.name
                && relation.target != *entity
                && !chain.iter().any(|held| **held == relation.target)
        })
        .filter_map(|relation| {
            let field = related::set_from_input(outcome, &relation.via)?;
            command
                .input
                .iter()
                .any(|held| held.name == field)
                .then_some((field, relation))
        })
        .collect()
}

/// How many rows the value a creating step sends in `field` names: its list's length under
/// `many`, one for a present plain value; `None` where nothing is to be arranged or the list is
/// longer than [`MOST`].
fn named(relation: &ResolvedRelation, sent: Option<&ScenarioValue>) -> Option<usize> {
    match (relation.cardinality, sent) {
        (
            Cardinality::Many,
            Some(ScenarioValue::Literal {
                value: Node::Seq(items),
            }),
        ) => (1..=MOST).contains(&items.len()).then_some(items.len()),
        (
            Cardinality::One,
            Some(ScenarioValue::Literal {
                value: Node::Seq(_) | Node::Null,
            }),
        ) => None,
        (Cardinality::One, Some(ScenarioValue::Literal { .. })) => Some(1),
        _ => None,
    }
}

/// One row of `entity`, where a creating branch leaves it: with the rows its own references name
/// arranged first, while `depth` allows it.
fn row(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    ids: &mut Distinctions,
    arranging: &[&EntityHandle],
    depth: usize,
) -> Result<Arrangement, Unreachable> {
    let at = ids.next();
    if depth < DEPTH {
        if let Some(chained) = chained(ir, entity, actors, at, ids, arranging, depth) {
            return Ok(chained);
        }
    }
    let initial = ir.entity(entity).lifecycle.initial.clone();
    let mut targets = vec![initial];
    let all = ir.drivers();
    for creator in all
        .get(entity)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        let landed = born(ir, creator);
        if !targets.contains(landed) {
            targets.push(landed.clone());
        }
    }
    arrange_first(ir, entity, &targets, actors, at, arranging)
}

/// A row of `entity` made by a creating branch that writes references from its input, each
/// created ahead of it and named in the creating step. `None` where no creating branch does, or
/// the one that does is another family's to arrange.
fn chained(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: Distinction,
    ids: &mut Distinctions,
    arranging: &[&EntityHandle],
    depth: usize,
) -> Option<Arrangement> {
    if ir.owner_of(entity).is_some() {
        return None;
    }
    let all = ir.drivers();
    let initial = &ir.entity(entity).lifecycle.initial;
    let mut creators: Vec<&Driver<'_>> = all
        .get(entity)?
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
        .collect();
    creators.sort_by_key(|creator| born(ir, creator) != initial);
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([entity]).collect();
    for creator in creators {
        let landed = born(ir, creator);
        if related_guard::routes(creator.command, creator.outcome)
            || has_subject_guards(creator.command)
            || binding_effects::resting(ir, creator.outcome, landed).as_ref() != Some(landed)
        {
            continue;
        }
        let references = written(ir, entity, creator.command, creator.outcome, &chain);
        if references.is_empty() {
            continue;
        }
        let made = created_by(ir, entity, creator, at, None, |_, _| {
            let mut invoked = invoke(
                ir,
                creator,
                None,
                None,
                actors,
                at,
                &BTreeMap::new(),
                &chain,
            )?;
            let sending = invoked.steps.iter().position(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if *command.name() == creator.command.name)
            });
            let mut prelude = Vec::new();
            for (field, relation) in &references {
                let Some(sending) = sending else { break };
                let ScenarioStep::ExecuteCommand { input, .. } = &invoked.steps[sending] else {
                    continue;
                };
                let Some(count) = named(relation, input.get(*field)) else {
                    continue;
                };
                let rows: Result<Vec<_>, _> = (0..count)
                    .map(|_| row(ir, &relation.target, actors, ids, &chain, depth + 1))
                    .collect();
                let Ok(rows) = rows else { continue };
                let items: Vec<ScenarioValue> = rows
                    .iter()
                    .map(|made| ScenarioValue::Instance {
                        instance: made.instance.clone(),
                    })
                    .collect();
                for made in rows {
                    prelude.extend(made.steps);
                    invoked.source.extend(made.source);
                }
                invoked
                    .source
                    .insert(EntityRef::from(&relation.target).into());
                let value = match relation.cardinality {
                    Cardinality::Many => ScenarioValue::List { items },
                    Cardinality::One => items
                        .into_iter()
                        .next()
                        .expect("one row is arranged for a `one` relation"),
                };
                if let ScenarioStep::ExecuteCommand { input, .. } = &mut invoked.steps[sending] {
                    input.insert((*field).to_owned(), value);
                }
            }
            prelude.append(&mut invoked.steps);
            invoked.steps = prelude;
            Ok::<_, Unreachable>(invoked)
        });
        if let Ok(arrangement) = made {
            return Some(arrangement);
        }
    }
    None
}

/// The inputs `outcome` is sent naming rows arranged ahead of `setup`'s own steps, by field.
///
/// Refused, naming the relation, where a `cardinality: one` reference has no route that creates
/// the row it names, and where a list names more rows than [`MOST`].
pub(super) fn arrange(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    witness: Witness,
    setup: &mut Setup,
    input: &BTreeMap<String, Node>,
) -> Result<BTreeMap<String, ScenarioValue>, RefusalCause> {
    let mut sent = BTreeMap::new();
    let Some(subject) = &outcome.subject else {
        return Ok(sent);
    };
    if outcome.error.is_some() || outcome.replays.is_some() {
        return Ok(sent);
    }
    let entity = ir.entity(&subject.entity);
    let mut arranged = Vec::new();
    let mut sources = Vec::new();
    let mut ids = Distinctions {
        base: BASE + slot(witness) * RUN,
        taken: 0,
    };
    let unsent = |relation: &ResolvedRelation, reason: &'static str| {
        RefusalCause::NoWitness(WitnessGap {
            path: format!("{}.{}", entity.name, relation.name),
            type_ref: ir.entity(&relation.target).name.to_string(),
            reason,
        })
    };
    for relation in &entity.relations {
        if relation.kind != RelationKind::References
            || relation.via == entity.identity.name
            || relation.target == subject.entity
        {
            continue;
        }
        let Some(field) = related::set_from_input(outcome, &relation.via) else {
            continue;
        };
        if setup.bound.contains_key(field)
            || guards_on(outcome, field)
            || !command.input.iter().any(|held| held.name == field)
        {
            continue;
        }
        let wanted = match (relation.cardinality, input.get(field)) {
            (Cardinality::Many, Some(Node::Seq(items))) if items.len() > MOST => {
                return Err(unsent(
                    relation,
                    "is sent a list naming more rows than synthesis arranges for one reference",
                ));
            }
            (Cardinality::Many, Some(Node::Seq(items))) if !items.is_empty() => items.len(),
            (Cardinality::One, Some(value)) if !matches!(value, Node::Seq(_) | Node::Null) => 1,
            _ => continue,
        };
        let rows: Result<Vec<_>, _> = (0..wanted)
            .map(|_| {
                row(
                    ir,
                    &relation.target,
                    actors,
                    &mut ids,
                    &[&subject.entity],
                    0,
                )
            })
            .collect();
        let Ok(rows) = rows else {
            let empty = Node::Seq(Vec::new());
            if relation.cardinality == Cardinality::Many
                && super::keeps_branch(ir, command, input, field, &empty)
            {
                sent.insert(field.to_owned(), ScenarioValue::literal(empty));
                continue;
            }
            return Err(unsent(
                relation,
                "no command creates the entity this reference names in a state the scenario can \
                 arrange, and the empty list would not keep the branch and the entity's \
                 invariants, so it cannot send an input naming a row",
            ));
        };
        let instances: Vec<ScenarioValue> = rows
            .iter()
            .map(|made| ScenarioValue::Instance {
                instance: made.instance.clone(),
            })
            .collect();
        for made in rows {
            arranged.extend(made.steps);
            sources.extend(made.source);
        }
        sources.push(EntityRef::from(&relation.target).into());
        let value = match relation.cardinality {
            Cardinality::Many => ScenarioValue::List { items: instances },
            Cardinality::One => instances
                .into_iter()
                .next()
                .expect("one row is arranged for a `one` relation"),
        };
        sent.insert(field.to_owned(), value);
    }
    arranged.append(&mut setup.steps);
    setup.steps = arranged;
    setup.source.extend(sources);
    Ok(sent)
}
