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
    Driver, EntityHandle, EssIr, ResolvedBinding, ResolvedBody, ResolvedCommand, ResolvedCondition,
    ResolvedEffect, ResolvedFailure, ResolvedInstance, ResolvedMappingValue, ResolvedOutcome,
    ResolvedPayloadField, ResolvedPayloadValue, ResolvedRefusalAction, ResolvedTypeRef,
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

mod set_guard;

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
    /// Another row, whose identity the row the publishing step acts on holds in its stored field
    /// `field`: the event field is `{subject: <field>}` (a downstream adopter's report of
    /// 2026-10-20).
    TriggerSubjectField {
        /// Its entity.
        entity: &'a EntityHandle,
        /// The stored field of the trigger's row carrying the identity.
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
    payload_declared(trigger, event, field)
        .map(|declared| (&declared.value, declared.conversion.is_some()))
}

/// The trigger's declaration of its event field `field` of `event`.
fn payload_declared<'a>(
    trigger: &'a ResolvedOutcome,
    event: &ess_compiler::ir::EventHandle,
    field: &str,
) -> Option<&'a ResolvedPayloadField> {
    trigger
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find(|declared| declared.target == field)
}

/// Whether a value of type `from` arrives as `to` exactly as it was sent: no conversion, or the
/// declared conversion `conversion` between two newtypes wrapping one representation, neither
/// held to more than it (no alphabet, prefix or invariant). Such a conversion has nothing to
/// compute, so an identity a scenario arranges and sends as the one type is the identity the other
/// names, which is what the interpreter does with it too. Any other conversion is the
/// implementation's to compute.
fn crosses_unchanged(
    ir: &EssIr,
    from: &ResolvedTypeRef,
    to: &ResolvedTypeRef,
    conversion: Option<&str>,
) -> bool {
    let Some(named) = conversion else {
        return true;
    };
    let declared = ir.conversions().iter().any(|candidate| {
        candidate.from == *from && candidate.to == *to && candidate.because == named
    });
    let (ResolvedTypeRef::Declared { name: source }, ResolvedTypeRef::Declared { name: target }) =
        (from, to)
    else {
        return false;
    };
    let (source, target) = (&ir.named_type(source).body, &ir.named_type(target).body);
    declared
        && !source.is_constrained()
        && !target.is_constrained()
        && matches!(
            (source, target),
            (ResolvedBody::Newtype { of: wrapped, .. }, ResolvedBody::Newtype { of: wraps, .. })
                if wrapped == wraps
        )
}

/// [`payload_source`], with the bool saying whether the value changes on the way into the event:
/// a conversion from an input that [`crosses_unchanged`] does not.
fn payload_followed<'a>(
    ir: &EssIr,
    trigger: &'a ResolvedOutcome,
    event: &ess_compiler::ir::EventHandle,
    field: &str,
) -> Option<(&'a ResolvedPayloadValue, bool)> {
    payload_declared(trigger, event, field).map(|declared| {
        let changed = match &declared.value {
            ResolvedPayloadValue::InputField { type_ref, .. } => !crosses_unchanged(
                ir,
                type_ref,
                &declared.target_type,
                declared.conversion.as_deref(),
            ),
            _ => declared.conversion.is_some(),
        };
        (&declared.value, changed)
    })
}

/// The event field the binding fills `input` from, where the value crosses into it unchanged
/// ([`crosses_unchanged`]).
fn mapped_unchanged<'a>(ir: &EssIr, binding: &'a ResolvedBinding, input: &str) -> Option<&'a str> {
    let mapped = binding
        .mapping
        .iter()
        .find(|mapped| mapped.target == input)?;
    match &mapped.value {
        ResolvedMappingValue::EventField { field, type_ref }
            if crosses_unchanged(
                ir,
                type_ref,
                &mapped.target_type,
                mapped.conversion.as_deref(),
            ) =>
        {
            Some(field.as_str())
        }
        _ => None,
    }
}

/// The declared conversion that keeps [`related_decided`] from naming the related row: on the way
/// from the trigger's input into the event, or from the event into the invoked command's input
/// the row is read through, the first that does not [cross unchanged](crosses_unchanged). `gap`
/// becomes [`BindingGap::BranchConverted`] naming it, and is returned as it was where there is
/// none.
fn converted(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    published_by: &ResolvedOutcome,
    gap: BindingGap,
) -> BindingGap {
    let BindingGap::BranchUndecided { command, branches } = gap else {
        return gap;
    };
    match unfollowed(ir, binding, invoked, published_by) {
        Some((input, from, to)) => BindingGap::BranchConverted {
            command,
            branches,
            conversion: Box::new(super::ConvertedInput {
                input: input.to_owned(),
                from: from.to_string(),
                to: to.to_string(),
            }),
        },
        None => BindingGap::BranchUndecided { command, branches },
    }
}

