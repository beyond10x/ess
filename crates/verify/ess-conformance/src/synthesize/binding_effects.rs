//! Bindings that move the row a step acts on, and the row a binding scenario addresses
//! (beyond10x/ess#266, beyond10x/ess#267, `docs/design/binding-arrangement-and-drop.md`).
//!
//! A binding is part of the running system and delivers eventually. So a step whose event sets off
//! a binding on its **own** row has not left the row where its branch says: the row rests where the
//! binding chain leaves it, once it has run. Arrangement routes only through rows at rest, adds the
//! eventual observation that establishes each one, and never sends a bound command explicitly to a
//! row a binding is about to move — that would race the binding and, against a target that runs
//! it, invoke the bound command twice.
//!
//! | what a step sets off on its own row | what arrangement does |
//! |---|---|
//! | nothing, or only refusals | the row rests where the branch leaves it, as before |
//! | one deterministic chain | it rests at the chain's end, observed by an [`EventuallyView`](ScenarioStep::EventuallyView) of an unfiltered view of identity and state, or by an [`EventuallyEvent`](ScenarioStep::EventuallyEvent) of the last binding's event carrying the row's literal identity |
//! | two bindings at once, a branch an input decides, a deletion, a cycle, or neither observation | no rest: the route is not taken, and a route needing it is refused naming the binding |
//!
//! Only bindings whose mapped identity *is* the step's own row are followed here. A binding that
//! addresses another row the step's input names, or a row the arrangement cannot know, moves a row
//! this arrangement is not building; general completion over arbitrary rows is #361/#362's.
//!
//! The second half is the binding scenarios' own destination: which row the invoked command acts
//! on, whether its identity is knowable before the trigger, the state it must be arranged in for
//! an accepting branch to be reached, and — for `drop` — whether it can be read unchanged.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    Driver, EntityHandle, EssIr, ResolvedBinding, ResolvedCommand, ResolvedCondition,
    ResolvedEffect, ResolvedInstance, ResolvedMappingValue, ResolvedOutcome, ResolvedPayloadValue,
    ResolvedView,
};
use ess_domain::command::TestStrategy;
use ess_domain::entity::{EntitySpec, StateName};
use ess_domain::view::{AssertionStyle, Consistency};
use ess_primitives::node::Node;

use super::{
    accepting_component, admits_held_state, arrange_first, captured, clipped, forced_eligibility,
    instance_name, paging, payload_shape, reachable_branch, Arrangement, BindingGap, Built,
    Distinction, Refusal, RefusalCause, Run, Unreachable,
};
use crate::scenario::{
    ActorRef, BindingRef, CommandRef, EntityRef, EssSemanticRef, EventRef, InstanceName,
    OutcomeRef, ScenarioId, ScenarioStep, ScenarioValue, ViewExpectation, ViewRef,
};
use ess_domain::name::QualifiedName;

/// How long a chain of bindings may run on one row before it is called unsettled.
const CHAIN_LIMIT: usize = 32;

// ---- which row a binding addresses -------------------------------------------------------------

/// The row a binding's invoked command acts on, seen from the step that publishes its event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Destination<'a> {
    /// The invoked command acts on no row that must already exist: it names no subject, or it
    /// creates one.
    Unaddressed,
    /// The publishing step's own subject: the mapped identity is the one the step acted on.
    SameRow {
        /// Its entity.
        entity: &'a EntityHandle,
    },
    /// Another row, whose identity the publishing command's caller supplies in `field`.
    TriggerInput {
        /// Its entity.
        entity: &'a EntityHandle,
        /// The publishing command's input carrying the identity.
        field: String,
    },
    /// No identity for the row is knowable before the trigger runs.
    Unavailable {
        /// The invoked command's input naming the row.
        input: String,
        /// Why it is not knowable beforehand.
        why: &'static str,
    },
}

/// The entity and the input naming the existing row the invoked command acts on, where it acts on
/// one. Every branch acting on an existing row names it through the same input, or none is chosen.
fn addressed(invoked: &ResolvedCommand) -> Option<(&EntityHandle, &str)> {
    let mut found: Option<(&EntityHandle, &str)> = None;
    for outcome in &invoked.outcomes {
        let Some(subject) = &outcome.subject else {
            continue;
        };
        if matches!(subject.effect, ResolvedEffect::Creates) {
            continue;
        }
        let ResolvedInstance::Supplied { field } = &subject.instance else {
            continue;
        };
        match found {
            None => found = Some((&subject.entity, field.name.as_str())),
            Some((entity, name)) if entity == &subject.entity && name == field.name => {}
            Some(_) => return None,
        }
    }
    found
}

/// Where the trigger's event field `field` of `event` comes from, as the trigger declares it.
fn payload_source<'a>(
    trigger: &'a ResolvedOutcome,
    event: &ess_compiler::ir::EventHandle,
    field: &str,
) -> Option<(&'a ResolvedPayloadValue, bool)> {
    trigger
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find(|declared| declared.target == field)
        .map(|declared| (&declared.value, declared.conversion.is_some()))
}

/// The row `binding` addresses when `trigger` publishes its event.
pub(super) fn destination<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    trigger: &'a ResolvedOutcome,
) -> Destination<'a> {
    let invoked = ir.command(&binding.command);
    let Some((entity, input)) = addressed(invoked) else {
        return Destination::Unaddressed;
    };
    let unavailable = |why| Destination::Unavailable {
        input: input.to_owned(),
        why,
    };
    let Some(event) = binding.cause.event() else {
        return unavailable("no event carries it");
    };
    let Some(mapped) = binding.mapping.iter().find(|mapped| mapped.target == input) else {
        return unavailable("the binding does not map it");
    };
    let field = match (&mapped.value, &mapped.conversion) {
        (ResolvedMappingValue::EventField { field, .. }, None) => field.as_str(),
        (ResolvedMappingValue::EventField { .. }, Some(_)) => {
            return unavailable("it is converted on the way in, and a conversion is not a value")
        }
        _ => return unavailable("it is not read from a field of the triggering event"),
    };
    let source = payload_source(trigger, event, field);
    if let Some(subject) = &trigger.subject {
        if &subject.entity == entity {
            let same = match &subject.instance {
                ResolvedInstance::Observed {
                    event: published,
                    field: identity,
                } => published == event && identity.name == field,
                ResolvedInstance::Supplied { field: identity } => matches!(
                    source,
                    Some((ResolvedPayloadValue::InputField { field: from, .. }, false))
                        if *from == identity.name
                ),
            };
            if same {
                return Destination::SameRow { entity };
            }
        }
    }
    match source {
        // A path (ess/22, A4) reads inside a struct input, where no arranged identity is sent.
        Some((ResolvedPayloadValue::InputField { field: from, .. }, false))
            if ess_domain::command::input_path::is_path(from) =>
        {
            unavailable("it is read from a member of a struct input")
        }
        Some((ResolvedPayloadValue::InputField { field: from, .. }, false)) => {
            Destination::TriggerInput {
                entity,
                field: from.clone(),
            }
        }
        Some((ResolvedPayloadValue::Generated, _)) => unavailable(
            "the triggering event carries an identity the implementation mints, known only after \
             the trigger",
        ),
        Some((ResolvedPayloadValue::ResponseField { .. }, _)) => {
            unavailable("the triggering event carries a response value, known only after the trigger")
        }
        _ => unavailable(
            "the triggering event field is not one the trigger's caller supplies, so no row can be \
             arranged for it beforehand",
        ),
    }
}

