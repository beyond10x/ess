//! Bounded arrangement over lifecycle AND independently observed enum history.
use super::{
    absorb, arrange, candidates, decides, flatten, invoke, require, settled, shows, supply, when,
    ActorRef, Arrangement, AssertionStyle, BTreeMap, BTreeSet, CommandRef, Distinction, Driver,
    EntityHandle, EntityRef, EntitySpec, EssIr, EssSemanticRef, InstanceNeed, Invocation, Node,
    OutcomeRef, QualifiedName, RefusalCause, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedOutcome, ScenarioStep, ScenarioValue, Setup, StateName, VecDeque, ViewExpectation,
    ViewRef, WitnessGap,
};

pub(super) fn uses(command: &ResolvedCommand) -> bool {
    command
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, ResolvedCondition::SubjectField { .. }))
}

fn field(command: &ResolvedCommand) -> Option<&str> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            ResolvedCondition::SubjectField { field, .. } => Some(field.as_str()),
            _ => None,
        })
}

fn missing(entity: &EntityHandle, field: &str, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path: format!("{entity}.{field}"),
        type_ref: entity.to_string(),
        reason,
    })
}

fn fact<'a>(arrangement: &'a Arrangement, field: &str) -> Option<&'a str> {
    match &arrangement.settled.get(field)?.value {
        ScenarioValue::Literal {
            value: Node::Text(value),
        } => Some(value),
        _ => None,
    }
}

fn reach_fact(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    state: &StateName,
    held: &str,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let guards: Vec<_> = command
        .outcomes
        .iter()
        .filter_map(|branch| match &branch.condition {
            ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
            _ => when(branch),
        })
        .collect();
    let inputs =
        candidates(ir, command, &guards, Distinction::PLAIN).map_err(RefusalCause::NoWitness)?;
    for input in &inputs {
        let facts = flatten(ir, command, input).map_err(RefusalCause::WitnessRejected)?;
        let mut selected = Vec::new();
        for branch in &command.outcomes {
            let matches = match &branch.condition {
                ResolvedCondition::When { predicate } => decides(&facts, &[predicate], true)?,
                ResolvedCondition::SubjectField {
                    equals, predicate, ..
                } => {
                    held == equals && decides(&facts, &predicate.iter().collect::<Vec<_>>(), true)?
                }
                ResolvedCondition::SubjectState { .. }
                | ResolvedCondition::StateChange { .. }
                | ResolvedCondition::Otherwise
                | ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                | ResolvedCondition::WrongState => false,
            };
            if matches {
                selected.push(branch);
            }
        }
        let selected = if selected.is_empty() {
            command
                .outcomes
                .iter()
                .find(|branch| branch.condition == ResolvedCondition::Otherwise)
        } else if selected.len() == 1 {
            Some(selected[0])
        } else {
            None
        };
        if selected.is_some_and(|branch| branch.name == outcome.name)
            && outcome
                .subject
                .as_ref()
                .and_then(|subject| subject.effect.transition())
                .is_none_or(|transition| transition.from.contains(state))
        {
            return Ok(input.clone());
        }
    }
    Err(RefusalCause::GuardUnsatisfiable {
        predicate: format!("input and observed subject fact {held}"),
        tried: inputs.len(),
    })
}