/// For [`converted`]: the invoked command's input the related row is read through, and the source
/// and target types of the first conversion on its way from the trigger that does not
/// [cross unchanged](crosses_unchanged).
fn unfollowed<'a>(
    ir: &EssIr,
    binding: &'a ResolvedBinding,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
) -> Option<(&'a str, &'a ResolvedTypeRef, &'a ResolvedTypeRef)> {
    let (via, _) = super::related_guard::reads(invoked)?;
    let event = binding.cause.event()?;
    let mapped = binding.mapping.iter().find(|mapped| mapped.target == via)?;
    let ResolvedMappingValue::EventField { field, type_ref } = &mapped.value else {
        return None;
    };
    if !crosses_unchanged(
        ir,
        type_ref,
        &mapped.target_type,
        mapped.conversion.as_deref(),
    ) {
        return Some((via, type_ref, &mapped.target_type));
    }
    let declared = payload_declared(published_by, event, field)?;
    match &declared.value {
        ResolvedPayloadValue::InputField { type_ref, .. }
            if !crosses_unchanged(
                ir,
                type_ref,
                &declared.target_type,
                declared.conversion.as_deref(),
            ) =>
        {
            Some((via, type_ref, &declared.target_type))
        }
        _ => None,
    }
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
        Some((ResolvedPayloadValue::Generated { .. }, _)) => unavailable(
            "the triggering event carries an identity the implementation mints, known only after \
             the trigger",
        ),
        Some((ResolvedPayloadValue::ResponseField { .. }, _)) => {
            unavailable("the triggering event carries a response value, known only after the trigger")
        }
        Some((ResolvedPayloadValue::SubjectField { field: stored, .. }, false))
            if trigger.subject.is_some() =>
        {
            Destination::TriggerSubjectField {
                entity,
                field: stored.clone(),
            }
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
            | ResolvedCondition::ExternalSubject { .. }
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

/// Arranges, before the branch under test, the row each unbounded retry its events set off needs,
/// where the run names that row by a literal (a downstream adopter's report of 2026-10-20). Such a
/// binding retries for as long as the bound command refuses, so a trigger naming no row it can find
/// never finishes, and the reference interpreter answers the scenario unsupported. The row is the
/// one the binding's own scenarios arrange ([`prepare`]): the literal an input of the trigger, or of
/// the step creating the trigger's row, was sent is replaced by it, and so is every value the run
/// determined from that literal. A retry selected per refusal counts where any refusal retries
/// unbounded. A run already naming a row, a bounded or non-retrying policy, and a binding whose
/// invocation needs no row keep the run as it was.
pub(super) fn arrange_retried(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    run: &mut Run,
) {
    if outcome.error.is_some() {
        return;
    }
    let retried = ir.bindings().values().filter(|binding| {
        binding.context.is_none()
            && binding.condition.is_none()
            && unbounded_retry(binding)
            && binding
                .cause
                .event()
                .is_some_and(|event| outcome.emits.contains(event))
    });
    for binding in retried {
        let invoked = ir.command(&binding.command);
        let Ok(prepared) = prepare(ir, binding, invoked, outcome, run, actors) else {
            continue;
        };
        if prepared.reached.is_err() || !prepared.capture.is_empty() {
            continue;
        }
        let renamed = renamed(
            run.setup.iter().chain(&run.invoke),
            prepared.setup.iter().chain(&prepared.invoke),
        );
        if renamed.is_empty() {
            continue;
        }
        run.setup = prepared.setup;
        run.invoke = prepared.invoke;
        run.source.extend(prepared.source);
        let rename = |value: &mut ScenarioValue| {
            if let Some((_, named)) = renamed.iter().find(|(literal, _)| literal == value) {
                *value = named.clone();
            }
        };
        run.input.values_mut().for_each(rename);
        for determined in run
            .settled
            .values_mut()
            .chain(run.before_settled.values_mut())
        {
            rename(&mut determined.value);
        }
    }
}

/// Whether `binding` may retry a refusal without bound: the universal `retry` with no count, or a
/// policy selected per refusal of which any choice is.
fn unbounded_retry(binding: &ResolvedBinding) -> bool {
    match binding.on_failure() {
        ResolvedFailure::Retry => true,
        ResolvedFailure::ByRefusal { policy } => policy
            .actions()
            .any(|action| matches!(action, ResolvedRefusalAction::Retry { bound: None })),
        _ => false,
    }
}

/// Each literal an invocation in `before` was sent that the same invocation in `after` is sent as
/// an instance instead, paired with that instance. The invocation is matched by its command and
/// every other input it carries; one sent unchanged in `after` renames nothing.
fn renamed<'s>(
    before: impl Iterator<Item = &'s ScenarioStep>,
    after: impl Iterator<Item = &'s ScenarioStep>,
) -> Vec<(ScenarioValue, ScenarioValue)> {
    let sent = |step: &'s ScenarioStep| match step {
        ScenarioStep::ExecuteCommand { command, input, .. } => Some((command, input)),
        _ => None,
    };
    let after: Vec<_> = after.filter_map(sent).collect();
    let mut renamed = Vec::new();
    for (command, input) in before.filter_map(sent) {
        if after.contains(&(command, input)) {
            continue;
        }
        for (field, value) in input {
            if !matches!(value, ScenarioValue::Literal { .. }) {
                continue;
            }
            let named = after.iter().find_map(|(other, sent)| {
                let same = *other == command
                    && sent.len() == input.len()
                    && sent
                        .iter()
                        .all(|(key, held)| key == field || input.get(key) == Some(held));
                sent.get(field)
                    .filter(|named| same && matches!(named, ScenarioValue::Instance { .. }))
            });
            if let Some(named) = named {
                renamed.push((value.clone(), named.clone()));
            }
        }
    }
    renamed
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
        Destination::Unaddressed => {
            let reached = reachable_branch(invoked);
            if reached.is_err() {
                if let Some(prepared) =
                    related_decided(ir, binding, invoked, published_by, trigger, actors)
                {
                    return Ok(prepared);
                }
                if let Some(prepared) =
                    set_guard::decided(ir, binding, invoked, published_by, trigger, actors)
                {
                    return Ok(prepared);
                }
            }
            // A conversion that stopped the related row's arrangement is named in the gap.
            let reached = reached.map_err(|gap| converted(ir, binding, invoked, published_by, gap));
            if let Ok(branch) = reached {
                if let Some(prepared) = per_row_decided(
                    ir,
                    binding,
                    (invoked, branch),
                    published_by,
                    trigger,
                    actors,
                ) {
                    return Ok(prepared);
                }
            }
            Ok(Prepared {
                setup: trigger.setup.clone(),
                invoke: trigger.invoke.clone(),
                capture: Vec::new(),
                reached,
                row: None,
                source: BTreeSet::new(),
            })
        }
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
        Destination::TriggerSubjectField { entity, field } => {
            stored_destination(ir, invoked, published_by, entity, &field, trigger, actors)
        }
    }
}

