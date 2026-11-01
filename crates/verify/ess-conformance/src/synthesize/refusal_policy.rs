//! The scenarios for a refusal-selected failure policy (ess/22, beyond10x/ess#269): one per
//! declared refusal of the invoked command, filed as `<binding>/binding/refusal/<outcome>`.
//!
//! Each refusal is forced the way §18 forces every failure — by injection, on a branch the
//! specification declares `external:` — and observed through the attempt count, the escalation
//! event, and its absence for the whole window
//! ([`ExpectNoPublication`](crate::ScenarioStep::ExpectNoPublication): the escalation is the
//! binding's, so the last command's direct events cannot show it). A `when_related: {via:
//! input.<f>, exists: false}` refusal is answered rather than forced, on every attempt, where the
//! trigger can be sent naming a row no row carries ([`super::binding_effects::related_missing`]),
//! and a `when_related: {via: input.<f>, predicate: …}` refusal where a row the predicate refuses
//! can be arranged before the trigger ([`super::binding_effects::related_refused`]).
//! An unforceable refusal is refused by name, a coverage limitation and never a fabricated result. The untyped fallback is no
//! declared refusal and no suite can force one; the runtime controls witness it.
//!
//! | the refusal's policy | forced | required |
//! |---|---|---|
//! | `drop` | once | exactly one invocation; the escalation event, where the binding has one, never |
//! | `escalate` | once | exactly one invocation, and the escalation event published exactly once |
//! | a bounded `retry`, the refusal `final` | once | exactly one invocation; no escalation |
//! | a bounded `retry` | on every one of `attempts` invocations | exactly `attempts` invocations; no escalation |
//! | an unbounded `retry` | once, after the arrangement `binding_effects::prepare` made | what the branch it decides publishes, the row at rest; no escalation |

use std::num::NonZeroU32;

use ess_compiler::ir::{EventHandle, ResolvedRefusalAction, ResolvedRefusalPolicy};

use super::{
    accepting_component, clipped, insert, payload_shape, publishes, ActorRef, BTreeMap, BTreeSet,
    BindingGap, BindingRef, Built, CommandRef, ConformanceScenario, ConformanceSuite, EssIr,
    EssSemanticRef, EventRef, OutcomeRef, QualifiedName, Refusal, RefusalCause, ResolvedBinding,
    ResolvedCommand, ResolvedOutcome, Run, ScenarioId, ScenarioStep, TestStrategy,
};

/// Every declared refusal's scenario, or the refusal that says why there is none.
#[allow(clippy::too_many_arguments)]
pub(super) fn witnesses(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    invoked: &ResolvedCommand,
    (published_by, trigger): (&ResolvedOutcome, &Run),
    prepared: &Result<super::binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    source: &BTreeSet<EssSemanticRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let subject = BindingRef::new(binding.name.clone());
    for rule in &policy.refusals {
        let id = ScenarioId::BindingRefusal {
            binding: subject.clone(),
            outcome: rule.outcome.clone(),
        };
        let built = if matches!(rule.action, ResolvedRefusalAction::Retry { bound: None }) {
            retried(
                ir,
                binding,
                policy,
                &rule.outcome,
                invoked,
                prepared,
                (published_by, trigger),
                event,
            )
        } else {
            witness(
                ir,
                binding,
                policy,
                &rule.outcome,
                &rule.action,
                invoked,
                (published_by, trigger),
                event,
                actors,
            )
        };
        match built {
            Ok((steps, purpose, extra)) => {
                let mut depends = source.clone();
                depends.extend(extra);
                insert(
                    suite,
                    id,
                    ConformanceScenario::new(purpose, steps, depends),
                    refusals,
                );
            }
            Err(gap) => refusals.push(Refusal::about(
                &id,
                RefusalCause::BindingUnobservable {
                    binding: subject.clone(),
                    gap,
                },
            )),
        }
    }
}