fn observe(
    ir: &EssIr,
    entity: &EntityHandle,
    field: &str,
    arrangement: &Arrangement,
) -> Result<(Vec<ScenarioStep>, ViewRef), RefusalCause> {
    let declared = ir.entity(entity);
    let Some(value) = arrangement.settled.get(field) else {
        return Err(missing(
            entity,
            field,
            "subject fact has no determined arrangement value",
        ));
    };
    let view = ir
        .views()
        .values()
        .find(|view| {
            view.source == *entity
                && view.params.is_empty()
                && view.filter.is_none()
                && view.assertion_style == AssertionStyle::Expect
                && view
                    .field(&declared.identity.name)
                    .is_some_and(|f| f.type_ref == declared.identity.type_ref)
                && view
                    .field(EntitySpec::STATE)
                    .is_some_and(|f| f.type_ref == declared.state_field().type_ref)
                && view
                    .field(field)
                    .is_some_and(|f| f.type_ref == value.type_ref)
        })
        .ok_or_else(|| {
            missing(
                entity,
                field,
                "subject fact selection requires an immediate unfiltered identity/state/fact view",
            )
        })?;
    let name = ViewRef::new(view.name.clone());
    let fields = [
        (
            declared.identity.name.clone(),
            ScenarioValue::instance(arrangement.instance.clone()),
        ),
        (
            EntitySpec::STATE.into(),
            ScenarioValue::literal(Node::Text(arrangement.state.to_string())),
        ),
        (field.into(), value.value.clone()),
    ]
    .into_iter()
    .collect();
    let mut steps = Vec::new();
    require(
        view,
        &name,
        BTreeMap::new(),
        ViewExpectation::Contains { fields },
        &mut steps,
    );
    Ok((steps, name))
}

/// Histories that share a lifecycle state stay distinct nodes in the search.
pub(super) fn prepare(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .ok_or(RefusalCause::StrategyWithoutGuard {
            strategy: outcome.test_strategy,
        })?;
    let entity = &subject.entity;
    let field = field(command).expect("uses established a field");
    let initial = &ir.entity(entity).lifecycle.initial;
    let arranged =
        arrange(ir, entity, initial, actors, Distinction::PLAIN, &[]).map_err(|reason| {
            RefusalCause::InstanceRequired {
                entity: EntityRef::from(entity),
                need: InstanceNeed::Updates,
                reason,
            }
        })?;
    let all = ir.drivers();
    let drivers = all.get(entity).map_or(&[][..], Vec::as_slice);
    let mut queue = VecDeque::from([arranged]);
    let mut seen = BTreeSet::new();
    while let Some(mut arrangement) = queue.pop_front() {
        let held = fact(&arrangement, field)
            .ok_or_else(|| {
                missing(
                    entity,
                    field,
                    "the arrangement did not establish a known enum fact",
                )
            })?
            .to_owned();
        if !seen.insert((arrangement.state.clone(), held.clone())) {
            continue;
        }
        if seen.len() > 64 {
            return Err(missing(
                entity,
                field,
                "subject fact arrangement exceeds 64 lifecycle/fact combinations",
            ));
        }
        if let Ok(input) = reach_fact(ir, command, outcome, &arrangement.state, &held) {
            let (steps, view) = observe(ir, entity, field, &arrangement)?;
            arrangement.steps.extend(steps);
            arrangement.source.insert(view.into());
            let after = subject.effect.transition().map_or_else(
                || arrangement.state.clone(),
                |transition| transition.to.clone(),
            );
            return Ok((
                Setup {
                    steps: arrangement.steps,
                    instance: Some(arrangement.instance),
                    bound: BTreeMap::new(),
                    source: arrangement.source,
                    after: Some(after),
                    settled: arrangement.settled,
                },
                input,
            ));
        }
        for driver in drivers {
            if matches!(
                driver.effect,
                ResolvedEffect::Creates | ResolvedEffect::Preserves
            ) {
                continue;
            }
            if driver
                .effect
                .transition()
                .is_some_and(|transition| !transition.from.contains(&arrangement.state))
            {
                continue;
            }
            if let Ok(next) = advance(ir, entity, driver, &arrangement, actors) {
                queue.push_back(next);
            }
        }
    }
    Err(missing(
        entity,
        field,
        "no bounded arrangement establishes the subject fact and eligible input",
    ))
}