/// The branch of a `when_related` guard a binding's invocation takes, decided by arranging the
/// related row before the trigger (downstream report, 2026-10-18).
///
/// The invoked command acts on no existing row of its own, and every branch it can take
/// unforced is decided by one related row alone: an `exists: false` branch, a predicate over the
/// row's stored fields that reads no input, or the default. The input naming the row is mapped
/// from an event field the trigger fills from an input of its own, each step unconverted or across
/// a declared conversion that hands the value on as it is ([`crosses_unchanged`], a downstream
/// adopter's report of 2026-10-26). Then the row is a value the scenario chooses rather than one
/// the upstream implementation does: it is arranged through its creating command so the accepting
/// branch is selected, as that branch's own scenario arranges it, and the trigger is sent naming
/// it. A trigger that refuses that same row missing before it publishes ([`refuses_only_missing`])
/// is sent naming the row arranged here, which exists, so it still publishes.
///
/// The accepting branch is the default where it accepts, else the first predicate branch that
/// does. `None` where any of that does not hold, so the binding keeps the
/// [`BindingGap::BranchUndecided`] it had — [`BindingGap::BranchConverted`] where a conversion
/// stopped it ([`converted`]) — and a model without such a guard keeps its suite bytes.
fn related_decided<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Prepared<'a>> {
    use super::related_guard;

    if !related_guard::uses(invoked)
        || related_guard::several(invoked)
        || related_guard::stored::field(invoked).is_some()
    {
        return None;
    }
    let (via, entity) = related_guard::reads(invoked)?;
    let accepting = accepting_branch(ir, invoked, via, entity)?;

    // The identity reaches the invocation unchanged from an input of the trigger's caller, across
    // no conversion or one that hands it on as it is.
    let event = binding.cause.event()?;
    let field = mapped_unchanged(ir, binding, via)?;
    // Sent in the trigger's own input, or stored on the trigger's row by the step creating it.
    let (from, stored) = match payload_followed(ir, published_by, event, field)? {
        (ResolvedPayloadValue::InputField { field: from, .. }, false)
            if !ess_domain::command::input_path::is_path(from) =>
        {
            (from.clone(), None)
        }
        (ResolvedPayloadValue::SubjectField { field: stored, .. }, false) => {
            let (sent, _, from) = stored_creation(ir, published_by, stored, trigger)?;
            (from, Some(sent))
        }
        _ => return None,
    };

    let (_, distinction) = unused_name(ir, entity, trigger);
    let models = super::caller::InvocationModels::plain(ir);
    let taken = captured(trigger);
    // A trigger that refuses the same missing row before it publishes is arranged around rows of
    // that entity itself, at the plain distinction: the related row is arranged past them, at the
    // first further distinction naming none of them (a downstream adopter's report of 2026-10-26).
    let past = stored.is_none()
        && command_of(ir, published_by)
            .is_some_and(|publisher| refuses_only_missing(publisher, &from));
    let further = (1..=taken.len() + 1)
        .map(Distinction::further)
        .filter(|_| past);
    let arranged = std::iter::once(distinction)
        .chain(further)
        .find_map(|distinction| {
            let (arranged, _) = related_guard::prepare_at_in(
                &models, invoked, accepting, actors, distinction, None,
            )
            .ok()?;
            let collides = arranged.steps.iter().any(|step| {
                matches!(step, ScenarioStep::CaptureInstance { instance, .. } if taken.contains(instance))
            });
            (!collides).then_some(arranged)
        })?;
    // Only the related row is the scenario's to choose: anything else the branch's own
    // arrangement binds is a value the event fills.
    if arranged.instance.is_some() || arranged.bound.keys().any(|bound| bound != via) {
        return None;
    }
    let row = arranged.bound.get(via)?.clone();
    let (setup, invoke) = if let Some(sent) = stored {
        // The row first, then the trigger's arrangement creating its own row naming it.
        let mut created = trigger.setup.clone();
        let ScenarioStep::ExecuteCommand { input, .. } = &mut created[sent] else {
            return None;
        };
        input.insert(from, ScenarioValue::instance(row));
        let mut setup = arranged.steps;
        setup.extend(created);
        (setup, trigger.invoke.clone())
    } else {
        let mut invoke = trigger.invoke.clone();
        let named = invoke.iter_mut().rev().find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } if input.contains_key(&from) => {
                input.insert(from.clone(), ScenarioValue::instance(row.clone()));
                Some(())
            }
            _ => None,
        });
        named?;
        let mut setup = trigger.setup.clone();
        setup.extend(arranged.steps);
        (setup, invoke)
    };
    Some(Prepared {
        setup,
        invoke,
        capture: Vec::new(),
        reached: Ok(accepting),
        row: None,
        source: arranged.source,
    })
}

