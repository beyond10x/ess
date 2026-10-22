//! A `when_related:` guard reading a stored field of the addressed subject (`via: <field>`, source
//! format `ess/22`, beyond10x/ess#304): "a task is completed only once the task its stored
//! `blocked_by` names is done".
//!
//! The command's input names only the subject, so the related row is reached by arranging the row
//! and then pointing the subject's stored field at it. The field is written by the act that
//! created the subject, from that act's input unchanged — the link [`super::super::related`]
//! already reads for a `{related: …}` source — so pointing it is rewriting that act's input:
//!
//! * a present row: the row is searched for as for an input reference, sits between decoys on
//!   which the same request answers otherwise, and the subject's creation names it;
//! * a missing row: the creation names an identity no row carries, beside rows of the entity that
//!   carry others — where the act that writes the field stores an identity unchecked. Where every
//!   writer refuses such an identity before storing it but a branch removes a row of the entity,
//!   the creation names a row that is removed once the subject is arranged (beyond10x/ess#509).
//!   Where neither holds — the proof validation reads from `ess/30` — no run holds such a
//!   reference and the branch is refused as unreachable;
//! * an absent Optional reference: the creation leaves the field out, between rows that would each
//!   refuse the request were the subject to name them.
//!
//! An arranging run of such a command — the move that brings a row along its lifecycle — is sent
//! with the reference left out where that selects its branch ([`step`]), so a row of an entity
//! whose own rows block each other is driven without arranging a second row first.

use std::collections::BTreeMap;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedCommand, ResolvedEffect, ResolvedOutcome, ResolvedPayloadValue,
};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::{
    absent_input, acting, block_start, candidates, decides, entity_ref, flatten, input_guard,
    is_absent, observe_read, own_arrangement, predicates, read, row_at, search_rows, selects,
    surround, unarranged, with_row, with_row_where, ActorRef, Arrangement, Distinction, Goal,
    RefusalCause, Setup, OWN,
};
use crate::scenario::{InstanceName, ScenarioStep, ScenarioValue};
use ess_compiler::ir::{ResolvedCondition, ResolvedRelatedTest, ResolvedRelatedVia};

/// The stored field of the addressed subject the command's related guards read, and the entity
/// whose row it names; `None` for a command reading its related row through its input.
pub(in crate::synthesize) fn field(command: &ResolvedCommand) -> Option<(&str, &EntityHandle)> {
    match read(command)? {
        (ResolvedRelatedVia::Subject { field, .. }, entity) => Some((field.as_str(), entity)),
        (ResolvedRelatedVia::Input { .. }, _) => None,
    }
}

/// What the subject's stored reference is pointed at.
enum Link<'a> {
    /// Left out: an Optional reference the subject does not hold.
    Absent,
    /// The row the scenario arranged.
    Row(&'a InstanceName),
    /// An identity no row of the entity carries.
    Fresh(&'a EntityHandle),
}

/// The addressed subject the scenario arranged: the branch's own, or the one the branch acting on
/// an existing row is sent for.
fn subject_of(command: &ResolvedCommand, setup: &Setup) -> Option<InstanceName> {
    setup
        .instance
        .clone()
        .or_else(|| acting(command).and_then(|(_, named)| setup.bound.get(named).cloned()))
}

/// The refusal of a stored reference the arrangement cannot point.
fn unpointed(field: &str, why: &str) -> RefusalCause {
    RefusalCause::GuardUnsatisfiable {
        predicate: format!("the stored `{field}` of the addressed subject: {why}"),
        tried: 0,
    }
}

/// Whether `command` refuses, before storing it, an identity no row carries in `input`: it reads
/// the row that input names through `when_related:` and declares an `exists: false` branch.
fn checks(command: &ResolvedCommand, input: &str) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            &outcome.condition,
            ResolvedCondition::Related {
                via: ResolvedRelatedVia::Input { field, .. },
                test: ResolvedRelatedTest::Absent,
                ..
            } if field == input
        )
    })
}

