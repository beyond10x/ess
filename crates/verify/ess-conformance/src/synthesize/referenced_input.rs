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
//! * a `many` relation with no route is sent the empty list, which names no missing row, and it
//!   fails closed: (a) the input is decided again as it will be sent, arranged rows (a fresh
//!   identity each) and every `[]` at once, the lists together, the precedence order still
//!   selecting the branch (a stored-row guard reading a changed input is undecided) and no
//!   invariant broken by that one input; a chained row's own list is held to the same rule at
//!   every depth and a chain it fails is given up, never left naming witness identities; (b)
//!   the row the branch stores binds a field only in a form understood exactly (an input copy, a
//!   literal, `{input: x, else: <literal>}`, the empty list, a created row's state), any other
//!   form being undecided, never absent, as is a field a branch that does not create leaves
//!   unset; (c) every invariant reading a list, a field the branch sets in any form or `state` is
//!   evaluated over that row. Where they do not all hold, or a read is undecided, every such
//!   list's scenario is refused by name;
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
    Driver, EntityHandle, EssIr, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedFallback, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue,
    ResolvedRelation,
};
use ess_domain::entity::{Cardinality, EntitySpec, RelationKind};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;
use ess_primitives::predicate::Truth;

use super::{
    arrange_first, binding_effects, born, created_by, has_subject_guards, invoke, related,
    related_guard, ActorRef, Arrangement, RefusalCause, ScenarioStep, ScenarioValue, Setup,
    Unreachable, Witness,
};
use crate::input::Completeness;
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

/// Whether `sent` is a list of more rows than [`MOST`] under a `many` relation.
fn too_long(relation: &ResolvedRelation, sent: Option<&ScenarioValue>) -> bool {
    relation.cardinality == Cardinality::Many
        && matches!(sent, Some(ScenarioValue::Literal {
            value: Node::Seq(items),
        }) if items.len() > MOST)
}