/// The branch of `invoked` [`related_decided`] arranges: the default where it accepts, else the
/// first predicate branch on the row read through `via` that does. `None` where a branch is decided
/// by anything that arrangement does not choose.
fn accepting_branch<'a>(
    ir: &EssIr,
    invoked: &'a ResolvedCommand,
    via: &str,
    entity: &EntityHandle,
) -> Option<&'a ResolvedOutcome> {
    use ess_compiler::ir::ResolvedRelatedTest;

    let mut accepting: Option<&ResolvedOutcome> = None;
    for outcome in &invoked.outcomes {
        if outcome.test_strategy == TestStrategy::InjectFault {
            continue;
        }
        if outcome.replays.is_some() {
            return None;
        }
        let candidate = match &outcome.condition {
            ResolvedCondition::Otherwise => true,
            // Answered first, for an identity a row already carries; the event fills a fresh one.
            ResolvedCondition::ExistingInstance => false,
            // A branch also requiring further rows (ess/28, `all:`) is decided by rows this does not
            // arrange.
            ResolvedCondition::Related { also, .. } if !also.is_empty() => return None,
            ResolvedCondition::Related {
                via: read,
                test,
                input: None,
                ..
            } if read.field() == via => match test {
                ResolvedRelatedTest::Absent => false,
                ResolvedRelatedTest::Holds { predicate } => {
                    if super::subject_fact::reads_input(ir, entity, predicate) {
                        return None;
                    }
                    true
                }
            },
            _ => return None,
        };
        if candidate && outcome.error.is_none() {
            let default = matches!(outcome.condition, ResolvedCondition::Otherwise);
            if accepting.is_none() || default {
                accepting = Some(outcome);
            }
        }
    }
    accepting
}

/// Whether `publisher` reads the related row its input `from` names, and only to refuse it missing
/// (`when_related: {via: input.<from>, exists: false}` with an error, reading no further row): it
/// publishes only while that row exists, whichever row it is, so [`related_decided`] may point it at
/// the row arranged for the invoked command.
fn refuses_only_missing(publisher: &ResolvedCommand, from: &str) -> bool {
    use ess_compiler::ir::{ResolvedRelatedTest, ResolvedRelatedVia};

    let mut reads = publisher
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via: ResolvedRelatedVia::Input { field, .. },
                test,
                also,
                ..
            } if field == from => Some((outcome, test, also)),
            _ => None,
        })
        .peekable();
    reads.peek().is_some()
        && reads.all(|(outcome, test, also)| {
            matches!(test, ResolvedRelatedTest::Absent)
                && also.is_empty()
                && outcome.error.is_some()
        })
}

/// The trigger of a binding whose invocation reaches a set effect publishing an event once per row
/// it changes (`each: changed`), with one row its filter selects arranged first, resting where a
/// per-row event's `when_state:` publishes it (a downstream adopter's report of 2026-10-20): with
/// no such row the honest system publishes nothing, and a flow waiting for the event cannot pass.
///
/// The row is arranged against the input the bound invocation will carry: every input the filter
/// reads is mapped unconverted from an event field the trigger fills unconverted from an input it
/// is sent as a literal, or mapped from a literal. Where that does not hold, or no arranging
/// command produces such a row, the branch is the gap [`BindingGap::ChangedRowUnarranged`], so the
/// flow and delivery scenarios are refused by name rather than filed unpassable. `None` for a
/// branch publishing no per-row event, which keeps the arrangement it had.
fn per_row_decided<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    (invoked, reached): (&'a ResolvedCommand, &'a ResolvedOutcome),
    published_by: &'a ResolvedOutcome,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Option<Prepared<'a>> {
    let set = reached.instances.as_ref()?;
    if set.events.is_empty() {
        return None;
    }
    let unarranged = || BindingGap::ChangedRowUnarranged {
        outcome: OutcomeRef::new(CommandRef::new(invoked.name.clone()), reached.name.clone()),
        entity: EntityRef::from(&set.entity),
    };
    let arranged = per_row_input(ir, binding, published_by, trigger).and_then(|input| {
        let movable = |state: &StateName| match &set.effect {
            ResolvedEffect::Moves { transition } => transition.from.contains(state),
            _ => true,
        };
        let rests = |state: &StateName| {
            movable(state)
                && set.events.iter().any(|entry| {
                    entry.when_state.is_empty()
                        || entry.when_state.iter().any(|held| held == state.as_str())
                })
        };
        let mut taken = captured(trigger);
        let before = taken.clone();
        let row = super::set_effects::selected_row(
            ir,
            (invoked, reached, set),
            &input,
            actors,
            &rests,
            &mut taken,
        )?;
        let collides = row.steps.iter().any(|step| {
            matches!(step, ScenarioStep::CaptureInstance { instance, .. } if before.contains(instance))
        });
        (!collides).then_some(row)
    });
    let Some(row) = arranged else {
        return Some(Prepared {
            setup: trigger.setup.clone(),
            invoke: trigger.invoke.clone(),
            capture: Vec::new(),
            reached: Err(unarranged()),
            row: None,
            source: BTreeSet::new(),
        });
    };
    let mut setup = trigger.setup.clone();
    setup.extend(row.steps.iter().cloned());
    let mut source = row.source.clone();
    source.insert(EntityRef::from(&set.entity).into());
    Some(Prepared {
        setup,
        invoke: trigger.invoke.clone(),
        capture: Vec::new(),
        reached: Ok(reached),
        row: None,
        source,
    })
}

