//! The rows an input written into a `references` field names
//! (`docs/design/referenced-input-arrangement.md`).
//!
//! A branch whose `sets:` copies an input unchanged into a field of its subject that carries a
//! `references` relation (`cardinality: one` for a plain field, `many` for a `List`) is sent an
//! input naming rows the scenario arranged first, each created through the referenced entity's
//! creating route, so a target resolving the references it stores accepts the branch. The unit
//! changes a sent value only where nothing reads it, decided by an allow-list: the input field is
//! unread only when every use of it anywhere in the resolved command (every branch's guards of
//! any form, sets, payload, affects, instances and filters) is the one `sets:` entry that copies it
//! into the reference field, and nothing reads that stored field (no invariant of the entity, no
//! guard, copy, expression or filter of any command acting on it, `subject.f` included). Anything
//! else, or anything in doubt, is
//! a read, and a field read is sent the witness value, as it was before.
//!
//! * a `many` relation with a route is sent exactly as many rows as its witness list holds, and an
//!   empty list stays empty;
//! * a `many` relation with no route is sent the empty list, which names no missing row;
//! * a `one` relation with no route refuses the scenario by naming the relation, as a plain
//!   reference has no empty value to fall back to;
//! * a list of more than [`MOST`] rows refuses the scenario by naming the relation.
//!
//! A referenced row rests where a creating branch leaves it (`into:`), and when that branch
//! writes a reference of its own from its input the row it names is arranged first, under the same
//! allow-list (a field the creating command uses elsewhere or the created entity reads keeps its
//! witness value): the row a command names and the [`DEPTH`] rows chained below it, four rows in
//! all; the fourth is created plain, so its own references stay what the witness chose, and so do
//! those of a creating branch another family arranges (a related guard, a held-state guard, an
//! owner). Where no route creates a chained row's own `many` list it is sent empty, and a `one`
//! reference there gives up the chain and refuses the outer scenario by name.
//!
//! Left alone: a refusal, a replay, a field something reads, a field another arrangement already
//! named, and a relation the subject's entity holds to itself. The caller leaves out every command
//! a related guard, a list of references or a row set arranges: they name their own rows.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    Driver, EntityHandle, EssIr, ResolvedCommand, ResolvedEffect, ResolvedOutcome,
    ResolvedPayloadValue, ResolvedRelation,
};
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;
use serde_json::Value;

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
/// The kinds of a `sets:` or `payload:` value that read the input, whose `field` is not a stored
/// one; a node tagged `from: input` (a related guard's `via`) names an input field the same way.
/// one.
const INPUT_KINDS: [&str; 2] = ["input_field", "input_or_generated"];

/// Whether any name in `value`, a resolved part of the model as JSON, passes `hit`: every string
/// is read as the dotted paths it holds, and every key as a name, so a path in a predicate, an
/// expression or a field is found wherever the model keeps it. The keys in `skipped` name no read,
/// and a node naming an input field is passed over when `stored` is asked for.
fn mentions(value: &Value, hit: &dyn Fn(&[&str]) -> bool, skipped: &[&str], stored: bool) -> bool {
    let named = |text: &str| {
        text.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
            .any(|path| {
                let segments: Vec<&str> = path.split('.').filter(|part| !part.is_empty()).collect();
                !segments.is_empty() && hit(&segments)
            })
    };
    match value {
        Value::String(text) => named(text),
        Value::Array(items) => items
            .iter()
            .any(|item| mentions(item, hit, skipped, stored)),
        Value::Object(map) => {
            let input = |map: &serde_json::Map<String, Value>| {
                map.get("kind")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| INPUT_KINDS.contains(&kind))
                    || map.get("from").and_then(Value::as_str) == Some("input")
            };
            if stored && input(map) {
                return false;
            }
            map.iter()
                .filter(|(key, _)| !skipped.contains(&key.as_str()))
                .any(|(key, item)| named(key) || mentions(item, hit, skipped, stored))
        }
        _ => false,
    }
}

/// Whether the input `field` of `command` is read anywhere but by the `sets:` entry of `outcome`
/// that copies it into the subject field `via`: in any branch's guard of any form, set, payload,
/// affect, instance or filter, or anything else the resolved command holds. Only the command's
/// own declarations (names, input, response, naming) are left out. A command that cannot be
/// written out counts as reading it.
fn input_read(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    via: &str,
    field: &str,
) -> bool {
    let Ok(Value::Object(mut json)) = serde_json::to_value(command) else {
        return true;
    };
    for key in [
        "name",
        "domain",
        "input",
        "response",
        "examples",
        "fixture_inputs",
        "naming",
        "refs",
        "undeclared_fields",
    ] {
        json.remove(key);
    }
    let Some(Value::Array(branches)) = json.get_mut("outcomes") else {
        return true;
    };
    for (branch, value) in command.outcomes.iter().zip(branches) {
        let Value::Object(map) = value else {
            return true;
        };
        map.remove("name");
        if !std::ptr::eq(branch, outcome) {
            continue;
        }
        let claimed = outcome.sets.iter().position(|set| {
            set.target == via
                && set.conversion.is_none()
                && matches!(&set.value, ResolvedPayloadValue::InputField { field: input, .. }
                    if input == field)
        });
        if let (Some(index), Some(Value::Array(sets))) = (claimed, map.get_mut("sets")) {
            sets.remove(index);
        }
    }
    mentions(
        &Value::Object(json),
        &|path| path[0] == field || (path[0] == "input" && path.get(1) == Some(&field)),
        &["target"],
        false,
    )
}