/// Why no run holds, in `field` of a `subject` row, an identity no `far` row carries — the proof
/// validation reads from `ess/30` (`ess-domain`'s `command::reference_integrity`,
/// beyond10x/ess#509): every act writing the field stores nothing, the row's own value or an input
/// its command refuses first where no row carries it, and no branch removes a `far` row. `None`
/// where some act stores an identity unchecked or some branch removes a `far` row: a run then
/// holds one.
fn unreachable(
    ir: &EssIr,
    subject: &EntityHandle,
    field: &str,
    far: &EntityHandle,
) -> Option<String> {
    let mut writers = Vec::new();
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            if removes(outcome, far) {
                return None;
            }
            let own = outcome
                .subject
                .as_ref()
                .is_some_and(|named| named.entity == *subject)
                || outcome
                    .instances
                    .as_ref()
                    .is_some_and(|named| named.entity == *subject);
            let written = outcome
                .sets
                .iter()
                .filter(|_| own)
                .chain(
                    outcome
                        .affects
                        .iter()
                        .filter(|affect| affect.entity == *subject)
                        .flat_map(|affect| &affect.sets),
                )
                .filter(|set| set.target == field);
            let mut checked = false;
            for set in written {
                match &set.value {
                    ResolvedPayloadValue::Cleared => {}
                    ResolvedPayloadValue::SubjectField { field: read, .. }
                        if own && read == field => {}
                    ResolvedPayloadValue::InputField { field: input, .. }
                        if set.conversion.is_none() && checks(command, input) =>
                    {
                        checked = true;
                    }
                    _ => return None,
                }
            }
            if checked {
                writers.push(format!("`{}/{}`", command.name, outcome.name));
            }
        }
    }
    Some(format!(
        "unreachable: every act writing `{field}` ({}) refuses an identity no row carries before \
         storing it, and no branch removes a `{far}` row, so no run holds one",
        if writers.is_empty() {
            "none".to_owned()
        } else {
            writers.join(", ")
        }
    ))
}

/// Whether `outcome` removes a row of `entity`: its subject, the rows its `instances:` selection
/// names, or an `affects:` entry deleting them.
fn removes(outcome: &ResolvedOutcome, entity: &EntityHandle) -> bool {
    outcome
        .subject
        .as_ref()
        .is_some_and(|named| named.entity == *entity && named.effect == ResolvedEffect::Deletes)
        || outcome
            .instances
            .as_ref()
            .is_some_and(|named| named.entity == *entity && named.effect == ResolvedEffect::Deletes)
        || outcome
            .affects
            .iter()
            .any(|affect| affect.entity == *entity && affect.deletes)
}

/// The step in `steps` that created `instance`, and the command and branch it took.
fn creation<'ir>(
    ir: &'ir EssIr,
    steps: &[ScenarioStep],
    instance: &InstanceName,
    field: &str,
) -> Result<(usize, &'ir ResolvedCommand, &'ir ResolvedOutcome), RefusalCause> {
    let gap = |why: &str| unpointed(field, why);
    let captured = steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::CaptureInstance { instance: named, .. } if named == instance)
        })
        .ok_or_else(|| gap("no step of the arrangement captures the subject"))?;
    let created = steps[..captured]
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .ok_or_else(|| gap("no act of the arrangement creates the subject"))?;
    let Some(ScenarioStep::ExpectOutcome { outcome: taken }) = steps.get(created + 1) else {
        return Err(gap("the act creating the subject asserts no outcome"));
    };
    let command = ir
        .commands()
        .values()
        .find(|command| super::super::CommandRef::new(command.name.clone()) == taken.command)
        .ok_or_else(|| gap("the act creating the subject names no declared command"))?;
    let creating = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name == taken.outcome)
        .ok_or_else(|| gap("the act creating the subject names no declared outcome"))?;
    Ok((created, command, creating))
}

