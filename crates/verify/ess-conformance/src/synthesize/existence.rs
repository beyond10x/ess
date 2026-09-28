//! Outcomes selected by whether the addressed record exists (ess/16, beyond10x/ess#164,
//! `docs/design/outcome-shapes.md`): each witnessed by calls that share one identity.
//!
//! * **Create-or-update.** The creating branch marked `unknown_instance: true` is filed like any
//!   creation, sent for an identity no other scenario sends ([`fresh_created`]). Its updating
//!   sibling is filed by the ordinary outcome family, whose arrangement creates the row through
//!   that same command — the first call — and then sends the identity again; this module adds the
//!   claim that the second call left **one** row for it ([`one_row`]).
//! * **Create-or-refuse.** The `existing_instance:` branch gets a scenario of its own
//!   ([`existing_instance`]): the creation with a fresh identity, the row snapshotted, the same
//!   identity sent again with other field values, then the declared error, no event and the row
//!   unchanged.
//!
//! Every step used is an existing one, so a suite carrying either form keeps the format its other
//! steps select.

use ess_compiler::ir::{Driver, ResolvedPayloadValue};
use ess_domain::view::Consistency;

use crate::witness::{candidates, WitnessGap, MAX_CANDIDATES};

use super::{
    clipped, created, insert, is_input_guarded_refusal, not_emitted, outcome_scenario, reach, run,
    shows, subject_fact, supply, ActorRef, AssertionStyle, BTreeMap, BTreeSet, CommandRef,
    ConformanceScenario, ConformanceSuite, Distinction, EntityRef, ErrorRef, EssIr, EssSemanticRef,
    InstanceNeed, Node, OutcomeRef, QualifiedName, Refusal, RefusalCause, ResolvedCommand,
    ResolvedCondition, ResolvedEffect, ResolvedInstance, ResolvedOutcome, ResolvedSubject,
    ResolvedView, Run, ScenarioId, ScenarioStep, ScenarioValue, Setup, ViewRef, FRESH_WITNESSES,
};

/// `true` for the creating half of create-or-update: a `creates:` branch marked
/// `unknown_instance: true`.
pub(super) fn creates_unknown(outcome: &ResolvedOutcome) -> bool {
    outcome.condition == ResolvedCondition::UnknownInstance
        && outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.effect == ResolvedEffect::Creates)
}

/// `true` when an identity no record carries creates a record on this command, rather than being
/// answered by a refusal or a no-op.
pub(super) fn creates_on_unknown(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(creates_unknown)
}

/// The input field a creating branch takes its new identity from: the `input.` source its payload
/// declares for the event field the creation publishes the identity in, or the optional input read
/// by `{input: f, else: {generated: true}}` — a caller that sends `f` names the identity.
fn identity_input(outcome: &ResolvedOutcome) -> Option<&str> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        return None;
    };
    outcome
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .filter(|entry| entry.target == field.name)
        .find_map(|entry| match &entry.value {
            ResolvedPayloadValue::InputField { field, .. }
            | ResolvedPayloadValue::InputOrGenerated {
                field,
                otherwise: None,
                ..
            } => Some(field.as_str()),
            _ => None,
        })
}

/// The first witness the identities of this module are drawn from: far past
/// [`Distinction::UNKNOWN`], which an ess/15 `unknown_instance:` refusal on the same entity sends,
/// and past every witness an arrangement numbers.
const FRESH_BASE: usize = Distinction::UNKNOWN.get() + 16;

/// How many witnesses past [`FRESH_BASE`] this module may use, and checks a fresh identity against.
const FRESH_SLOTS: usize = 64;

/// Which call a fresh identity is for; each has its own witness, so no two scenarios — and no two
/// calls of one scenario — send the same identity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fresh {
    /// A creation selected by existence, in its own scenario.
    Created,
    /// The same creation again, in the invocation that leaves out every input read only through
    /// an `else:` literal.
    CreatedAgain,
    /// The `nth` row a two-call segment stores before sending its identity again.
    Stored(usize),
}

