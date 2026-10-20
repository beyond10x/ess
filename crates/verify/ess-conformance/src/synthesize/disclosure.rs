//! Finite source-owned disclosure inventory, with named refusals for unarrangeable cells.
use super::caller::{InvocationModels, Readings};
use super::{
    exercise, granted_actors, has_subject_guards, purpose, related_guard, subject_fact, ActorRef,
    BTreeMap, CommandRef, ConformanceScenario, Distinction, EssIr, EventRef, OutcomeRef,
    QualifiedName, Refusal, RefusalCause, ResolvedCommand, ResolvedCondition, ResolvedOutcome,
    ScenarioId, ScenarioStep, ScenarioValue, Synthesis, ViewRef, WitnessGap,
};
use crate::one_time_response::{Aspect, Cell};

pub(super) fn augment(ir: &EssIr, synthesis: &mut Synthesis) {
    if !crate::one_time_response::marked_model(ir) {
        return;
    }
    for (id, scenario) in &mut synthesis.suite.scenarios {
        if let Err(reason) = crate::one_time_response::produce::attach(ir, scenario) {
            synthesis.refusals.push(refusal(id, &reason));
        }
    }
    let defaults = granted_actors(ir);
    let readings = super::caller::Readings::of(ir);
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            if outcome.one_time_response.is_empty() {
                continue;
            }
            let origin =
                OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone());
            for actor in originating_actors(ir, command) {
                let mut assigned = defaults.clone();
                if let Some(actor) = &actor {
                    assigned.insert(command.name.clone(), actor.clone());
                }
                for field in &outcome.one_time_response {
                    for aspect in [Aspect::Origin, Aspect::Retry, Aspect::Rotation] {
                        let id = ScenarioId::Disclosure {
                            cell: Box::new(Cell {
                                origin: origin.clone(),
                                field: field.clone(),
                                aspect: aspect.clone(),
                                actor: actor.clone(),
                            }),
                        };
                        let Some((steps, source, origin_run)) = exercise(
                            ir,
                            command,
                            outcome,
                            &assigned,
                            &id,
                            &mut synthesis.refusals,
                        ) else {
                            continue;
                        };
                        let mut scenario =
                            ConformanceScenario::new(purpose(command, outcome), steps, source);
                        if matches!(aspect, Aspect::Retry) && !stateless(command, outcome) {
                            let mut retry_refusals = Vec::new();
                            if let Some((steps, source, _)) = retry(
                                ir,
                                command,
                                outcome,
                                &origin_run,
                                &assigned,
                                &id,
                                &mut retry_refusals,
                            ) {
                                scenario.steps.extend(steps);
                                scenario.source.extend(source);
                                insert(ir, synthesis, id, scenario);
                            } else if supplied_identity(outcome, &origin_run).is_none()
                                && repeat_exact(ir, &mut scenario, &origin_run)
                            {
                                insert(ir, synthesis, id, scenario);
                            } else {
                                synthesis.refusals.extend(retry_refusals);
                            }
                            continue;
                        }
                        if matches!(aspect, Aspect::Retry) {
                            scenario.steps.extend(scenario.steps.clone());
                        } else if matches!(aspect, Aspect::Rotation) {
                            let repeated = scenario.clone();
                            if let Err(reason) =
                                super::caller::append_independent(ir, &mut scenario, &repeated)
                            {
                                synthesis.refusals.push(refusal(&id, reason));
                                continue;
                            }
                        }
                        insert(ir, synthesis, id, scenario);
                    }
                }
            }
            for field in &outcome.one_time_response {
                followups(
                    ir,
                    readings.as_ref(),
                    synthesis,
                    (command, outcome, field),
                    &defaults,
                );
            }
        }
    }
    link_causes(&mut synthesis.refusals);
    synthesis.suite.select_fresh_format_for(ir);
}

