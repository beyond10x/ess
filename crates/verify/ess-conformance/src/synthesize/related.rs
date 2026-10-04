//! The rows a `{related: {via, field}}` source reads (source format `ess/16`, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8).
//!
//! A branch that reads a field of the row its subject (or its input) references is only tested by
//! a scenario in which that row exists, holds a value the scenario knows, and is not the only row of
//! its entity. So for each `via` the branch reads, three rows of the referenced entity are
//! arranged ahead of everything else — a **decoy**, the row the subject is pointed at, and a second
//! decoy, each decoy holding the value of a further witness — and the referenced row's value is
//! carried into the arrangement's settled fields under [`key`], where [`super::expression_value`]
//! finds it. An implementation that reads a decoy, the row registered first or last, or any row but
//! the one named publishes another value and fails the assertion.
//!
//! Where any part of that cannot be built — the referenced entity has no creating branch, the
//! subject's creating branch does not store `via` from its input, a later act rewrote `via` — the
//! arrangement is left as it was and the source stays covered by the payload shape alone, as an
//! undetermined `{subject: …}` is.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedEffect, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField,
    ResolvedPayloadValue, ResolvedRelatedVia,
};
use ess_domain::command::TestStrategy;
use ess_domain::entity::{Cardinality, RelationKind};
use ess_domain::name::QualifiedName;

use super::{
    arrange_first, arrange_owner, invoke, Arrangement, CommandRef, Determined, Invocation,
    RefusalCause, Setup,
};
use crate::scenario::{ActorRef, InstanceName, ScenarioStep, ScenarioValue};
use crate::witness::Distinction;

/// The step between the referenced rows' distinctions, and the base of every block of them.
///
/// The referenced row is arranged at a multiple of this number and its decoys at one below and one
/// above it. A decoy is one away from the referenced row, so under an enum of any number of
/// variants (the witness cycles variants by distinction) its value differs. And where the
/// referenced row is an owner the arrangement created at the plain witness (distinction 0), the
/// decoys are one away from a multiple of 27720 = lcm(1..=12), which every enum of up to twelve
/// variants reads as the variant next to the plain one. Past every further instance an
/// arrangement or a candidate search numbers (those stop at `MAX_CANDIDATES`), so the instance
/// names and witness values are the rows' own.
const RELATED_WITNESS: usize = 27_720;

/// How many further witnesses are tried for the input of a branch that changes a field the source
/// reads, looking for a value neither the referenced row nor a decoy held.
const CHANGE_CANDIDATES: usize = 12;

/// The settled key a referenced row's value is carried under.
///
/// Not a field name and not a fact path — it holds spaces — so every reader of the settled fields
/// that looks a field up by name, projects a view row or binds a filter's facts passes over it.
pub(super) fn key(via: &ResolvedRelatedVia, field: &str) -> String {
    let prefix = match via {
        ResolvedRelatedVia::Subject { .. } => "",
        ResolvedRelatedVia::Input { .. } => "input.",
    };
    format!("related {prefix}{} {field}", via.field())
}

/// One `via` a branch reads, the entity it names, and every field read through it.
struct Read<'a> {
    via: &'a ResolvedRelatedVia,
    entity: &'a EntityHandle,
    fields: Vec<&'a str>,
}