/// The input the bound invocation carries, where the scenario chose it: each input mapped from a
/// literal, or unconverted from an event field the trigger fills unconverted from an input it is
/// sent as a literal. Inputs filled any other way are left out; `None` where the binding has no
/// event.
fn per_row_input(
    ir: &EssIr,
    binding: &ResolvedBinding,
    published_by: &ResolvedOutcome,
    trigger: &Run,
) -> Option<BTreeMap<String, Node>> {
    let event = binding.cause.event()?;
    let sent = trigger.invoke.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { input, .. } => Some(input),
        _ => None,
    })?;
    let mut input = BTreeMap::new();
    for mapped in &binding.mapping {
        let value = match (&mapped.value, &mapped.conversion) {
            (ResolvedMappingValue::Literal { value }, _) => {
                crate::input::mapping_literal(ir, &mapped.target_type, value)
            }
            (ResolvedMappingValue::EventField { field, .. }, None) => {
                match payload_source(published_by, event, field) {
                    Some((ResolvedPayloadValue::InputField { field: from, .. }, false))
                        if !ess_domain::command::input_path::is_path(from) =>
                    {
                        match sent.get(from) {
                            Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        if let Some(value) = value {
            input.insert(mapped.target.clone(), value);
        }
    }
    Some(input)
}

/// The trigger's invocation sent naming a related row no row carries, so that the bound invocation
/// answers `refusal`, a `when_related: {via: input.<f>, exists: false}` branch, as the honest
/// system does: no injection, and nothing to arrange (a downstream adopter's report of 2026-10-20).
///
/// As [`related_decided`] requires for the accepting branch: the invoked command acts on no
/// existing row of its own, and every branch it can take unforced reads the one related row the
/// refusal reads, through no input guard, or is the default; the input naming the row is mapped
/// from an event field the trigger fills from an input of its own, each step unconverted or across
/// a conversion that [crosses unchanged](crosses_unchanged). `refusal`
/// is the first `exists: false` branch declared, so no other one answers before it. `None` where
/// any of that does not hold, so the refusal keeps the gap it had.
///
/// [`BindingGap::RelatedRowNeededByTrigger`] where the trigger's own command has an
/// `exists: false` branch over the input it copies into the event: sent naming no row, the trigger
/// answers that refusal and never publishes, so the binding is never delivered (a downstream
/// adopter's report of 2026-10-22). The same where the trigger's publishing branch acts on that
/// related row, named by that input, and its command refuses a row no record carries
/// (`unknown_instance:`).
pub(super) fn related_missing(
    ir: &EssIr,
    binding: &ResolvedBinding,
    invoked: &ResolvedCommand,
    published_by: &ResolvedOutcome,
    refusal: &ResolvedOutcome,
    trigger: &Run,
) -> Option<Result<Vec<ScenarioStep>, BindingGap>> {
    use super::related_guard;
    use ess_compiler::ir::{ResolvedRelatedTest, ResolvedRelatedVia};

    if refusal.error.is_none()
        || addressed(invoked).is_some()
        || related_guard::several(invoked)
        || related_guard::stored::field(invoked).is_some()
    {
        return None;
    }
    let ResolvedCondition::Related {
        via: via @ ResolvedRelatedVia::Input { .. },
        entity: related,
        test: ResolvedRelatedTest::Absent,
        input: None,
        ..
    } = &refusal.condition
    else {
        return None;
    };
    let via = via.field();
    let mut earlier = true;
    for outcome in &invoked.outcomes {
        if std::ptr::eq(outcome, refusal) {
            earlier = false;
        }
        match &outcome.condition {
            _ if outcome.test_strategy == TestStrategy::InjectFault => {}
            ResolvedCondition::Otherwise => {}
            ResolvedCondition::Related {
                via: read,
                test,
                input: None,
                ..
            } if read.field() == via => {
                // An `exists: false` branch over the same row declared earlier answers first.
                if earlier && matches!(test, ResolvedRelatedTest::Absent) {
                    return None;
                }
            }
            _ => return None,
        }
        if outcome.replays.is_some() {
            return None;
        }
    }

    let event = binding.cause.event()?;
    let field = mapped_unchanged(ir, binding, via)?;
    let from = match payload_followed(ir, published_by, event, field)? {
        (ResolvedPayloadValue::InputField { field: from, .. }, false)
            if !ess_domain::command::input_path::is_path(from) =>
        {
            from.clone()
        }
        _ => return None,
    };
    let publisher = command_of(ir, published_by)?;
    let refuses_missing = publisher.outcomes.iter().any(|outcome| {
        !std::ptr::eq(outcome, published_by)
            && matches!(
                &outcome.condition,
                ResolvedCondition::Related {
                    via: ResolvedRelatedVia::Input { field, .. },
                    test: ResolvedRelatedTest::Absent,
                    ..
                } if *field == from
            )
    });
    // The publishing branch acts on the related row itself, named by that same input, and the
    // trigger refuses a row no record carries (`unknown_instance:`): sent naming no row, it
    // answers that refusal and never publishes either.
    let acts_on_missing = published_by.subject.as_ref().is_some_and(|subject| {
        subject.entity == *related
            && !matches!(subject.effect, ResolvedEffect::Creates)
            && matches!(&subject.instance, ResolvedInstance::Supplied { field } if field.name == from)
    }) && publisher
        .outcomes
        .iter()
        .any(|outcome| outcome.condition == ResolvedCondition::UnknownInstance);
    if refuses_missing || acts_on_missing {
        return Some(Err(BindingGap::RelatedRowNeededByTrigger {
            refusal: OutcomeRef::new(CommandRef::new(invoked.name.clone()), refusal.name.clone()),
            trigger: CommandRef::new(publisher.name.clone()),
        }));
    }
    let fresh = super::fresh_identity(ir, invoked, via, None).ok()?;
    let mut invoke = trigger.invoke.clone();
    invoke.iter_mut().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { input, .. } if input.contains_key(&from) => {
            input.insert(from.clone(), ScenarioValue::literal(fresh.clone()));
            Some(())
        }
        _ => None,
    })?;
    Some(Ok(invoke))
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
    let destination = || Destination::TriggerInput {
        entity,
        field: field.to_owned(),
    };
    if let Some((instance, held)) = input_arranged(ir, published_by, entity, field, trigger) {
        let Reached::Accepted(reached) = bound_branch(invoked, &held)? else {
            return Err(BindingGap::DestinationIneligible {
                command: command(),
                state: held,
            });
        };
        return Ok(Prepared {
            setup: trigger.setup.clone(),
            invoke: trigger.invoke.clone(),
            capture: Vec::new(),
            reached: Ok(reached),
            row: Some(row(
                ir,
                published_by,
                entity,
                ScenarioValue::instance(instance),
                held,
                fan_out(ir, published_by, &destination()),
                Ok(()),
            )),
            source: BTreeSet::new(),
        });
    }
    // The trigger reads the stored fields of the row it names, so its arrangement made exactly
    // that row selectable: a freshly arranged row in its place is one the trigger may refuse.
    if command_of(ir, published_by).is_some_and(|publisher| reads_stored(publisher, field)) {
        return Err(BindingGap::DestinationIdentityUnavailable {
            command: command(),
            input: field_name(invoked),
            why: "the trigger reads the stored fields of the row it names, and the state its \
                  arrangement leaves that row in cannot be followed",
        });
    }
    let (arrangement, reached) = eligible_row(ir, invoked, entity, trigger, actors)?;
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
            fan_out(ir, published_by, &destination()),
            Ok(()),
        )),
        source: arrangement.source,
    })
}

