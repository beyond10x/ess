//! The scenarios for a bounded retry (ess/16 `on_failure: {retry: {attempts, final}}`,
//! beyond10x/ess#165): exactly `attempts` invocations when every one fails with a retried refusal,
//! and exactly one when the refusal is `final`.
//!
//! Both are forced the way §18 forces every failure — by injection, on a branch the specification
//! declares `external:` — and both are observed through the invocation count, which is the only
//! thing a bound changes. Neither requires the invoked command's success event: after the last
//! attempt the event's effect is lost, so a sender that stops is conformant and one that reaches
//! the success branch by attempting once more is not.
//!
//! | aspect | forced | required |
//! |---|---|---|
//! | `on-failure` | a retried refusal, on the next `attempts` invocations | exactly `attempts` invocations |
//! | `final-failure` | a `final` refusal, on the next invocation | exactly one invocation |
//!
//! `final-failure` exists only for a bound that names `final` refusals: a binding that states no
//! final refusal makes no claim for it to witness.

use std::num::NonZeroU32;

use ess_compiler::ir::{EventHandle, ResolvedRetryBound};

use super::{
    accepting_component, binding_source, clipped, insert, payload_shape, run, ActorRef, BTreeMap,
    BTreeSet, BindingAspect, BindingGap, BindingRef, Built, CommandRef, ConformanceScenario,
    ConformanceSuite, EssIr, EssSemanticRef, EventRef, OutcomeRef, QualifiedName, Refusal,
    RefusalCause, ResolvedBinding, ResolvedCommand, ResolvedFailure, ResolvedOutcome, Run,
    ScenarioId, ScenarioStep, TestStrategy,
};

/// `on-failure` for a bounded retry: a retried refusal forced on every attempt, and exactly
/// `attempts` invocations.
pub(super) fn exhausted(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    trigger: &Run,
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Built {
    let ResolvedFailure::BoundedRetry { bound } = binding.on_failure() else {
        unreachable!("`bindings` calls this for a bounded retry only")
    };
    let command = CommandRef::new(invoked.name.clone());
    // A failure the retry repeats: a refusal (`error:`) that is not final. An `external:` branch
    // that carries no error is a success the adapter decides, and a correct sender stops on it.
    let forced = forcible(invoked, |outcome| {
        outcome.error.is_some() && !bound.is_final(&outcome.name)
    })?
    .ok_or(BindingGap::RetriedUnforcible {
        command: command.clone(),
    })?;
    let attempts = NonZeroU32::new(bound.attempts).unwrap_or(NonZeroU32::MIN);
    let alternative = clean_trigger(ir, binding, trigger, actors)?;
    let trigger = alternative.as_ref().map_or(trigger, |(run, _)| run);
    let (steps, mut source) =
        forced_run(ir, binding, invoked, trigger, event, forced, Some(attempts));
    if let Some((_, extra)) = alternative {
        source.extend(extra);
    }
    let text = format!(
        "`{}` stops after {attempts} attempts at `{}`, each answered `{}`",
        binding.name, invoked.name, forced.name
    );
    Ok((steps, clipped(&text), source))
}

/// `final-failure`, filed beside the four aspects every binding has, for a bound that names
/// `final` refusals: the scenario, or the refusal that says why there is none.
#[allow(clippy::too_many_arguments)]
pub(super) fn final_failure(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    trigger: &Run,
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    source: &BTreeSet<EssSemanticRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let Some(bound) = binding.retry.as_ref() else {
        return;
    };
    if bound.final_outcomes.is_empty() {
        return;
    }
    let subject = BindingRef::new(binding.name.clone());
    let id = ScenarioId::Binding {
        binding: subject.clone(),
        aspect: BindingAspect::FinalFailure,
    };
    match single_attempt(ir, binding, bound, invoked, trigger, event, actors) {
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
                binding: subject,
                gap,
            },
        )),
    }
}