/// Every `via` the branch's payload and `sets:` read, in the order first written.
fn reads(outcome: &ResolvedOutcome) -> Vec<Read<'_>> {
    fn collect<'a>(field: &'a ResolvedPayloadField, out: &mut Vec<Read<'a>>) {
        match &field.value {
            ResolvedPayloadValue::RelatedField {
                via, entity, field, ..
            } => match out.iter_mut().find(|read| read.via == via) {
                Some(read) => {
                    if !read.fields.contains(&field.as_str()) {
                        read.fields.push(field);
                    }
                }
                None => out.push(Read {
                    via,
                    entity,
                    fields: vec![field],
                }),
            },
            ResolvedPayloadValue::Struct { fields } => {
                for leaf in fields {
                    collect(leaf, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for field in outcome
        .payload
        .iter()
        .flat_map(|payload| &payload.fields)
        .chain(&outcome.sets)
    {
        collect(field, &mut out);
    }
    out
}

/// `setup` with the rows every related source of `outcome` reads arranged ahead of it, the subject
/// or the input pointed at the referenced one, and the values that row holds settled under
/// [`key`]. A branch that reads no related row gets `setup` back unchanged.
///
/// Each referenced row sits between two decoys — one created before it and one after — so an
/// implementation reading the first row of the entity and one reading the last both read a decoy.
/// Where the input `via` already names the owner the arrangement created for an owned subject,
/// that owner is the referenced row and the decoys are arranged around it.
///
/// Where the document declares an `updates:` branch that writes a field the source reads from its
/// input, that branch is run on the referenced row last, after the subject exists and just before
/// the branch under test, and the value it wrote is the one asserted: the source reads the row as
/// it is at the branch, and an implementation that copied the value when the subject was created
/// publishes the old one.
pub(super) fn arrange(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    setup: Setup,
) -> Result<Setup, RefusalCause> {
    arrange_except(ir, outcome, actors, distinction, setup, None)
}

/// A row a `when_related:` guard reads: the input that names it and the entity it is a row of.
pub(super) type Guarded<'a> = (&'a str, &'a EntityHandle);

/// [`arrange`], but for the reads of the row `guarded` names: that row is the one the guard reads
/// too, which [`super::related_guard`] arranges so that it selects the branch, and whose values
/// [`settle`] carries once it exists (beyond10x/ess#270). Arranging it here as well would point the
/// input at a second row before the guard could point it at its own.
pub(super) fn arrange_except(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    setup: Setup,
    guarded: Option<Guarded<'_>>,
) -> Result<Setup, RefusalCause> {
    arrange_within(ir, outcome, actors, distinction, setup, guarded, &[])
}

/// Related sources inside an existing creation search retain its cycle guard (#360).
pub(super) fn arrange_within(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    setup: Setup,
    guarded: Option<Guarded<'_>>,
    arranging: &[&EntityHandle],
) -> Result<Setup, RefusalCause> {
    arrange_within_each(
        ir,
        outcome,
        actors,
        distinction,
        setup,
        guarded.as_slice(),
        arranging,
    )
}

/// [`arrange_except`] for a branch of a command whose guards read several related rows
/// (beyond10x/ess#283): the reads of every row in `guarded` are left to the guards' arrangement.
pub(super) fn arrange_except_each(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    setup: Setup,
    guarded: &[Guarded<'_>],
) -> Result<Setup, RefusalCause> {
    arrange_within_each(ir, outcome, actors, distinction, setup, guarded, &[])
}

fn arrange_within_each(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    mut setup: Setup,
    guarded: &[Guarded<'_>],
    arranging: &[&EntityHandle],
) -> Result<Setup, RefusalCause> {
    let reads = reads(outcome)
        .into_iter()
        .filter(|read| !guarded_by_any(ir, outcome, read, guarded));
    for (nth, read) in reads.enumerate() {
        check_chain(read.entity, arranging)?;
        let first = RELATED_WITNESS * (1 + distinction.get() + 8 * nth);
        let row = |at: usize| {
            arrange_first(
                ir,
                read.entity,
                std::slice::from_ref(&ir.entity(read.entity).lifecycle.initial),
                actors,
                Distinction::further(at),
                arranging,
            )
            .ok()
        };
        let (referenced, arranged_here) =
            if let Some(owner) = bound_owner(ir, outcome, actors, &setup, &read, arranging) {
                (owner, false)
            } else {
                let Some(referenced) = row(first) else {
                    continue;
                };
                if !point_at(ir, outcome, &mut setup, &read, &referenced.instance) {
                    // A read through the subject's own identity (beyond10x/ess#230) whose creating
                    // act generates that identity leaves nothing to point at the row: refused by
                    // name rather than run without the row it reads.
                    if through_identity(ir, outcome, &read) {
                        return Err(super::related_guard::unarranged());
                    }
                    continue;
                }
                (referenced, true)
            };
        // A singleton entity has no row beside the referenced one to read instead
        // (beyond10x/ess#287).
        let (before, after) = if super::singleton::is_singleton(ir, read.entity) {
            (None, None)
        } else {
            (row(first - 1), row(first + 1))
        };
        let decoys: Vec<&BTreeMap<String, Determined>> = before
            .iter()
            .chain(after.iter())
            .map(|decoy| &decoy.settled)
            .collect();
        let change = changed(ir, &read, &referenced, actors, first, &decoys);
        let identity = &ir.entity(read.entity).identity;
        for field in &read.fields {
            let held = if identity.name == *field {
                Some(Determined {
                    value: ScenarioValue::instance(referenced.instance.clone()),
                    type_ref: identity.type_ref.clone(),
                })
            } else {
                match &change {
                    Some((writes, invocation)) if writes.iter().any(|written| written == field) => {
                        invocation.settled.get(*field).cloned()
                    }
                    _ => referenced.settled.get(*field).cloned(),
                }
            };
            match held {
                Some(held) => {
                    setup.settled.insert(key(read.via, field), held);
                }
                None => {
                    setup.settled.remove(&key(read.via, field));
                }
            }
        }
        let mut steps = Vec::new();
        if let Some(decoy) = before {
            steps.extend(decoy.steps);
            setup.source.extend(decoy.source);
        }
        let mut after_steps = Vec::new();
        if let Some(decoy) = after {
            after_steps = decoy.steps;
            setup.source.extend(decoy.source);
        }
        if arranged_here {
            steps.extend(referenced.steps);
            setup.source.extend(referenced.source);
            steps.extend(after_steps);
        } else {
            // The owner's steps are already the arrangement's: the later decoy goes right after
            // the step that captures it, before anything that names it.
            let at = setup
                .steps
                .iter()
                .position(|step| {
                    matches!(step, ScenarioStep::CaptureInstance { instance, .. }
                        if *instance == referenced.instance)
                })
                .map_or(0, |index| index + 1);
            setup.steps.splice(at..at, after_steps);
        }
        steps.append(&mut setup.steps);
        if let Some((_, invocation)) = change {
            steps.extend(invocation.steps);
            setup.source.extend(invocation.source);
        }
        setup.steps = steps;
    }
    Ok(setup)
}

fn check_chain(entity: &EntityHandle, arranging: &[&EntityHandle]) -> Result<(), RefusalCause> {
    if arranging.contains(&entity) {
        return Err(RefusalCause::GuardUnsatisfiable {
            predicate: format!("cyclic related source arrangement through `{entity}`"),
            tried: arranging.len(),
        });
    }
    Ok(())
}

/// Whether `read` reads a row any of `guarded` names ([`is_guarded`]).
fn guarded_by_any(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    read: &Read<'_>,
    guarded: &[Guarded<'_>],
) -> bool {
    guarded
        .iter()
        .any(|guarded| is_guarded(ir, outcome, read, Some(*guarded)))
}

/// Whether `read` reads the row `guarded` names: through the same input, of the same entity.
fn is_guarded(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    read: &Read<'_>,
    guarded: Option<Guarded<'_>>,
) -> bool {
    guarded.is_some_and(|(field, entity)| {
        read.entity == entity && input_read(ir, outcome, read.via) == Some(field)
    })
}

/// The stored fields — not the identity — `outcome` reads through the input `guarded` names
/// ([`arrange_except`]): the values a row beside the guarded one should hold otherwise, so that an
/// implementation copying from another row publishes another value.
pub(super) fn guarded_fields<'a>(
    ir: &EssIr,
    outcome: &'a ResolvedOutcome,
    guarded: Guarded<'_>,
) -> Vec<&'a str> {
    let mut out = Vec::new();
    for read in reads(outcome)
        .iter()
        .filter(|read| is_guarded(ir, outcome, read, Some(guarded)))
    {
        let identity = &ir.entity(read.entity).identity.name;
        for field in &read.fields {
            if identity != field && !out.contains(field) {
                out.push(*field);
            }
        }
    }
    out
}

/// Carries, under [`key`], what `row` — the row a `when_related:` guard reads, arranged to select
/// the branch — holds for every field `outcome` reads of it through the same input
/// ([`arrange_except`], beyond10x/ess#270): its identity, or the value it settled. A field the row
/// left undetermined is carried as nothing, and stays covered by the payload shape alone.
pub(super) fn settle(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &mut Setup,
    guarded: Guarded<'_>,
    row: &Arrangement,
) {
    for read in reads(outcome)
        .iter()
        .filter(|read| is_guarded(ir, outcome, read, Some(guarded)))
    {
        let identity = &ir.entity(read.entity).identity;
        for field in &read.fields {
            let held = if identity.name == *field {
                Some(Determined {
                    value: ScenarioValue::instance(row.instance.clone()),
                    type_ref: identity.type_ref.clone(),
                })
            } else {
                row.settled.get(*field).cloned()
            };
            match held {
                Some(held) => {
                    setup.settled.insert(key(read.via, field), held);
                }
                None => {
                    setup.settled.remove(&key(read.via, field));
                }
            }
        }
    }
}

/// Whether `read` follows the subject's own identity to a row of another entity, through a
/// relation the identity carries (beyond10x/ess#230).
fn through_identity(ir: &EssIr, outcome: &ResolvedOutcome, read: &Read<'_>) -> bool {
    let ResolvedRelatedVia::Subject { field, .. } = read.via else {
        return false;
    };
    outcome.subject.as_ref().is_some_and(|subject| {
        ir.entity(&subject.entity).identity.name == *field
            && subject.entity != *read.entity
            && identity_reference(ir, &subject.entity)
    })
}

/// A run, on the referenced row, of the first `updates:` branch of its entity that writes a field
/// the source reads from its input unchanged, and the fields it writes that the source reads.
///
/// Its input is the first of [`CHANGE_CANDIDATES`] further witnesses that writes a value neither
/// the referenced row nor a decoy held in every such field, else the first that at least moves off
/// the referenced row's value. A branch chosen by the row's stored fields, or decided by an
/// injected external outcome, is not used: its input is not this function's to choose.
fn changed(
    ir: &EssIr,
    read: &Read<'_>,
    referenced: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    first: usize,
    decoys: &[&BTreeMap<String, Determined>],
) -> Option<(Vec<String>, Invocation)> {
    let all = ir.drivers();
    let driver = all.get(read.entity)?.iter().find(|driver| {
        matches!(driver.effect, ResolvedEffect::Updates)
            && driver.outcome.test_strategy != TestStrategy::InjectFault
            && !super::subject_fact::uses(driver.command)
            && writes(driver.outcome, read).next().is_some()
    })?;
    let written: Vec<String> = writes(driver.outcome, read).map(str::to_owned).collect();
    let value = |settled: &BTreeMap<String, Determined>, field: &str| {
        settled.get(field).map(|held| held.value.clone())
    };
    let mut moved = None;
    for nth in 2..2 + CHANGE_CANDIDATES {
        let Ok(invocation) = invoke(
            ir,
            driver,
            Some(&referenced.instance),
            Some(&referenced.state),
            actors,
            Distinction::further(first + nth),
            &BTreeMap::new(),
            &[read.entity],
        ) else {
            continue;
        };
        let new = |field: &String| value(&invocation.settled, field);
        let off_referenced = written
            .iter()
            .all(|field| new(field).is_some() && new(field) != value(&referenced.settled, field));
        if !off_referenced {
            continue;
        }
        let off_decoys = written
            .iter()
            .all(|field| decoys.iter().all(|decoy| value(decoy, field) != new(field)));
        if off_decoys {
            return Some((written, invocation));
        }
        moved.get_or_insert(invocation);
    }
    moved.map(|invocation| (written, invocation))
}

/// The fields a branch writes from its input unchanged that `read` reads.
fn writes<'a>(
    outcome: &'a ResolvedOutcome,
    read: &'a Read<'_>,
) -> impl Iterator<Item = &'a str> + 'a {
    outcome.sets.iter().filter_map(move |set| {
        let copied = matches!(set.value, ResolvedPayloadValue::InputField { .. })
            && set.conversion.is_none();
        (copied && read.fields.contains(&set.target.as_str())).then_some(set.target.as_str())
    })
}

/// The input a related source reads: `input.<field>`, or, on a `creates:` branch, the subject
/// field the branch sets from its input unchanged (there is no row before the outcome, so the
/// field holds that input) — or the subject's identity, where the branch fills it from its input
/// unchanged ([`identity_input`]): an identity may carry the relation (beyond10x/ess#230).
fn input_read<'a>(
    ir: &EssIr,
    outcome: &'a ResolvedOutcome,
    via: &'a ResolvedRelatedVia,
) -> Option<&'a str> {
    match via {
        ResolvedRelatedVia::Input { field, .. } => Some(field),
        ResolvedRelatedVia::Subject { field, .. } => {
            let subject = outcome
                .subject
                .as_ref()
                .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
            set_from_input(outcome, field).or_else(|| {
                (ir.entity(&subject.entity).identity.name == *field
                    && identity_reference(ir, &subject.entity))
                .then(|| identity_input(outcome))
                .flatten()
            })
        }
    }
}

/// Whether `entity`'s identity carries a `references`, `cardinality: one` relation to another
/// entity: the only case in which the identity is a relation carrier (beyond10x/ess#230), as
/// `ess_domain::command::related_value::identity_reference` admits it.
fn identity_reference(ir: &EssIr, entity: &EntityHandle) -> bool {
    let resolved = ir.entity(entity);
    resolved.relations.iter().any(|relation| {
        relation.kind == RelationKind::References
            && relation.cardinality == Cardinality::One
            && relation.via == resolved.identity.name
            && relation.target != *entity
    })
}

/// The input a branch's `sets:` writes into the subject field `field` unchanged.
pub(super) fn set_from_input<'a>(outcome: &'a ResolvedOutcome, field: &str) -> Option<&'a str> {
    outcome.sets.iter().find_map(|set| match &set.value {
        ResolvedPayloadValue::InputField { field: input, .. }
            if set.target == field && set.conversion.is_none() =>
        {
            Some(input.as_str())
        }
        _ => None,
    })
}