/// Points the stored `field` of `instance` as `link` says, by rewriting the input of the act in
/// `steps` that created it, and what `settled` says the subject holds there with it.
fn relink(
    ir: &EssIr,
    steps: &mut [ScenarioStep],
    settled: Option<&mut BTreeMap<String, super::super::Determined>>,
    instance: &InstanceName,
    field: &str,
    link: &Link<'_>,
) -> Result<(), RefusalCause> {
    let gap = |why: &str| unpointed(field, why);
    let (created, command, creating) = creation(ir, steps, instance, field)?;
    let subject = creating
        .subject
        .as_ref()
        .map(|subject| &subject.entity)
        .ok_or_else(|| gap("the act creating the subject names no subject"))?;
    let input = super::super::related::set_from_input(creating, field)
        .ok_or_else(|| {
            gap("the act creating the subject does not write it from its input unchanged")
        })?
        .to_owned();
    let declared = command
        .input
        .iter()
        .find(|held| held.name == input)
        .map(|held| held.type_ref.clone())
        .ok_or_else(|| gap("the creating act's input declares no such field"))?;
    let stored = ir
        .entity(subject)
        .fields
        .iter()
        .find(|held| held.name == field)
        .map(|held| held.type_ref.clone())
        .ok_or_else(|| gap("the subject declares no such field"))?;
    let value = match link {
        Link::Absent => {
            if !declared.is_optional() {
                return Err(gap("the act creating the subject requires it"));
            }
            None
        }
        Link::Row(row) => Some(ScenarioValue::instance((*row).clone())),
        Link::Fresh(far) => {
            if checks(command, &input) {
                return Err(gap(&unreachable(ir, subject, field, far).unwrap_or_else(
                    || {
                        format!(
                            "`{}` refuses an identity no row carries before storing it, and the \
                         arrangement creates the subject through it",
                            command.name
                        )
                    },
                )));
            }
            let fresh = super::super::fresh_identity(ir, command, &input, None)?;
            Some(crate::now_offset::sent(ir, command, &input, &fresh))
        }
    };
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &mut steps[created] else {
        return Err(gap("the act creating the subject is not a command"));
    };
    // Only where the subject still holds what that act sent: no later act of the arrangement
    // wrote the field, so the row the branch reads is the one pointed at here. An optional input
    // the act left out (beyond10x/ess#546) is held as a null copy.
    if let Some(settled) = settled.as_deref() {
        let held = settled.get(field).map(|held| &held.value);
        let kept = match (held, sent.get(&input)) {
            (Some(ScenarioValue::Literal { value: Node::Null }), None) => true,
            (held, sent) => held == sent,
        };
        if !kept {
            return Err(gap("a later act of the arrangement rewrote it"));
        }
    }
    match &value {
        Some(value) => {
            sent.insert(input, value.clone());
        }
        None => {
            sent.remove(&input);
        }
    }
    if let Some(settled) = settled {
        match value {
            Some(value) => {
                settled.insert(
                    field.to_owned(),
                    super::super::Determined {
                        value,
                        type_ref: stored,
                    },
                );
            }
            None => {
                settled.remove(field);
            }
        }
    }
    Ok(())
}

/// An input the command is sent with where the addressed row's existence and held state have
/// answered nothing: one no input-guarded refusal takes, so the stored reference decides.
fn unrefused(
    ir: &EssIr,
    command: &ResolvedCommand,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let refusals: Vec<&ess_primitives::predicate::Predicate> = command
        .outcomes
        .iter()
        .filter(|outcome| super::super::is_input_guarded_refusal(outcome))
        .filter_map(input_guard)
        .collect();
    let guards: Vec<_> = command.outcomes.iter().filter_map(input_guard).collect();
    for input in candidates(ir, command, &guards, distinction).map_err(RefusalCause::NoWitness)? {
        let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
        let mut refused = false;
        for guard in &refusals {
            refused |= decides(&facts, &[*guard], true)?;
        }
        if !refused {
            return Ok(input);
        }
    }
    Err(unarranged())
}