/// Links each refused disclosure cell to the refusal whose cause it repeats, so a reader can print
/// one block per cause rather than one per actor and follow-up. Every cell keeps its own refusal.
///
/// First to the ordinary outcome scenario refused for the same cause, one the cell sends: its
/// follow-up branch, then any branch of its follow-up or denied command, then its origin. A
/// missing creation then reads once, under the outcome that needs it. The cause must be equal,
/// not merely present.
///
/// Then, among the cells of one marked field still unlinked, to the first cell of the same aspect
/// refused for the same reason: the read cells of every actor not granted the origin, the retries
/// the model does not answer.
fn link_causes(refusals: &mut [Refusal]) {
    let ordinary: Vec<(OutcomeRef, ScenarioId, RefusalCause)> = refusals
        .iter()
        .filter(|refusal| !refusal.stands && refusal.caused_by.is_none())
        .filter_map(|refusal| match &refusal.scenario {
            Some(id @ ScenarioId::Outcome { outcome }) => {
                Some((outcome.clone(), id.clone(), refusal.cause.clone()))
            }
            _ => None,
        })
        .collect();
    for refusal in refusals.iter_mut() {
        let Some(ScenarioId::Disclosure { cell }) = &refusal.scenario else {
            continue;
        };
        let same = |entry: &&(OutcomeRef, ScenarioId, RefusalCause)| entry.2 == refusal.cause;
        let exact = match &cell.aspect {
            Aspect::Command(outcome) => ordinary
                .iter()
                .filter(same)
                .find(|(declared, _, _)| declared == outcome),
            _ => None,
        };
        let sent: Vec<&CommandRef> = match &cell.aspect {
            Aspect::Command(outcome) => vec![&outcome.command, &cell.origin.command],
            Aspect::Denied(command) => vec![command, &cell.origin.command],
            _ => vec![&cell.origin.command],
        };
        let found = exact.or_else(|| {
            sent.iter().find_map(|command| {
                ordinary
                    .iter()
                    .filter(same)
                    .find(|(declared, _, _)| &declared.command == *command)
            })
        });
        refusal.caused_by = found.map(|(_, id, _)| id.clone());
    }
    let mut first: Vec<(SameReason, ScenarioId)> = Vec::new();
    for refusal in refusals.iter_mut() {
        if refusal.caused_by.is_some() || refusal.stands {
            continue;
        }
        let Some(id @ ScenarioId::Disclosure { cell }) = &refusal.scenario else {
            continue;
        };
        let key = (
            cell.origin.clone(),
            cell.field.clone(),
            aspect_kind(&cell.aspect),
            reason(&refusal.cause),
        );
        match first.iter().find(|(seen, _)| *seen == key) {
            Some((_, cause)) => refusal.caused_by = Some(cause.clone()),
            None => first.push((key, id.clone())),
        }
    }
}

/// What two cells refused for one reason share: their origin, marked field and aspect keyword, and
/// the cause without the cell's own id.
type SameReason = (OutcomeRef, String, &'static str, RefusalCause);

/// The aspect's keyword, without the view, command or outcome it names.
fn aspect_kind(aspect: &Aspect) -> &'static str {
    match aspect {
        Aspect::Origin => "origin",
        Aspect::Retry => "retry",
        Aspect::Rotation => "rotation",
        Aspect::Read(_) => "read",
        Aspect::Command(_) => "command",
        Aspect::Denied(_) => "denied",
    }
}

/// A cell's cause without the cell's own id, which a disclosure cell's own reason carries in its
/// path: two cells refused for one reason then compare equal.
fn reason(cause: &RefusalCause) -> RefusalCause {
    match cause {
        RefusalCause::NoWitness(gap) if gap.type_ref == INVENTORY => {
            let mut gap = gap.clone();
            if let Some((_, detail)) = gap.path.split_once(": ") {
                gap.path = detail.to_owned();
            }
            RefusalCause::NoWitness(gap)
        }
        other => other.clone(),
    }
}

