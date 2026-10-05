//! The scenarios for a refusal-selected failure policy (ess/22, beyond10x/ess#269): one per
//! declared refusal of the invoked command, filed as `<binding>/binding/refusal/<outcome>`.
//!
//! Each refusal is forced the way §18 forces every failure — by injection, on a branch the
//! specification declares `external:` — and observed through the attempt count, the escalation
//! event, and its absence for the whole window
//! ([`ExpectNoPublication`](crate::ScenarioStep::ExpectNoPublication): the escalation is the
//! binding's, so the last command's direct events cannot show it). An unforceable refusal is
//! refused by name, a coverage limitation and never a fabricated result. The untyped fallback is no
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
    ResolvedCommand, Run, ScenarioId, ScenarioStep, TestStrategy,
};

/// Every declared refusal's scenario, or the refusal that says why there is none.
#[allow(clippy::too_many_arguments)]
pub(super) fn witnesses(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    invoked: &ResolvedCommand,
    trigger: &Run,
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
            retried(ir, binding, policy, &rule.outcome, invoked, prepared, event)
        } else {
            witness(
                ir,
                binding,
                policy,
                &rule.outcome,
                &rule.action,
                invoked,
                trigger,
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
    trigger: &Run,
    event: &EventRef,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Built {
    let command = CommandRef::new(invoked.name.clone());
    let forced_ref = forcible(invoked, outcome)?;
    let alternative = super::bounded_retry::clean_trigger(ir, binding, trigger, actors)?;
    let trigger = alternative.as_ref().map_or(trigger, |(run, _)| run);
    let mut source: BTreeSet<EssSemanticRef> = [command.clone().into(), forced_ref.clone().into()]
        .into_iter()
        .collect();
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    if let Some((_, extra)) = &alternative {
        source.extend(extra.iter().cloned());
    }
    let Owed {
        times,
        invocations,
        after,
        text,
    } = owed(ir, binding, policy, outcome, action, invoked, &mut source);
    let mut steps = trigger.setup.clone();
    steps.push(super::bounded_retry::force(forced_ref, times));
    steps.extend(trigger.invoke.iter().cloned());
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

/// The refusal as a scenario forces it: an `external:` branch whose eligibility needs no
/// observation of the mapped input, or the gap that names why not.
fn forcible(
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
    if let Some(gap) = super::forced_eligibility(invoked, forced) {
        return Err(gap);
    }
    Ok(forced_ref)
}

/// An unbounded retry of one refusal, witnessed as the universal `retry` is (§18,
/// [`super::on_failure`]): through the arrangement `binding_effects::prepare` made, so a row the
/// invoked command addresses exists before the trigger and decides the branch the next attempt
/// takes; the refusal forced once, then what that branch publishes, the row where it rests, and no
/// escalation. Not a count: the arrangement may itself invoke the binding, and an unbounded retry
/// promises the consequence, not a number.
fn retried(
    ir: &EssIr,
    binding: &ResolvedBinding,
    policy: &ResolvedRefusalPolicy,
    outcome: &ess_domain::command::OutcomeName,
    invoked: &ResolvedCommand,
    prepared: &Result<super::binding_effects::Prepared<'_>, BindingGap>,
    event: &EventRef,
) -> Built {
    let forced_ref = forcible(invoked, outcome)?;
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
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
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