// ---- which branch the invoked command takes on a row in a known state --------------------------

/// What an invocation on a row resting in a known state reaches.
#[derive(Debug, Clone, Copy)]
pub(super) enum Reached<'a> {
    /// An accepting branch.
    Accepted(&'a ResolvedOutcome),
    /// A refusal — `wrong_state:`, or nothing the command declares — which changes nothing.
    Refused,
}

/// Whether `outcome`'s own move, if it has one, runs from `held`.
fn moves_from(outcome: &ResolvedOutcome, held: &StateName) -> bool {
    outcome
        .subject
        .as_ref()
        .and_then(|subject| subject.effect.transition())
        .is_none_or(|transition| transition.from.contains(held))
}

/// Where `outcome` leaves a row that rested in `held`.
pub(super) fn moved_to(outcome: &ResolvedOutcome, held: &StateName) -> StateName {
    outcome
        .subject
        .as_ref()
        .and_then(|subject| subject.effect.transition())
        .map_or_else(|| held.clone(), |transition| transition.to.clone())
}

/// The branch a bound invocation takes on an existing row resting in `held` (binding-arrangement-
/// and-drop.md, "Eligible destination state").
///
/// `wrong_state:` describes the states no accepting branch admits; it is reached only there, and is
/// never a second candidate beside an accepting branch the held state selects. An `external:`
/// branch is taken only when forced. A branch an input or a stored value decides is a choice no
/// arranged state settles, and keeps [`BindingGap::BranchUndecided`] — this never picks one by
/// position.
pub(super) fn bound_branch<'a>(
    invoked: &'a ResolvedCommand,
    held: &StateName,
) -> Result<Reached<'a>, BindingGap> {
    let undecided = || BindingGap::BranchUndecided {
        command: CommandRef::new(invoked.name.clone()),
        branches: invoked
            .outcomes
            .iter()
            .filter(|outcome| outcome.test_strategy != TestStrategy::InjectFault)
            .map(|outcome| outcome.name.clone())
            .collect(),
    };
    let mut accepting = Vec::new();
    for outcome in &invoked.outcomes {
        match &outcome.condition {
            ResolvedCondition::External { .. }
            | ResolvedCondition::ExternalWhen { .. }
            | ResolvedCondition::WrongState
            | ResolvedCondition::UnknownInstance
            | ResolvedCondition::ExistingInstance
            | ResolvedCondition::InputAbsent => continue,
            ResolvedCondition::Otherwise => {}
            ResolvedCondition::SubjectState {
                predicate: None, ..
            }
            | ResolvedCondition::StateChange {
                predicate: None, ..
            } => {
                if !admits_held_state(&outcome.condition, held) {
                    continue;
                }
            }
            _ => return Err(undecided()),
        }
        if outcome.replays.is_some() {
            return Err(undecided());
        }
        if moves_from(outcome, held) {
            accepting.push(outcome);
        }
    }
    match accepting.as_slice() {
        [] => Ok(Reached::Refused),
        [only] => Ok(Reached::Accepted(only)),
        several => {
            // A branch the held state selects precedes the default beside it.
            let selected: Vec<&&ResolvedOutcome> = several
                .iter()
                .filter(|outcome| !matches!(outcome.condition, ResolvedCondition::Otherwise))
                .collect();
            match selected.as_slice() {
                [one] => Ok(Reached::Accepted(one)),
                _ => Err(undecided()),
            }
        }
    }
}

// ---- where a row rests -------------------------------------------------------------------------

/// One binding run on a row: the binding, the branch it reaches, and where that leaves the row.
#[derive(Debug, Clone)]
pub(super) struct Link<'a> {
    /// The binding.
    pub(super) binding: &'a ResolvedBinding,
    /// The branch of its invoked command the row's state selects.
    pub(super) outcome: &'a ResolvedOutcome,
}

/// Where a row a step acted on rests once every binding the step sets off on it has run.
#[derive(Debug, Clone)]
pub(super) enum Settled<'a> {
    /// At rest in `state`, after `chain` ran (empty where the step set nothing off on it).
    Rests {
        /// Where it rests.
        state: StateName,
        /// The bindings that ran on it, in order.
        chain: Vec<Link<'a>>,
    },
    /// No rest that synthesis can name.
    Unsettled {
        /// The binding the chain could not get past.
        binding: BindingRef,
        /// Why.
        why: String,
    },
}

/// The event-caused bindings `outcome`'s events set off on the row `outcome` acts on.
fn same_row<'a>(ir: &'a EssIr, outcome: &'a ResolvedOutcome) -> Vec<&'a ResolvedBinding> {
    if outcome.subject.is_none() || outcome.emits.is_empty() {
        return Vec::new();
    }
    ir.bindings()
        .values()
        .filter(|binding| binding.context.is_none() && binding.cause.periodic().is_none())
        .filter(|binding| {
            binding
                .cause
                .event()
                .is_some_and(|event| outcome.emits.contains(event))
        })
        .filter(|binding| {
            matches!(
                destination(ir, binding, outcome),
                Destination::SameRow { .. }
            )
        })
        .collect()
}

/// Every event-caused binding `trigger`'s events set off that addresses `row`: the whole fan-out a
/// binding scenario's trigger sends to its destination, the binding under test among them.
fn fan_out<'a>(
    ir: &'a EssIr,
    trigger: &'a ResolvedOutcome,
    row: &Destination<'_>,
) -> Vec<&'a ResolvedBinding> {
    ir.bindings()
        .values()
        .filter(|binding| binding.context.is_none() && binding.cause.periodic().is_none())
        .filter(|binding| {
            binding
                .cause
                .event()
                .is_some_and(|event| trigger.emits.contains(event))
        })
        .filter(|binding| destination(ir, binding, trigger) == *row)
        .collect()
}