fn originating_actors(ir: &EssIr, command: &ResolvedCommand) -> Vec<Option<ActorRef>> {
    let actors: Vec<_> = ir
        .actors()
        .values()
        .filter(|actor| actor.may.iter().any(|grant| grant.name() == &command.name))
        .map(|actor| Some(ActorRef::new(actor.name.clone())))
        .collect();
    // An inaccessible origin is still an inventoried obligation, not an empty success.
    if actors.is_empty() {
        vec![None]
    } else {
        actors
    }
}

/// A creation may mint a new identity on an identical invocation. Reuse the composition
/// machinery to capture that identity, but refuse if it changed any invocation input or caller.
fn repeat_exact(ir: &EssIr, scenario: &mut ConformanceScenario, origin: &super::Run) -> bool {
    let Some(invocation) = origin
        .invoke
        .iter()
        .find(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
    else {
        return false;
    };
    let ScenarioStep::ExecuteCommand { command, .. } = invocation else {
        return false;
    };
    let repeated = scenario.clone();
    let before = scenario.steps.len();
    if super::caller::append_independent(ir, scenario, &repeated).is_err() {
        return false;
    }
    scenario.steps[before..].iter().rev().find(|step| matches!(step, ScenarioStep::ExecuteCommand { command: named, .. } if named == command)) == Some(invocation)
}

/// Repeat the actual invocation over the state it left, with the ordinary assertions for the
/// branch that state selects. Replaying the origin's arrangement would silently reset its row.
fn retry(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    origin: &super::Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<(
    Vec<ScenarioStep>,
    std::collections::BTreeSet<super::EssSemanticRef>,
    super::Run,
)> {
    if let Some(field) = supplied_identity(outcome, origin) {
        return supplied_retry(ir, command, field, origin, actors, id, refusals);
    }
    let (Some(subject), Some(instance), Some(_)) = (
        command.selection_subject(outcome),
        origin.instance.as_ref(),
        origin.after.as_ref(),
    ) else {
        refusals.push(refusal(
            id,
            "the retry has no established surviving subject",
        ));
        return None;
    };
    let eligible: Vec<_> = command
        .outcomes
        .iter()
        .filter_map(|branch| {
            super::replay_eligibility(ir, command, branch, origin, subject, instance)
                .ok()
                .map(|observations| (branch, observations))
        })
        .collect();
    let [(branch, (observed, sources))] = eligible.as_slice() else {
        refusals.push(refusal(
            id,
            "the identical retry has no uniquely witnessed post-state branch",
        ));
        return None;
    };
    if branch
        .subject
        .as_ref()
        .is_some_and(|subject| matches!(subject.effect, super::ResolvedEffect::Creates))
    {
        refusals.push(refusal(
            id,
            "the retry creates another instance whose identity must be captured separately",
        ));
        return None;
    }
    let mut invocation = origin.invoke.iter().find(|step| matches!(step, ScenarioStep::ExecuteCommand { command: named, .. } if named.name() == &command.name))?.clone();
    let actor = actors.get(&command.name).cloned();
    if let ScenarioStep::ExecuteCommand { actor: caller, .. } = &mut invocation {
        caller.clone_from(&actor);
    }
    let selected = OutcomeRef::new(CommandRef::new(command.name.clone()), branch.name.clone());
    let mut invoke = Vec::new();
    if branch.test_strategy == ess_domain::command::TestStrategy::InjectFault {
        invoke.push(ScenarioStep::ConfigureExternalOutcome {
            force: selected.clone(),
            times: None,
        });
    }
    invoke.extend([
        invocation,
        ScenarioStep::ExpectOutcome { outcome: selected },
    ]);
    let after = match branch.subject.as_ref().map(|subject| &subject.effect) {
        Some(super::ResolvedEffect::Moves { transition }) => Some(transition.to.clone()),
        Some(super::ResolvedEffect::Deletes) => None,
        _ => origin.after.clone(),
    };
    let mut settled = origin.settled.clone();
    super::absorb(
        &mut settled,
        branch,
        super::settled(ir, branch, &origin.input, &origin.settled),
    );
    let mut source = origin.source.clone();
    source.extend(sources.iter().cloned());
    let run = super::Run {
        setup: observed.clone(),
        invoke,
        before: origin.after.clone(),
        after,
        instance: origin.instance.clone(),
        actor,
        input: origin.input.clone(),
        source,
        after_steps: Vec::new(),
        before_settled: origin.settled.clone(),
        settled,
        refused: Vec::new(),
        unclaimed: Vec::new(),
        unobserved: std::collections::BTreeSet::new(),
    };
    super::exercise_run(ir, command, branch, actors, id, refusals, run)
}

/// The input field a creation takes its identity from where the origin's caller supplied it,
/// outside create-or-update, whose identical repeat updates the row instead. An identity the
/// origin left to be generated is minted afresh by the repeat, as for any minted identity.
fn supplied_identity<'a>(outcome: &'a ResolvedOutcome, origin: &super::Run) -> Option<&'a str> {
    super::existence::identity_input(outcome).filter(|field| {
        !super::existence::creates_unknown(outcome) && origin.input.contains_key(*field)
    })
}