/// Whether the stored field `via` of `entity` is read: by an invariant of the entity, or by any
/// branch of any command that acts on the entity (its guard, a copy of the field, an expression, a
/// selector or filter), where a branch's own input is not a stored read. A name counts wherever it
/// stands in a path (`subject.via` included, not only first). Anything that cannot be written out
/// counts as a read.
fn stored_read(ir: &EssIr, entity: &EntityHandle, via: &str) -> bool {
    let hit = |path: &[&str]| path.contains(&via);
    let by_invariant = ir.entity(entity).invariants.iter().any(|invariant| {
        serde_json::to_value(&invariant.predicate)
            .map_or(true, |predicate| mentions(&predicate, &hit, &[], false))
    });
    by_invariant
        || ir.commands().values().any(|command| {
            command.outcomes.iter().any(|branch| {
                let acts = branch
                    .subject
                    .as_ref()
                    .is_some_and(|subject| subject.entity == *entity)
                    || branch
                        .instances
                        .as_ref()
                        .is_some_and(|instances| instances.entity == *entity)
                    || branch.affects.iter().any(|affect| affect.entity == *entity);
                acts && serde_json::to_value(branch).map_or(true, |branch| {
                    mentions(&branch, &hit, &["target", "name"], true)
                })
            })
        })
}

/// Whether anything in the model reads the input `field` that `outcome` of `command` copies into
/// the subject field `via` of `entity`: any use of the input but that copy, or any read of the
/// stored field. What is read is sent the witness value, which already satisfies it. In doubt it
/// is read.
fn read(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    entity: &EntityHandle,
    via: &str,
    field: &str,
) -> bool {
    input_read(command, outcome, via, field) || stored_read(ir, entity, via)
}

/// The references `outcome` writes unchanged from its command's input into its entity, as the
/// input field and the relation, leaving out a relation to the entity itself, one to an entity
/// already being arranged and one anything else reads.
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
            let held = command.input.iter().any(|held| held.name == field);
            (held && !read(ir, command, outcome, entity, &relation.via, field))
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
        if let Some(chained) = chained(ir, entity, actors, at, ids, arranging, depth)? {
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

/// The value an input sends for the arranged rows `items`: the list under `many`, the one row under
/// `one`.
fn shaped(relation: &ResolvedRelation, items: Vec<ScenarioValue>) -> ScenarioValue {
    match relation.cardinality {
        Cardinality::Many => ScenarioValue::List { items },
        Cardinality::One => items
            .into_iter()
            .next()
            .expect("one row is arranged for a `one` relation"),
    }
}

/// A row of `entity` made by a creating branch that writes references from its input, each
/// created ahead of it and named in the creating step. `None` where no creating branch does, or
/// the one that does is another family's to arrange; an error where no route creates a `one`
/// reference of the branch, so the row cannot be named without naming a missing one.
fn chained(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: Distinction,
    ids: &mut Distinctions,
    arranging: &[&EntityHandle],
    depth: usize,
) -> Result<Option<Arrangement>, Unreachable> {
    if ir.owner_of(entity).is_some() {
        return Ok(None);
    }
    let all = ir.drivers();
    let initial = &ir.entity(entity).lifecycle.initial;
    let Some(drivers) = all.get(entity) else {
        return Ok(None);
    };
    let mut creators: Vec<&Driver<'_>> = drivers
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
                let (sent, rows) = match rows {
                    Ok(rows) => (None, rows),
                    Err(_) if relation.cardinality == Cardinality::Many => (
                        Some(ScenarioValue::literal(Node::Seq(Vec::new()))),
                        Vec::new(),
                    ),
                    Err(_) => return Err(Unreachable::NothingCreates),
                };
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
                if sent.is_none() {
                    invoked
                        .source
                        .insert(EntityRef::from(&relation.target).into());
                }
                if let ScenarioStep::ExecuteCommand { input, .. } = &mut invoked.steps[sending] {
                    input.insert(
                        (*field).to_owned(),
                        sent.unwrap_or_else(|| shaped(relation, items)),
                    );
                }
            }
            prelude.append(&mut invoked.steps);
            invoked.steps = prelude;
            Ok::<_, Unreachable>(invoked)
        });
        match made {
            Ok(arrangement) => return Ok(Some(arrangement)),
            Err(Unreachable::NothingCreates) => return Err(Unreachable::NothingCreates),
            Err(_) => {}
        }
    }
    Ok(None)
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
            || !command.input.iter().any(|held| held.name == field)
            || read(ir, command, outcome, &subject.entity, &relation.via, field)
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
            if relation.cardinality == Cardinality::Many {
                sent.insert(
                    field.to_owned(),
                    ScenarioValue::literal(Node::Seq(Vec::new())),
                );
                continue;
            }
            return Err(unsent(
                relation,
                "no command creates the entity this reference names in a state the scenario can \
                 arrange, so it cannot send an input naming a row",
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
        sent.insert(field.to_owned(), shaped(relation, instances));
    }
    arranged.append(&mut setup.steps);
    setup.steps = arranged;
    setup.source.extend(sources);
    Ok(sent)
}