/// Where the row `outcome` left in `state` rests once the bindings it sets off on it have run.
pub(super) fn settle<'a>(
    ir: &'a EssIr,
    outcome: &'a ResolvedOutcome,
    state: &StateName,
) -> Settled<'a> {
    settle_from(ir, same_row(ir, outcome), state)
}

/// [`settle`], for a row in `state` that `first` is delivered to at once; every binding a link's
/// own events set off on the row follows, as there.
fn settle_from<'a>(
    ir: &'a EssIr,
    first: Vec<&'a ResolvedBinding>,
    state: &StateName,
) -> Settled<'a> {
    let mut state = state.clone();
    let mut next = first;
    let mut chain: Vec<Link<'a>> = Vec::new();
    let mut seen: BTreeSet<(String, StateName)> = BTreeSet::new();
    loop {
        let mut acting: Vec<(&ResolvedBinding, &ResolvedOutcome)> = Vec::new();
        for binding in next {
            // Whether a conditioned binding fires depends on the payload this step publishes
            // (ess/22, beyond10x/ess#268), which arrangement does not decide: no rest is named.
            if let Some(condition) = &binding.condition {
                return Settled::Unsettled {
                    binding: BindingRef::new(binding.name.clone()),
                    why: format!(
                        "whether it runs on a row in `{state}` depends on its event-payload \
                         condition `{}`, which arrangement does not decide",
                        condition.plan.predicate
                    ),
                };
            }
            match bound_branch(ir.command(&binding.command), &state) {
                Ok(Reached::Refused) => {}
                Ok(Reached::Accepted(branch)) => acting.push((binding, branch)),
                Err(gap) => {
                    return Settled::Unsettled {
                        binding: BindingRef::new(binding.name.clone()),
                        why: format!("on a row in `{state}`, it {gap}"),
                    }
                }
            }
        }
        let (binding, branch) = match acting.as_slice() {
            [] => return Settled::Rests { state, chain },
            [one] => *one,
            several => {
                return Settled::Unsettled {
                    binding: BindingRef::new(several[0].0.name.clone()),
                    why: format!(
                        "{} act on one row in `{state}` at once, and which runs first is the \
                         transport's",
                        several
                            .iter()
                            .map(|(binding, _)| format!("`{}`", binding.name))
                            .collect::<Vec<_>>()
                            .join(" and ")
                    ),
                }
            }
        };
        let name = BindingRef::new(binding.name.clone());
        if branch
            .subject
            .as_ref()
            .is_some_and(|subject| matches!(subject.effect, ResolvedEffect::Deletes))
        {
            return Settled::Unsettled {
                binding: name,
                why: format!("it removes the row through `{}`", branch.name),
            };
        }
        if !seen.insert((binding.name.to_string(), state.clone())) || chain.len() >= CHAIN_LIMIT {
            return Settled::Unsettled {
                binding: name,
                why: format!(
                    "it sets itself off again from `{state}`, so the chain never comes to rest"
                ),
            };
        }
        state = moved_to(branch, &state);
        chain.push(Link {
            binding,
            outcome: branch,
        });
        next = same_row(ir, branch);
    }
}

/// The view an eventual observation of a row's state reads: unfiltered, unparameterised, read
/// whole, projecting the identity and the state at the entity's own types. The first by name.
fn state_view<'a>(ir: &'a EssIr, entity: &EntityHandle) -> Option<&'a ResolvedView> {
    let declared = ir.entity(entity);
    ir.views().values().find(|view| {
        !view.is_aggregate()
            && &view.source == entity
            && view.filter.is_none()
            && view.params.is_empty()
            && paging::read_whole(view)
            && view
                .field(&declared.identity.name)
                .is_some_and(|field| field.type_ref == declared.identity.type_ref)
            && view
                .field(EntitySpec::STATE)
                .is_some_and(|field| field.type_ref == declared.state_field().type_ref)
    })
}

/// The eventual observation that a row named `identity` rests in `state`.
pub(super) fn observe(
    ir: &EssIr,
    entity: &EntityHandle,
    identity: ScenarioValue,
    state: &StateName,
) -> Option<ScenarioStep> {
    let view = state_view(ir, entity)?;
    let fields = [
        (ir.entity(entity).identity.name.clone(), identity),
        (
            EntitySpec::STATE.to_owned(),
            ScenarioValue::literal(Node::Text(state.to_string())),
        ),
    ]
    .into_iter()
    .collect();
    Some(ScenarioStep::EventuallyView {
        view: ViewRef::new(view.name.clone()),
        params: BTreeMap::new(),
        expectation: ViewExpectation::Contains { fields },
    })
}

/// Where a route may take a row `outcome` leaves in `state`: where it rests, where that rest can be
/// observed; `None` where it cannot be named.
pub(super) fn resting(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    state: &StateName,
) -> Option<StateName> {
    match settle(ir, outcome, state) {
        Settled::Rests { state, chain } if chain.is_empty() => Some(state),
        Settled::Rests { state, chain } => {
            let entity = &outcome.subject.as_ref()?.entity;
            (state_view(ir, entity).is_some() || identity_event(chain.last()?).is_some())
                .then_some(state)
        }
        Settled::Unsettled { .. } => None,
    }
}

/// Why a rest a binding chain reaches cannot be observed.
const UNOBSERVABLE: &str = "no unfiltered view shows the row's identity and state, and the last \
                            binding's event does not carry an identity the arrangement supplied, \
                            so where the binding leaves the row cannot be observed";

/// The event the link's branch publishes that carries the row's identity, and the field it is
/// carried in: where the event's `payload:` copies the invoked command's identity input unconverted.
fn identity_event(link: &Link<'_>) -> Option<(EventRef, String)> {
    let subject = link.outcome.subject.as_ref()?;
    let ResolvedInstance::Supplied { field: identity } = &subject.instance else {
        return None;
    };
    link.outcome.payload.iter().find_map(|payload| {
        payload
            .fields
            .iter()
            .find_map(|declared| match &declared.value {
                ResolvedPayloadValue::InputField { field, .. }
                    if *field == identity.name && declared.conversion.is_none() =>
                {
                    Some((EventRef::from(&payload.event), declared.target.clone()))
                }
                _ => None,
            })
    })
}

