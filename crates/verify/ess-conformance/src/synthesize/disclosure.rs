//! Finite source-owned disclosure inventory, with named refusals for unarrangeable cells.
use super::{
    exercise, granted_actors, has_subject_guards, purpose, related_guard, run, subject_fact,
    ActorRef, BTreeMap, CommandRef, ConformanceScenario, Distinction, EssIr, EventRef, OutcomeRef,
    QualifiedName, Refusal, RefusalCause, ResolvedCommand, ResolvedOutcome, ScenarioId,
    ScenarioStep, ScenarioValue, Synthesis, ViewRef, WitnessGap,
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
                            } else if repeat_exact(ir, &mut scenario, &origin_run) {
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
                followups(ir, synthesis, command, outcome, field, &defaults);
            }
        }
    }
    synthesis.suite.select_fresh_format_for(ir);
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
    synthesis: &mut Synthesis,
    origin: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    field: &str,
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
                if let Some((steps, source, _)) =
                    exercise(ir, command, branch, &assigned, &id, &mut synthesis.refusals)
                {
                    let followup =
                        ConformanceScenario::new(purpose(command, branch), steps, source);
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

fn denied_cell(
    ir: &EssIr,
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
    let Some(branch) = command.outcomes.first() else {
        return;
    };
    let run = match run(ir, command, branch, defaults) {
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
    if let ScenarioStep::ExecuteCommand { actor: caller, .. } = &mut invoke {
        *caller = Some(actor.clone());
    }
    steps.push(invoke);
    steps.push(ScenarioStep::ExpectNotGranted {
        actor: actor.clone(),
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
    if let Err(reason) = crate::one_time_response::produce::attach(ir, &mut scenario) {
        synthesis.refusals.push(refusal(&id, &reason));
        return;
    }
    synthesis.suite.scenarios.insert(id, scenario);
}

fn refusal(id: &ScenarioId, detail: &str) -> Refusal {
    Refusal::about(
        id,
        RefusalCause::NoWitness(WitnessGap {
            path: format!("{id}: {detail}"),
            type_ref: "one-time disclosure inventory".into(),
            reason: "the source-owned disclosure cell cannot be arranged or observed",
        }),
    )
}