/// Whether `creator`'s step, sending `input`, keeps its branch and invariants with the `many` list
/// `relation` names in `field` sent empty ([`empties_keep_branch`]).
fn keeps_empty(
    ir: &EssIr,
    creator: &Driver<'_>,
    input: &BTreeMap<String, ScenarioValue>,
    field: &str,
    relation: &ResolvedRelation,
) -> bool {
    relation.cardinality == Cardinality::Many
        && nodes_of(input).is_some_and(|current| {
            empties_keep_branch(
                ir,
                creator.command,
                creator.outcome,
                &current,
                &current,
                &[(field, relation.via.as_str())],
                &[],
            )
        })
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
                if too_long(relation, input.get(*field)) {
                    return Err(Unreachable::NothingCreates);
                }
                let Some(count) = named(relation, input.get(*field)) else {
                    continue;
                };
                let kept = keeps_empty(ir, creator, input, field, relation);
                let rows: Result<Vec<_>, _> = (0..count)
                    .map(|_| row(ir, &relation.target, actors, ids, &chain, depth + 1))
                    .collect();
                let Ok(rows) = rows else {
                    if !kept {
                        return Err(Unreachable::NothingCreates);
                    }
                    if let ScenarioStep::ExecuteCommand { input, .. } = &mut invoked.steps[sending]
                    {
                        input.insert(
                            (*field).to_owned(),
                            ScenarioValue::literal(Node::Seq(Vec::new())),
                        );
                    }
                    continue;
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
                invoked
                    .source
                    .insert(EntityRef::from(&relation.target).into());
                if let ScenarioStep::ExecuteCommand { input, .. } = &mut invoked.steps[sending] {
                    input.insert((*field).to_owned(), shaped(relation, items));
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

/// The input at `path` (declared segments joined by `.`), where it holds a value.
fn input_at(input: &BTreeMap<String, Node>, path: &str) -> Option<Node> {
    let mut segments = path.split('.');
    let mut current = input.get(segments.next()?)?;
    for segment in segments {
        let Node::Map(fields) = current else {
            return None;
        };
        current = fields.get(segment)?;
    }
    Some(current.clone())
}

/// The row `outcome` would store, as far as it is understood exactly.
struct StoredRow {
    /// The fields whose stored value is known: a copied input (or the literal an `else:` names
    /// where it is absent), a literal, the empty list.
    known: BTreeMap<String, Node>,
    /// The fields `sets:` fills in a form this does not follow (an `else:` naming an input, a
    /// struct, a conversion, an expression, a generated value, a member path): undecided, never
    /// absent.
    undecided: Vec<String>,
    /// Every field `sets:` fills, whatever the form.
    set: Vec<String>,
}

/// The value one `sets:` entry stores, where its form is one this follows exactly.
fn stored_value(
    ir: &EssIr,
    set: &ResolvedPayloadField,
    input: &BTreeMap<String, Node>,
) -> Option<Node> {
    if set.conversion.is_some() || set.target.contains('.') {
        return None;
    }
    match &set.value {
        ResolvedPayloadValue::InputField { field, .. } => input_at(input, field),
        ResolvedPayloadValue::InputOrGenerated {
            field, otherwise, ..
        } => match input_at(input, field) {
            Some(given) if given != Node::Null => Some(given),
            _ => match otherwise {
                Some(ResolvedFallback::Literal(written)) => {
                    super::literal_value(ir, &set.target_type, written, 0)
                }
                _ => None,
            },
        },
        ResolvedPayloadValue::Literal { value } => {
            super::literal_value(ir, &set.target_type, value, 0)
        }
        ResolvedPayloadValue::EmptyList => Some(Node::Seq(Vec::new())),
        _ => None,
    }
}

/// The row `outcome` stores for `input`.
fn stored_by(ir: &EssIr, outcome: &ResolvedOutcome, input: &BTreeMap<String, Node>) -> StoredRow {
    let mut row = StoredRow {
        known: BTreeMap::new(),
        undecided: Vec::new(),
        set: Vec::new(),
    };
    for set in &outcome.sets {
        let root = set.target.split('.').next().unwrap_or(&set.target);
        row.set.push(root.to_owned());
        match stored_value(ir, set, input) {
            Some(value) => {
                row.known.insert(root.to_owned(), value);
            }
            None => row.undecided.push(root.to_owned()),
        }
    }
    row
}

/// The input guard `branch` reads, as the precedence order decides it from the input alone.
fn input_guard_of(branch: &ResolvedOutcome) -> Option<&ess_primitives::predicate::Predicate> {
    match &branch.condition {
        ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
        ResolvedCondition::SubjectPredicate { input, .. }
        | ResolvedCondition::ExternalSubject { input, .. }
        | ResolvedCondition::Related { input, .. } => input.as_ref(),
        _ => crate::decision::when(branch),
    }
}

/// Whether every guard the precedence order reads up to and including `outcome` decides `moved`
/// as it decides `original`: each branch's input guard (a held-state guard reads no input and is
/// skipped).
/// A guard this cannot decide is not held to be unchanged.
fn guards_unchanged(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    original: &BTreeMap<String, Node>,
    moved: &BTreeMap<String, Node>,
) -> bool {
    let Some(last) = command
        .outcomes
        .iter()
        .position(|branch| branch.name == outcome.name)
    else {
        return false;
    };
    let (Ok(before), Ok(after)) = (
        crate::input::flatten(ir, command, original),
        crate::input::flatten(ir, command, moved),
    ) else {
        return false;
    };
    command.outcomes[..=last].iter().all(|branch| {
        input_guard_of(branch).is_none_or(|guard| {
            std::mem::discriminant(&before.decide(guard))
                == std::mem::discriminant(&after.decide(guard))
        })
    })
}

/// Whether a stored-row guard (a `when_subject:` predicate) of a branch the precedence order reads
/// up to and including `outcome` reads an input `moved` changed from `original`: it is decided
/// against the row the scenario arranges, which synthesis does not hold here, so it is undecided.
fn stored_guard_reads_changed(
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    original: &BTreeMap<String, Node>,
    moved: &BTreeMap<String, Node>,
) -> bool {
    let Some(last) = command
        .outcomes
        .iter()
        .position(|branch| branch.name == outcome.name)
    else {
        return true;
    };
    let changed: Vec<&str> = moved
        .iter()
        .filter(|(name, value)| original.get(*name) != Some(*value))
        .map(|(name, _)| name.as_str())
        .collect();
    command.outcomes[..=last].iter().any(|branch| {
        let stored = match &branch.condition {
            ResolvedCondition::SubjectPredicate { predicate, .. } => Some(predicate),
            ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
            _ => None,
        };
        stored.is_some_and(|predicate| {
            predicate.fact_paths().iter().any(|path| {
                let segments = path.segments();
                let Some(name) = segments
                    .get(1)
                    .filter(|_| segments.first().is_some_and(|root| root == "input"))
                else {
                    return false;
                };
                let same_length = matches!(
                    (original.get(name), moved.get(name)),
                    (Some(Node::Seq(before)), Some(Node::Seq(after))) if before.len() == after.len()
                );
                changed.contains(&name.as_str())
                    && !(same_length && segments.len() == 3 && segments[2] == "count")
            })
        })
    })
}

/// Whether `moved` still takes `outcome`: `selects_branch` says so, or, where it cannot decide (a
/// stored-row guard needs the held row), every guard the precedence order reads decides `moved` as
/// it decided `original`. Where `original` itself was not selected there is nothing to keep, and
/// `strict` says whether that counts as kept.
fn still_selected(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    original: &BTreeMap<String, Node>,
    moved: &BTreeMap<String, Node>,
    strict: bool,
) -> bool {
    if stored_guard_reads_changed(command, outcome, original, moved) {
        return false;
    }
    let before = super::selects_branch(ir, command, outcome, None, original);
    match super::selects_branch(ir, command, outcome, None, moved) {
        Ok(true) => true,
        Ok(false) => !strict && !matches!(before, Ok(true)),
        Err(_) => guards_unchanged(ir, command, outcome, original, moved),
    }
}

/// Whether sending every list of `lists` (input field, stored field) the empty list at once
/// leaves `outcome` the branch the command takes and its subject's invariants holding: the
/// precedence order selects `outcome` for the one moved input (the selection `reach` makes, so a
/// later accepting branch does not count), `invariant_broken_by` finds none broken, and every
/// invariant that reads a list, a field the branch sets in any form or `state` is satisfied by the
/// row `outcome` would store. A field the branch sets in a form [`stored_by`] does not follow is
/// undecided, and on a branch that does not create so is a field it does not set; an undecided
/// read is not held to be kept.
fn empties_keep_branch(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    original: &BTreeMap<String, Node>,
    base: &BTreeMap<String, Node>,
    emptied: &[(&str, &str)],
    arranged: &[&str],
) -> bool {
    let empty = Node::Seq(Vec::new());
    let mut moved = base.clone();
    for (field, _) in emptied {
        let Some(declared) = super::input_type(ir, command, field) else {
            return false;
        };
        if crate::input::validate_typed_value(ir, declared, &empty).is_err() {
            return false;
        }
        moved.insert((*field).to_owned(), empty.clone());
    }
    if !still_selected(ir, command, outcome, original, &moved, !emptied.is_empty()) {
        return false;
    }
    let Some(subject) = &outcome.subject else {
        return true;
    };
    if crate::witness::invariant_broken_by(ir, command, outcome, &moved).is_some() {
        return false;
    }
    let entity = ir.entity(&subject.entity);
    let creates = matches!(subject.effect, ResolvedEffect::Creates);
    let mut row = stored_by(ir, outcome, &moved);
    let mut fields = entity.fields.clone();
    if creates {
        let landed = subject.into.as_ref().unwrap_or(&entity.lifecycle.initial);
        fields.push(entity.state_field());
        row.known
            .insert(EntitySpec::STATE.to_owned(), Node::Text(landed.to_string()));
    }
    let Ok(facts) = crate::input::bind(ir, &fields, &row.known, Completeness::Partial) else {
        return false;
    };
    let facts = crate::input::TypedFacts::new(ir, &fields, facts);
    entity.invariants.iter().all(|invariant| {
        let reads: Vec<&str> = invariant
            .predicate
            .fact_paths()
            .iter()
            .filter_map(|path| path.segments().first().map(String::as_str))
            .collect();
        let touched = reads.iter().any(|root| {
            *root == EntitySpec::STATE
                || row.set.iter().any(|set| set == root)
                || emptied.iter().any(|(_, stored)| stored == root)
                || arranged.contains(root)
        });
        if !touched {
            return true;
        }
        let decided = reads.iter().all(|root| {
            if row.undecided.iter().any(|undecided| undecided == root) {
                return false;
            }
            let declared =
                *root == EntitySpec::STATE || entity.fields.iter().any(|held| held.name == *root);
            creates || !declared || row.known.contains_key(*root)
        });
        decided && invariant.predicate.evaluate(&facts) == Truth::True
    })
}

/// The input a step sends as a map of values the guards and invariants can read, an arranged row's
/// identity standing as one fresh text no other value equals (what the target generates is never
/// another input's); `None` where a value is neither a literal nor an arranged row.
fn nodes_of(sent: &BTreeMap<String, ScenarioValue>) -> Option<BTreeMap<String, Node>> {
    let mut fresh = 0_u64;
    let mut identity = || {
        fresh += 1;
        Node::Text(format!(
            "00000000-0000-4000-8000-{:012x}",
            0xa11c_e000_u64 + fresh
        ))
    };
    let mut one = |value: &ScenarioValue| match value {
        ScenarioValue::Literal { value } => Some(value.clone()),
        ScenarioValue::Instance { .. } => Some(identity()),
        _ => None,
    };
    sent.iter()
        .map(|(name, value)| {
            let node = match value {
                ScenarioValue::List { items } => {
                    Node::Seq(items.iter().map(&mut one).collect::<Option<Vec<_>>>()?)
                }
                other => one(other)?,
            };
            Some((name.clone(), node))
        })
        .collect()
}

/// What `outcome` sends once the arrangement has built its input: the rows arranged for `arranged`
/// (field, relation, value) in place of the witness's, and the empty list for every list of
/// `unarranged` no route creates a row for. The final input is decided again by
/// [`empties_keep_branch`], all the lists together; refused, naming the relations, where it does
/// not hold.
fn send_decided(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    input: &BTreeMap<String, Node>,
    arranged: &BTreeMap<String, ScenarioValue>,
    unarranged: &[(&str, &ResolvedRelation)],
) -> Result<BTreeMap<String, ScenarioValue>, RefusalCause> {
    let (Some(subject), false) = (
        &outcome.subject,
        arranged.is_empty() && unarranged.is_empty(),
    ) else {
        return Ok(BTreeMap::new());
    };
    let entity = ir.entity(&subject.entity);
    let named: Vec<&ResolvedRelation> = entity
        .relations
        .iter()
        .filter(|relation| {
            unarranged
                .iter()
                .any(|(_, held)| std::ptr::eq(*held, *relation))
                || related::set_from_input(outcome, &relation.via)
                    .is_some_and(|field| arranged.contains_key(field))
        })
        .collect();
    let refuse = |reason: &'static str| {
        RefusalCause::NoWitness(WitnessGap {
            path: named
                .iter()
                .map(|relation| format!("{}.{}", entity.name, relation.name))
                .collect::<Vec<_>>()
                .join(" and "),
            type_ref: named.first().map_or_else(String::new, |relation| {
                ir.entity(&relation.target).name.to_string()
            }),
            reason,
        })
    };
    let Some(rows) = nodes_of(arranged) else {
        return Err(refuse(
            "is sent a value synthesis cannot read back to decide the branch again",
        ));
    };
    let mut base = input.clone();
    base.extend(rows);
    let emptied: Vec<(&str, &str)> = unarranged
        .iter()
        .map(|(field, relation)| (*field, relation.via.as_str()))
        .collect();
    let stored: Vec<&str> = named.iter().map(|relation| relation.via.as_str()).collect();
    if !empties_keep_branch(ir, command, outcome, input, &base, &emptied, &stored) {
        return Err(refuse(
            "is sent rows or the empty list that would change the branch the command takes or \
             break an invariant of the entity, or that synthesis cannot decide, so it cannot send \
             an input naming a row",
        ));
    }
    Ok(unarranged
        .iter()
        .map(|(field, _)| {
            (
                (*field).to_owned(),
                ScenarioValue::literal(Node::Seq(Vec::new())),
            )
        })
        .collect())
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
    let mut unarranged: Vec<(&str, &ResolvedRelation)> = Vec::new();
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
            if relation.cardinality == Cardinality::Many {
                unarranged.push((field, relation));
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
    let empties = send_decided(ir, command, outcome, input, &sent, &unarranged)?;
    sent.extend(empties);
    arranged.append(&mut setup.steps);
    setup.steps = arranged;
    setup.source.extend(sources);
    Ok(sent)
}