/// [`super::prepare_at_in`] for a stored reference: the setup, the input, and whether the scenario
/// is unaccompanied (never: a stored reference copies nothing here).
pub(super) fn arranged_at(
    models: &super::super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    goal: Option<&Goal>,
) -> Result<(Setup, BTreeMap<String, Node>, bool), RefusalCause> {
    let ir = models.arrangement;
    let (field, entity) = field(command).ok_or_else(unarranged)?;
    if goal.is_some() && is_absent(outcome) {
        return Err(unarranged());
    }
    let mut setup = own_arrangement(
        ir,
        command,
        outcome,
        actors,
        distinction,
        (field, entity),
        &[],
    )?;
    let subject = subject_of(command, &setup).ok_or_else(unarranged)?;
    // The subject's own arrangement must not have run this command on it: those runs read the
    // stored reference as it was, and repointing it afterwards would contradict them.
    if runs(&setup.steps, command) {
        return Err(unpointed(
            field,
            "the subject's arrangement already runs the command that reads it",
        ));
    }
    let mut steps = Vec::new();
    // Steps run once the subject is arranged, before the command is sent.
    let mut after = Vec::new();
    let input = if is_absent(outcome) {
        let first = block_start(OWN, distinction);
        let fresh = relink(
            ir,
            &mut setup.steps,
            setup.instance.is_some().then_some(&mut setup.settled),
            &subject,
            field,
            &Link::Fresh(entity),
        );
        if let Err(fresh) = fresh {
            // Every writer refuses an identity no row carries before storing it: the subject is
            // pointed at a row a branch removes once the subject is arranged (beyond10x/ess#509).
            let (row, removal) = removed(ir, entity, actors, first).ok_or(fresh)?;
            relink(
                ir,
                &mut setup.steps,
                setup.instance.is_some().then_some(&mut setup.settled),
                &subject,
                field,
                &Link::Row(&row.instance),
            )?;
            steps.extend(row.steps);
            setup.source.extend(row.source);
            after.extend(removal.steps);
            setup.source.extend(removal.source);
        }
        // Rows of the entity exist, each carrying an identity other than the one stored.
        if !super::super::singleton::is_singleton(ir, entity) {
            for at in [first - 1, first + 1] {
                if let Some(decoy) = row_at(ir, entity, actors, at, &[]) {
                    steps.extend(decoy.steps);
                    setup.source.extend(decoy.source);
                }
            }
        }
        unrefused(ir, command, distinction)?
    } else {
        let (row, first, input) = reachable_row(
            ir,
            (command, outcome, actors),
            (entity, &subject, field),
            &setup,
            (distinction, goal),
        )?;
        relink(
            ir,
            &mut setup.steps,
            setup.instance.is_some().then_some(&mut setup.settled),
            &subject,
            field,
            &Link::Row(&row.instance),
        )?;
        let named = ordered(ir, &setup, (entity, &subject, field), &row, &mut after)?;
        surround(
            ir,
            (command, outcome, entity),
            actors,
            (first, &input),
            (&named, &[]),
            (&mut steps, &mut setup.source),
        );
        let observing = if after.is_empty() {
            &mut steps
        } else {
            &mut after
        };
        observe_read(ir, command, entity, &row, observing, &mut setup.source);
        input
    };
    // The related rows go ahead of the subject's own arrangement, whose creation names one.
    std::mem::swap(&mut steps, &mut setup.steps);
    setup.steps.append(&mut steps);
    setup.steps.append(&mut after);
    setup.source.insert(entity_ref(entity));
    Ok((setup, input, false))
}

/// The related row [`arranged_at`] points the subject at, the distinction it was arranged at and
/// the input: the row [`with_row`] reaches first, unless no order of the subject's arrangement
/// `setup` reaches the branch on it ([`order`]) — a row created straight into the state the branch
/// reads, by a creator the arrangement refuses it from — and another creator's row reaches that
/// state by moves after the arrangement. That row is searched for, and the first row stands, to be
/// refused by [`ordered`], where none is found.
fn reachable_row(
    ir: &EssIr,
    (command, outcome, actors): (
        &ResolvedCommand,
        &ResolvedOutcome,
        &BTreeMap<QualifiedName, ActorRef>,
    ),
    (entity, subject, field): (&EntityHandle, &InstanceName, &str),
    setup: &Setup,
    (distinction, goal): (Distinction, Option<&Goal>),
) -> Result<(Arrangement, usize, BTreeMap<String, Node>), RefusalCause> {
    let search = |admits: &dyn Fn(&Arrangement) -> bool| {
        with_row_where(
            ir,
            command,
            outcome,
            entity,
            actors,
            (OWN, distinction, &[]),
            (None, goal, None),
            (&BTreeMap::new(), &setup.steps, &BTreeMap::new()),
            admits,
        )
    };
    // The subject's arrangement as it runs once its creation names `row`.
    let pointed = |row: &Arrangement| {
        let mut steps = setup.steps.clone();
        let mut settled = setup.settled.clone();
        relink(
            ir,
            &mut steps,
            setup.instance.is_some().then_some(&mut settled),
            subject,
            field,
            &Link::Row(&row.instance),
        )
        .map(|()| steps)
    };
    let reaches = |row: &Arrangement| {
        pointed(row).is_ok_and(|steps| order(ir, &steps, (entity, subject, field), row).is_ok())
    };
    let found = search(&|_| true)?;
    if reaches(&found.0) {
        return Ok(found);
    }
    Ok(search(&reaches).unwrap_or(found))
}