impl Fresh {
    fn distinction(self) -> Distinction {
        Distinction::further(
            FRESH_BASE
                + match self {
                    Self::Created => 0,
                    Self::CreatedAgain => 1,
                    Self::Stored(nth) => 2 + nth.min(FRESH_SLOTS - 3),
                },
        )
    }
}

/// `true` for a creation beside `existing_instance:`: a `creates:` taking its identity from input,
/// on a command that declares the marker.
fn creates_beside_existing(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    identity_input(outcome).is_some()
        && command
            .outcomes
            .iter()
            .any(|other| other.condition == ResolvedCondition::ExistingInstance)
}

/// The value of the identity field at `fresh`'s witness, refused where the type has too few values
/// to keep it apart from every witness an arrangement, the unknown-identity scenario or another
/// call of this module sends.
fn identity_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    fresh: Fresh,
) -> Result<Node, RefusalCause> {
    let value = |distinction: Distinction| {
        candidates(ir, command, &[], distinction)
            .map_err(RefusalCause::NoWitness)
            .map(|inputs| {
                inputs
                    .into_iter()
                    .next()
                    .and_then(|input| input.get(field).cloned())
            })
    };
    let unfresh = || {
        RefusalCause::NoWitness(WitnessGap {
            path: field.to_owned(),
            type_ref: command
                .input
                .iter()
                .find(|input| input.name == field)
                .map(|input| input.type_ref.to_string())
                .unwrap_or_default(),
            reason: "has too few values to name an identity no other scenario sends, so no \
                     instance is known to be new",
        })
    };
    let at = fresh.distinction();
    let mine = value(at)?.ok_or_else(unfresh)?;
    let taken = (0..=MAX_CANDIDATES)
        .map(Distinction::further)
        .chain([Distinction::UNKNOWN])
        .chain((0..FRESH_SLOTS).map(|slot| Distinction::further(FRESH_BASE + slot)));
    for other in taken.filter(|other| *other != at) {
        if value(other)?.as_ref() == Some(&mine) {
            return Err(unfresh());
        }
    }
    Ok(mine)
}

/// The input a creation selected by existence is sent with in its own scenario: the one that
/// reaches it, with the identity replaced by one no other scenario sends. On a target the
/// scenarios share, the plain witness may be a row another scenario's arrangement made, which
/// would answer the update (create-or-update) or the refusal (create-or-refuse) instead; the
/// unknown-identity witness may be one a creation stored before an ess/15 refusal sends it. `again`
/// is the invocation that leaves out every input read only through an `else:` literal, which
/// creates a second record and needs an identity of its own. Any other branch's input is returned
/// as it is.
pub(super) fn fresh_created(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    mut input: BTreeMap<String, Node>,
    again: bool,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let Some(field) = identity_input(outcome) else {
        return Ok(input);
    };
    if creates_unknown(outcome) || creates_beside_existing(command, outcome) {
        let fresh = if again {
            Fresh::CreatedAgain
        } else {
            Fresh::Created
        };
        input.insert(field.to_owned(), identity_at(ir, command, field, fresh)?);
    }
    Ok(input)
}

/// Both forms, for every command declaring one.
pub(super) fn existence(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for command in ir.commands().values() {
        for creating in command
            .outcomes
            .iter()
            .filter(|outcome| creates_unknown(outcome))
        {
            if let Some((id, scenario)) = outcome_scenario(ir, command, creating, actors, refusals)
            {
                insert(suite, id, scenario, refusals);
            }
            for updating in paired(command, creating) {
                one_row(ir, command, updating, actors, suite, refusals);
            }
        }
        for declared in command
            .outcomes
            .iter()
            .filter(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
        {
            let id = outcome_id(command, declared);
            match existing_instance(ir, command, declared, actors) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(cause) => refusals.push(Refusal::about(&id, cause)),
            }
        }
        refusals_on_a_stored_row(ir, command, actors, suite, refusals);
        refusals_on_an_arranged_row(ir, command, actors, suite, refusals);
    }
}

