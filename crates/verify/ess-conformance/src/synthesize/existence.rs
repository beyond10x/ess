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

use super::caller::{InvocationModels, InvocationPhase};
use super::{
    clipped, created, insert, is_input_guarded_refusal, not_emitted, reach, shows, subject_fact,
    supply, ActorRef, AssertionStyle, BTreeMap, BTreeSet, CommandRef, ConformanceScenario,
    ConformanceSuite, Distinction, EntityRef, ErrorRef, EssIr, EssSemanticRef, Focus, InstanceNeed,
    Node, OutcomeRef, QualifiedName, Refusal, RefusalCause, ResolvedCommand, ResolvedCondition,
    ResolvedEffect, ResolvedInstance, ResolvedOutcome, ResolvedSubject, ResolvedView, Run,
    ScenarioId, ScenarioStep, ScenarioValue, Setup, ViewRef, FRESH_WITNESSES,
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
pub(super) fn identity_input(outcome: &ResolvedOutcome) -> Option<&str> {
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
    /// The same creation in the `n`th further fallback run after the first (ess/22, A4), each
    /// leaving out one Optional.
    CreatedAgainIn(usize),
    /// The `nth` row a two-call segment stores before sending its identity again.
    Stored(usize),
    /// The `nth` candidate for a further boundary row of a creation the input names
    /// (beyond10x/ess#471). [`boundary_identity`] skips one the scenario already sent.
    Boundary(usize),
}

impl Fresh {
    fn slot(self) -> usize {
        match self {
            Self::Created => 0,
            Self::CreatedAgain => 1,
            Self::CreatedAgainIn(run) => FRESH_SLOTS - 16 + run.min(15),
            Self::Stored(nth) | Self::Boundary(nth) => 2 + nth.min(FRESH_SLOTS - 3),
        }
    }

    fn distinction(self) -> Distinction {
        Distinction::further(FRESH_BASE + self.slot())
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
///
/// Where that value would change the branch `input` takes — a guard reads the identity — the
/// slot's own value inside the guards is taken instead ([`super::guided_identity`]), after the one
/// the unknown-identity scenario takes, and the scenario is refused where the guards leave none
/// (beyond10x/ess#275).
///
/// An identity whose type has one value is that value wherever the scenario creates one row, and
/// refused for [`Fresh::CreatedAgain`], a second row in the same run (beyond10x/ess#287).
fn identity_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    fresh: Fresh,
    input: &BTreeMap<String, Node>,
) -> Result<Node, RefusalCause> {
    let value = |distinction: Distinction| {
        candidates(ir, command, &[], distinction)
            .map_err(RefusalCause::NoWitness)
            .map(|inputs| {
                inputs
                    .into_iter()
                    .next()
                    .and_then(|input| ess_compiler::ir::read_input(&input, field).cloned())
            })
    };
    let unfresh = || {
        RefusalCause::NoWitness(WitnessGap {
            path: field.to_owned(),
            type_ref: super::input_type(ir, command, field)
                .map(ToString::to_string)
                .unwrap_or_default(),
            reason: "has too few values to name an identity no other scenario sends, so no \
                     instance is known to be new",
        })
    };
    let at = fresh.distinction();
    let mine = value(at)?.ok_or_else(unfresh)?;
    if !super::keeps_branch(ir, command, input, field, &mine) {
        return super::guided_identity(ir, command, field, input, 1 + fresh.slot())
            .ok_or_else(|| super::unguided(command, field));
    }
    // The one row of a singleton entity is new in the scenario that creates it first, and a second
    // creation in the same run has no identity of its own (beyond10x/ess#287).
    if super::singleton::names_the_one_row(ir, command, field) {
        return match fresh {
            Fresh::CreatedAgain | Fresh::CreatedAgainIn(_) | Fresh::Boundary(_) => {
                Err(super::singleton::second_row(command, field))
            }
            Fresh::Created | Fresh::Stored(_) => Ok(mine),
        };
    }
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

/// The identity a further boundary row of a creation the input names is sent with
/// (beyond10x/ess#471): the first [`Fresh::Boundary`] value that keeps `input` on its branch and
/// that this scenario has not `sent` yet. `creates:` never replaces the instance an identity
/// already names, so a row sent the plain witness's identity again asks for a branch the model does
/// not describe.
///
/// `None` where the type has no such value — a singleton's one row, a `Boolean` — and the row is
/// then not sent.
pub(super) fn boundary_identity(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    input: &BTreeMap<String, Node>,
    sent: &BTreeSet<Node>,
) -> Option<Node> {
    (0..FRESH_SLOTS - 2).find_map(|nth| {
        identity_at(ir, command, field, Fresh::Boundary(nth), input)
            .ok()
            .filter(|identity| !sent.contains(identity))
    })
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
        let identity = identity_at(ir, command, field, fresh, &input)?;
        super::set_at(&mut input, field, Some(identity));
    }
    Ok(input)
}

/// [`fresh_created`] for the `run`th further fallback run: the first takes the identity the one run
/// always took, and every later one (ess/22, A4) one of its own.
pub(super) fn fresh_created_again(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    mut input: BTreeMap<String, Node>,
    run: usize,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    if run == 0 {
        return fresh_created(ir, command, outcome, input, true);
    }
    let Some(field) = identity_input(outcome) else {
        return Ok(input);
    };
    if creates_unknown(outcome) || creates_beside_existing(command, outcome) {
        let identity = identity_at(ir, command, field, Fresh::CreatedAgainIn(run), &input)?;
        super::set_at(&mut input, field, Some(identity));
    }
    Ok(input)
}

/// Both forms, for every command declaring one.
pub(super) fn existence(
    models: &InvocationModels<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    focus: Focus<'_>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for command in models.acting.commands().values() {
        if !focus.takes(&command.name) {
            continue;
        }
        for creating in command
            .outcomes
            .iter()
            .filter(|outcome| creates_unknown(outcome))
        {
            if let Some((id, scenario)) =
                super::outcome_scenario_in(models, command, creating, actors, refusals)
            {
                insert(suite, id, scenario, refusals);
            }
            for updating in paired(command, creating) {
                one_row(models, command, updating, actors, suite, refusals);
            }
        }
        for declared in command
            .outcomes
            .iter()
            .filter(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
        {
            let id = outcome_id(command, declared);
            match existing_instance(models, command, declared, actors) {
                Ok(scenario) => insert(suite, id, scenario, refusals),
                Err(cause) => refusals.push(Refusal::about(&id, cause)),
            }
        }
        refusals_on_a_stored_row(models, command, actors, suite, refusals);
        refusals_on_an_arranged_row(models, command, actors, suite, refusals);
        refusals_in_each_held_state(models, command, actors, suite, refusals);
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
    models: &InvocationModels<'_>,
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
        match arranged_refusal(models, command, addressing, refusal, actors, &taken) {
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
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    addressing: &ResolvedOutcome,
    refusal: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &BTreeSet<super::InstanceName>,
) -> Result<Segment, RefusalCause> {
    let ir = models.arrangement;
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
    models.mark(InvocationPhase::Arrange, &mut steps);
    steps.extend(preservation.before);
    let supplied = supply(
        ir,
        command,
        &refused,
        Some(subject),
        setup.instance.as_ref(),
        &setup.bound,
    );
    let expected = super::expect_error(ir, refusal, error, &supplied, &setup.settled);
    steps.push(ScenarioStep::ExecuteCommand {
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supplied,
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
    steps.push(expected);
    source.insert(named.into());
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));
    steps.extend(preservation.after);
    models.mark(InvocationPhase::Act, &mut steps);
    Ok(Segment { steps, source })
}

/// Every input-guarded refusal naming no subject beside branches that read the held state, sent
/// again for a row arranged in each held state (beyond10x/ess#227).
///
/// The refusal is answered before existence and before the held state, so its own scenario opens
/// with the refused input for an identity nothing stored, as [`refusals_on_an_arranged_row`] keeps
/// for a command without held-state branches. This adds the other half once per state of the
/// subject's lifecycle, in declaration order, each on a record of its own: a row arranged in that
/// state and observed there, the refused input sent for it, and the refusal required with its
/// error, no event and the row unchanged. Every held state is one a sibling answers in — the
/// joint partition proves some branch is selected for every state — so a target reading the held
/// state first answers a sibling in each and fails there, and a target that checks the input in
/// every state but one fails in that one.
///
/// Where a sibling that runs in a state also reads the input, the refused input is sent there
/// again at their overlap, refuting every other input refusal: a target that tries the sibling
/// first takes it. Where no arrangement reaches a state, the scenario is withdrawn and refused
/// with the arrangement's cause, never left without that state's witness.
fn refusals_in_each_held_state(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for refusal in command.outcomes.iter().filter(|outcome| {
        super::is_state_input_refusal(command, outcome)
            && !subject_fact::reads_identity(command, outcome)
    }) {
        let id = outcome_id(command, refusal);
        let Some(filed) = suite.scenarios.get(&id) else {
            // The plain send was refused under this id already, with its cause.
            continue;
        };
        let taken = super::bound_instances(&filed.steps);
        match held_state_refusals(models, command, refusal, actors, taken) {
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

/// The segments of [`refusals_in_each_held_state`] for one refusal, one per held state and input.
fn held_state_refusals(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    refusal: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    mut taken: BTreeSet<super::InstanceName>,
) -> Result<Segment, RefusalCause> {
    let ir = models.arrangement;
    let without = || RefusalCause::StrategyWithoutGuard {
        strategy: refusal.test_strategy,
    };
    let error = refusal.error.as_ref().ok_or_else(without)?;
    let subject = subject_fact::common(command).ok_or_else(without)?;
    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), refusal.name.clone());
    let named = ErrorRef::from(error);
    let forbidden = not_emitted(ir, &[]);
    let mut part = Segment {
        steps: Vec::new(),
        source: BTreeSet::new(),
    };
    part.source.insert(command_ref.clone().into());
    part.source.insert(branch.clone().into());
    part.source.insert(named.clone().into());
    part.source.insert(EntityRef::from(&subject.entity).into());
    part.source
        .extend(forbidden.iter().cloned().map(EssSemanticRef::from));
    if let Some(actor) = actors.get(&command.name) {
        part.source.insert(actor.clone().into());
    }
    let mut nth = 0;
    for state in &ir.entity(&subject.entity).lifecycle.states {
        for input in refused_inputs_in(ir, command, refusal, state)? {
            // Each send on a record of its own, under names nothing earlier in the scenario
            // bound: the unchanged-row comparison spans one command.
            let mut arrangement =
                arranged_in(ir, subject, state, actors, &mut nth, &mut taken, without)?;
            let setup = Setup {
                instance: Some(arrangement.instance.clone()),
                after: Some(state.clone()),
                before: Some(state.clone()),
                settled: arrangement.settled.clone(),
                ..Setup::none()
            };
            models.mark(InvocationPhase::Arrange, &mut arrangement.steps);
            part.steps.extend(arrangement.steps);
            part.source.extend(arrangement.source);
            // Observed in the state it was arranged in, where a view shows it; the held-state
            // siblings need that view themselves, so its absence is theirs to refuse.
            let preservation = match super::observe_selection_subject(
                ir,
                subject,
                &arrangement.instance,
                state,
                &refusal.name.to_string(),
            ) {
                Ok((observed, view)) => {
                    part.steps.extend(observed);
                    part.source.insert(view.into());
                    subject_fact::preserve_refused_subject(ir, subject, &setup)?
                }
                Err(_) => subject_fact::Preservation {
                    before: Vec::new(),
                    after: Vec::new(),
                    source: BTreeSet::new(),
                    unobserved: Vec::new(),
                },
            };
            part.steps.extend(preservation.before);
            part.source.extend(preservation.source);
            let supplied = supply(
                ir,
                command,
                &input,
                Some(subject),
                Some(&arrangement.instance),
                &BTreeMap::new(),
            );
            let expected = super::expect_error(ir, refusal, error, &supplied, &setup.settled);
            part.steps.push(ScenarioStep::ExecuteCommand {
                command: command_ref.clone(),
                actor: actors.get(&command.name).cloned(),
                input: supplied,
                caller: BTreeMap::new(),
            });
            part.steps.push(ScenarioStep::ExpectOutcome {
                outcome: branch.clone(),
            });
            part.steps.push(expected);
            for event in &forbidden {
                part.steps.push(ScenarioStep::ExpectNoEvent {
                    event: event.clone(),
                });
            }
            part.steps.extend(preservation.after);
        }
    }
    models.mark(InvocationPhase::Act, &mut part.steps);
    Ok(part)
}

/// The inputs `refusal` is sent in held state `state`: the one [`super::reach_in_state`] decides
/// there, then, for each sibling that runs in `state` and reads the input, one both guards admit
/// that refutes every other input refusal — the send a target trying that sibling first answers
/// wrongly.
fn refused_inputs_in(
    ir: &EssIr,
    command: &ResolvedCommand,
    refusal: &ResolvedOutcome,
    state: &ess_domain::entity::StateName,
) -> Result<Vec<BTreeMap<String, Node>>, RefusalCause> {
    let mut inputs = vec![super::reach_in_state(
        ir,
        command,
        refusal,
        state,
        Distinction::PLAIN,
    )?];
    let Some(own) = super::when(refusal) else {
        return Ok(inputs);
    };
    let others: Vec<_> = super::sibling_refusals(command, refusal)
        .filter_map(super::when)
        .collect();
    for sibling in command.outcomes.iter().filter(|sibling| {
        sibling.name != refusal.name
            && !super::is_state_input_refusal(command, sibling)
            && super::admits_held_state(&sibling.condition, state)
    }) {
        let Some(guard) = super::when(sibling) else {
            continue;
        };
        let mut searched = vec![own, guard];
        searched.extend(others.iter().copied());
        let Ok(candidates) = candidates(ir, command, &searched, Distinction::PLAIN) else {
            continue;
        };
        let found = candidates.into_iter().find(|input| {
            super::flatten(ir, command, input).is_ok_and(|facts| {
                super::decides(&facts, &[own, guard], true).unwrap_or(false)
                    && super::decides(&facts, &others, false).unwrap_or(false)
            })
        });
        if let Some(input) = found.filter(|input| !inputs.contains(input)) {
            inputs.push(input);
        }
    }
    Ok(inputs)
}

/// A record of `subject`'s entity arranged in `state` under names outside `taken`, which it then
/// joins; `nth` is the last distinction tried, so every call draws further ones.
fn arranged_in(
    ir: &EssIr,
    subject: &ResolvedSubject,
    state: &ess_domain::entity::StateName,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    nth: &mut usize,
    taken: &mut BTreeSet<super::InstanceName>,
    without: impl Fn() -> RefusalCause,
) -> Result<super::Arrangement, RefusalCause> {
    let mut last = None;
    while *nth <= MAX_CANDIDATES {
        *nth += 1;
        match super::arrange_first(
            ir,
            &subject.entity,
            std::slice::from_ref(state),
            actors,
            Distinction::further(*nth),
            &[],
        ) {
            Ok(found) if super::bound_instances(&found.steps).is_disjoint(taken) => {
                taken.extend(super::bound_instances(&found.steps));
                return Ok(found);
            }
            Ok(_) => {}
            Err(reason) => last = Some(reason),
        }
    }
    Err(match last {
        Some(reason) => RefusalCause::InstanceRequired {
            entity: EntityRef::from(&subject.entity),
            need: InstanceNeed::InState {
                state: state.clone(),
            },
            reason,
        },
        None => without(),
    })
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
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    updating: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let ir = models.arrangement;
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
    let Ok(run) = super::run_as(models, command, updating, actors, super::Witness::Full) else {
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

/// The segment, or `None` where `declared` is an input-guarded refusal no input sent for the
/// stored identity selects: its guard is decided by the identity alone, which every stored row
/// refutes, so no target can answer existence before it and the plain send witnesses it alone
/// (beyond10x/ess#479). Sent the stored identity over the refused input, `{item_id: ""}` became
/// `{item_id: item_id-1048595}`, which `item_id == ""` refutes. Where another input keeps the guard
/// with the stored identity — `item_id == "" or title == ""` sent `title: ""` — that one is sent.
fn segment(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    creating: (&ResolvedOutcome, &str),
    second: BTreeMap<String, Node>,
    declared: &ResolvedOutcome,
    fresh: Fresh,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Option<Segment>, RefusalCause> {
    let ir = models.arrangement;
    let (creating, field) = creating;
    let creator = &ir.commands()[&command.name];
    let creating = creator
        .outcomes
        .iter()
        .find(|outcome| outcome.name == creating.name)
        .expect("caller interpretation preserves source outcomes");
    let no_error = || RefusalCause::StrategyWithoutGuard {
        strategy: declared.test_strategy,
    };
    let subject = creating.subject.as_ref().ok_or_else(no_error)?;
    let error = declared.error.as_ref().ok_or_else(no_error)?;
    let at = fresh.distinction();
    let mut first = creating_input(ir, creator, creating, at, actors)?;
    let identity = identity_at(ir, creator, field, fresh, &first)?;
    super::set_at(&mut first, field, Some(identity.clone()));
    let second = if declared.condition == ResolvedCondition::ExistingInstance {
        let mut second = second;
        super::set_at(&mut second, field, Some(identity));
        second
    } else {
        match super::reach_pinned(ir, command, declared, second, field, &identity)? {
            Some(second) => second,
            None => return Ok(None),
        }
    };

    let driver = Driver {
        command: creator,
        outcome: creating,
        effect: &subject.effect,
    };
    let mut arrangement = created(ir, &subject.entity, &driver, actors, at, &[], Some(&first))
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(&subject.entity),
            need: InstanceNeed::InState {
                state: ir.entity(&subject.entity).lifecycle.initial.clone(),
            },
            reason,
        })?;
    // Only the existence refusal carries recreation; an input refusal using this same helper
    // keeps its ordinary two-call precedence check.
    models.mark(InvocationPhase::Arrange, &mut arrangement.steps);
    let recreation = if declared.condition == ResolvedCondition::ExistingInstance {
        recreation(ir, creator, creating, &arrangement, actors)?
    } else {
        None
    };
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
    steps.extend(preservation.before.iter().cloned());
    let supplied = supply(ir, command, &second, None, None, &BTreeMap::new());
    let expected = super::expect_error(ir, declared, error, &supplied, &setup.settled);
    steps.push(ScenarioStep::ExecuteCommand {
        command: command_ref.clone(),
        actor: actors.get(&command.name).cloned(),
        input: supplied,
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
    steps.push(expected);
    source.insert(named.into());
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));
    steps.extend(preservation.after);
    models.mark(InvocationPhase::Act, &mut steps);
    if let Some((deleted, again)) = recreation {
        let mut part = recreated(ir, creator, creating, deleted, again, preservation.before);
        models.mark(InvocationPhase::Arrange, &mut part.steps);
        steps.extend(part.steps);
        source.extend(part.source);
    }
    Ok(Some(Segment { steps, source }))
}

fn recreated(
    ir: &EssIr,
    command: &ResolvedCommand,
    creating: &ResolvedOutcome,
    deleted: super::Arrangement,
    again: ScenarioStep,
    observed: Vec<ScenarioStep>,
) -> Segment {
    let mut steps = deleted.steps;
    let mut source = deleted.source;
    // The same immediate projections that saw the original row must now hold none of it.
    for step in &observed {
        steps.push(match step {
            ScenarioStep::SnapshotSubject { view, subject }
            | ScenarioStep::SnapshotCompleteSubject { view, subject, .. } => {
                ScenarioStep::ExpectSubjectAbsent {
                    view: view.clone(),
                    subject: subject.clone(),
                }
            }
            query => query.clone(),
        });
    }
    let ScenarioStep::ExecuteCommand { input, .. } = &again else {
        unreachable!("the original creating invocation")
    };
    let events: Vec<_> = creating
        .emits
        .iter()
        .map(|event| {
            let event = super::EventRef::from(event);
            source.insert(event.clone().into());
            ScenarioStep::ExpectEvent {
                payload: super::determined_payload(ir, creating, &event, input, &BTreeMap::new()),
                shape: crate::response::event_shape(ir, &event, creating),
                event,
            }
        })
        .collect();
    let outcome = OutcomeRef::new(CommandRef::new(command.name.clone()), creating.name.clone());
    // The first invocation consumed its external control. Recreating through that same branch
    // requires a new one immediately before the repeated invocation, as invoke_with supplies.
    if creating.test_strategy == ess_domain::command::TestStrategy::InjectFault {
        steps.push(ScenarioStep::ConfigureExternalOutcome {
            force: outcome.clone(),
            times: None,
        });
    }
    steps.push(again);
    steps.push(ScenarioStep::ExpectOutcome { outcome });
    steps.push(ScenarioStep::ExpectNoError);
    steps.extend(events);
    // Exactly one recreated row, without comparing generated fields to its former incarnation.
    steps.extend(observed.into_iter().map(|step| match step {
        ScenarioStep::SnapshotCompleteSubject { view, subject, .. } => {
            ScenarioStep::SnapshotSubject { view, subject }
        }
        other => other,
    }));
    Segment { steps, source }
}

fn recreation(
    ir: &EssIr,
    command: &ResolvedCommand,
    creating: &ResolvedOutcome,
    arrangement: &super::Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Option<(super::Arrangement, ScenarioStep)>, RefusalCause> {
    let entity = &creating.subject.as_ref().expect("creating branch").entity;
    let Some(deleted) = subject_fact::delete_existing(ir, entity, arrangement.clone(), actors)?
    else {
        return Ok(None);
    };
    if super::related_guard::uses(command) {
        return Err(RefusalCause::NoWitness(WitnessGap {
            path: command.name.to_string(),
            type_ref: entity.to_string(),
            reason: "recreation has not proved that the creating command's related-row guards still hold after deletion",
        }));
    }
    let again = arrangement
        .steps
        .iter()
        .rev()
        .find(|step| {
            matches!(step,
                ScenarioStep::ExecuteCommand { command: sent, .. } if sent.name() == &command.name
            )
        })
        .expect("the arrangement invoked its creator")
        .clone();
    Ok(Some((deleted, again)))
}

/// The input a creating branch is sent with: the one that reaches it, or — for a command reading a
/// related row (ess/18), which only an arrangement of that row reaches — a plain one, which the
/// creation then sends with the related row arranged for it.
fn creating_input(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    if super::related_guard::uses(command) {
        super::related_guard::prepare_at_in(
            &InvocationModels::plain(ir),
            command,
            outcome,
            actors,
            distinction,
            None,
        )
        .map(|(_, input)| input)
    } else {
        reach(ir, command, outcome, distinction)
    }
}

/// The create-or-refuse witness: for **every** creating branch, create with a fresh identity,
/// snapshot the row, send the same identity again through that branch's own input with other
/// field values, and require the declared error, no event and the row as it was. A target that
/// looks for the stored record on one creating path and not another is caught on the other.
fn existing_instance(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    declared: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let ir = models.arrangement;
    let creating = creations(command);
    if creating.is_empty() {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: declared.test_strategy,
        });
    }
    // Every creating path stores its own row in this one run, which the one row of a singleton
    // entity cannot be twice (beyond10x/ess#287).
    if let [_, (_, field), ..] = creating.as_slice() {
        if super::singleton::names_the_one_row(ir, command, field) {
            return Err(super::singleton::second_row(command, field));
        }
    }
    let mut steps = Vec::new();
    let mut source = BTreeSet::new();
    for (nth, (outcome, field)) in creating.into_iter().enumerate() {
        let fresh = Fresh::Stored(nth);
        let first = creating_input(ir, command, outcome, fresh.distinction(), actors)?;
        // Other field values where the branch's own input allows them, so an overwrite is visible.
        let mut second = first.clone();
        for further in 0..=FRESH_WITNESSES {
            let Ok(other) =
                creating_input(ir, command, outcome, Distinction::further(further), actors)
            else {
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
        // Beside a related guard (ess/18, #211) the second call names a related row that does not
        // exist: the identity is checked first, so `existing_instance:` still answers.
        if super::related_guard::uses(command) {
            super::related_guard::point_at_missing(ir, command, &mut second)?;
        }
        // The existence refusal is sent the stored identity whatever its input, so a segment is
        // always built for it.
        if let Some(part) = segment(
            models,
            command,
            (outcome, field),
            second,
            declared,
            fresh,
            actors,
        )? {
            steps.extend(part.steps);
            source.extend(part.source);
        }
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
/// not witness. Where no input sent for a stored identity selects the refusal — a guard decided by
/// the identity alone, which no stored row meets — there is no half to build, and the plain send
/// stands alone (beyond10x/ess#479).
fn refusals_on_a_stored_row(
    models: &InvocationModels<'_>,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let ir = models.arrangement;
    // Related-row selection declares existence first, ahead of every input refusal. Its stored
    // row belongs to existing_instance; adding an input-refusal half here would assert the wrong
    // precedence (outcome-shapes.md, "Precedence").
    if super::related_guard::uses(command) {
        return;
    }
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
                models,
                command,
                first_creation,
                refused,
                refusal,
                Fresh::Stored(creating.len() + nth),
                actors,
            )
        });
        match built {
            Ok(Some(part)) => {
                let scenario = suite
                    .scenarios
                    .get_mut(&id)
                    .expect("checked to be filed above");
                scenario.steps.extend(part.steps);
                scenario.source.extend(part.source);
            }
            // No stored identity selects the refusal: its plain send is its whole witness.
            Ok(None) => {}
            Err(cause) => {
                suite.scenarios.remove(&id);
                refusals.push(Refusal::about(&id, cause));
            }
        }
    }
}