/// [`prepare`] where the row the trigger acts on holds the destination's identity in its stored
/// field `stored` (`{subject: <field>}` in the trigger's payload): the destination is arranged
/// first, in a state the one accepting branch admits, and the trigger's own arrangement creates its
/// row holding that identity (a downstream adopter's report of 2026-10-20).
///
/// Only where that arrangement creates the trigger's row with `stored` set unconverted from an
/// input, and the row still holds what that input was sent when the trigger runs; otherwise the
/// identity stays unavailable, named.
fn stored_destination<'a>(
    ir: &'a EssIr,
    invoked: &'a ResolvedCommand,
    published_by: &'a ResolvedOutcome,
    entity: &'a EntityHandle,
    stored: &str,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Prepared<'a>, BindingGap> {
    let unavailable = |why| BindingGap::DestinationIdentityUnavailable {
        command: CommandRef::new(invoked.name.clone()),
        input: field_name(invoked),
        why,
    };
    if let Some((instance, held)) = trigger_arranged(ir, published_by, entity, stored, trigger) {
        let Reached::Accepted(reached) = bound_branch(invoked, &held)? else {
            return Err(BindingGap::DestinationIneligible {
                command: CommandRef::new(invoked.name.clone()),
                state: held,
            });
        };
        return Ok(Prepared {
            setup: trigger.setup.clone(),
            invoke: trigger.invoke.clone(),
            capture: Vec::new(),
            reached: Ok(reached),
            row: Some(row(
                ir,
                published_by,
                entity,
                ScenarioValue::instance(instance),
                held,
                fan_out(
                    ir,
                    published_by,
                    &Destination::TriggerSubjectField {
                        entity,
                        field: stored.to_owned(),
                    },
                ),
                Ok(()),
            )),
            source: BTreeSet::new(),
        });
    }
    // The creator reads the stored fields of the row it names, so the arrangement made exactly
    // that row selectable: a freshly arranged row in its place is one the creator may refuse.
    if stored_creation(ir, published_by, stored, trigger)
        .is_some_and(|(_, creator, from)| reads_stored(creator, &from))
    {
        return Err(unavailable(
            "the command creating the trigger's row reads the stored fields of the row it names, \
             and the state the trigger's arrangement leaves that row in cannot be followed",
        ));
    }
    let (arrangement, reached) = eligible_row(ir, invoked, entity, trigger, actors)?;
    let identity = arrangement.identity();
    let named = stored_in_setup(ir, published_by, stored, trigger, &identity).ok_or_else(|| {
        unavailable(
            "the triggering event carries a stored field of the row the trigger acts on, and the \
             trigger's arrangement does not create that row with the field from an input it sends",
        )
    })?;
    let taken = captured(trigger);
    if arrangement.steps.iter().any(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance, .. } if taken.contains(instance))
    }) {
        return Err(unavailable(
            "arranging the row names an instance the trigger's arrangement already names",
        ));
    }
    let mut setup = arrangement.steps.clone();
    setup.extend(named);
    Ok(Prepared {
        setup,
        invoke: trigger.invoke.clone(),
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
                &Destination::TriggerSubjectField {
                    entity,
                    field: stored.to_owned(),
                },
            ),
            Ok(()),
        )),
        source: arrangement.source,
    })
}