/// The literal identity of an arranged row, where the creation that made it published an identity
/// copied unconverted from a literal it was sent.
fn literal_identity(ir: &EssIr, arrangement: &Arrangement) -> Option<Node> {
    let (at, event, field) =
        arrangement
            .steps
            .iter()
            .enumerate()
            .find_map(|(at, step)| match step {
                ScenarioStep::CaptureInstance {
                    instance,
                    event,
                    field,
                    ..
                } if *instance == arrangement.instance => Some((at, event, field)),
                _ => None,
            })?;
    let (command, input) = arrangement.steps[..at]
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => Some((command, input)),
            _ => None,
        })?;
    let command = ir.commands().get(command.name())?;
    let source = command.outcomes.iter().find_map(|outcome| {
        outcome
            .payload
            .iter()
            .filter(|payload| EventRef::from(&payload.event) == *event)
            .flat_map(|payload| &payload.fields)
            .find_map(|declared| match &declared.value {
                ResolvedPayloadValue::InputField { field: from, .. }
                    if declared.target == *field && declared.conversion.is_none() =>
                {
                    Some(from.clone())
                }
                _ => None,
            })
    })?;
    match super::supplied_at(input, &source)? {
        ScenarioValue::Literal { value } => Some(value),
        _ => None,
    }
}

/// The eventual observation that the last binding of `chain` ran on this row: its event, carrying
/// the row's literal identity. Only where both are known — an occurrence of the event for another
/// row is not this row's rest.
fn observe_published(
    ir: &EssIr,
    chain: &[Link<'_>],
    arrangement: &Arrangement,
) -> Option<ScenarioStep> {
    let (event, field) = identity_event(chain.last()?)?;
    let identity = literal_identity(ir, arrangement)?;
    Some(ScenarioStep::EventuallyEvent {
        shape: payload_shape(ir, &event),
        event,
        payload: [(field, identity)].into_iter().collect(),
    })
}

/// Carries an arrangement a step just left in `arrangement.state` to where its bindings rest it,
/// with the observation that establishes it, or says why it cannot.
fn carry(
    ir: &EssIr,
    entity: &EntityHandle,
    outcome: &ResolvedOutcome,
    arrangement: &mut Arrangement,
) -> Result<(), Unreachable> {
    let at = OutcomeRef::new(
        CommandRef::new(
            command_of(ir, outcome)
                .map_or_else(|| entity_name(ir, entity), |command| command.name.clone()),
        ),
        outcome.name.clone(),
    );
    match settle(ir, outcome, &arrangement.state) {
        Settled::Rests { chain, .. } if chain.is_empty() => Ok(()),
        Settled::Rests { state, chain } => {
            // A chain that leaves the state where it was is not established by reading the state:
            // its own event, carrying this row's identity, is the observation that it ran.
            let unmoved = state == arrangement.state;
            let by_view = || observe(ir, entity, arrangement.identity(), &state);
            let by_event = || observe_published(ir, &chain, arrangement);
            let step = if unmoved {
                by_event().or_else(by_view)
            } else {
                by_view().or_else(by_event)
            }
            .ok_or_else(|| Unreachable::BindingUnsettled {
                binding: BindingRef::new(chain[0].binding.name.clone()),
                outcome: Box::new(at.clone()),
                why: UNOBSERVABLE.into(),
            })?;
            arrangement.steps.push(step);
            for link in &chain {
                arrangement.absorb(link.outcome, BTreeMap::new());
                arrangement
                    .source
                    .insert(BindingRef::new(link.binding.name.clone()).into());
            }
            arrangement.state = state;
            Ok(())
        }
        Settled::Unsettled { binding, why } => Err(Unreachable::BindingUnsettled {
            binding,
            outcome: Box::new(at),
            why: why.into(),
        }),
    }
}

/// [`carry`] for a row a creation just made. A rest that cannot be named leaves the row where the
/// creation put it: every route [`super::arrange`] takes avoids such a creation already, through
/// [`resting`].
pub(super) fn settle_created(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &ResolvedOutcome,
    arrangement: &mut Arrangement,
) {
    let mut carried = arrangement.clone();
    if carry(ir, entity, creator, &mut carried).is_ok() {
        *arrangement = carried;
    }
}

/// [`carry`] for a row a route step just moved.
pub(super) fn settle_moved(
    ir: &EssIr,
    entity: &EntityHandle,
    outcome: &ResolvedOutcome,
    arrangement: &mut Arrangement,
) -> Result<(), Unreachable> {
    carry(ir, entity, outcome, arrangement)
}

/// Why no route reaches `target` at rest, where a binding is the reason: every step landing there
/// sets one off that moves the row on, or one whose effect cannot be settled.
pub(super) fn bound_away(
    ir: &EssIr,
    drivers: &[Driver<'_>],
    target: &StateName,
) -> Option<Unreachable> {
    for driver in drivers {
        let landing = match driver.effect {
            ResolvedEffect::Creates => super::born(ir, driver).clone(),
            ResolvedEffect::Moves { transition } => transition.to.clone(),
            _ => continue,
        };
        if &landing != target {
            continue;
        }
        let at = OutcomeRef::new(
            CommandRef::new(driver.command.name.clone()),
            driver.outcome.name.clone(),
        );
        match settle(ir, driver.outcome, &landing) {
            Settled::Rests { state, chain } if !chain.is_empty() => {
                return Some(if &state == target {
                    Unreachable::BindingUnsettled {
                        binding: BindingRef::new(chain[0].binding.name.clone()),
                        outcome: Box::new(at),
                        why: UNOBSERVABLE.into(),
                    }
                } else {
                    Unreachable::BoundAway {
                        state: target.clone(),
                        binding: BindingRef::new(chain[0].binding.name.clone()),
                    }
                })
            }
            Settled::Unsettled { binding, why } => {
                return Some(Unreachable::BindingUnsettled {
                    binding,
                    outcome: Box::new(at),
                    why: why.into(),
                })
            }
            Settled::Rests { .. } => {}
        }
    }
    // A creation every route starts from, whose rest cannot be named: the binding is the reason
    // nothing reaches the state, not the lifecycle.
    for driver in drivers
        .iter()
        .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates))
    {
        let born = super::born(ir, driver);
        if resting(ir, driver.outcome, born).is_some() {
            continue;
        }
        let at = Box::new(OutcomeRef::new(
            CommandRef::new(driver.command.name.clone()),
            driver.outcome.name.clone(),
        ));
        return Some(match settle(ir, driver.outcome, born) {
            Settled::Unsettled { binding, why } => Unreachable::BindingUnsettled {
                binding,
                outcome: at,
                why: why.into(),
            },
            Settled::Rests { chain, .. } => Unreachable::BindingUnsettled {
                binding: BindingRef::new(chain.first()?.binding.name.clone()),
                outcome: at,
                why: UNOBSERVABLE.into(),
            },
        });
    }
    None
}