/// The input a branch names its subject's instance by, unchanged: the supplied input field, or on
/// `creates:` the input the event publishing the new identity fills that field from.
fn identity_input(outcome: &ResolvedOutcome) -> Option<&str> {
    match &outcome.subject.as_ref()?.instance {
        ResolvedInstance::Supplied { field } => Some(&field.name),
        ResolvedInstance::Observed { event, field } => outcome
            .payload
            .iter()
            .find(|payload| payload.event == *event)?
            .fields
            .iter()
            .find(|filled| filled.target == field.name && filled.conversion.is_none())
            .and_then(|filled| match &filled.value {
                ResolvedPayloadValue::InputField { field, .. } => Some(field.as_str()),
                _ => None,
            }),
    }
}

/// The owner the arrangement already created for a `creates:` subject and bound to the input
/// `via` reads, where `via` names that owner's entity: [`super::arrange_owner`] asked again with
/// the arguments [`super::prepare_subject`] gave it, which answers with the same rows, now with
/// what they settled.
fn bound_owner(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &Setup,
    read: &Read<'_>,
    arranging: &[&EntityHandle],
) -> Option<Arrangement> {
    let field = input_read(ir, outcome, read.via)?;
    let bound = setup.bound.get(field)?;
    let subject = outcome.subject.as_ref()?;
    if subject.effect != ResolvedEffect::Creates {
        return None;
    }
    let (carried, owner) = arrange_owner(
        ir,
        outcome,
        &subject.entity,
        actors,
        Distinction::PLAIN,
        arranging,
    )?;
    let names = ir.owner_of(&subject.entity)?.owner == *read.entity;
    (names && carried == field && owner.instance == *bound).then_some(owner)
}