/// The retry of a creation whose identity the caller supplies (beyond10x/ess#574). Sent again
/// unchanged, it names the row its origin created, so the command's `existing_instance:` refusal
/// answers it over the origin's post-state. Without one the model declares no answer, and the cell
/// is refused naming that declaration rather than redrawing the identity or inventing an outcome.
fn supplied_retry(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &str,
    origin: &super::Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<(
    Vec<ScenarioStep>,
    std::collections::BTreeSet<super::EssSemanticRef>,
    super::Run,
)> {
    let Some(branch) = command
        .outcomes
        .iter()
        .find(|branch| branch.condition == ResolvedCondition::ExistingInstance)
    else {
        refusals.push(refusal(
            id,
            &format!(
                "the identical retry sends again the caller-supplied identity `{field}` of the row \
                 the origin created, and `{}` declares no `existing_instance:` outcome to answer \
                 it; declare one to make this cell reachable",
                command.name
            ),
        ));
        return None;
    };
    let Some(mut invocation) = origin.invoke.iter().find(|step| matches!(step, ScenarioStep::ExecuteCommand { command: named, .. } if named.name() == &command.name)).cloned() else {
        refusals.push(refusal(id, "the origin has no typed command invocation to repeat"));
        return None;
    };
    let actor = actors.get(&command.name).cloned();
    if let ScenarioStep::ExecuteCommand { actor: caller, .. } = &mut invocation {
        caller.clone_from(&actor);
    }
    let selected = OutcomeRef::new(CommandRef::new(command.name.clone()), branch.name.clone());
    let run = super::Run {
        setup: Vec::new(),
        invoke: vec![
            invocation,
            ScenarioStep::ExpectOutcome { outcome: selected },
        ],
        before: origin.after.clone(),
        after: origin.after.clone(),
        instance: origin.instance.clone(),
        actor,
        input: origin.input.clone(),
        source: origin.source.clone(),
        after_steps: Vec::new(),
        before_settled: origin.settled.clone(),
        settled: origin.settled.clone(),
        refused: Vec::new(),
        unclaimed: Vec::new(),
        unobserved: std::collections::BTreeSet::new(),
    };
    super::exercise_run(ir, command, branch, actors, id, refusals, run)
}

fn cell(
    origin: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    field: &str,
    aspect: Aspect,
    actor: Option<ActorRef>,
) -> ScenarioId {
    ScenarioId::Disclosure {
        cell: Box::new(Cell {
            origin: OutcomeRef::new(CommandRef::new(origin.name.clone()), outcome.name.clone()),
            field: field.into(),
            aspect,
            actor,
        }),
    }
}

fn prefix(
    ir: &EssIr,
    synthesis: &mut Synthesis,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
) -> Option<ConformanceScenario> {
    exercise(ir, command, outcome, actors, id, &mut synthesis.refusals).map(|(steps, source, _)| {
        ConformanceScenario::new(purpose(command, outcome), steps, source)
    })
}

fn stateless(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    outcome.subject.is_none()
        && !has_subject_guards(command)
        && !related_guard::routes(command, outcome)
        && !subject_fact::uses(command)
}

fn followups(
    ir: &EssIr,
    readings: Option<&Readings>,
    synthesis: &mut Synthesis,
    (origin, outcome, field): (&ResolvedCommand, &ResolvedOutcome, &str),
    defaults: &BTreeMap<QualifiedName, ActorRef>,
) {
    let actors: Vec<_> = if ir.actors().is_empty() {
        vec![None]
    } else {
        ir.actors()
            .values()
            .map(|actor| Some(ActorRef::new(actor.name.clone())))
            .collect()
    };
    for actor in actors {
        for command in ir.commands().values() {
            let granted = actor.as_ref().is_none_or(|actor| {
                ir.actors()[actor.name()]
                    .may
                    .iter()
                    .any(|granted| granted.name() == &command.name)
            });
            if !granted {
                denied_cell(
                    ir,
                    readings,
                    synthesis,
                    (origin, outcome, field),
                    command,
                    actor.clone(),
                    defaults,
                );
                continue;
            }
            for branch in &command.outcomes {
                let id = cell(
                    origin,
                    outcome,
                    field,
                    Aspect::Command(OutcomeRef::new(
                        CommandRef::new(command.name.clone()),
                        branch.name.clone(),
                    )),
                    actor.clone(),
                );
                let Some((steps, source, origin_run)) =
                    exercise(ir, origin, outcome, defaults, &id, &mut synthesis.refusals)
                else {
                    continue;
                };
                let mut scenario =
                    ConformanceScenario::new(purpose(origin, outcome), steps, source);
                let mut assigned = defaults.clone();
                if let Some(actor) = &actor {
                    assigned.insert(command.name.clone(), actor.clone());
                }
                if command.name == origin.name {
                    let mut probe_refusals = Vec::new();
                    if let Some((steps, source, post)) = retry(
                        ir,
                        command,
                        outcome,
                        &origin_run,
                        &assigned,
                        &id,
                        &mut probe_refusals,
                    ) {
                        if post.invoke.iter().any(|step| matches!(step, ScenarioStep::ExpectOutcome { outcome } if outcome.outcome == branch.name)) {
                            scenario.steps.extend(steps);
                            scenario.source.extend(source);
                            insert(ir, synthesis, id, scenario);
                            continue;
                        }
                    }
                }
                if let Some(followup) = reached(
                    ir,
                    readings,
                    (command, branch),
                    &assigned,
                    &id,
                    &mut synthesis.refusals,
                ) {
                    if let Err(reason) =
                        super::caller::append_independent(ir, &mut scenario, &followup)
                    {
                        synthesis.refusals.push(refusal(&id, reason));
                        continue;
                    }
                    insert(ir, synthesis, id, scenario);
                }
            }
        }
        read_cells(
            ir,
            synthesis,
            origin,
            outcome,
            field,
            actor.as_ref(),
            defaults,
        );
    }
}

fn read_cells(
    ir: &EssIr,
    synthesis: &mut Synthesis,
    origin: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    field: &str,
    actor: Option<&ActorRef>,
    defaults: &BTreeMap<QualifiedName, ActorRef>,
) {
    for view in ir.views().values() {
        let id = cell(
            origin,
            outcome,
            field,
            Aspect::Read(ViewRef::new(view.name.clone())),
            actor.cloned(),
        );
        if actor.is_some_and(|actor| {
            !ir.actors()[actor.name()]
                .may
                .iter()
                .any(|grant| grant.name() == &origin.name)
        }) {
            synthesis.refusals.push(refusal(
                &id,
                "the read's originating caller has no grant to issue this value",
            ));
            continue;
        }
        // Views use the existing public query seam. The cell's caller binds the actual
        // originating invocation, as admission requires; it does not invent a view grant.
        let mut assigned = defaults.clone();
        if let Some(actor) = actor {
            assigned.insert(origin.name.clone(), actor.clone());
        }
        let Some((steps, source, origin_run)) =
            exercise(ir, origin, outcome, &assigned, &id, &mut synthesis.refusals)
        else {
            continue;
        };
        let mut scenario = ConformanceScenario::new(purpose(origin, outcome), steps, source);
        let mut params: BTreeMap<_, _> =
            match crate::witness::fields(ir, &view.params, Distinction::PLAIN) {
                Ok(params) => params
                    .into_iter()
                    .map(|(name, value)| (name, ScenarioValue::literal(value)))
                    .collect(),
                Err(reason) => {
                    synthesis
                        .refusals
                        .push(Refusal::about(&id, RefusalCause::NoWitness(reason)));
                    continue;
                }
            };
        if outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.entity == view.source)
        {
            let identity = origin_run.instance.clone().map(ScenarioValue::instance);
            params.extend(super::bound(
                ir,
                view,
                &origin_run.settled,
                identity.as_ref(),
            ));
        }
        let read = ScenarioStep::QueryView {
            view: ViewRef::new(view.name.clone()),
            params,
        };
        let rotation = scenario.clone();
        scenario.steps.push(read.clone());
        if let Err(reason) = super::caller::append_independent(ir, &mut scenario, &rotation) {
            synthesis.refusals.push(refusal(&id, reason));
            continue;
        }
        scenario.steps.push(read);
        scenario
            .source
            .insert(ViewRef::new(view.name.clone()).into());
        insert(ir, synthesis, id, scenario);
    }
}