fn command_of<'a>(ir: &'a EssIr, outcome: &ResolvedOutcome) -> Option<&'a ResolvedCommand> {
    ir.commands().values().find(|command| {
        command
            .outcomes
            .iter()
            .any(|declared| std::ptr::eq(declared, outcome))
    })
}

fn entity_name(ir: &EssIr, entity: &EntityHandle) -> QualifiedName {
    ir.entity(entity).name.clone()
}

// ---- the branch under test ---------------------------------------------------------------------

/// Carries the run of a branch under test to where the bindings it sets off on its own row leave
/// that row, so its view expectations describe the row at rest. `true` where the row moves on, and
/// every read of it has to be eventual; a rest that cannot be named is refused under `id` and
/// leaves the run with no state to read.
pub(super) fn settle_run(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    run: &mut Run,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) -> bool {
    let Some(state) = run.after.clone() else {
        return false;
    };
    match settle(ir, outcome, &state) {
        Settled::Rests { chain, .. } if chain.is_empty() => false,
        Settled::Rests { state, chain } => {
            for link in &chain {
                super::absorb(&mut run.settled, link.outcome, BTreeMap::new());
                run.source
                    .insert(BindingRef::new(link.binding.name.clone()).into());
            }
            run.after = Some(state);
            true
        }
        Settled::Unsettled { binding, why } => {
            let at = command_of(ir, outcome).map(|command| {
                OutcomeRef::new(CommandRef::new(command.name.clone()), outcome.name.clone())
            });
            if let Some(at) = at {
                let refusal = Refusal::about(
                    id,
                    RefusalCause::BindingUnobservable {
                        binding,
                        gap: BindingGap::EffectUnsettled { outcome: at, why },
                    },
                );
                if !refusals.contains(&refusal) {
                    refusals.push(refusal);
                }
            }
            run.after = None;
            false
        }
    }
}

/// `steps` with every immediate read of a view turned into an eventual one: what a row a binding is
/// still moving shows is a claim about where it comes to rest.
pub(super) fn eventually(steps: Vec<ScenarioStep>) -> Vec<ScenarioStep> {
    let mut params: BTreeMap<ViewRef, BTreeMap<String, ScenarioValue>> = BTreeMap::new();
    let mut out = Vec::with_capacity(steps.len());
    for step in steps {
        match step {
            ScenarioStep::QueryView { view, params: read } => {
                params.insert(view, read);
            }
            ScenarioStep::ExpectView { view, expectation } => {
                let read = params.get(&view).cloned().unwrap_or_default();
                out.push(ScenarioStep::EventuallyView {
                    view,
                    params: read,
                    expectation,
                });
            }
            other => out.push(other),
        }
    }
    out
}

// ---- a binding scenario's trigger and destination ----------------------------------------------

/// A binding scenario's trigger, with the row its invoked command addresses arranged before it.
pub(super) struct Prepared<'a> {
    /// Everything before the trigger's invocation: its own arrangement, then the destination's.
    pub(super) setup: Vec<ScenarioStep>,
    /// The trigger's invocation, naming the destination where its input carries it.
    pub(super) invoke: Vec<ScenarioStep>,
    /// Steps right after the trigger that name a destination the trigger itself made.
    pub(super) capture: Vec<ScenarioStep>,
    /// The branch the bound invocation reaches.
    pub(super) reached: Result<&'a ResolvedOutcome, BindingGap>,
    /// The destination row, where the invoked command addresses one.
    pub(super) row: Option<Row<'a>>,
    /// What the destination's arrangement depends on.
    pub(super) source: BTreeSet<EssSemanticRef>,
}

/// The existing row a bound invocation acts on.
pub(super) struct Row<'a> {
    /// Its entity.
    pub(super) entity: &'a EntityHandle,
    /// What the scenario calls it.
    pub(super) identity: ScenarioValue,
    /// The state it is in when the trigger's bindings are delivered to it.
    pub(super) start: StateName,
    /// Every binding the trigger's events set off on it, the one under test included.
    pub(super) fan: Vec<&'a ResolvedBinding>,
    /// Where it rests once the whole fan-out and anything it sets off have run, where that is
    /// observable.
    pub(super) settled: Option<StateName>,
    /// The binding the fan-out could not get past, the trigger, and why, where it does not settle:
    /// then no state of the row is asserted, and the scenario names the limitation.
    pub(super) unsettled: Option<(BindingRef, OutcomeRef, String)>,
    /// `Ok` where the row exists before the trigger and the trigger leaves it as it was, so it can
    /// be read before the trigger and compared after; otherwise why not.
    pub(super) before_trigger: Result<(), &'static str>,
}

impl Prepared<'_> {
    /// The arrangement, the trigger, and the capture, in order.
    pub(super) fn steps(&self) -> Vec<ScenarioStep> {
        let mut steps = self.setup.clone();
        steps.extend(self.invoke.iter().cloned());
        steps.extend(self.capture.iter().cloned());
        steps
    }

    /// The eventual observation of the destination at rest, where there is one to make.
    pub(super) fn settled_row(&self, ir: &EssIr) -> Option<ScenarioStep> {
        let row = self.row.as_ref()?;
        observe(ir, row.entity, row.identity.clone(), row.settled.as_ref()?)
    }
}

/// Names, under the scenario `id`, a destination row whose rest the trigger's fan-out does not
/// settle (adversary pass 1, F2): the flow, delivery and failure scenarios keep what they observe
/// of events and invocations and assert no state of the row, and this says why. The mapping
/// asserts no state of the row, and the drop scenario names its own refusal.
pub(super) fn name_unsettled(
    prepared: &Result<Prepared<'_>, BindingGap>,
    aspect: crate::scenario::BindingAspect,
    dropping: bool,
    id: &ScenarioId,
    refusals: &mut Vec<Refusal>,
) {
    use crate::scenario::BindingAspect;
    if matches!(aspect, BindingAspect::Mapping | BindingAspect::FinalFailure)
        || (aspect == BindingAspect::OnFailure && dropping)
    {
        return;
    }
    let Some((binding, outcome, why)) = prepared
        .as_ref()
        .ok()
        .and_then(|prepared| prepared.row.as_ref())
        .and_then(|row| row.unsettled.clone())
    else {
        return;
    };
    let refusal = Refusal::about(
        id,
        RefusalCause::BindingUnobservable {
            binding,
            gap: BindingGap::EffectUnsettled { outcome, why },
        },
    );
    if !refusals.contains(&refusal) {
        refusals.push(refusal);
    }
}