/// Points `via` at `referenced`: the input the branch reads it from, or the field the existing
/// subject stores.
///
/// An input that names the existing subject cannot be rebound — it already names an arranged row,
/// the subject — but where the subject's identity is what `via` follows to another entity, the
/// identity the subject was created with is pointed instead (beyond10x/ess#230).
fn point_at(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &mut Setup,
    read: &Read<'_>,
    referenced: &InstanceName,
) -> bool {
    let subject = outcome.subject.as_ref();
    if let Some(field) = input_read(ir, outcome, read.via) {
        let names_subject = subject.is_some_and(|subject| {
            matches!(&subject.instance, ResolvedInstance::Supplied { field: named }
                if named.name == field)
        });
        if names_subject {
            return subject.is_some_and(|subject| {
                subject.entity != *read.entity
                    && identity_reference(ir, &subject.entity)
                    && stored(
                        ir,
                        setup,
                        &ir.entity(&subject.entity).identity.name,
                        referenced,
                    )
            });
        }
        if setup.bound.contains_key(field) {
            return false;
        }
        setup.bound.insert(field.to_owned(), referenced.clone());
        return true;
    }
    match read.via {
        ResolvedRelatedVia::Subject { field, .. } => {
            let identity =
                subject.is_some_and(|subject| ir.entity(&subject.entity).identity.name == *field);
            // An identity pointed at a row of its own entity would be a second row with the same
            // key, not a link, and an identity that carries no relation to another entity links
            // nothing: both left to the payload shape.
            if identity
                && subject.is_some_and(|subject| {
                    subject.entity == *read.entity || !identity_reference(ir, &subject.entity)
                })
            {
                return false;
            }
            stored(ir, setup, field, referenced)
        }
        ResolvedRelatedVia::Input { .. } => false,
    }
}