/// Every declared refusal's scenario refused for one reason: the binding's trigger, or its
/// delivery, cannot be arranged.
pub(super) fn refuse_all(
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    gap: &BindingGap,
    refusals: &mut Vec<Refusal>,
) {
    let subject = BindingRef::new(binding.name.clone());
    for rule in &policy.refusals {
        refusals.push(Refusal::about(
            &ScenarioId::BindingRefusal {
                binding: subject.clone(),
                outcome: rule.outcome.clone(),
            },
            RefusalCause::BindingUnobservable {
                binding: subject.clone(),
                gap: gap.clone(),
            },
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn witness(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    outcome: &ess_domain::command::OutcomeName,
    action: &ResolvedRefusalAction,
    invoked: &ResolvedCommand,
    (published_by, trigger): (&ResolvedOutcome, &Run),
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Built {
    let command = CommandRef::new(invoked.name.clone());
    let forced = forcible(ir.format(), invoked, outcome);
    let alternative = super::bounded_retry::clean_trigger(ir, binding, trigger, actors)?;
    // A missing related row is a refusal the honest system answers on every attempt, where the
    // trigger names a row no row carries: no injection is needed to reach it (a downstream
    // adopter's report of 2026-10-20). Where the trigger refuses that missing row itself, another
    // publisher of the event that does not is sent instead.
    let answered = match (&forced, &alternative) {
        (Err(_), None) => match invoked
            .outcomes
            .iter()
            .find(|candidate| &candidate.name == outcome)
        {
            Some(refusal) => match super::binding_effects::related_missing(
                ir,
                binding,
                invoked,
                published_by,
                refusal,
                trigger,
            ) {
                None => None,
                Some(Ok(invoke)) => Some((invoke, None)),
                Some(Err(gap)) => Some(
                    unguarded_publisher(ir, binding, invoked, published_by, refusal, actors)
                        .ok_or(gap)?,
                ),
            },
            None => None,
        },
        _ => None,
    };
    let tried = matches!((&forced, &alternative, &answered), (Err(_), None, None));
    let arranged = if tried {
        refused_row(ir, binding, invoked, outcome, published_by, trigger, actors)
    } else {
        None
    };
    let forced_ref = match (forced, &answered, &arranged) {
        (Ok(forced_ref), _, _) => forced_ref,
        (Err(_), Some(_), _) | (Err(_), None, Some(_)) => {
            OutcomeRef::new(command.clone(), outcome.clone())
        }
        // Only an arrangement tried and refused says no row the predicate refuses can be arranged.
        (Err(gap), None, None) if tried => return Err(unarranged(invoked, outcome).unwrap_or(gap)),
        (Err(gap), None, None) => return Err(gap),
    };
    let (answered, publisher) = match answered {
        Some((invoke, publisher)) => (Some(invoke), publisher),
        None => (None, None),
    };
    let trigger = match &publisher {
        Some((run, _)) => run,
        None => alternative.as_ref().map_or(trigger, |(run, _)| run),
    };
    let mut source: BTreeSet<EssSemanticRef> = [command.clone().into(), forced_ref.clone().into()]
        .into_iter()
        .collect();
    if let Some((_, published)) = &publisher {
        source.extend(published.iter().cloned());
    }
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    if let Some((_, extra)) = &alternative {
        source.extend(extra.iter().cloned());
    }
    if let Some((_, _, rests_on)) = &arranged {
        source.extend(rests_on.iter().cloned());
    }
    let Owed {
        times,
        invocations,
        after,
        text,
    } = owed(ir, binding, policy, outcome, action, invoked, &mut source);
    let mut steps = match &arranged {
        Some((setup, _, _)) => setup.clone(),
        None => trigger.setup.clone(),
    };
    if let Some(invoke) = answered {
        steps.extend(invoke);
    } else if let Some((_, invoke, _)) = arranged {
        steps.extend(invoke);
    } else {
        steps.push(super::bounded_retry::force(forced_ref, times));
        steps.extend(trigger.invoke.iter().cloned());
    }
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    steps.push(super::bounded_retry::count(
        BindingRef::new(binding.name.clone()),
        command,
        invocations,
    ));
    steps.extend(after);
    Ok((steps, clipped(&text), source))
}

/// The invocation leaving the related row missing, sent from the first other publisher of the
/// binding's event (by command, then branch, in declaration order) that does not refuse the missing
/// row itself, with its arranged run and what that run rests on. `None` where every publisher
/// refuses it, so the refusal by name stands.
#[allow(clippy::type_complexity)]
fn unguarded_publisher(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    published_by: &ResolvedOutcome,
    refusal: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<(Vec<ScenarioStep>, Option<(Run, Vec<EssSemanticRef>)>)> {
    let event = binding.cause.event()?;
    ir.commands().values().find_map(|command| {
        command
            .outcomes
            .iter()
            .filter(|outcome| {
                !std::ptr::eq(*outcome, published_by)
                    && outcome.error.is_none()
                    && outcome.emits.contains(event)
            })
            .find_map(|outcome| {
                let run = super::run(ir, command, outcome, actors).ok()?;
                let invoke = super::binding_effects::related_missing(
                    ir, binding, invoked, outcome, refusal, &run,
                )?
                .ok()?;
                let name = CommandRef::new(command.name.clone());
                let published = vec![
                    name.clone().into(),
                    OutcomeRef::new(name, outcome.name.clone()).into(),
                ];
                Some((invoke, Some((run, published))))
            })
    })
}

/// A predicate refusal over a related row, answered on every attempt as a missing row is: a row the
/// predicate refuses arranged before the trigger, and the trigger sent naming it, as that refusal's
/// own scenario arranges one ([`super::binding_effects::related_refused`]). The setup, the
/// trigger's invocation and what the row rests on; `None` where no such row can be arranged, where
/// arranging it sets the binding off before the counted attempts, or where a step sets off another
/// binding that changes rows of the refused row's entity ([`rewritten`]): that binding may write
/// the row back before the trigger, and the honest system then answers otherwise.
#[allow(clippy::type_complexity)]
fn refused_row(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    outcome: &ess_domain::command::OutcomeName,
    published_by: &ResolvedOutcome,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<(
    Vec<ScenarioStep>,
    Vec<ScenarioStep>,
    BTreeSet<EssSemanticRef>,
)> {
    use ess_compiler::ir::ResolvedCondition;

    let refusal = invoked
        .outcomes
        .iter()
        .find(|candidate| &candidate.name == outcome)?;
    let ResolvedCondition::Related { entity, .. } = &refusal.condition else {
        return None;
    };
    super::binding_effects::related_refused(
        ir,
        binding,
        invoked,
        published_by,
        refusal,
        trigger,
        actors,
    )
    .filter(|(setup, invoke, _)| {
        !sets_off(ir, binding, setup)
            && !setup
                .iter()
                .chain(invoke)
                .any(|step| rewritten(ir, binding, step, entity))
    })
}

/// Whether executing a step of `setup` can publish the binding's event, so the binding would be
/// invoked before the counted attempts begin.
fn sets_off(ir: &EssIr, binding: &ResolvedBinding, setup: &[ScenarioStep]) -> bool {
    let Some(event) = binding.cause.event() else {
        return false;
    };
    setup.iter().any(|step| match step {
        ScenarioStep::ExecuteCommand { command, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => {
            super::bounded_retry::reaches(ir, command.name(), event)
        }
        _ => false,
    })
}

/// Whether executing `step` can set off a binding other than `binding` that invokes a command, or
/// sets off one further on, with a branch changing rows of `entity`: its subject, the rows of a set
/// effect, or an `affects:` entry. The step's own command is the arrangement and is not counted.
/// Read as [`super::bounded_retry::reaches`] reads a chain: every branch at every hop, so the
/// answer errs towards "rewritten", and each command is visited once.
fn rewritten(
    ir: &EssIr,
    binding: &ResolvedBinding,
    step: &ScenarioStep,
    entity: &ess_compiler::ir::EntityHandle,
) -> bool {
    let start = match step {
        ScenarioStep::ExecuteCommand { command, .. }
        | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => command.name(),
        _ => return false,
    };
    let changes = |outcome: &ResolvedOutcome| {
        outcome
            .subject
            .as_ref()
            .is_some_and(|subject| &subject.entity == entity)
            || outcome
                .instances
                .as_ref()
                .is_some_and(|set| &set.entity == entity)
            || outcome
                .affects
                .iter()
                .any(|affect| &affect.entity == entity)
    };
    let set_off = |command: &ResolvedCommand| -> Vec<QualifiedName> {
        let emitted: BTreeSet<&EventHandle> = command
            .outcomes
            .iter()
            .flat_map(|outcome| outcome.emits.iter())
            .collect();
        ir.bindings()
            .values()
            .filter(|other| other.name != binding.name)
            .filter(|other| other.cause.event().is_some_and(|on| emitted.contains(on)))
            .map(|other| other.command.name().clone())
            .collect()
    };
    let Some(first) = ir.commands().get(start) else {
        return false;
    };
    let mut visited = BTreeSet::from([start.clone()]);
    let mut pending = set_off(first);
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(command) = ir.commands().get(&name) else {
            continue;
        };
        if command.outcomes.iter().any(changes) {
            return true;
        }
        pending.extend(set_off(command));
    }
    false
}

/// The refusal as a scenario forces it: an `external:` branch whose eligibility needs no
/// observation of the mapped input, or the gap that names why not.
fn forcible(
    format: ess_domain::system::FormatVersion,
    invoked: &ResolvedCommand,
    outcome: &ess_domain::command::OutcomeName,
) -> Result<OutcomeRef, BindingGap> {
    let forced_ref = OutcomeRef::new(CommandRef::new(invoked.name.clone()), outcome.clone());
    let forced = invoked
        .outcomes
        .iter()
        .find(|candidate| &candidate.name == outcome)
        .filter(|candidate| candidate.test_strategy == TestStrategy::InjectFault)
        .ok_or_else(|| BindingGap::RefusalUnforcible {
            outcome: forced_ref.clone(),
        })?;
    if let Some(gap) = super::forced_eligibility(format, invoked, forced) {
        return Err(gap);
    }
    Ok(forced_ref)
}

/// [`BindingGap::RelatedRefusalUnarranged`] for a predicate refusal over a related row that is not
/// `external:`: a scenario answers it through an arranged row, and never by injection. Named only
/// where [`refused_row`] was tried and arranged none; an unbounded retry ([`retried`]) never tries
/// one, and keeps [`BindingGap::RefusalUnforcible`].
fn unarranged(
    invoked: &ResolvedCommand,
    outcome: &ess_domain::command::OutcomeName,
) -> Option<BindingGap> {
    use ess_compiler::ir::{ResolvedCondition, ResolvedRelatedTest};

    invoked
        .outcomes
        .iter()
        .find(|candidate| &candidate.name == outcome)
        .filter(|refusal| refusal.test_strategy != TestStrategy::InjectFault)
        .filter(|refusal| {
            matches!(
                &refusal.condition,
                ResolvedCondition::Related {
                    test: ResolvedRelatedTest::Holds { .. },
                    ..
                }
            )
        })
        .map(|_| BindingGap::RelatedRefusalUnarranged {
            outcome: OutcomeRef::new(CommandRef::new(invoked.name.clone()), outcome.clone()),
        })
}

/// An unbounded retry of one refusal, witnessed as the universal `retry` is (§18,
/// [`super::on_failure`]): through the arrangement `binding_effects::prepare` made, so a row the
/// invoked command addresses exists before the trigger and decides the branch the next attempt
/// takes; the refusal forced once, then what that branch publishes, the row where it rests, and no
/// escalation. Not a count: the arrangement may itself invoke the binding, and an unbounded retry
/// promises the consequence, not a number.
#[allow(clippy::too_many_arguments)]
fn retried(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    outcome: &ess_domain::command::OutcomeName,
    invoked: &ResolvedCommand,
    prepared: &Result<super::binding_effects::Prepared<'_>, BindingGap>,
    trigger: super::Published<'_>,
    event: &EventRef,
) -> Built {
    let forced_ref = forcible(ir.format(), invoked, outcome)?;
    let prepared = prepared.as_ref().map_err(Clone::clone)?;
    let reached = prepared.reached.clone()?;
    let published = publishes(invoked, reached)?;
    let mut source: BTreeSet<EssSemanticRef> = [
        CommandRef::new(invoked.name.clone()).into(),
        forced_ref.clone().into(),
    ]
    .into_iter()
    .collect();
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    source.extend(prepared.source.iter().cloned());
    source.extend(super::downstream(ir, invoked, reached));
    let mut steps = prepared.setup.clone();
    steps.push(super::bounded_retry::force(forced_ref, None));
    steps.extend(prepared.invoke.iter().cloned());
    steps.extend(prepared.capture.iter().cloned());
    steps.push(super::expect_trigger_event(
        ir,
        trigger,
        (&prepared.setup, &prepared.invoke),
        event,
    ));
    steps.extend(published.iter().map(|event| ScenarioStep::EventuallyEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    }));
    steps.extend(prepared.settled_row(ir));
    steps.extend(absent(policy.escalation(), &mut source));
    let text = format!(
        "`{}` retries a `{}` answered `{outcome}` until {} is published",
        binding.name,
        invoked.name,
        super::listed(&published)
    );
    Ok((steps, clipped(&text), source))
}

/// What one refusal's scenario owes, by the policy that answers it.
struct Owed {
    /// How many invocations in a row the refusal is forced on, beyond the next one.
    times: Option<NonZeroU32>,
    /// Exactly how many invocations the binding makes.
    invocations: NonZeroU32,
    /// What must, and must not, be published after them.
    after: Vec<ScenarioStep>,
    /// The scenario's purpose.
    text: String,
}

fn owed(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    outcome: &ess_domain::command::OutcomeName,
    action: &ResolvedRefusalAction,
    invoked: &ResolvedCommand,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Owed {
    let escalation = policy.escalation();
    let (times, invocations, after, text) = match action {
        ResolvedRefusalAction::Drop => (
            None,
            NonZeroU32::MIN,
            absent(escalation, source),
            format!(
                "`{}` drops a `{}` answered `{outcome}` after one attempt",
                binding.name, invoked.name
            ),
        ),
        ResolvedRefusalAction::Escalate { emits } => {
            let escalated = EventRef::from(emits);
            source.insert(escalated.clone().into());
            (
                None,
                NonZeroU32::MIN,
                vec![
                    ScenarioStep::EventuallyEvent {
                        event: escalated.clone(),
                        payload: BTreeMap::new(),
                        shape: payload_shape(ir, &escalated),
                    },
                    // Once for the one escalating attempt: a second publication is a duplicate
                    // escalation, whatever the attempt count says.
                    ScenarioStep::ExpectPublicationCount {
                        event: escalated.clone(),
                        count: NonZeroU32::MIN,
                    },
                ],
                format!(
                    "a `{}` answered `{outcome}` makes `{}` escalate into `{escalated}` once",
                    invoked.name, binding.name
                ),
            )
        }
        ResolvedRefusalAction::Retry { bound: Some(bound) } if bound.is_final(outcome) => (
            None,
            NonZeroU32::MIN,
            absent(escalation, source),
            format!(
                "`{}` makes one attempt at `{}` when it is answered with the final `{outcome}`",
                binding.name, invoked.name
            ),
        ),
        ResolvedRefusalAction::Retry { bound: Some(bound) } => {
            let attempts = NonZeroU32::new(bound.attempts).unwrap_or(NonZeroU32::MIN);
            (
                Some(attempts),
                attempts,
                absent(escalation, source),
                format!(
                    "`{}` stops after {attempts} attempts at `{}`, each answered `{outcome}`",
                    binding.name, invoked.name
                ),
            )
        }
        ResolvedRefusalAction::Retry { bound: None } => {
            unreachable!("an unbounded retry is built by `retried`, from the prepared arrangement")
        }
    };
    Owed {
        times,
        invocations,
        after,
        text,
    }
}

/// The escalation event never published, where the binding has one to publish: what tells a
/// dropped or retried refusal from one escalated by the wrong policy.
fn absent(
    escalation: Option<&EventHandle>,
    source: &mut BTreeSet<EssSemanticRef>,
) -> Vec<ScenarioStep> {
    escalation
        .map(|emits| {
            let event = EventRef::from(emits);
            source.insert(event.clone().into());
            ScenarioStep::ExpectNoPublication { event }
        })
        .into_iter()
        .collect()
}