/// An instance name for `entity` no step of `run` already uses.
fn unused_name(ir: &EssIr, entity: &EntityHandle, run: &Run) -> (InstanceName, Distinction) {
    let taken = captured(run);
    let name = &ir.entity(entity).name;
    let mut nth = 0;
    loop {
        let distinction = if nth == 0 {
            Distinction::PLAIN
        } else {
            Distinction::further(nth)
        };
        let instance = instance_name(name, distinction);
        if !taken.contains(&instance) {
            return (instance, distinction);
        }
        nth += 1;
    }
}

/// Where the trigger leaves the row it acts on, and whether that is a change to it.
fn trigger_changes(published_by: &ResolvedOutcome) -> Result<(), &'static str> {
    match published_by.subject.as_ref().map(|subject| &subject.effect) {
        Some(ResolvedEffect::Creates) => Err(
            "the trigger creates the row the binding addresses, so nothing can be read before it",
        ),
        Some(ResolvedEffect::Preserves) if published_by.sets.is_empty() => Ok(()),
        Some(_) => Err("the trigger itself changes the row the binding addresses"),
        None => Ok(()),
    }
}

/// The trigger of `binding`, with the row the invoked command addresses arranged in a state an
/// accepting branch admits — or the named gap that stops it.
pub(super) fn prepare<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Prepared<'a>, BindingGap> {
    let command = || CommandRef::new(invoked.name.clone());
    match destination(ir, binding, published_by) {
        Destination::Unaddressed => Ok(Prepared {
            setup: trigger.setup.clone(),
            invoke: trigger.invoke.clone(),
            capture: Vec::new(),
            reached: reachable_branch(invoked),
            row: None,
            source: BTreeSet::new(),
        }),
        Destination::Unavailable { input, why } => {
            Err(BindingGap::DestinationIdentityUnavailable {
                command: command(),
                input,
                why,
            })
        }
        Destination::SameRow { entity } => {
            same_row_destination(ir, invoked, published_by, entity, trigger)
        }
        Destination::TriggerInput { entity, field } => {
            arranged_destination(ir, invoked, published_by, entity, &field, trigger, actors)
        }
    }
}

/// [`prepare`] where the destination is the trigger's own row: its state is where the trigger
/// leaves it, and it is named by the trigger's arrangement or captured from the trigger's event.
fn same_row_destination<'a>(
    ir: &'a EssIr,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    entity: &'a EntityHandle,
    trigger: &Run,
) -> Result<Prepared<'a>, BindingGap> {
    let command = || CommandRef::new(invoked.name.clone());
    let held = trigger
        .after
        .clone()
        .ok_or_else(|| BindingGap::DestinationIdentityUnavailable {
            command: command(),
            input: field_name(invoked),
            why: "the trigger leaves no row the scenario can name",
        })?;
    let Reached::Accepted(reached) = bound_branch(invoked, &held)? else {
        return Err(BindingGap::DestinationIneligible {
            command: command(),
            state: held,
        });
    };
    let (identity, capture) = match (&trigger.instance, published_by.subject.as_ref()) {
        (Some(instance), _) => (ScenarioValue::instance(instance.clone()), Vec::new()),
        (None, Some(subject)) => {
            let ResolvedInstance::Observed { event, field } = &subject.instance else {
                return Err(BindingGap::DestinationIdentityUnavailable {
                    command: command(),
                    input: field_name(invoked),
                    why: "the trigger names no instance the scenario captured",
                });
            };
            let (instance, _) = unused_name(ir, entity, trigger);
            (
                ScenarioValue::instance(instance.clone()),
                vec![ScenarioStep::CaptureInstance {
                    instance,
                    entity: EntityRef::from(entity),
                    event: EventRef::from(event),
                    field: field.name.clone(),
                }],
            )
        }
        (None, None) => unreachable!("a same-row destination is a subject's row"),
    };
    Ok(Prepared {
        setup: trigger.setup.clone(),
        invoke: trigger.invoke.clone(),
        capture,
        reached: Ok(reached),
        row: Some(row(
            ir,
            published_by,
            entity,
            identity,
            held,
            fan_out(ir, published_by, &Destination::SameRow { entity }),
            trigger_changes(published_by),
        )),
        source: BTreeSet::new(),
    })
}

/// [`prepare`] where the trigger's caller names the destination in `field`: the row is arranged
/// first, through the settled routes, in a state the one accepting branch admits, and the trigger
/// is sent naming exactly it.
fn arranged_destination<'a>(
    ir: &'a EssIr,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    entity: &'a EntityHandle,
    field: &str,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Prepared<'a>, BindingGap> {
    let command = || CommandRef::new(invoked.name.clone());
    let mut eligible = Vec::new();
    let mut branches: BTreeSet<_> = BTreeSet::new();
    for state in &ir.entity(entity).lifecycle.states {
        if let Reached::Accepted(branch) = bound_branch(invoked, state)? {
            eligible.push(state.clone());
            branches.insert(branch.name.clone());
        }
    }
    // Two accepting branches, each in its own states, are two outcomes the arranged state would
    // pick between: that is a choice, and it stays named rather than made by position.
    if branches.len() > 1 {
        return Err(BindingGap::BranchUndecided {
            command: command(),
            branches: branches.into_iter().collect(),
        });
    }
    let Some(first) = eligible.first().cloned() else {
        return Err(BindingGap::DestinationIneligible {
            command: command(),
            state: ir.entity(entity).lifecycle.initial.clone(),
        });
    };
    let (_, distinction) = unused_name(ir, entity, trigger);
    let arrangement =
        arrange_first(ir, entity, &eligible, actors, distinction, &[]).map_err(|reason| {
            BindingGap::DestinationUnreachable {
                entity: EntityRef::from(entity),
                state: first,
                reason: Box::new(reason),
            }
        })?;
    let Reached::Accepted(reached) = bound_branch(invoked, &arrangement.state)? else {
        return Err(BindingGap::DestinationIneligible {
            command: command(),
            state: arrangement.state.clone(),
        });
    };
    let identity = arrangement.identity();
    let mut invoke = trigger.invoke.clone();
    let named = invoke.iter_mut().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { input, .. } if input.contains_key(field) => {
            input.insert(field.to_owned(), identity.clone());
            Some(())
        }
        _ => None,
    });
    if named.is_none() {
        return Err(BindingGap::DestinationIdentityUnavailable {
            command: command(),
            input: field_name(invoked),
            why: "the trigger is not sent with the input that names the row",
        });
    }
    let mut setup = trigger.setup.clone();
    setup.extend(arrangement.steps.iter().cloned());
    Ok(Prepared {
        setup,
        invoke,
        capture: Vec::new(),
        reached: Ok(reached),
        row: Some(row(
            ir,
            published_by,
            entity,
            identity,
            arrangement.state.clone(),
            fan_out(
                ir,
                published_by,
                &Destination::TriggerInput {
                    entity,
                    field: field.to_owned(),
                },
            ),
            Ok(()),
        )),
        source: arrangement.source,
    })
}