/// The follow-up `branch` of `command`, sent as `actors` assigns, built as the ordinary suite
/// reaches it ([`super::reached_branch`]): from the model itself, and else under each caller
/// reading of [`Readings`] in turn. Where none reaches it, the model's own refusals.
fn reached(
    ir: &EssIr,
    readings: Option<&Readings>,
    (command, branch): (&ResolvedCommand, &ResolvedOutcome),
    actors: &BTreeMap<QualifiedName, ActorRef>,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> Option<ConformanceScenario> {
    let mut own = Vec::new();
    let plain = InvocationModels::plain(ir);
    if let Some(scenario) = super::reached_branch(&plain, command, branch, actors, id, &mut own) {
        refusals.extend(own);
        return Some(scenario);
    }
    for models in readings
        .map(|readings| readings.models(ir))
        .into_iter()
        .flatten()
    {
        let Some((command, branch)) = read_as(&models, command, branch) else {
            continue;
        };
        let mut tried = Vec::new();
        if let Some(scenario) =
            super::reached_branch(&models, command, branch, actors, id, &mut tried)
        {
            refusals.extend(tried);
            return Some(scenario);
        }
    }
    refusals.extend(own);
    None
}

/// `command` and its `branch` as `models` act them: the reading writes the caller's values in.
fn read_as<'m>(
    models: &InvocationModels<'m>,
    command: &ResolvedCommand,
    branch: &ResolvedOutcome,
) -> Option<(&'m ResolvedCommand, &'m ResolvedOutcome)> {
    let command = models.acting.commands().get(&command.name)?;
    let branch = command
        .outcomes
        .iter()
        .find(|declared| declared.name == branch.name)?;
    Some((command, branch))
}