/// Every input-guarded refusal on a command that addresses an existing record, sent for a record
/// the scenario arranged (beyond10x/ess#209).
///
/// Its own scenario sends the refused input for an identity nothing stored, which the precedence
/// admits: an input refusal is answered before existence (`docs/design/outcome-shapes.md`
/// "Precedence"). A target that looks the record up first answers that send "not found", so this
/// adds the other half, as [`refusals_on_a_stored_row`] does for the two existence forms: the
/// record created through a declared creation and driven to a state the command runs from, the
/// refused input sent for it, and the refusal required with no event and the row unchanged — the
/// last only where some identity view shows the row. Where no arrangement reaches that record, the
/// scenario is withdrawn and refused with the arrangement's cause, never filed to be skipped at
/// run time.
///
/// Left to their own families: the two existence forms (their stored-row half is above), a command
/// reading stored fields (its refusals are already sent for an arranged row), a command whose
/// branches read the held state, and a refusal whose guard reads the identity field, which stays a
/// plain send (beyond10x/ess#178).
fn refusals_on_an_arranged_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    if !creations(command).is_empty()
        || subject_fact::uses(command)
        || super::has_subject_guards(command)
    {
        return;
    }
    // The branch acting on the input-named record: the arrangement it would be sent for.
    let Some(addressing) = command.outcomes.iter().find(|outcome| {
        outcome.subject.as_ref().is_some_and(|subject| {
            subject.effect != ResolvedEffect::Creates
                && matches!(subject.instance, ResolvedInstance::Supplied { .. })
        })
    }) else {
        return;
    };
    for refusal in command.outcomes.iter().filter(|outcome| {
        is_input_guarded_refusal(outcome)
            && outcome.subject.is_none()
            && !subject_fact::reads_identity(command, outcome)
    }) {
        let id = outcome_id(command, refusal);
        let Some(filed) = suite.scenarios.get(&id) else {
            continue;
        };
        let taken = super::bound_instances(&filed.steps);
        match arranged_refusal(ir, command, addressing, refusal, actors, &taken) {
            Ok(part) => {
                let scenario = suite
                    .scenarios
                    .get_mut(&id)
                    .expect("checked to be filed above");
                scenario.steps.extend(part.steps);
                scenario.source.extend(part.source);
            }
            Err(cause) => {
                suite.scenarios.remove(&id);
                refusals.push(Refusal::about(&id, cause));
            }
        }
    }
}

/// The arranged half of [`refusals_on_an_arranged_row`]: the record `addressing` acts on, arranged
/// under names the scenario has not bound, then `refusal`'s input sent for it.
fn arranged_refusal(
    ir: &EssIr,
    command: &ResolvedCommand,
    addressing: &ResolvedOutcome,
    refusal: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &BTreeSet<super::InstanceName>,
) -> Result<Segment, RefusalCause> {
    let subject = addressing
        .subject
        .as_ref()
        .expect("the addressing branch names a subject");
    let error = refusal
        .error
        .as_ref()
        .ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: refusal.test_strategy,
        })?;
    let mut setup = None;
    for distinction in
        std::iter::once(Distinction::PLAIN).chain((1..=MAX_CANDIDATES).map(Distinction::further))
    {
        let arranged = super::prepare_in(ir, addressing, actors, None, distinction)?;
        if super::bound_instances(&arranged.steps).is_disjoint(taken) {
            setup = Some(arranged);
            break;
        }
    }
    let mut setup = setup.ok_or(RefusalCause::StrategyWithoutGuard {
        strategy: refusal.test_strategy,
    })?;
    // Refused, the record stays where the arrangement left it.
    setup.after.clone_from(&setup.before);
    let refused = reach(ir, command, refusal, Distinction::PLAIN)?;
    // Where no identity view shows the arranged row, nothing can read it back, so the half is
    // filed without the unchanged-row claim and still requires the error and no event. The plain
    // send needs no view either, so the scenario is not lost over it (adversary pass 1).
    let observed = identity_views(ir, subject).any(|view| {
        setup.after.as_ref().is_some_and(|state| {
            shows(ir, view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
        })
    });
    let preservation = if observed {
        subject_fact::preserve_refused_subject(ir, subject, &setup)?
    } else {
        subject_fact::Preservation {
            before: Vec::new(),
            after: Vec::new(),
            source: BTreeSet::new(),
            unobserved: Vec::new(),
        }
    };

    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), refusal.name.clone());
    let mut steps = setup.steps.clone();
    steps.extend(preservation.before);
    steps.push(ScenarioStep::ExecuteCommand {
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supply(
            command,
            &refused,
            Some(subject),
            setup.instance.as_ref(),
            &setup.bound,
        ),
        caller: BTreeMap::new(),
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: branch.clone(),
    });
    let mut source: BTreeSet<EssSemanticRef> = setup.source;
    source.extend(preservation.source);
    source.insert(command_ref.into());
    source.insert(branch.into());
    source.insert(EntityRef::from(&subject.entity).into());
    if let Some(actor) = actors.get(&command.name) {
        source.insert(actor.clone().into());
    }
    let named = ErrorRef::from(error);
    steps.push(ScenarioStep::ExpectError {
        error: named.clone(),
        fields: BTreeMap::new(),
    });
    source.insert(named.into());
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));
    steps.extend(preservation.after);
    Ok(Segment { steps, source })
}