fn field_name(invoked: &ResolvedCommand) -> String {
    addressed(invoked).map_or_else(String::new, |(_, input)| input.to_owned())
}

/// The destination row as the trigger delivers to it: in `start`, sent the whole `fan` of bindings
/// the trigger's events set off on it — the one under test and every sibling — and where that
/// leaves it (adversary pass 1, F2). A sibling acting beside the binding under test is part of the
/// honest system, so the rest is the fan-out's, never the tested branch's chain alone.
fn row<'a>(
    ir: &'a EssIr,
    published_by: &'a ResolvedOutcome,
    entity: &'a EntityHandle,
    identity: ScenarioValue,
    start: StateName,
    fan: Vec<&'a ResolvedBinding>,
    before_trigger: Result<(), &'static str>,
) -> Row<'a> {
    let (settled, unsettled) = match settle_from(ir, fan.clone(), &start) {
        Settled::Rests { state, .. } => (state_view(ir, entity).map(|_| state), None),
        Settled::Unsettled { binding, why } => (
            None,
            command_of(ir, published_by).map(|command| {
                (
                    binding,
                    OutcomeRef::new(
                        CommandRef::new(command.name.clone()),
                        published_by.name.clone(),
                    ),
                    why,
                )
            }),
        ),
    };
    Row {
        entity,
        identity,
        start,
        fan,
        settled,
        unsettled,
        before_trigger,
    }
}

// ---- drop ---------------------------------------------------------------------------------------

/// The immediate view a destination is snapshotted through before the trigger and compared after:
/// read-your-writes, unfiltered, unparameterised, read whole, projecting the identity. First by
/// name.
fn snapshot_view<'a>(ir: &'a EssIr, entity: &EntityHandle) -> Option<&'a ResolvedView> {
    let declared = ir.entity(entity);
    ir.views().values().find(|view| {
        !view.is_aggregate()
            && &view.source == entity
            && view.filter.is_none()
            && view.params.is_empty()
            && paging::read_whole(view)
            && view.consistency == Consistency::ReadYourWrites
            && view.assertion_style == AssertionStyle::Expect
            && view
                .field(&declared.identity.name)
                .is_some_and(|field| field.type_ref == declared.identity.type_ref)
    })
}

/// §18 for `drop`, with what it can actually prove (binding-arrangement-and-drop.md, "What drop can
/// prove"): one forced refusal, exactly one attempt carrying the mapped input throughout the
/// eventual window, and the destination row as it was before the trigger.
///
/// A target with no external refusal to force, or a destination that cannot be read before the
/// trigger unchanged by it, keeps the precise limitation; an empty log is never taken for a drop.
pub(super) fn dropped(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    prepared: &Prepared<'_>,
    event: &EventRef,
) -> Built {
    let Some(row) = &prepared.row else {
        return Err(BindingGap::PolicySilent);
    };
    row.before_trigger
        .map_err(|why| BindingGap::UnchangedUnobservable { why })?;
    let view = snapshot_view(ir, row.entity).ok_or(BindingGap::UnchangedUnobservable {
        why: "no read-your-writes, unfiltered view shows the row by its identity",
    })?;
    let forced = invoked
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::InjectFault)
        .ok_or_else(|| BindingGap::NoForcibleFailure {
            command: CommandRef::new(invoked.name.clone()),
        })?;
    if let Some(gap) = forced_eligibility(invoked, forced) {
        return Err(gap);
    }
    let siblings = siblings_rest(ir, binding, row)?;
    let input = mapped_input(ir, binding, event)?;
    let command = CommandRef::new(invoked.name.clone());
    let forced_ref = OutcomeRef::new(command.clone(), forced.name.clone());
    let name = ViewRef::new(view.name.clone());
    let identity = ir.entity(row.entity).identity.name.clone();
    let bound = BindingRef::new(binding.name.clone());

    let mut steps = prepared.setup.clone();
    if siblings.is_none() {
        steps.push(ScenarioStep::QueryView {
            view: name.clone(),
            params: BTreeMap::new(),
        });
        steps.push(ScenarioStep::SnapshotSubject {
            view: name.clone(),
            subject: [(identity, row.identity.clone())].into_iter().collect(),
        });
    }
    steps.push(ScenarioStep::ConfigureExternalOutcome {
        force: forced_ref.clone(),
        times: None,
    });
    steps.extend(prepared.invoke.iter().cloned());
    steps.extend(prepared.capture.iter().cloned());
    steps.push(ScenarioStep::ExpectEvent {
        event: event.clone(),
        payload: BTreeMap::new(),
        shape: payload_shape(ir, event),
    });
    // Every attempt carries the mapped input — a malformed retry beside a correct one fails here —
    steps.push(ScenarioStep::ExpectEveryInvocation {
        binding: bound.clone(),
        command: command.clone(),
        selecting: BTreeMap::new(),
        input,
    });
    // — and there is exactly one of them for the whole window: none, or a retry, fails here.
    steps.push(ScenarioStep::ExpectInvocation {
        binding: bound,
        command: command.clone(),
        input: BTreeMap::new(),
        count: Some(std::num::NonZeroU32::MIN),
    });
    match siblings {
        // Unchanged: nothing else the trigger sets off acts on the row.
        None => {
            steps.push(ScenarioStep::QueryView {
                view: name.clone(),
                params: BTreeMap::new(),
            });
            steps.push(ScenarioStep::ExpectSubjectUnchanged { view: name.clone() });
        }
        // Where the siblings leave it, the dropped binding having changed nothing.
        Some(rest) => steps.push(rest),
    }

    let mut source: BTreeSet<EssSemanticRef> = [
        command.into(),
        forced_ref.into(),
        name.into(),
        EntityRef::from(row.entity).into(),
    ]
    .into_iter()
    .collect();
    if let Some(component) = accepting_component(ir, &invoked.name) {
        source.insert(component.into());
    }
    source.extend(prepared.source.iter().cloned());
    let text = format!(
        "`{}` gives a failed `{}` up after its one attempt, and the row stays as it was",
        binding.name, invoked.name
    );
    Ok((steps, clipped(&text), source))
}