/// A `final` refusal forced once, and exactly one invocation.
fn single_attempt(
    ir: &EssIr,
    binding: &ResolvedBinding,
    bound: &ResolvedRetryBound,
    invoked: &ResolvedCommand,
    trigger: &Run,
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Built {
    let command = CommandRef::new(invoked.name.clone());
    let forced = forcible(invoked, |outcome| bound.is_final(&outcome.name))?.ok_or(
        BindingGap::FinalUnforcible {
            command: command.clone(),
        },
    )?;
    let alternative = clean_trigger(ir, binding, trigger, actors)?;
    let trigger = alternative.as_ref().map_or(trigger, |(run, _)| run);
    let (steps, mut source) = forced_run(ir, binding, invoked, trigger, event, forced, None);
    if let Some((_, extra)) = alternative {
        source.extend(extra);
    }
    let text = format!(
        "`{}` makes one attempt at `{}` when it is answered with the final `{}`",
        binding.name, invoked.name, forced.name
    );
    Ok((steps, clipped(&text), source))
}

/// A trigger whose arrangement does not itself publish the binding's event, so every invocation
/// the count sees is one the bounded retry made: `None` where the one `bindings` chose is already
/// such a trigger, otherwise the first other publisher of the event, in the model's order, whose
/// arrangement is clean, with what it depends on.
///
/// A step of the arrangement sets the binding off when it executes a command that [`reaches`] the
/// event — read conservatively, whichever branch the step takes and whichever branch each command a
/// binding invokes on the way takes. Where every publisher needs such an arrangement, the count
/// would include attempts the bound did not make, so the scenario is refused by name.
pub(super) fn clean_trigger(
    ir: &EssIr,
    binding: &ResolvedBinding,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Option<(Run, BTreeSet<EssSemanticRef>)>, BindingGap> {
    let handle = binding
        .cause
        .event()
        .expect("a bounded retry is on an event binding");
    let sets_off = |setup: &[ScenarioStep]| {
        setup.iter().any(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => {
                reaches(ir, command.name(), handle)
            }
            _ => false,
        })
    };
    if !sets_off(&trigger.setup) {
        return Ok(None);
    }
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            if !outcome.emits.contains(handle) {
                continue;
            }
            let Ok(candidate) = run(ir, command, outcome, actors) else {
                continue;
            };
            if !sets_off(&candidate.setup) {
                let source = binding_source(ir, binding, command, outcome, &candidate);
                return Ok(Some((candidate, source)));
            }
        }
    }
    Err(BindingGap::ArrangementSetsOff {
        event: EventRef::from(handle),
    })
}

/// Whether executing `start` can publish `event`: a branch of it emits the event, or emits one an
/// event binding reacts to by invoking a command that, in turn, reaches it.
///
/// Any branch counts at every hop, and every binding on an emitted event is followed whatever its
/// selection or failure policy, so the answer errs towards "sets off". Each command is visited
/// once, so a cycle of bindings terminates.
fn reaches(ir: &EssIr, start: &QualifiedName, event: &EventHandle) -> bool {
    let mut visited = BTreeSet::new();
    let mut pending = vec![start.clone()];
    while let Some(name) = pending.pop() {
        if !visited.insert(name.clone()) {
            continue;
        }
        let Some(command) = ir.commands().get(&name) else {
            continue;
        };
        let emitted: BTreeSet<&EventHandle> = command
            .outcomes
            .iter()
            .flat_map(|outcome| outcome.emits.iter())
            .collect();
        if emitted.contains(event) {
            return true;
        }
        pending.extend(
            ir.bindings()
                .values()
                .filter(|other| other.cause.event().is_some_and(|on| emitted.contains(on)))
                .map(|other| other.command.name().clone())
                .filter(|next| !visited.contains(next)),
        );
    }
    false
}

/// The first branch a scenario can force that `wanted` accepts, in declaration order, or `None`.
///
/// The same reading [`super::on_failure`] makes of an unbounded policy: only an `external:` branch
/// can be forced, and one whose eligibility a guard over the mapped input decides needs an
/// observation of that input first.
fn forcible(
    invoked: &ResolvedCommand,
    wanted: impl Fn(&ResolvedOutcome) -> bool,
) -> Result<Option<&ResolvedOutcome>, BindingGap> {
    let Some(forced) = invoked
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::InjectFault && wanted(outcome))
    else {
        return Ok(None);
    };
    if let Some(gap) = super::forced_eligibility(invoked, forced) {
        return Err(gap);
    }
    Ok(Some(forced))
}

/// The arrangement, the forced branch armed before the trigger, the trigger, and the count.
///
/// Armed after the arrangement for [`super::on_failure`]'s reason: a control armed first is spent
/// on whatever the arrangement's own commands set off.
fn forced_run(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    trigger: &Run,
    event: &EventRef,
    forced: &ResolvedOutcome,
    times: Option<NonZeroU32>,
) -> (Vec<ScenarioStep>, BTreeSet<EssSemanticRef>) {
    let command = CommandRef::new(invoked.name.clone());
    let forced_ref = OutcomeRef::new(command.clone(), forced.name.clone());
    let mut steps = trigger.setup.clone();
    steps.push(force(forced_ref.clone(), times));
    steps.extend(trigger.invoke.iter().cloned());
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    steps.push(count(
        BindingRef::new(binding.name.clone()),
        command.clone(),
        times.unwrap_or(NonZeroU32::MIN),
    ));

    let mut source: BTreeSet<EssSemanticRef> =
        [command.into(), forced_ref.into()].into_iter().collect();
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    (steps, source)
}

/// Force `outcome` on the next invocation, or on the next `times` of them.
pub(super) fn force(outcome: OutcomeRef, times: Option<NonZeroU32>) -> ScenarioStep {
    ScenarioStep::ConfigureExternalOutcome {
        force: outcome,
        times,
    }
}

/// Require exactly `count` invocations of `command` by `binding`.
pub(super) fn count(binding: BindingRef, command: CommandRef, count: NonZeroU32) -> ScenarioStep {
    ScenarioStep::ExpectInvocation {
        binding,
        command,
        input: BTreeMap::new(),
        count: Some(count),
    }
}