/// The scenario id of one outcome.
fn outcome_id(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> ScenarioId {
    ScenarioId::Outcome {
        outcome: OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone()),
    }
}

/// The siblings of a creating `unknown_instance:` branch that act on the record its identity names:
/// the branches a second call with that identity takes.
fn paired<'c>(
    command: &'c ResolvedCommand,
    creating: &'c ResolvedOutcome,
) -> impl Iterator<Item = &'c ResolvedOutcome> {
    let entity = creating.subject.as_ref().map(|subject| &subject.entity);
    let identity = identity_input(creating);
    command.outcomes.iter().filter(move |outcome| {
        outcome.subject.as_ref().is_some_and(|subject| {
            Some(&subject.entity) == entity
                && matches!(
                    subject.effect,
                    ResolvedEffect::Moves { .. } | ResolvedEffect::Updates
                )
                && matches!(&subject.instance, ResolvedInstance::Supplied { field }
                    if Some(field.name.as_str()) == identity)
        })
    })
}

/// The row-level views of `subject`'s entity that project its identity under its own type: the
/// views a claim about one row for that identity could be read from.
fn identity_views<'i>(
    ir: &'i EssIr,
    subject: &'i ResolvedSubject,
) -> impl Iterator<Item = &'i ResolvedView> {
    let entity = ir.entity(&subject.entity);
    ir.views().values().filter(move |view| {
        !view.is_aggregate()
            && view.source == subject.entity
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    })
}

/// Whether the one-row snapshot can be read from `view` after the updating branch: an immediate,
/// unparameterised view whose filter — where it has one — is decided to admit the row the branch
/// leaves, from its state and the fields the scenario determined.
fn one_row_view(ir: &EssIr, view: &ResolvedView, run: &Run) -> bool {
    super::paging::read_whole(view)
        && view.consistency == Consistency::ReadYourWrites
        && view.assertion_style == AssertionStyle::Expect
        && (view.filter.is_none()
            || run.after.as_ref().is_some_and(|state| {
                shows(ir, view, state, &run.settled, &BTreeMap::new()) == Ok(true)
            }))
}

