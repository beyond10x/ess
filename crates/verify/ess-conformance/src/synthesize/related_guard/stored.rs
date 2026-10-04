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
//!   writer refuses an identity no row carries before storing it, no run holds such a reference and
//!   the branch is refused as unreachable;
//! * an absent Optional reference: the creation leaves the field out, between rows that would each
//!   refuse the request were the subject to name them.
//!
//! An arranging run of such a command — the move that brings a row along its lifecycle — is sent
//! with the reference left out where that selects its branch ([`step`]), so a row of an entity
//! whose own rows block each other is driven without arranging a second row first.

use std::collections::BTreeMap;

use ess_compiler::ir::{EntityHandle, EssIr, ResolvedCommand, ResolvedOutcome};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::{
    absent_input, acting, block_start, candidates, decides, entity_ref, flatten, input_guard,
    is_absent, observe_read, own_arrangement, predicates, read, row_at, search_rows, selects,
    surround, unarranged, with_row, ActorRef, Arrangement, Distinction, Goal, RefusalCause, Setup,
    OWN,
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
    /// An identity no row carries.
    Fresh,
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

/// Why no run stores an identity no row carries in `field` of `subject`: every act writing it from
/// its input refuses one first. `None` where some act stores it unchecked.
fn unreachable(ir: &EssIr, subject: &EntityHandle, field: &str) -> Option<String> {
    let all = ir.drivers();
    let mut writers = Vec::new();
    for driver in all.get(subject).map_or(&[][..], Vec::as_slice) {
        let Some(input) = super::super::related::set_from_input(driver.outcome, field) else {
            continue;
        };
        if !checks(driver.command, input) {
            return None;
        }
        writers.push(format!("`{}/{}`", driver.command.name, driver.outcome.name));
    }
    Some(format!(
        "unreachable: every act writing `{field}` ({}) refuses an identity no row carries before \
         storing it, so no run holds one",
        if writers.is_empty() {
            "none".to_owned()
        } else {
            writers.join(", ")
        }
    ))
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
        Link::Fresh => {
            if checks(command, &input) {
                return Err(gap(&unreachable(ir, subject, field).unwrap_or_else(|| {
                    format!(
                        "`{}` refuses an identity no row carries before storing it, and the \
                         arrangement creates the subject through it",
                        command.name
                    )
                })));
            }
            let fresh = super::super::fresh_identity(ir, command, &input, None)?;
            Some(crate::now_offset::sent(command, &input, &fresh))
        }
    };
    let ScenarioStep::ExecuteCommand { input: sent, .. } = &mut steps[created] else {
        return Err(gap("the act creating the subject is not a command"));
    };
    // Only where the subject still holds what that act sent: no later act of the arrangement
    // wrote the field, so the row the branch reads is the one pointed at here.
    if let Some(settled) = settled.as_deref() {
        if settled.get(field).map(|held| &held.value) != sent.get(&input) {
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
    let mut setup = own_arrangement(ir, command, outcome, actors, distinction, (field, entity))?;
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
    let input = if is_absent(outcome) {
        relink(
            ir,
            &mut setup.steps,
            setup.instance.is_some().then_some(&mut setup.settled),
            &subject,
            field,
            &Link::Fresh,
        )?;
        // Rows of the entity exist, each carrying an identity other than the one stored.
        let first = block_start(OWN, distinction);
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
        let (row, first, input) = with_row(
            ir,
            command,
            outcome,
            entity,
            actors,
            (OWN, distinction, &[]),
            None,
            goal,
            (&BTreeMap::new(), &setup.steps),
        )?;
        surround(
            ir,
            (command, outcome, entity),
            actors,
            (first, &input),
            (&row, &[]),
            (&mut steps, &mut setup.source),
        );
        observe_read(ir, command, entity, &row, &mut steps, &mut setup.source);
        relink(
            ir,
            &mut setup.steps,
            setup.instance.is_some().then_some(&mut setup.settled),
            &subject,
            field,
            &Link::Row(&row.instance),
        )?;
        input
    };
    // The related rows go ahead of the subject's own arrangement, whose creation names one.
    std::mem::swap(&mut steps, &mut setup.steps);
    setup.steps.append(&mut steps);
    setup.source.insert(entity_ref(entity));
    Ok((setup, input, false))
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
    let mut setup = own_arrangement(ir, command, outcome, actors, distinction, (field, entity))?;
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

/// The block the related row an arranging run points a stored reference at is created in: apart
/// from every block [`super`] arranges rows in, so it names no row a scenario, a driver, a nested
/// arrangement or a companion does.
const STORED: usize = 13;

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
        (&BTreeMap::new(), &[]),
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