/// A row of `entity` where its lifecycle starts, under the further witness `at`, and the steps of
/// a declared deletion that remove it; `None` where no branch addressing such a row deletes it.
fn removed(
    ir: &EssIr,
    entity: &EntityHandle,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    at: usize,
) -> Option<(Arrangement, Arrangement)> {
    let row = row_at(ir, entity, actors, at, &[])?;
    let removal =
        super::super::subject_fact::delete_existing(ir, entity, row.clone(), actors).ok()??;
    Some((row, removal))
}

/// [`super::prepare_absent_in`] for a stored reference: the subject created with the reference left
/// out, after two rows of the entity each selecting a present-related refusal where one is found.
pub(super) fn absent_at(
    models: &super::super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let ir = models.arrangement;
    let (field, entity) = field(command).ok_or_else(unarranged)?;
    let input = absent_input(ir, command, outcome, distinction)?;
    let mut setup = own_arrangement(
        ir,
        command,
        outcome,
        actors,
        distinction,
        (field, entity),
        &[],
    )?;
    let subject = subject_of(command, &setup).ok_or_else(unarranged)?;
    if runs(&setup.steps, command) {
        return Err(unpointed(
            field,
            "the subject's arrangement already runs the command that reads it",
        ));
    }
    relink(
        ir,
        &mut setup.steps,
        setup.instance.is_some().then_some(&mut setup.settled),
        &subject,
        field,
        &Link::Absent,
    )?;
    let mut steps = Vec::new();
    if !super::super::singleton::is_singleton(ir, entity) {
        let refusing = |node: &Arrangement| {
            selects(ir, command, entity, Some(node), &input).map(|branch| {
                branch.is_some_and(|branch| {
                    branch.error.is_some()
                        && matches!(
                            branch.condition,
                            ResolvedCondition::Related {
                                test: ResolvedRelatedTest::Holds { .. },
                                ..
                            }
                        )
                })
            })
        };
        let first = block_start(OWN, distinction);
        for at in [first - 1, first + 1] {
            let decoy = search_rows(
                ir,
                entity,
                actors,
                (at, None),
                &predicates(command),
                refusing,
            )
            .or_else(|| row_at(ir, entity, actors, at, &[]));
            if let Some(decoy) = decoy {
                steps.extend(decoy.steps);
                setup.source.extend(decoy.source);
            }
        }
    }
    std::mem::swap(&mut steps, &mut setup.steps);
    setup.steps.append(&mut steps);
    setup.source.insert(entity_ref(entity));
    Ok((setup, input))
}

/// The input of the wrong-state scenario for a command reading a stored reference: the held state
/// answers before the reference is read, so the subject is sent as its arrangement left it.
pub(super) fn wrong_state_input(
    ir: &EssIr,
    command: &ResolvedCommand,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    unrefused(ir, command, Distinction::PLAIN)
}

/// Whether `steps` run `command`.
fn runs(steps: &[ScenarioStep], command: &ResolvedCommand) -> bool {
    steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. } if sent.name() == &command.name)
    })
}

/// The related row as it goes ahead of the subject's arrangement `setup`; the moves that
/// follow that arrangement are pushed onto `after`.
///
/// The subject's arrangement is sent with the related row as it is when it runs. Where an act of
/// it refuses the row in the state the branch reads — the creator of a machine refusing a frozen
/// site — the row is created first and moved there once the subject is arranged; where the act
/// refuses the row as created too, no order reaches the branch and it is refused by name.
fn ordered(
    ir: &EssIr,
    setup: &Setup,
    (entity, subject, field): (&EntityHandle, &InstanceName, &str),
    row: &Arrangement,
    after: &mut Vec<ScenarioStep>,
) -> Result<Arrangement, RefusalCause> {
    let Some((created, later)) = order(ir, &setup.steps, (entity, subject, field), row)? else {
        return Ok(row.clone());
    };
    let mut named = row.clone();
    named.steps = created;
    after.extend(later);
    Ok(named)
}

/// The steps that create a related row, and the moves that bring it on from there.
type Split = (Vec<ScenarioStep>, Vec<ScenarioStep>);