/// Rewrites the act that created the subject so the field `via` it stores names `referenced`.
///
/// The link is the creating branch's `sets:`, exactly as [`super::arrange_owner`] reads it:
/// `via: input.<field>` with no conversion — or, where `via` is the subject's identity, the input
/// the creating branch fills the new identity from ([`identity_input`]). The rewrite is made only
/// where the subject still holds what that act sent — no later act of the arrangement wrote `via`
/// — so the row the branch under test reads is the row the scenario arranged. An identity is never
/// rewritten by a later act, so it is checked only where the arrangement settled it.
fn stored(ir: &EssIr, setup: &mut Setup, via: &str, referenced: &InstanceName) -> bool {
    let Some(subject) = setup.instance.clone() else {
        return false;
    };
    let Some(captured) = setup.steps.iter().position(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance, .. } if *instance == subject)
    }) else {
        return false;
    };
    let Some(created) = setup.steps[..captured]
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
    else {
        return false;
    };
    let Some(ScenarioStep::ExpectOutcome { outcome: taken }) = setup.steps.get(created + 1) else {
        return false;
    };
    let creating = ir
        .commands()
        .values()
        .filter(|command| CommandRef::new(command.name.clone()) == taken.command)
        .flat_map(|command| &command.outcomes)
        .find(|outcome| outcome.name == taken.outcome);
    let Some(creating) = creating else {
        return false;
    };
    let identity = creating.subject.as_ref().is_some_and(|subject| {
        ir.entity(&subject.entity).identity.name == via && identity_reference(ir, &subject.entity)
    });
    let input = set_from_input(creating, via)
        .or_else(|| identity.then(|| identity_input(creating)).flatten());
    let Some(input) = input.map(str::to_owned) else {
        return false;
    };
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &mut setup.steps[created] else {
        return false;
    };
    let Some(before) = sent.get(&input).cloned() else {
        return false;
    };
    let pointed = ScenarioValue::instance(referenced.clone());
    match setup.settled.get_mut(via) {
        Some(held) if held.value == before => held.value = pointed.clone(),
        None if identity => {}
        _ => return false,
    }
    sent.insert(input, pointed);
    true
}