/// An invocation of `command` a granted caller would send, for a denied cell to send as one who is
/// not: the first branch, in declared order, with an arrangement and an input, from the model
/// itself and else under each caller reading. A branch only a family of its own reaches (an
/// identity no record carries, a state the moves leave out) has none, and the next is taken.
/// Where none has one, the first branch's refusal.
fn denied_run(
    ir: &EssIr,
    readings: Option<&Readings>,
    command: &ResolvedCommand,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<super::Run, RefusalCause> {
    let mut first = None;
    let plain = InvocationModels::plain(ir);
    let readings = readings.map(|readings| readings.models(ir));
    for models in std::iter::once(&plain).chain(readings.iter().flatten()) {
        let Some(command) = models.acting.commands().get(&command.name) else {
            continue;
        };
        for branch in &command.outcomes {
            match super::run_as(models, command, branch, actors, super::Witness::Full) {
                Ok(run) => return Ok(run),
                Err(cause) => {
                    first.get_or_insert(cause);
                }
            }
        }
    }
    Err(first.unwrap_or_else(|| {
        RefusalCause::NoWitness(WitnessGap {
            path: command.name.to_string(),
            type_ref: INVENTORY.into(),
            reason: "the denied command declares no branch to send",
        })
    }))
}

fn denied_cell(
    ir: &EssIr,
    readings: Option<&Readings>,
    synthesis: &mut Synthesis,
    origin: (&ResolvedCommand, &ResolvedOutcome, &str),
    command: &ResolvedCommand,
    actor: Option<ActorRef>,
    defaults: &BTreeMap<QualifiedName, ActorRef>,
) {
    let (origin, outcome, field) = origin;
    let id = cell(
        origin,
        outcome,
        field,
        Aspect::Denied(CommandRef::new(command.name.clone())),
        actor.clone(),
    );
    let Some(mut scenario) = prefix(ir, synthesis, origin, outcome, defaults, &id) else {
        return;
    };
    let Some(actor) = actor else {
        return;
    };
    if command.outcomes.is_empty() {
        return;
    }
    let run = match denied_run(ir, readings, command, defaults) {
        Ok(run) => run,
        Err(cause) => {
            synthesis.refusals.push(Refusal::about(&id, cause));
            return;
        }
    };
    let mut steps = run.setup;
    let Some(mut invoke) = run.invoke.into_iter().find(|step| matches!(step, ScenarioStep::ExecuteCommand { command: named, .. } if named.name() == &command.name)) else {
        synthesis.refusals.push(refusal(&id, "denied witness has no typed command invocation"));
        return;
    };
    // Sent as the denied actor: a caller a reading marked it with was the granted actor's.
    if let ScenarioStep::ExecuteCommand {
        actor: sender,
        caller,
        ..
    } = &mut invoke
    {
        *sender = Some(actor.clone());
        caller.clear();
    }
    steps.push(invoke);
    steps.push(ScenarioStep::ExpectNotGranted {
        actor: Some(actor.clone()),
        unpublished: ir
            .events()
            .values()
            .map(|event| EventRef::new(event.name.clone()))
            .collect(),
    });
    let mut denied = ConformanceScenario::new(purpose(origin, outcome), steps, run.source);
    denied.source.insert(actor.into());
    denied
        .source
        .insert(CommandRef::new(command.name.clone()).into());
    if let Err(reason) = super::caller::append_independent(ir, &mut scenario, &denied) {
        synthesis.refusals.push(refusal(&id, reason));
        return;
    }
    insert(ir, synthesis, id, scenario);
}

fn insert(
    ir: &EssIr,
    synthesis: &mut Synthesis,
    id: ScenarioId,
    mut scenario: ConformanceScenario,
) {
    // The cell is built from the model itself, outside the caller readings that mark every other
    // scenario: a command it sends is sent as the suite's caller, whose attributes the outcome
    // may read (`{caller: <attribute>}`, beyond10x/ess#550).
    super::caller::send_as_first(ir, &mut scenario);
    if let Err(reason) = crate::one_time_response::produce::attach(ir, &mut scenario) {
        synthesis.refusals.push(refusal(&id, &reason));
        return;
    }
    synthesis.suite.scenarios.insert(id, scenario);
}

/// The type a disclosure cell's own refusal names, which is what [`RefusalCause`] reads it by.
pub(super) const INVENTORY: &str = "one-time disclosure inventory";

fn refusal(id: &ScenarioId, detail: &str) -> Refusal {
    Refusal::about(
        id,
        RefusalCause::NoWitness(WitnessGap {
            path: format!("{id}: {detail}"),
            type_ref: INVENTORY.into(),
            reason: "the source-owned disclosure cell cannot be arranged or observed",
        }),
    )
}