/// How [`ordered`] splits `row` around the subject's arrangement `steps`: `None` where no act of it
/// refuses the row as the branch reads it, else the steps that create the row and the moves that
/// follow the arrangement. Refused by name where the act refuses the row as created too, or the row
/// is created straight into the state the branch reads.
fn order(
    ir: &EssIr,
    steps: &[ScenarioStep],
    (entity, subject, field): (&EntityHandle, &InstanceName, &str),
    row: &Arrangement,
) -> Result<Option<Split>, RefusalCause> {
    let Some(refusing) = refuses(ir, steps, entity, subject, field, row) else {
        return Ok(None);
    };
    let (created, later) = split_moves(row);
    let born = created_state(ir, entity, &created);
    let as_created = born.as_ref().and_then(|state| {
        refuses_in(
            ir,
            steps,
            (entity, subject, field, &row.instance),
            state,
            None,
        )
    });
    if later.is_empty() || born.is_none() || as_created.is_some() {
        return Err(unpointed(
            field,
            &format!(
                "`{refusing}`, which arranges the subject, refuses the related row in the state \
                 the branch reads and where the row is created, so no order of the arrangement \
                 reaches the branch"
            ),
        ));
    }
    Ok(Some((created, later)))
}

/// The steps of `row` up to the one capturing it, and the moves that bring it on from there to the
/// state it was arranged in.
fn split_moves(row: &Arrangement) -> (Vec<ScenarioStep>, Vec<ScenarioStep>) {
    let captured = row.steps.iter().position(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance, .. } if *instance == row.instance)
    });
    match captured {
        Some(at) => {
            let mut created = row.steps.clone();
            let moves = created.split_off(at + 1);
            (created, moves)
        }
        None => (row.steps.clone(), Vec::new()),
    }
}

/// The state the act ending `created` leaves the row it creates in: the state its `into:` names,
/// or where the entity's lifecycle starts. `None` where that act is not a declared creation.
fn created_state(
    ir: &EssIr,
    entity: &EntityHandle,
    created: &[ScenarioStep],
) -> Option<super::super::StateName> {
    let taken = created.iter().rev().find_map(|step| match step {
        ScenarioStep::ExpectOutcome { outcome } => Some(outcome),
        _ => None,
    })?;
    let command = ir
        .commands()
        .values()
        .find(|command| super::super::CommandRef::new(command.name.clone()) == taken.command)?;
    let subject = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name == taken.outcome)?
        .subject
        .as_ref()
        .filter(|subject| subject.entity == *entity && subject.effect == ResolvedEffect::Creates)?;
    Some(
        subject
            .into
            .clone()
            .unwrap_or_else(|| ir.entity(entity).lifecycle.initial.clone()),
    )
}

/// The command of an act in `steps` that a `when_related:` refusal it does not expect to take
/// answers once the related row is held as `row` holds it: an act naming the row through its
/// input, or reading the stored `field` of `subject` that names it.
fn refuses(
    ir: &EssIr,
    steps: &[ScenarioStep],
    entity: &EntityHandle,
    subject: &InstanceName,
    field: &str,
    row: &Arrangement,
) -> Option<QualifiedName> {
    refuses_in(
        ir,
        steps,
        (entity, subject, field, &row.instance),
        &row.state,
        Some((&row.settled, &row.unwritten)),
    )
}