/// The trigger's arrangement with the row the trigger acts on created holding `identity` in its
/// stored field `stored`: the creating step's input that `sets:` copies into it, unconverted,
/// replaced. `None` where the arrangement does not create the row so, or where the row no longer
/// holds what that input was sent by the time the trigger runs.
fn stored_in_setup(
    ir: &EssIr,
    published_by: &ResolvedOutcome,
    stored: &str,
    trigger: &Run,
    identity: &ScenarioValue,
) -> Option<Vec<ScenarioStep>> {
    let (sent, _, from) = stored_creation(ir, published_by, stored, trigger)?;
    let mut setup = trigger.setup.clone();
    let ScenarioStep::ExecuteCommand { input, .. } = &mut setup[sent] else {
        return None;
    };
    input.insert(from, identity.clone());
    Some(setup)
}

/// Where the trigger's arrangement creates the row the trigger acts on with its stored field
/// `stored` copied, unconverted, from an input: the creating step's index in the arrangement, the
/// creating command, and that input, which the row still holds when the trigger runs.
fn stored_creation<'a>(
    ir: &'a EssIr,
    published_by: &ResolvedOutcome,
    stored: &str,
    trigger: &Run,
) -> Option<(usize, &'a ResolvedCommand, String)> {
    let subject = published_by.subject.as_ref()?;
    let instance = trigger.instance.as_ref()?;
    let held = trigger.before_settled.get(stored)?;
    let setup = &trigger.setup;
    let at = setup.iter().position(|step| {
        matches!(step, ScenarioStep::CaptureInstance { instance: captured, .. } if captured == instance)
    })?;
    let sent = setup[..at]
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))?;
    let taken = match setup.get(sent + 1) {
        Some(ScenarioStep::ExpectOutcome { outcome }) => outcome.outcome.clone(),
        _ => return None,
    };
    let ScenarioStep::ExecuteCommand { command, input, .. } = &setup[sent] else {
        return None;
    };
    let command = ir.commands().get(command.name())?;
    let creator = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name == taken)?;
    let creates = creator.subject.as_ref().is_some_and(|created| {
        matches!(created.effect, ResolvedEffect::Creates) && created.entity == subject.entity
    });
    if !creates {
        return None;
    }
    let from = creator
        .sets
        .iter()
        .find(|set| set.target == stored && set.conversion.is_none())
        .and_then(|set| match &set.value {
            ResolvedPayloadValue::InputField { field, .. }
                if !ess_domain::command::input_path::is_path(field) =>
            {
                Some(field.clone())
            }
            _ => None,
        })?;
    if input.get(&from) != Some(&held.value) {
        return None;
    }
    Some((sent, command, from))
}

/// The row of `entity` the trigger's own arrangement creates its row naming, and the state the
/// arrangement leaves it in, where the creating command reads that row's stored fields
/// (`when_related: {via: input.<from>, predicate: …}`): the arrangement made exactly that row
/// selectable, so it is the binding's destination, and no second row is arranged in its place (a
/// downstream adopter's report of 2026-10-22).
///
/// `None` where the creator reads no stored field of the row through that input, so a guard that
/// asks only whether the row exists keeps the separately arranged destination and its suite bytes;
/// and where the arrangement's steps do not leave the row in one state synthesis can name.
fn trigger_arranged(
    ir: &EssIr,
    published_by: &ResolvedOutcome,
    entity: &EntityHandle,
    stored: &str,
    trigger: &Run,
) -> Option<(InstanceName, StateName)> {
    let (sent, creator, from) = stored_creation(ir, published_by, stored, trigger)?;
    if !reads_stored(creator, &from) {
        return None;
    }
    let ScenarioStep::ExecuteCommand { input, .. } = &trigger.setup[sent] else {
        return None;
    };
    let ScenarioValue::Instance { instance } = input.get(&from)? else {
        return None;
    };
    let state = arranged_state(ir, entity, instance, &trigger.setup)?;
    Some((instance.clone(), state))
}

/// [`trigger_arranged`] where the trigger names the destination in its own input `field`: the row
/// of `entity` its arrangement made selectable for a trigger reading that row's stored fields
/// through `field`, and the state the arrangement leaves it in. `None` where the trigger reads no
/// stored field through it, so an existence guard keeps the separately arranged destination and its
/// suite bytes, and where the row's state cannot be named.
fn input_arranged(
    ir: &EssIr,
    published_by: &ResolvedOutcome,
    entity: &EntityHandle,
    field: &str,
    trigger: &Run,
) -> Option<(InstanceName, StateName)> {
    let publisher = command_of(ir, published_by)?;
    if !reads_stored(publisher, field) {
        return None;
    }
    let instance = trigger.invoke.iter().rev().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { input, .. } => match input.get(field)? {
            ScenarioValue::Instance { instance } => Some(instance),
            _ => None,
        },
        _ => None,
    })?;
    let state = arranged_state(ir, entity, instance, &trigger.setup)?;
    Some((instance.clone(), state))
}

