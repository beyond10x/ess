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

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedEffect, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedHop, ResolvedRelatedVia,
    ResolvedTypeRef,
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
use ess_primitives::node::Node;

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
///
/// A chained read (ess/22, beyond10x/ess#285) adds the field of every further reference it follows,
/// so a one-hop read keeps the key it always had.
pub(super) fn key(via: &ResolvedRelatedVia, through: &[ResolvedRelatedHop], field: &str) -> String {
    let prefix = match via {
        ResolvedRelatedVia::Subject { .. } => "",
        ResolvedRelatedVia::Input { .. } => "input.",
    };
    let mut key = format!("related {prefix}{}", via.field());
    for hop in through {
        key.push(' ');
        key.push_str(&hop.field);
    }
    format!("{key} {field}")
}

/// One `via` a branch reads — with the further references it follows, where it is chained — the
/// entity the last one names, and every field read through it.
struct Read<'a> {
    via: &'a ResolvedRelatedVia,
    through: &'a [ResolvedRelatedHop],
    entity: &'a EntityHandle,
    fields: Vec<&'a str>,
}

impl Read<'_> {
    /// The entity `via` itself names: the first hop's, where the read is chained.
    fn named(&self) -> &EntityHandle {
        self.through.first().map_or(self.entity, |hop| &hop.entity)
    }

    /// The settled key of `field`.
    fn key(&self, field: &str) -> String {
        key(self.via, self.through, field)
    }
}