/// [`refuses`] with the row in `state`, its fields as `fields` hold them where known.
fn refuses_in(
    ir: &EssIr,
    steps: &[ScenarioStep],
    (entity, subject, field, instance): (&EntityHandle, &InstanceName, &str, &InstanceName),
    state: &super::super::StateName,
    fields: Option<(
        &BTreeMap<String, super::super::Determined>,
        &std::collections::BTreeSet<String>,
    )>,
) -> Option<QualifiedName> {
    let (empty, none) = (BTreeMap::new(), std::collections::BTreeSet::new());
    let (settled, unwritten) = fields.unwrap_or((&empty, &none));
    let named = ScenarioValue::instance(instance.clone());
    let addressed = ScenarioValue::instance(subject.clone());
    steps.iter().enumerate().find_map(|(at, step)| {
        let ScenarioStep::ExecuteCommand {
            command: sent,
            input,
            ..
        } = step
        else {
            return None;
        };
        let taken = match steps.get(at + 1) {
            Some(ScenarioStep::ExpectOutcome { outcome }) => Some(&outcome.outcome),
            _ => None,
        };
        let command = ir
            .commands()
            .values()
            .find(|command| sent.name() == &command.name)?;
        let refused = command.outcomes.iter().any(|branch| {
            let ResolvedCondition::Related {
                via,
                entity: read,
                test: ResolvedRelatedTest::Holds { predicate },
                input: None,
                also,
            } = &branch.condition
            else {
                return false;
            };
            let reads_row = match via {
                ResolvedRelatedVia::Input { field: from, .. } => input.get(from) == Some(&named),
                ResolvedRelatedVia::Subject { field: from, .. } => {
                    from == field && input.values().any(|value| *value == addressed)
                }
            };
            branch.error.is_some()
                && taken != Some(&branch.name)
                && read == entity
                && also.is_empty()
                && reads_row
                && super::super::subject_fact::guard_truth_with(
                    ir,
                    entity,
                    settled,
                    unwritten,
                    Some(state),
                    predicate,
                    None,
                ) == ess_primitives::predicate::Truth::True
        });
        refused.then(|| command.name.clone())
    })
}

/// The block the related row an arranging run points a stored reference at is created in: past
/// every block [`super`] arranges rows in, so it names no row a scenario, a driver, a nested
/// arrangement, a companion, a row beside or an overlap does, and apart from the block the rows a
/// list names are created in ([`super::super::related_list`], `25`) and the one a copied field's
/// related row is ([`super::super::subject_fact`], `13`). It was `13`, the first block of the rows
/// beside, and a machine row's site was then opened under the name of the site beside it, which
/// rebound that name before the send (an adopter's report); then `25`, and the site it opened was
/// named as the first site a list in the same scenario names.
const STORED: usize = super::OVERLAP + 5;

// Every block another strategy creates rows in is checked here, so a block that moves onto this
// one fails the build instead of naming one row twice in a scenario.
const _: () = {
    let others = [
        super::OWN,
        super::DRIVEN,
        super::NESTED,
        super::COMPANION,
        super::super::related_list::ROWS / super::BLOCK,
        super::super::subject_fact::COPIED / super::BLOCK,
    ];
    let mut at = 0;
    while at < others.len() {
        assert!(STORED != others[at]);
        at += 1;
    }
    // The rows beside, a driver's rows beside and the overlaps, each four blocks wide.
    assert!(STORED >= super::OVERLAP + 4 || STORED < super::OWN);
};

/// One arranging run of a branch of a command reading a stored reference, on the row `arrangement`
/// brought into being, and the row as the run leaves it.
///
/// The run is sent with the reference left out — the row's creation rewritten to leave it out
/// where it wrote one — where that selects the branch. Where it does not, or the reference is
/// required (correction round 1, F2), a row of the related entity that selects the branch is
/// arranged first, in a block of its own, and the creation rewritten to name it — one level deep:
/// a related row of an entity `arranging` already names (the driven row's own included) is not
/// arranged again. Refused naming both reasons where neither run can be built.
pub(in crate::synthesize) fn step(
    ir: &EssIr,
    driver: &super::super::Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
) -> Result<Arrangement, RefusalCause> {
    let (field, related) = field(driver.command).ok_or_else(unarranged)?;
    // A reference naming the row a search is advancing — the incident a review is filed under,
    // while the incident is arranged — is decided on that row, and left as it is
    // (beyond10x/ess#553).
    if let Some(ScenarioValue::Instance { instance }) =
        arrangement.settled.get(field).map(|held| &held.value)
    {
        if let Some(row) = super::super::subject_fact::held(ir, related, instance) {
            return on_held(ir, driver, arrangement, actors, distinction, related, &row);
        }
    }
    let absent = without(ir, driver, arrangement, actors, distinction, field);
    let Err(absent) = absent else {
        return absent;
    };
    let pointed = pointed(
        ir,
        driver,
        arrangement,
        actors,
        distinction,
        (field, related),
        arranging,
    );
    pointed.map_err(|pointed| {
        unpointed(
            field,
            &format!(
                "an arranging run of `{}/{}` needs it left out ({}) or naming a row of `{}` that \
                 selects the branch ({})",
                driver.command.name,
                driver.outcome.name,
                reason(&absent),
                related.name(),
                reason(&pointed)
            ),
        )
    })
}