/// Whether `command` has a branch reading the stored fields of the row its input `from` names
/// (`when_related: {via: input.<from>, predicate: …}`, or an `all:` member through it).
fn reads_stored(command: &ResolvedCommand, from: &str) -> bool {
    use ess_compiler::ir::{ResolvedRelatedTest, ResolvedRelatedVia};

    let reads = |via: &ResolvedRelatedVia| matches!(via, ResolvedRelatedVia::Input { field, .. } if field == from);
    command
        .outcomes
        .iter()
        .any(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via, test, also, ..
            } => {
                (reads(via) && matches!(test, ResolvedRelatedTest::Holds { .. }))
                    || also.iter().any(|also| reads(&also.via))
            }
            _ => false,
        })
}

/// The state `steps` leave the row of `entity` captured as `instance` in: created by the step
/// whose event it is captured from, then moved by every step sent naming it. `None` where a step
/// acts on it any other way — established, awaited, deleted, or named by an observed identity —
/// or where any step sets off a binding into a command acting on `entity`, whose delivery the
/// steps do not follow.
fn arranged_state(
    ir: &EssIr,
    entity: &EntityHandle,
    instance: &InstanceName,
    steps: &[ScenarioStep],
) -> Option<StateName> {
    let named = ScenarioValue::instance(instance.clone());
    let acts_on_entity = |command: &ResolvedCommand| {
        command.outcomes.iter().any(|outcome| {
            outcome
                .subject
                .as_ref()
                .is_some_and(|s| s.entity == *entity)
        })
    };
    let mut sent: Option<(&CommandRef, &BTreeMap<String, ScenarioValue>)> = None;
    let mut created: Option<StateName> = None;
    let mut state: Option<StateName> = None;
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => sent = Some((command, input)),
            ScenarioStep::ExecuteCommandWithoutInput { .. } => sent = None,
            ScenarioStep::EstablishEntity { instance: of, .. } if of == instance => return None,
            ScenarioStep::AwaitOutcome { instance: of, .. } if *of == named => return None,
            ScenarioStep::ExpectOutcome { outcome } => {
                let Some((command, input)) = sent.take() else {
                    continue;
                };
                let taken = ir
                    .commands()
                    .get(command.name())?
                    .outcomes
                    .iter()
                    .find(|declared| declared.name == outcome.outcome)?;
                let sets_off = ir.bindings().values().any(|binding| {
                    binding
                        .cause
                        .event()
                        .is_some_and(|event| taken.emits.contains(event))
                        && acts_on_entity(ir.command(&binding.command))
                });
                if sets_off {
                    return None;
                }
                created = None;
                let Some(subject) = taken.subject.as_ref().filter(|s| s.entity == *entity) else {
                    continue;
                };
                match (&subject.effect, &subject.instance) {
                    (ResolvedEffect::Creates, _) => {
                        created = Some(
                            subject
                                .into
                                .clone()
                                .unwrap_or_else(|| ir.entity(entity).lifecycle.initial.clone()),
                        );
                    }
                    (effect, ResolvedInstance::Supplied { field }) => {
                        if input.get(&field.name) != Some(&named) {
                            continue;
                        }
                        let held = state.as_ref()?;
                        state = match effect {
                            ResolvedEffect::Moves { transition } => Some(transition.to.clone()),
                            ResolvedEffect::Updates | ResolvedEffect::Preserves => {
                                Some(held.clone())
                            }
                            ResolvedEffect::Deletes | ResolvedEffect::Creates => return None,
                        };
                    }
                    (_, ResolvedInstance::Observed { .. }) => return None,
                }
            }
            ScenarioStep::CaptureInstance {
                instance: captured,
                entity: of,
                ..
            } if captured == instance => {
                if state.is_some() || *of != EntityRef::from(entity) {
                    return None;
                }
                state = Some(created.take()?);
            }
            _ => {}
        }
    }
    state
}

/// The row the invoked command acts on, arranged in a state its one accepting branch admits under
/// a name no step of `trigger` uses, and that branch.
fn eligible_row<'a>(
    ir: &'a EssIr,
    invoked: &'a ResolvedCommand,
    entity: &'a EntityHandle,
    trigger: &Run,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Arrangement, &'a ResolvedOutcome), BindingGap> {
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
    Ok((arrangement, reached))
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
        // A refusal: an `external:` branch that accepts is no failure to force.
        .find(|outcome| {
            outcome.test_strategy == TestStrategy::InjectFault && outcome.error.is_some()
        })
        .ok_or_else(|| BindingGap::NoForcibleFailure {
            command: CommandRef::new(invoked.name.clone()),
        })?;
    if let Some(gap) = forced_eligibility(ir.format(), invoked, forced) {
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
/// knows what the upstream implementation published there, and a literal becomes the value the
/// binding wrote, typed against the input it fills.
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
                // A literal over text or an enum is the text; over a `Boolean`, `Integer` or
                // `Decimal` it is that value (beyond10x/ess#445). `ess-domain` refuses any other
                // target, so no conversion is being invented here.
                ResolvedMappingValue::Literal { value } => ScenarioValue::literal(
                    crate::input::mapping_literal(ir, &mapped.target_type, value).ok_or_else(
                        || BindingGap::AccessorObservation {
                            reason: format!(
                                "the constant `{value}` is not a value of `{}`",
                                mapped.target_type
                            ),
                        },
                    )?,
                ),
            };
            Ok((mapped.target.clone(), value))
        })
        .collect()
}
