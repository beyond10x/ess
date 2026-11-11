//! The rows an input written into a `references` field names
//! (`docs/design/referenced-input-arrangement.md`).
//!
//! A branch whose `sets:` copies an input unchanged into a field of its subject that carries a
//! `references` relation (`cardinality: one` for a plain field, `many` for a `List`) is sent an
//! input naming rows the scenario arranged first, each created through the referenced entity's
//! creating route, so a target resolving the references it stores accepts the branch:
//!
//! * a `many` relation with a route is sent as many rows as its witness list holds, and at least
//!   one, so the reference is exercised and not passed by an empty list;
//! * a `many` relation with no route is sent the empty list, which names no missing row;
//! * a `one` relation with no route refuses the scenario by naming the relation, as a plain
//!   reference has no empty value to fall back to.
//!
//! Left alone: a refusal, a replay, a branch whose own `when:` guard reads the field (its witness
//! list is what selects it), a field another arrangement already named, and a relation the
//! subject's entity holds to itself. The caller leaves out every command a related guard, a list
//! of references or a row set arranges: they name their own rows.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedOutcome};
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::{arrange_first, related, ActorRef, RefusalCause, ScenarioValue, Setup, Witness};
use crate::scenario::EntityRef;
use crate::witness::{Distinction, WitnessGap};

/// The first distinction the referenced rows are arranged at: past every block the other
/// arrangements number from, so no row is another arrangement's.
const BASE: usize = 27_720 * 60;

/// The distinctions one run reserves, so two runs of one scenario arrange different rows.
const RUN: usize = 8;

/// The most rows a `many` relation's list is arranged with.
const MOST: usize = 3;

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

/// Whether `outcome`'s own `when:` guard reads the input `field`.
fn guards_on(outcome: &ResolvedOutcome, field: &str) -> bool {
    crate::decision::when(outcome).is_some_and(|guard| {
        guard
            .fact_paths()
            .iter()
            .any(|path| path.segments().first().is_some_and(|root| root == field))
    })
}

/// The inputs `outcome` is sent naming rows arranged ahead of `setup`'s own steps, by field.
///
/// Refused, naming the relation, where a `cardinality: one` reference has no route that creates
/// the row it names.
pub(super) fn arrange(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    witness: Witness,
    setup: &mut Setup,
    input: &BTreeMap<String, Node>,
) -> Result<BTreeMap<String, ScenarioValue>, RefusalCause> {
    let mut named = BTreeMap::new();
    let Some(subject) = &outcome.subject else {
        return Ok(named);
    };
    if outcome.error.is_some() || outcome.replays.is_some() {
        return Ok(named);
    }
    let entity = ir.entity(&subject.entity);
    let mut arranged = Vec::new();
    let mut sources = Vec::new();
    let mut taken = 0;
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
            (Cardinality::Many, Some(Node::Seq(items))) => items.len().clamp(1, MOST),
            (Cardinality::One, Some(value)) if !matches!(value, Node::Seq(_) | Node::Null) => 1,
            _ => continue,
        };
        let initial = ir.entity(&relation.target).lifecycle.initial.clone();
        let reached: Result<Vec<_>, _> = (0..wanted)
            .map(|_| {
                let distinction = Distinction::further(BASE + slot(witness) * RUN + taken);
                taken += 1;
                arrange_first(
                    ir,
                    &relation.target,
                    std::slice::from_ref(&initial),
                    actors,
                    distinction,
                    &[&subject.entity],
                )
            })
            .collect();
        let Ok(rows) = reached else {
            if relation.cardinality == Cardinality::Many {
                named.insert(
                    field.to_owned(),
                    ScenarioValue::literal(Node::Seq(Vec::new())),
                );
                continue;
            }
            return Err(RefusalCause::NoWitness(WitnessGap {
                path: format!("{}.{}", entity.name, relation.name),
                type_ref: ir.entity(&relation.target).name.to_string(),
                reason: "no command creates the entity this reference names, so the scenario \
                         cannot send an input naming a row",
            }));
        };
        let instances: Vec<ScenarioValue> = rows
            .iter()
            .map(|row| ScenarioValue::Instance {
                instance: row.instance.clone(),
            })
            .collect();
        for row in rows {
            arranged.extend(row.steps);
            sources.extend(row.source);
        }
        sources.push(EntityRef::from(&relation.target).into());
        let value = match relation.cardinality {
            Cardinality::Many => ScenarioValue::List { items: instances },
            Cardinality::One => instances
                .into_iter()
                .next()
                .expect("one row is arranged for a `one` relation"),
        };
        named.insert(field.to_owned(), value);
    }
    arranged.append(&mut setup.steps);
    setup.steps = arranged;
    setup.source.extend(sources);
    Ok(named)
}