/// Adds to the updating branch's own scenario that exactly one row carries the identity after the
/// second call: a snapshot selects exactly one row or fails, so an implementation that stored a
/// second row for the identity rather than updating the first is caught even where it answered the
/// update branch.
///
/// Read from every immediate view the row is decided to appear in, a filtered one included. Where
/// views project the identity and none of them can carry the claim — each eventual, parameterised
/// or filtered by something the scenario cannot decide — the scenario is withdrawn and refused
/// naming them, rather than filed without the claim. A model with no view projecting the identity
/// observes no row at all, for this branch or any update, and keeps its scenario.
fn one_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    updating: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let Some(subject) = &updating.subject else {
        return;
    };
    let id = ScenarioId::Outcome {
        outcome: OutcomeRef::new(CommandRef::new(command.name.clone()), updating.name.clone()),
    };
    if !suite.scenarios.contains_key(&id) {
        return;
    }
    let candidates: Vec<&ResolvedView> = identity_views(ir, subject).collect();
    if candidates.is_empty() {
        return;
    }
    // The run the scenario was built from, again: synthesis is deterministic, so this is the row
    // the scenario's second call leaves, and the instance its arrangement bound.
    let Ok(run) = run(ir, command, updating, actors) else {
        return;
    };
    let usable: Vec<&ResolvedView> = candidates
        .iter()
        .copied()
        .filter(|view| one_row_view(ir, view, &run))
        .collect();
    let Some(instance) = run.instance.clone().filter(|_| !usable.is_empty()) else {
        suite.scenarios.remove(&id);
        refusals.push(Refusal::about(
            &id,
            RefusalCause::NoWitness(WitnessGap {
                path: candidates
                    .iter()
                    .map(|view| view.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                type_ref: subject.entity.to_string(),
                reason: "the one-row check after the second call needs an immediate, \
                         unparameterised view whose filter is decided to admit the row, and \
                         none of these is",
            }),
        ));
        return;
    };
    let scenario = suite
        .scenarios
        .get_mut(&id)
        .expect("checked to be filed above");
    let identity = ir.entity(&subject.entity).identity.name.clone();
    for view in usable {
        let name = ViewRef::new(view.name.clone());
        scenario.steps.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        scenario.steps.push(ScenarioStep::SnapshotSubject {
            view: name.clone(),
            subject: [(identity.clone(), ScenarioValue::instance(instance.clone()))]
                .into_iter()
                .collect(),
        });
        scenario.source.insert(name.into());
    }
}

/// The creating branches of a command selected by existence whose identity the input supplies:
/// the creation of create-or-update, or every creation beside `existing_instance:`.
fn creations(command: &ResolvedCommand) -> Vec<(&ResolvedOutcome, &str)> {
    command
        .outcomes
        .iter()
        .filter(|outcome| creates_unknown(outcome) || creates_beside_existing(command, outcome))
        .filter_map(|outcome| identity_input(outcome).map(|field| (outcome, field)))
        .collect()
}

/// One segment of calls sharing an identity: `creating` stores a row under a fresh identity, the
/// row is snapshotted, `second` is sent for that identity, and `declared` must answer with its
/// error, no event, and the row as it was.
struct Segment {
    steps: Vec<ScenarioStep>,
    source: BTreeSet<EssSemanticRef>,
}

fn segment(
    ir: &EssIr,
    command: &ResolvedCommand,
    creating: (&ResolvedOutcome, &str),
    second: BTreeMap<String, Node>,
    declared: &ResolvedOutcome,
    fresh: Fresh,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Segment, RefusalCause> {
    let (creating, field) = creating;
    let no_error = || RefusalCause::StrategyWithoutGuard {
        strategy: declared.test_strategy,
    };
    let subject = creating.subject.as_ref().ok_or_else(no_error)?;
    let error = declared.error.as_ref().ok_or_else(no_error)?;
    let at = fresh.distinction();
    let identity = identity_at(ir, command, field, fresh)?;
    let mut first = reach(ir, command, creating, at)?;
    first.insert(field.to_owned(), identity.clone());
    let mut second = second;
    second.insert(field.to_owned(), identity);

    let driver = Driver {
        command,
        outcome: creating,
        effect: &subject.effect,
    };
    let arrangement = created(ir, &subject.entity, &driver, actors, at, &[], Some(&first))
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(&subject.entity),
            need: InstanceNeed::InState {
                state: ir.entity(&subject.entity).lifecycle.initial.clone(),
            },
            reason,
        })?;
    let setup = Setup {
        steps: arrangement.steps,
        instance: Some(arrangement.instance),
        bound: BTreeMap::new(),
        source: arrangement.source,
        after: Some(arrangement.state),
        before: None,
        settled: arrangement.settled,
    };
    let preservation = subject_fact::preserve_refused_subject(ir, subject, &setup)?;

    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), declared.name.clone());
    let mut steps = setup.steps.clone();
    steps.extend(preservation.before);
    steps.push(ScenarioStep::ExecuteCommand {
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supply(command, &second, None, None, &BTreeMap::new()),
        caller: BTreeMap::new(),
    });
    steps.push(ScenarioStep::ExpectOutcome {
        outcome: branch.clone(),
    });
    let mut source: BTreeSet<EssSemanticRef> = setup.source;
    source.extend(preservation.source);
    source.insert(command_ref.into());
    source.insert(branch.into());
    source.insert(EntityRef::from(&subject.entity).into());
    if let Some(actor) = actors.get(&command.name) {
        source.insert(actor.clone().into());
    }
    let named = ErrorRef::from(error);
    steps.push(ScenarioStep::ExpectError {
        error: named.clone(),
        fields: BTreeMap::new(),
    });
    source.insert(named.into());
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));
    steps.extend(preservation.after);
    Ok(Segment { steps, source })
}