/// Every `via` the branch's payload and `sets:` read, in the order first written.
fn reads(outcome: &ResolvedOutcome) -> Vec<Read<'_>> {
    fn collect<'a>(field: &'a ResolvedPayloadField, out: &mut Vec<Read<'a>>) {
        match &field.value {
            ResolvedPayloadValue::RelatedField {
                via,
                through,
                entity,
                field,
                ..
            } => match out
                .iter_mut()
                .find(|read| read.via == via && read.through == through.as_slice())
            {
                Some(read) => {
                    if !read.fields.contains(&field.as_str()) {
                        read.fields.push(field);
                    }
                }
                None => out.push(Read {
                    via,
                    through,
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
        None,
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
    arrange_within_each(ir, outcome, actors, distinction, setup, guarded, &[], None)
}

/// One reference a related read follows that a witness can leave absent (ess/22,
/// beyond10x/ess#285): the read, by its position among the outcome's reads, and the reference —
/// `0` for its `via`, `1` for the next reference a chained read follows.
pub(super) type Point = (usize, usize);

/// An absent-reference witness under construction (ess/22, beyond10x/ess#285): the one reference
/// it leaves absent, the inputs the invocation leaves out, and whether it was left absent.
pub(super) struct Absence<'c> {
    /// The command whose input is left out.
    command: &'c ResolvedCommand,
    /// The reference this run leaves absent; every other is arranged present.
    point: Point,
    /// The inputs the invocation leaves out.
    omitted: BTreeSet<String>,
    /// The `via`s left absent: every read through one of them reads nothing.
    vias: Vec<ResolvedRelatedVia>,
    /// Whether the reference was left absent.
    done: bool,
}

/// Every reference the related reads of `outcome` follow that a witness can leave absent (ess/22,
/// beyond10x/ess#285), each once, in the order the reads are written: a `via` that is
/// `Optional<…>` and filled from an Optional input the branch can be sent without — or, on an
/// existing subject, stored by a branch creating it from an Optional input — and a chained read's
/// `Optional` next reference that a branch creating that row fills from an Optional input. Each is
/// witnessed by a run of its own, with every other reference present ([`arrange_absent`]).
pub(super) fn absence_points(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<Point> {
    let mut out = Vec::new();
    let mut vias: Vec<&ResolvedRelatedVia> = Vec::new();
    for (nth, read) in reads(outcome).iter().enumerate() {
        if !vias.contains(&read.via) && via_may_be_left_absent(ir, command, outcome, read.via) {
            vias.push(read.via);
            out.push((nth, 0));
        }
        if let [hop] = read.through {
            if hop.type_ref.is_optional() && created_absent(ir, &hop.entity, &hop.field) {
                out.push((nth, 1));
            }
        }
    }
    out
}

/// The reference `point` names, as a refusal names it: `<entity>.<field>`, or `input.<field>`.
pub(super) fn point_reference(ir: &EssIr, outcome: &ResolvedOutcome, point: usize) -> String {
    let reads = reads(outcome);
    let Some(read) = reads.get(point) else {
        return format!("reference {point}");
    };
    via_reference(ir, outcome, read.via)
}

/// `via` as a refusal names it.
fn via_reference(ir: &EssIr, outcome: &ResolvedOutcome, via: &ResolvedRelatedVia) -> String {
    match (via, outcome.subject.as_ref()) {
        (ResolvedRelatedVia::Subject { field, .. }, Some(subject)) => {
            format!("{}.{field}", ir.entity(&subject.entity).name)
        }
        _ => via.to_string(),
    }
}

/// Every reference the related reads of `outcome` follow that may be absent where the branch runs
/// and that no run can leave absent, with why (ess/22, beyond10x/ess#285): an `Optional` stored
/// reference no branch creating its row fills from an Optional input, or an Optional input that
/// names the branch's own subject. A reference filled from a required input is never absent where
/// the branch runs, and is not one of them.
pub(super) fn unwitnessable(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut vias: Vec<&ResolvedRelatedVia> = Vec::new();
    for read in &reads(outcome) {
        if read.via.type_ref().is_optional()
            && !vias.contains(&read.via)
            && !via_may_be_left_absent(ir, command, outcome, read.via)
        {
            vias.push(read.via);
            let reason = match (input_read(ir, outcome, read.via), read.via) {
                (Some(input), _) if names_subject(outcome, input) => {
                    Some("it names the branch's own subject".to_owned())
                }
                (None, ResolvedRelatedVia::Subject { field, .. }) => {
                    outcome.subject.as_ref().map(|subject| {
                        format!(
                            "no branch creating `{}` fills `{field}` from an Optional input",
                            ir.entity(&subject.entity).name
                        )
                    })
                }
                _ => None,
            };
            if let Some(reason) = reason {
                out.push((via_reference(ir, outcome, read.via), reason));
            }
        }
        if let [hop] = read.through {
            if hop.type_ref.is_optional() && !created_absent(ir, &hop.entity, &hop.field) {
                let entity = &ir.entity(&hop.entity).name;
                out.push((
                    format!("{entity}.{}", hop.field),
                    format!(
                        "no branch creating `{entity}` fills `{}` from an Optional input",
                        hop.field
                    ),
                ));
            }
        }
    }
    out
}

/// Whether `via` may be left absent by the run that witnesses it ([`absence_points`]).
fn via_may_be_left_absent(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    via: &ResolvedRelatedVia,
) -> bool {
    if !via.type_ref().is_optional() {
        return false;
    }
    if let Some(input) = input_read(ir, outcome, via) {
        return !names_subject(outcome, input)
            && command
                .input
                .iter()
                .any(|field| field.name == input && field.type_ref.is_optional());
    }
    match via {
        ResolvedRelatedVia::Subject { field, .. } => {
            outcome.subject.as_ref().is_some_and(|subject| {
                subject.effect != ResolvedEffect::Creates
                    && created_absent(ir, &subject.entity, field)
            })
        }
        ResolvedRelatedVia::Input { .. } => false,
    }
}

/// Whether some branch creating a row of `entity` fills `field` from an Optional input unchanged,
/// so a row can be created with `field` absent.
fn created_absent(ir: &EssIr, entity: &EntityHandle, field: &str) -> bool {
    ir.drivers().get(entity).is_some_and(|drivers| {
        drivers.iter().any(|driver| {
            matches!(driver.effect, ResolvedEffect::Creates)
                && set_from_input(driver.outcome, field).is_some_and(|input| {
                    driver
                        .command
                        .input
                        .iter()
                        .any(|declared| declared.name == input && declared.type_ref.is_optional())
                })
        })
    })
}

/// Whether `input` names the branch's own subject.
fn names_subject(outcome: &ResolvedOutcome, input: &str) -> bool {
    outcome.subject.as_ref().is_some_and(|subject| {
        matches!(&subject.instance, ResolvedInstance::Supplied { field } if field.name == input)
    })
}

/// [`arrange`] for the witness of the absent reference `point` names (ess/22, beyond10x/ess#285):
/// that reference is left absent — the Optional input `via` reads left out, the existing subject
/// created without the reference it stores, or the row `via` names created without the Optional
/// reference a chained read follows next — and every value read through it settled as absent;
/// every other read is arranged as [`arrange`] does. `None` where it cannot be left absent, with
/// the inputs the invocation leaves out otherwise.
pub(super) fn arrange_absent(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (distinction, point): (Distinction, Point),
    setup: Setup,
) -> Result<Option<(Setup, BTreeSet<String>)>, RefusalCause> {
    let mut absence = Absence {
        command,
        point,
        omitted: BTreeSet::new(),
        vias: Vec::new(),
        done: false,
    };
    let setup = arrange_within_each(
        ir,
        outcome,
        actors,
        distinction,
        setup,
        &[],
        &[],
        Some(&mut absence),
    )?;
    Ok(absence.done.then_some((setup, absence.omitted)))
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn arrange_within_each(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    mut setup: Setup,
    guarded: &[Guarded<'_>],
    arranging: &[&EntityHandle],
    mut absence: Option<&mut Absence<'_>>,
) -> Result<Setup, RefusalCause> {
    let reads = reads(outcome)
        .into_iter()
        .filter(|read| !guarded_by_any(ir, outcome, read, guarded));
    // The row each `via` was pointed at here: a later read through the same `via` reads that row,
    // in one hop or on along its own chain (ess/22, beyond10x/ess#285).
    let mut named: Vec<(&ResolvedRelatedVia, Arrangement)> = Vec::new();
    for (nth, read) in reads.enumerate() {
        check_chain(read.entity, arranging)?;
        check_chain(read.named(), arranging)?;
        let first = RELATED_WITNESS * (1 + distinction.get() + 8 * nth);
        if let Some(absence) = absence.as_deref_mut() {
            if absence.vias.contains(read.via) {
                settle_absent(ir, &mut setup, &read);
                continue;
            }
            if !absence.done
                && absence.point == (nth, 0)
                && leave_via_absent(ir, outcome, &mut setup, &read, absence)
            {
                continue;
            }
        }
        let absent_hop = absence
            .as_deref()
            .is_some_and(|absence| !absence.done && absence.point == (nth, 1));
        if let Some((_, row)) = named.iter_mut().find(|(via, _)| *via == read.via) {
            let left = shared(
                ir, actors, &mut setup, &read, row, first, absent_hop, arranging,
            )?;
            if let Some(absence) = absence.as_deref_mut().filter(|_| left) {
                absence.done = true;
            }
            continue;
        }
        if !read.through.is_empty() {
            let chain = Chain {
                read: &read,
                first,
                absent_hop,
                arranging,
            };
            if let Some((middle, left)) = chained(ir, outcome, actors, &mut setup, &chain) {
                if let Some(absence) = absence.as_deref_mut().filter(|_| left) {
                    absence.done = true;
                }
                named.push((read.via, middle));
            }
            continue;
        }
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
        named.push((
            read.via,
            Arrangement {
                steps: Vec::new(),
                ..referenced.clone()
            },
        ));
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
        settle_read(ir, &mut setup, &read, &referenced, change.as_ref());
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
        read.through.is_empty()
            && read.entity == entity
            && input_read(ir, outcome, read.via) == Some(field)
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
                    setup.settled.insert(read.key(field), held);
                }
                None => {
                    setup.settled.remove(&read.key(field));
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
            && subject.entity != *read.named()
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
    // A path (ess/22, A4) reads inside a struct input, where no arranged row's identity is sent:
    // the related row is not arranged through one.
    let read = match via {
        ResolvedRelatedVia::Input { field, .. } => Some(field.as_str()),
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
    };
    read.filter(|read| !ess_domain::command::input_path::is_path(read))
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
    let names = ir.owner_of(&subject.entity)?.owner == *read.named();
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
                subject.entity != *read.named()
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
                    subject.entity == *read.named() || !identity_reference(ir, &subject.entity)
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

/// Carries, under the read's keys, what `referenced` holds for every field `read` reads: its
/// identity, the value the `updates:` branch `change` wrote, or the value it settled. A field the
/// row left undetermined is carried as nothing, and stays covered by the payload shape alone.
fn settle_read(
    ir: &EssIr,
    setup: &mut Setup,
    read: &Read<'_>,
    referenced: &Arrangement,
    change: Option<&(Vec<String>, Invocation)>,
) {
    let identity = &ir.entity(read.entity).identity;
    for field in &read.fields {
        let held = if identity.name == *field {
            Some(Determined {
                value: ScenarioValue::instance(referenced.instance.clone()),
                type_ref: identity.type_ref.clone(),
            })
        } else {
            match change {
                Some((writes, invocation)) if writes.iter().any(|written| written == field) => {
                    invocation.settled.get(*field).cloned()
                }
                _ => referenced.settled.get(*field).cloned(),
            }
        };
        match held {
            Some(held) => {
                setup.settled.insert(read.key(field), held);
            }
            None => {
                setup.settled.remove(&read.key(field));
            }
        }
    }
}

/// Carries every field `read` reads as absent (ess/22, beyond10x/ess#285): a reference it follows
/// was left absent, so the value is.
fn settle_absent(ir: &EssIr, setup: &mut Setup, read: &Read<'_>) {
    let entity = ir.entity(read.entity);
    for field in &read.fields {
        let type_ref = std::iter::once(&entity.identity)
            .chain(&entity.fields)
            .find(|held| held.name == *field)
            .map_or_else(
                || entity.identity.type_ref.clone(),
                |held| held.type_ref.clone(),
            );
        setup.settled.insert(
            read.key(field),
            Determined {
                value: ScenarioValue::literal(Node::Null),
                type_ref: wrapped(type_ref),
            },
        );
    }
}

/// `Optional<…>` of `type_ref`, unless it already is.
fn wrapped(type_ref: ResolvedTypeRef) -> ResolvedTypeRef {
    if type_ref.is_optional() {
        type_ref
    } else {
        ResolvedTypeRef::Optional {
            of: Box::new(type_ref),
        }
    }
}

/// A chained read under arrangement ([`chained`]): the read, the first distinction of its block,
/// whether this run leaves its next reference absent, and the entities already being arranged.
struct Chain<'r, 'a> {
    read: &'r Read<'a>,
    first: usize,
    absent_hop: bool,
    arranging: &'r [&'r EntityHandle],
}

/// A chained read (ess/22, beyond10x/ess#285): the row of the last entity it names, between two
/// decoys as a one-hop read's is; the row `via` names, created with its next reference pointed at
/// that row, between two decoys of its own pointed crosswise at the last entity's decoys; and
/// `via` pointed at that middle row.
///
/// Every row of the chain is read as it is at the branch: where the document declares an
/// `updates:` branch that changes the middle row's next reference, that branch is run on it last,
/// pointing it at a further row of the last entity ([`retargeted`]); and where it declares one
/// that changes the field read on the last row, that branch is run on the row then named
/// ([`changed`]). The value that row holds is settled under the read's key.
///
/// So an implementation reading another row of either entity, following another reference, or
/// reading either reference as it was first written publishes another value. Where the run leaves
/// the next reference absent, the middle row is created without it and the value settled absent.
/// Where the last row cannot be arranged, an Optional next reference is left absent; where nothing
/// can be built, the read is left to the payload shape, as an unarranged one-hop read is. Returns
/// the middle row, and whether the run's absent reference was left absent here.
fn chained(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    chain: &Chain<'_, '_>,
) -> Option<(Arrangement, bool)> {
    let (read, first, arranging) = (chain.read, chain.first, chain.arranging);
    let [hop] = read.through else {
        return None;
    };
    let last = |at: usize| row_at(ir, read.entity, actors, at, arranging);
    let middle = |at: usize, to: Option<&InstanceName>| {
        let mut row = row_at(ir, &hop.entity, actors, at, arranging)?;
        rewrite_reference(ir, &mut row, &hop.field, to).then_some(row)
    };
    // The next reference left absent: by this run, or because no last row can be arranged.
    let absent = |setup: &mut Setup| {
        let named = middle(first + 3, None)?;
        if !point_at(ir, outcome, setup, read, &named.instance) {
            return None;
        }
        let mut steps = named.steps.clone();
        steps.append(&mut setup.steps);
        setup.steps = steps;
        setup.source.extend(named.source.iter().cloned());
        settle_absent(ir, setup, read);
        Some(Arrangement {
            steps: Vec::new(),
            ..named
        })
    };
    if chain.absent_hop {
        if let Some(named) = absent(setup) {
            return Some((named, true));
        }
    }
    let Some(referenced) = last(first) else {
        return absent(setup).map(|named| (named, false));
    };
    let (before, after) = if super::singleton::is_singleton(ir, read.entity) {
        (None, None)
    } else {
        (last(first - 1), last(first + 1))
    };
    let mut named = middle(first + 3, Some(&referenced.instance))?;
    let beside = !super::singleton::is_singleton(ir, &hop.entity);
    let middle_before = after
        .as_ref()
        .filter(|_| beside)
        .and_then(|to| middle(first + 2, Some(&to.instance)));
    let middle_after = before
        .as_ref()
        .filter(|_| beside)
        .and_then(|to| middle(first + 4, Some(&to.instance)));
    if !point_at(ir, outcome, setup, read, &named.instance) {
        return None;
    }
    let retarget = retargeted(ir, read, hop, &named, actors, first, arranging);
    let read_from = retarget.as_ref().map_or(&referenced, |(row, _)| row);
    let decoys: Vec<&BTreeMap<String, Determined>> = before
        .iter()
        .chain(after.iter())
        .chain(retarget.as_ref().map(|_| &referenced))
        .map(|decoy| &decoy.settled)
        .collect();
    let change = changed(ir, read, read_from, actors, first, &decoys);
    settle_read(ir, setup, read, read_from, change.as_ref());
    if let Some((_, invocation)) = &retarget {
        named.settled.extend(invocation.settled.clone());
    }
    let record = Arrangement {
        steps: Vec::new(),
        ..named.clone()
    };
    let (moved, retarget) = match retarget {
        Some((row, invocation)) => (Some(row), Some(invocation)),
        None => (None, None),
    };
    let rows = [
        before,
        Some(referenced),
        after,
        moved,
        middle_before,
        Some(named),
        middle_after,
    ];
    surround(
        setup,
        rows.into_iter().flatten(),
        [retarget, change.map(|(_, run)| run)],
    );
    Some((record, false))
}

/// A row of `entity` where its lifecycle starts, arranged at the `at`th further distinction.
fn row_at(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
    arranging: &[&EntityHandle],
) -> Option<Arrangement> {
    let initial = std::slice::from_ref(&ir.entity(entity).lifecycle.initial);
    arrange_first(
        ir,
        entity,
        initial,
        actors,
        Distinction::further(at),
        arranging,
    )
    .ok()
}

/// `setup` with the steps of `rows` before its own and the steps of `invocations` after them.
fn surround(
    setup: &mut Setup,
    rows: impl IntoIterator<Item = Arrangement>,
    invocations: impl IntoIterator<Item = Option<Invocation>>,
) {
    let mut steps = Vec::new();
    for row in rows {
        steps.extend(row.steps);
        setup.source.extend(row.source);
    }
    steps.append(&mut setup.steps);
    for invocation in invocations.into_iter().flatten() {
        steps.extend(invocation.steps);
        setup.source.extend(invocation.source);
    }
    setup.steps = steps;
}

/// A further row of the last entity a chained read names, and a run, on the middle row `named`, of
/// the first `updates:` branch of its entity that writes the read's next reference from its input
/// unchanged, pointing it at that row (ess/22, beyond10x/ess#285): the reference is read as it is
/// at the branch, so an implementation following it as first written reads another row. A branch
/// chosen by the row's stored fields, or decided by an injected external outcome, is not used.
fn retargeted(
    ir: &EssIr,
    read: &Read<'_>,
    hop: &ResolvedRelatedHop,
    named: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    first: usize,
    arranging: &[&EntityHandle],
) -> Option<(Arrangement, Invocation)> {
    let all = ir.drivers();
    let (driver, input) = all.get(&hop.entity)?.iter().find_map(|driver| {
        let usable = matches!(driver.effect, ResolvedEffect::Updates)
            && driver.outcome.test_strategy != TestStrategy::InjectFault
            && !super::subject_fact::uses(driver.command);
        usable
            .then(|| set_from_input(driver.outcome, &hop.field))
            .flatten()
            .map(|input| (driver, input))
    })?;
    let moved = arrange_first(
        ir,
        read.entity,
        std::slice::from_ref(&ir.entity(read.entity).lifecycle.initial),
        actors,
        Distinction::further(first + 5),
        arranging,
    )
    .ok()?;
    let invocation = invoke(
        ir,
        driver,
        Some(&named.instance),
        Some(&named.state),
        actors,
        Distinction::further(first + 6),
        &BTreeMap::from([(input.to_owned(), moved.instance.clone())]),
        &[&hop.entity],
    )
    .ok()?;
    let pointed = invocation
        .settled
        .get(&hop.field)
        .is_some_and(|held| held.value == ScenarioValue::instance(moved.instance.clone()));
    pointed.then_some((moved, invocation))
}

/// A read through a `via` an earlier read already pointed at `row` (ess/22, beyond10x/ess#285):
/// a one-hop read reads that row; a chained one points the row's next reference at a row of the
/// last entity it names, arranged between two decoys — or, where this run leaves that reference
/// absent, or no such row can be arranged and the reference may be absent, leaves it absent. A
/// required reference no row can be arranged for is refused by name. Returns whether the run's
/// absent reference was left absent here.
#[allow(clippy::too_many_arguments)]
fn shared(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    setup: &mut Setup,
    read: &Read<'_>,
    row: &mut Arrangement,
    first: usize,
    absent_hop: bool,
    arranging: &[&EntityHandle],
) -> Result<bool, RefusalCause> {
    let [hop] = read.through else {
        settle_read(ir, setup, read, row, None);
        return Ok(false);
    };
    let instance = row.instance.clone();
    let mut repoint = |setup: &mut Setup, to: Option<&InstanceName>| {
        rewrite_in(
            ir,
            &mut setup.steps,
            &mut row.settled,
            &instance,
            &hop.field,
            to,
        )
    };
    if absent_hop && repoint(setup, None) {
        settle_absent(ir, setup, read);
        return Ok(true);
    }
    let last = |at: usize| {
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
    if let Some(referenced) = last(first) {
        if repoint(setup, Some(&referenced.instance)) {
            let (before, after) = if super::singleton::is_singleton(ir, read.entity) {
                (None, None)
            } else {
                (last(first - 1), last(first + 1))
            };
            let decoys: Vec<&BTreeMap<String, Determined>> = before
                .iter()
                .chain(after.iter())
                .map(|decoy| &decoy.settled)
                .collect();
            let change = changed(ir, read, &referenced, actors, first, &decoys);
            settle_read(ir, setup, read, &referenced, change.as_ref());
            let rows = [before, Some(referenced), after];
            surround(
                setup,
                rows.into_iter().flatten(),
                [change.map(|(_, run)| run)],
            );
            return Ok(false);
        }
    }
    if hop.type_ref.is_optional() && repoint(setup, None) {
        settle_absent(ir, setup, read);
        return Ok(false);
    }
    Err(RefusalCause::GuardUnsatisfiable {
        predicate: format!(
            "`{}` names one row for two reads, and its `{hop}` can be pointed at no arranged \
             `{}` row",
            read.via,
            ir.entity(read.entity).name
        ),
        tried: 1,
    })
}

/// Leaves `read`'s `via` absent for the run that witnesses it (ess/22, beyond10x/ess#285): the
/// Optional input it reads left out of the invocation, or — on an existing subject — the subject
/// created without the reference it stores ([`stored_absent`]). Every value read through it is
/// settled absent. `false`, with nothing changed, where neither can be built.
fn leave_via_absent(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &mut Setup,
    read: &Read<'_>,
    absence: &mut Absence<'_>,
) -> bool {
    if let Some(input) = input_read(ir, outcome, read.via) {
        let optional = absence
            .command
            .input
            .iter()
            .any(|field| field.name == input && field.type_ref.is_optional());
        if !optional || names_subject(outcome, input) || setup.bound.contains_key(input) {
            return false;
        }
        absence.omitted.insert(input.to_owned());
    } else {
        let ResolvedRelatedVia::Subject { field, .. } = read.via else {
            return false;
        };
        if !stored_absent(ir, setup, field) {
            return false;
        }
    }
    absence.vias.push(read.via.clone());
    settle_absent(ir, setup, read);
    absence.done = true;
    true
}

/// [`stored`], leaving the field `via` the existing subject stores absent (ess/22,
/// beyond10x/ess#285): the act that created the subject is sent without the Optional input its
/// branch fills `via` from, where the subject still holds what that act sent.
fn stored_absent(ir: &EssIr, setup: &mut Setup, via: &str) -> bool {
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
    let Some((command, creating)) = creating_branch(ir, taken) else {
        return false;
    };
    let Some(input) = set_from_input(creating, via).map(str::to_owned) else {
        return false;
    };
    if !command
        .input
        .iter()
        .any(|declared| declared.name == input && declared.type_ref.is_optional())
    {
        return false;
    }
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &mut setup.steps[created] else {
        return false;
    };
    let Some(before) = sent.get(&input).cloned() else {
        return false;
    };
    match setup.settled.get_mut(via) {
        Some(held) if held.value == before => held.value = ScenarioValue::literal(Node::Null),
        _ => return false,
    }
    sent.remove(&input);
    true
}

/// The command and branch a scenario's `expect_outcome` step names.
fn creating_branch<'ir>(
    ir: &'ir EssIr,
    taken: &crate::scenario::OutcomeRef,
) -> Option<(&'ir ResolvedCommand, &'ir ResolvedOutcome)> {
    ir.commands()
        .values()
        .filter(|command| CommandRef::new(command.name.clone()) == taken.command)
        .flat_map(|command| {
            command
                .outcomes
                .iter()
                .map(move |outcome| (command, outcome))
        })
        .find(|(_, outcome)| outcome.name == taken.outcome)
}

/// Rewrites the act that created `row` so the reference `field` it stores names `to`, or — with
/// `to` `None` — is left absent, its Optional input left out (ess/22, beyond10x/ess#285).
pub(super) fn rewrite_reference(
    ir: &EssIr,
    row: &mut Arrangement,
    field: &str,
    to: Option<&InstanceName>,
) -> bool {
    let Arrangement {
        instance,
        steps,
        settled,
        ..
    } = row;
    rewrite_in(ir, steps, settled, instance, field, to)
}

/// [`rewrite_reference`] over the steps that created `instance` and what it settled.
///
/// The link is the creating branch's `sets: <field>: input.<input>` with no conversion, as
/// [`stored`] reads it. The rewrite is made only where no act after the creating one names the
/// row, so nothing after it wrote the field; leaving it absent only where the input is
/// `Optional<…>`.
fn rewrite_in(
    ir: &EssIr,
    steps: &mut [ScenarioStep],
    settled: &mut BTreeMap<String, Determined>,
    instance: &InstanceName,
    field: &str,
    to: Option<&InstanceName>,
) -> bool {
    let Some(captured) = steps.iter().position(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance: captured, .. } if captured == instance)
    }) else {
        return false;
    };
    let Some(created) = steps[..captured]
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
    else {
        return false;
    };
    let Some(ScenarioStep::ExpectOutcome { outcome: taken }) = steps.get(created + 1) else {
        return false;
    };
    let Some((command, creating)) = creating_branch(ir, taken) else {
        return false;
    };
    // A later act on the row itself — one that names it and acts on its entity — may have
    // written the field; one that only names it, as a reference, has not.
    let entity = creating.subject.as_ref().map(|subject| &subject.entity);
    let pointed = ScenarioValue::instance(instance.clone());
    let acted_on_later = steps[captured..].windows(2).any(|pair| match pair {
        [ScenarioStep::ExecuteCommand { input, .. }, ScenarioStep::ExpectOutcome { outcome: taken }] => {
            input.values().any(|value| *value == pointed)
                && creating_branch(ir, taken).is_some_and(|(_, outcome)| {
                    outcome.subject.as_ref().is_some_and(|subject| {
                        Some(&subject.entity) == entity
                            && subject.effect != ResolvedEffect::Creates
                    })
                })
        }
        _ => false,
    });
    if acted_on_later {
        return false;
    }
    let Some(input) = set_from_input(creating, field).map(str::to_owned) else {
        return false;
    };
    let Some(filled) = creating.sets.iter().find(|set| set.target == field) else {
        return false;
    };
    let optional = command
        .input
        .iter()
        .any(|declared| declared.name == input && declared.type_ref.is_optional());
    if to.is_none() && !optional {
        return false;
    }
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &mut steps[created] else {
        return false;
    };
    let value = if let Some(named) = to {
        let pointed = ScenarioValue::instance(named.clone());
        sent.insert(input, pointed.clone());
        pointed
    } else {
        sent.remove(&input);
        ScenarioValue::literal(Node::Null)
    };
    settled.insert(
        field.to_owned(),
        Determined {
            value,
            type_ref: filled.target_type.clone(),
        },
    );
    true
}