/// What the drop scenario requires of the row besides one refused attempt, where another binding
/// the trigger sets off acts on it too (adversary pass 1, F1): `None` where none does, so the row
/// stays exactly as the pre-trigger snapshot showed it; otherwise the eventual read of the row where
/// those siblings leave it — its identity, its state, and every field their branches write a
/// literal to — since the honest system changes it and "unchanged" would fail a correct target.
///
/// Refused as [`BindingGap::UnchangedUnobservable`] where the siblings' rest cannot be named or
/// read, or where a sibling invokes the forced command and could take the forced refusal itself.
fn siblings_rest(
    ir: &EssIr,
    binding: &ResolvedBinding,
    row: &Row<'_>,
) -> Result<Option<ScenarioStep>, BindingGap> {
    let siblings: Vec<&ResolvedBinding> = row
        .fan
        .iter()
        .copied()
        .filter(|sibling| sibling.name != binding.name)
        .collect();
    if siblings
        .iter()
        .any(|sibling| sibling.command == binding.command)
    {
        return Err(BindingGap::UnchangedUnobservable {
            why: "another binding the trigger sets off invokes the same command, so the forced \
                  refusal is not this binding's alone",
        });
    }
    let (state, chain) = match settle_from(ir, siblings, &row.start) {
        Settled::Rests { chain, .. } if chain.is_empty() => return Ok(None),
        Settled::Rests { state, chain } => (state, chain),
        Settled::Unsettled { .. } => {
            return Err(BindingGap::UnchangedUnobservable {
                why: "another binding the trigger sets off acts on the row, and where it rests \
                      cannot be named",
            })
        }
    };
    let unreadable = BindingGap::UnchangedUnobservable {
        why: "another binding the trigger sets off changes the row, and no unfiltered view shows \
              its identity and state",
    };
    let view = state_view(ir, row.entity).ok_or_else(|| unreadable.clone())?;
    let declared = ir.entity(row.entity);
    let mut fields: BTreeMap<String, ScenarioValue> = BTreeMap::new();
    for link in &chain {
        for set in &link.outcome.sets {
            let written = match &set.value {
                ResolvedPayloadValue::Literal { value } if set.conversion.is_none() => {
                    super::literal_value(ir, &set.target_type, value, 0)
                }
                _ => None,
            };
            match written.filter(|_| {
                view.field(&set.target)
                    .is_some_and(|field| field.type_ref == set.target_type)
            }) {
                Some(value) => {
                    fields.insert(set.target.clone(), ScenarioValue::literal(value));
                }
                None => {
                    fields.remove(&set.target);
                }
            }
        }
    }
    fields.insert(declared.identity.name.clone(), row.identity.clone());
    fields.insert(
        EntitySpec::STATE.to_owned(),
        ScenarioValue::literal(Node::Text(state.to_string())),
    );
    Ok(Some(ScenarioStep::EventuallyView {
        view: ViewRef::new(view.name.clone()),
        params: BTreeMap::new(),
        expectation: ViewExpectation::Contains { fields },
    }))
}

/// What a binding's mapping fills each input of its invoked command with, as a scenario observes
/// it: a field of the triggering event becomes [`ScenarioValue::Observed`], because no generator
/// knows what the upstream implementation published there, and a literal becomes the text the
/// binding wrote.
pub(super) fn mapped_input(
    ir: &EssIr,
    binding: &ResolvedBinding,
    event: &EventRef,
) -> Result<BTreeMap<String, ScenarioValue>, BindingGap> {
    binding
        .mapping
        .iter()
        .map(|mapped| {
            let value = match &mapped.value {
                ResolvedMappingValue::HostContext { .. }
                | ResolvedMappingValue::HostRead { .. } => {
                    return Err(BindingGap::AccessorObservation {
                        reason: "PeriodicHostMapping: no event supplies this input".into(),
                    })
                }
                ResolvedMappingValue::DeliveryContext { .. } => {
                    return Err(BindingGap::AccessorObservation {
                        reason: "DeliveryContext: only a delivered event carries a context".into(),
                    })
                }
                ResolvedMappingValue::Selection {
                    selector,
                    projection,
                    ..
                } => {
                    if mapped.conversion.is_some() {
                        return Err(BindingGap::AccessorObservation {
                            reason: "selection result requires an explicit host conversion".into(),
                        });
                    }
                    let selection = crate::selection::Observation::of(
                        ir,
                        binding,
                        *selector,
                        projection,
                        &mapped.target_type,
                    )
                    .map_err(|reason| BindingGap::AccessorObservation { reason })?;
                    ScenarioValue::ObservedSelection {
                        event: event.clone(),
                        selection,
                    }
                }
                ResolvedMappingValue::EventField { field, .. } => {
                    ScenarioValue::observed(event.clone(), field.clone())
                }
                ResolvedMappingValue::EventAccessor { plan, types, .. } => {
                    if mapped.conversion.is_some() {
                        return Err(BindingGap::AccessorObservation {
                            reason: format!(
                                "OpaqueAccessorConversion: {} to {} requires the declared host \
                                 implementation",
                                plan.path(),
                                mapped.target_type
                            ),
                        });
                    }
                    // A required input the binding's condition proves present (ess/22,
                    // beyond10x/ess#194) is observed as its Optional, the value the condition
                    // guarantees: never an unwrap of a value that could be absent.
                    let target = if binding.condition.is_some() && !mapped.target_type.is_optional()
                    {
                        ess_compiler::ir::ResolvedTypeRef::Optional {
                            of: Box::new(mapped.target_type.clone()),
                        }
                    } else {
                        mapped.target_type.clone()
                    };
                    let accessor = crate::accessor::Observation::of(ir, plan, types, &target)
                        .map_err(|reason| BindingGap::AccessorObservation { reason })?;
                    ScenarioValue::ObservedAccessor {
                        event: event.clone(),
                        accessor,
                    }
                }
                // A literal reaches the model as text and fills a target that is a `String` or an
                // enum underneath — `ess-domain` refuses any other target — so the text is the value
                // and no conversion is being invented here.
                ResolvedMappingValue::Literal { value } => {
                    ScenarioValue::literal(Node::Text(value.clone()))
                }
            };
            Ok((mapped.target.clone(), value))
        })
        .collect()
}