/// The create-or-refuse witness: for **every** creating branch, create with a fresh identity,
/// snapshot the row, send the same identity again through that branch's own input with other
/// field values, and require the declared error, no event and the row as it was. A target that
/// looks for the stored record on one creating path and not another is caught on the other.
fn existing_instance(
    ir: &EssIr,
    command: &ResolvedCommand,
    declared: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let creating = creations(command);
    if creating.is_empty() {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: declared.test_strategy,
        });
    }
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    for (nth, (outcome, field)) in creating.into_iter().enumerate() {
        let fresh = Fresh::Stored(nth);
        let first = reach(ir, command, outcome, fresh.distinction())?;
        // Other field values where the branch's own input allows them, so an overwrite is visible.
        let mut second = first.clone();
        for further in 0..=FRESH_WITNESSES {
            let Ok(other) = reach(ir, command, outcome, Distinction::further(further)) else {
                continue;
            };
            let differs = other
                .iter()
                .any(|(key, value)| key != field && first.get(key) != Some(value));
            if differs {
                second = other;
                break;
            }
        }
        let part = segment(
            ir,
            command,
            (outcome, field),
            second,
            declared,
            fresh,
            actors,
        )?;
        steps.extend(part.steps);
        source.extend(part.source);
    }
    let text = format!(
        "`{}` sent twice for one identity takes `{}` the second time, on every creating path, and \
         leaves the row as the first call made it",
        command.name, declared.name
    );
    Ok(ConformanceScenario::new(clipped(&text), steps, source))
}

/// Every input-guarded refusal beside either form is answered before existence (the coordinator's
/// decision, `docs/design/outcome-shapes.md` "Precedence"). Its own scenario sends the refused
/// input for an identity nothing stored; this adds the other half — a row stored under a fresh
/// identity, the refused input sent for that identity, and the refusal required with no event and
/// the row unchanged — so a target answering existence first is caught. Where the half cannot be
/// built the scenario is withdrawn and refused, rather than filed claiming the precedence it does
/// not witness.
fn refusals_on_a_stored_row(
    ir: &EssIr,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let creating = creations(command);
    let Some(first_creation) = creating.first().copied() else {
        return;
    };
    for (nth, refusal) in command
        .outcomes
        .iter()
        .filter(|outcome| is_input_guarded_refusal(outcome))
        .enumerate()
    {
        let id = outcome_id(command, refusal);
        if !suite.scenarios.contains_key(&id) {
            continue;
        }
        let built = reach(ir, command, refusal, Distinction::PLAIN).and_then(|refused| {
            segment(
                ir,
                command,
                first_creation,
                refused,
                refusal,
                Fresh::Stored(creating.len() + nth),
                actors,
            )
        });
        match built {
            Ok(part) => {
                let scenario = suite
                    .scenarios
                    .get_mut(&id)
                    .expect("checked to be filed above");
                scenario.steps.extend(part.steps);
                scenario.source.extend(part.source);
            }
            Err(cause) => {
                suite.scenarios.remove(&id);
                refusals.push(Refusal::about(&id, cause));
            }
        }
    }
}