/// [`step`] on the row a search is advancing, which the stored reference already names: sent with
/// an input on which that row selects the branch.
fn on_held(
    ir: &EssIr,
    driver: &super::super::Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    related: &EntityHandle,
    row: &Arrangement,
) -> Result<Arrangement, RefusalCause> {
    let inputs = super::held_inputs(ir, driver.command, related, row, None, distinction)?;
    let tried = inputs.len();
    let input = inputs
        .into_iter()
        .find(|input| {
            selects(ir, driver.command, related, Some(row), input)
                .ok()
                .flatten()
                .is_some_and(|branch| branch.name == driver.outcome.name)
        })
        .ok_or_else(|| RefusalCause::GuardUnsatisfiable {
            predicate: format!(
                "the row of `{}` the arrangement is advancing, which the stored reference of the \
                 addressed subject names, does not select `{}/{}` as the steps so far leave it",
                related.name(),
                driver.command.name,
                driver.outcome.name
            ),
            tried,
        })?;
    Ok(advanced(ir, driver, arrangement.clone(), actors, &input))
}

/// A refusal's reason, without its code.
fn reason(cause: &RefusalCause) -> String {
    match cause {
        RefusalCause::GuardUnsatisfiable { predicate, .. } => predicate.clone(),
        other => other.to_string(),
    }
}

/// [`step`] with the reference left out.
fn without(
    ir: &EssIr,
    driver: &super::super::Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    field: &str,
) -> Result<Arrangement, RefusalCause> {
    let input = absent_input(ir, driver.command, driver.outcome, distinction)?;
    let mut next = arrangement.clone();
    let instance = next.instance.clone();
    // A field no step of the arrangement wrote is already absent.
    if !next.unwritten.contains(field) {
        relink(
            ir,
            &mut next.steps,
            Some(&mut next.settled),
            &instance,
            field,
            &Link::Absent,
        )?;
        next.unwritten.insert(field.to_owned());
    }
    Ok(advanced(ir, driver, next, actors, &input))
}

/// [`step`] with the reference naming a row of `related` arranged for it.
fn pointed(
    ir: &EssIr,
    driver: &super::super::Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (field, related): (&str, &EntityHandle),
    arranging: &[&EntityHandle],
) -> Result<Arrangement, RefusalCause> {
    let moved_entity = driver
        .outcome
        .subject
        .as_ref()
        .map(|subject| &subject.entity)
        .ok_or_else(unarranged)?;
    let chain: Vec<&EntityHandle> = arranging.iter().copied().chain([moved_entity]).collect();
    if chain.contains(&related) {
        return Err(unpointed(
            field,
            &format!(
                "a row of `{}` is already being arranged, and a related row of it is arranged one \
                 level deep only",
                related.name()
            ),
        ));
    }
    let (row, _, input) = with_row(
        ir,
        driver.command,
        driver.outcome,
        related,
        actors,
        (STORED, distinction, &chain),
        None,
        None,
        None,
        (&BTreeMap::new(), &[], &BTreeMap::new()),
    )?;
    let mut next = arrangement.clone();
    let instance = next.instance.clone();
    relink(
        ir,
        &mut next.steps,
        Some(&mut next.settled),
        &instance,
        field,
        &Link::Row(&row.instance),
    )?;
    // The row goes ahead of the creation that names it.
    let mut steps = row.steps;
    steps.append(&mut next.steps);
    next.steps = steps;
    next.source.extend(row.source);
    next.source.insert(entity_ref(related));
    Ok(advanced(ir, driver, next, actors, &input))
}

/// `next` with the run of `driver` on it, sent with `input`.
fn advanced(
    ir: &EssIr,
    driver: &super::super::Driver<'_>,
    mut next: Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    input: &BTreeMap<String, Node>,
) -> Arrangement {
    let instance = next.instance.clone();
    let invoked =
        super::super::invoke_with(ir, driver, Some(&instance), actors, &BTreeMap::new(), input);
    next.steps.extend(invoked.steps);
    next.source.extend(invoked.source);
    next.absorb(driver.outcome, invoked.settled);
    if let Some(transition) = driver.effect.transition() {
        next.state = transition.to.clone();
    }
    next
}
