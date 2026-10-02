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
            let actors: Vec<_> = if ir.actors().is_empty() {
                vec![None]
            } else {
                ir.actors()
                    .values()
                    .filter(|actor| {
                        actor
                            .may
                            .iter()
                            .any(|granted| granted.name() == &command.name)
                    })
                    .map(|actor| Some(ActorRef::new(actor.name.clone())))
                    .collect()
            };
            // An inaccessible origin is still an inventoried obligation, not an empty success.
            let actors = if actors.is_empty() {
                vec![None]
            } else {
                actors
            };
            for actor in actors {
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
                        let Some((steps, source, _)) = exercise(
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
                        if !matches!(aspect, Aspect::Origin) {
                            if outcome.subject.is_some()
                                || has_subject_guards(command)
                                || related_guard::routes(command, outcome)
                                || subject_fact::uses(command)
                            {
                                synthesis.refusals.push(refusal(&id, "a repeated origin requires a separately arranged post-state witness"));
                                continue;
                            }
                            let repeated = scenario.steps.clone();
                            scenario.steps.extend(repeated);
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
                let Some(mut scenario) = prefix(ir, synthesis, origin, outcome, defaults, &id)
                else {
                    continue;
                };
                if !stateless(origin, outcome) {
                    synthesis.refusals.push(refusal(
                        &id,
                        "the follow-up arrangement needs a witness over the origin's post-state",
                    ));
                    continue;
                }
                let mut assigned = defaults.clone();
                if let Some(actor) = &actor {
                    assigned.insert(command.name.clone(), actor.clone());
                }
                if let Some((steps, source, _)) =
                    exercise(ir, command, branch, &assigned, &id, &mut synthesis.refusals)
                {
                    scenario.steps.extend(steps);
                    scenario.source.extend(source);
                    insert(ir, synthesis, id, scenario);
                }
            }
        }
        read_cells(ir, synthesis, origin, outcome, field, actor.as_ref(), defaults);
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
        if actor.is_some() {
            synthesis.refusals.push(refusal(
                &id,
                "the public query observation has no actor authority seam",
            ));
            continue;
        }
        let Some(mut scenario) = prefix(ir, synthesis, origin, outcome, defaults, &id) else {
            continue;
        };
        if !stateless(origin, outcome) {
            synthesis.refusals.push(refusal(
                &id,
                "the rotated read requires an origin post-state witness",
            ));
            continue;
        }
        let params = match crate::witness::fields(ir, &view.params, Distinction::PLAIN) {
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
        let read = ScenarioStep::QueryView {
            view: ViewRef::new(view.name.clone()),
            params,
        };
        let rotation = scenario.steps.clone();
        scenario.steps.push(read.clone());
        scenario.steps.extend(rotation);
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
    if !stateless(origin, outcome) {
        synthesis.refusals.push(refusal(
            &id,
            "the denied follow-up needs an origin post-state witness",
        ));
        return;
    }
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
    scenario.steps.extend(run.setup);
    let Some(mut invoke) = run.invoke.into_iter().find(|step| matches!(step, ScenarioStep::ExecuteCommand { command: named, .. } if named.name() == &command.name)) else {
        synthesis.refusals.push(refusal(&id, "denied witness has no typed command invocation"));
        return;
    };
    if let ScenarioStep::ExecuteCommand { actor: caller, .. } = &mut invoke {
        *caller = Some(actor.clone());
    }
    scenario.steps.push(invoke);
    scenario.steps.push(ScenarioStep::ExpectNotGranted {
        actor: actor.clone(),
        unpublished: ir
            .events()
            .values()
            .map(|event| EventRef::new(event.name.clone()))
            .collect(),
    });
    scenario.source.extend(run.source);
    scenario.source.insert(actor.into());
    scenario
        .source
        .insert(CommandRef::new(command.name.clone()).into());
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