fn advance(
    ir: &EssIr,
    entity: &EntityHandle,
    driver: &Driver<'_>,
    arrangement: &Arrangement,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Arrangement, RefusalCause> {
    let mut next = arrangement.clone();
    let invoked = if uses(driver.command) {
        let driver_field = field(driver.command).expect("uses established a field");
        let value = fact(arrangement, driver_field)
            .ok_or_else(|| missing(entity, driver_field, "driver subject history is unknown"))?;
        let input = reach_fact(
            ir,
            driver.command,
            driver.outcome,
            &arrangement.state,
            value,
        )?;
        let (observed, view) = observe(ir, entity, driver_field, arrangement)?;
        next.steps.extend(observed);
        next.source.insert(view.into());
        let supplied = supply(
            driver.command,
            &input,
            driver.outcome.subject.as_ref(),
            Some(&arrangement.instance),
            &BTreeMap::new(),
        );
        let command_ref = CommandRef::new(driver.command.name.clone());
        let outcome_ref = OutcomeRef::new(command_ref.clone(), driver.outcome.name.clone());
        Invocation {
            steps: vec![
                ScenarioStep::ExecuteCommand {
                    command: command_ref.clone(),
                    actor: actors.get(&driver.command.name).cloned(),
                    input: supplied.clone(),
                },
                ScenarioStep::ExpectOutcome {
                    outcome: outcome_ref.clone(),
                },
                ScenarioStep::ExpectNoError,
            ],
            source: [command_ref.into(), outcome_ref.into()]
                .into_iter()
                .collect(),
            settled: settled(ir, driver.outcome, &supplied),
        }
    } else {
        invoke(
            ir,
            driver,
            Some(&arrangement.instance),
            Some(&arrangement.state),
            actors,
            Distinction::PLAIN,
            &BTreeMap::new(),
        )
        .map_err(|reason| RefusalCause::InstanceRequired {
            entity: EntityRef::from(entity),
            need: InstanceNeed::Updates,
            reason,
        })?
    };
    next.steps.extend(invoked.steps);
    next.source.extend(invoked.source);
    absorb(&mut next.settled, driver.outcome, invoked.settled);
    if let Some(transition) = driver.effect.transition() {
        next.state = transition.to.clone();
    }
    Ok(next)
}

pub(super) struct Preservation {
    pub before: Vec<ScenarioStep>,
    pub after: Vec<ScenarioStep>,
    pub source: BTreeSet<EssSemanticRef>,
}

/// Every declared field must be observed. Unknown generated values are captured from
/// the implementation before the command, never filled from the expected outcome.
pub(super) fn preservation(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    setup: &Setup,
) -> Result<Preservation, RefusalCause> {
    let subject = outcome
        .subject
        .as_ref()
        .expect("preservation has a subject");
    let entity = ir.entity(&subject.entity);
    let instance = setup
        .instance
        .as_ref()
        .expect("preservation arranged an existing subject");
    let mut required: BTreeSet<String> = entity
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect();
    required.insert(entity.identity.name.clone());
    required.insert(EntitySpec::STATE.into());
    let mut before = Vec::new();
    let mut after = Vec::new();
    let mut source = BTreeSet::new();
    for view in ir.views().values().filter(|view| {
        view.source == subject.entity
            && view.params.is_empty()
            && setup.after.as_ref().is_some_and(|state| {
                shows(view, state, &setup.settled, &BTreeMap::new()) == Ok(true)
            })
            && view.assertion_style == AssertionStyle::Expect
            && view
                .field(&entity.identity.name)
                .is_some_and(|field| field.type_ref == entity.identity.type_ref)
    }) {
        let name = ViewRef::new(view.name.clone());
        for field in &view.fields {
            let exact = if field.name == entity.identity.name {
                field.type_ref == entity.identity.type_ref
            } else if field.name == EntitySpec::STATE {
                field.type_ref == entity.state_field().type_ref
            } else {
                entity.fields.iter().any(|declared| {
                    declared.name == field.name && declared.type_ref == field.type_ref
                })
            };
            if exact {
                required.remove(&field.name);
            }
        }
        before.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        before.push(ScenarioStep::SnapshotSubject {
            view: name.clone(),
            subject: [(
                entity.identity.name.clone(),
                ScenarioValue::instance(instance.clone()),
            )]
            .into_iter()
            .collect(),
        });
        after.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        after.push(ScenarioStep::ExpectSubjectUnchanged { view: name.clone() });
        source.insert(name.into());
    }
    if !required.is_empty() {
        return Err(missing(
            &subject.entity,
            &required.into_iter().collect::<Vec<_>>().join(","),
            "preservation requires immediate identity views covering every subject field",
        ));
    }
    Ok(Preservation {
        before,
        after,
        source,
    })
}
