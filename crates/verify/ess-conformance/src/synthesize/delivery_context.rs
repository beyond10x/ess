//! The four claims of a binding that reads its delivery context (ess/18, beyond10x/ess#195).
//!
//! Such a binding reacts to an event nothing in the specification publishes — `ess-domain` admits
//! a context only there — so no command the suite runs can make it fire. The suite delivers the
//! event itself, through [`ScenarioStep::DeliverEvent`], and chooses both its fields and its
//! context. That is what makes the mapping checkable exactly: every expected input is a value the
//! suite put in, never an observation of what an upstream implementation published.
//!
//! | aspect | steps | what a wrong target does here |
//! |---|---|---|
//! | `mapping` | deliver under one context, expect the invocation; deliver under a second, expect it again | ignoring the context, or keeping the first one, fails an invocation |
//! | `delivery` | deliver under two contexts, redeliver, require every invocation for the second occurrence to carry the second context | redelivering with the first delivery's context fails |
//! | `flow` | deliver, then the invoked branch's events eventually | as for any binding |
//! | `on-failure` | force the failure, deliver, then the policy's consequence | as for any binding |
//!
//! A mapping this module cannot turn into an exact expectation — through a declared conversion, a
//! bounded accessor, a selection — refuses its scenario with the reason, never a guess.
//!
//! A binding with an event-payload condition (ess/22, beyond10x/ess#268) gets its delivered
//! payloads chosen for the condition as well, varying only the members it reads
//! (`binding_condition::delivered`): the four aspects above deliver payloads it holds for, and
//! `condition-false` and `condition-absent` deliver one it fails for with every member present and
//! one per proved Optional level left out, each then requiring zero invocations for the whole
//! window. Where no value set makes it hold, each aspect is refused naming why.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedCommand, ResolvedCondition, ResolvedDeliveryContext,
    ResolvedFailure, ResolvedField, ResolvedMappingValue,
};
use ess_domain::binding::Delivery;
use ess_domain::command::TestStrategy;
use ess_primitives::node::Node;

use super::{
    accepting_component, clipped, downstream, input_types, insert, listed, payload_shape,
    publishes, reachable_branch, reachable_types, BindingAspect, BindingGap, BindingRef, Built,
    CommandRef, ConformanceScenario, ConformanceSuite, EssSemanticRef, EventRef, OutcomeRef,
    Refusal, RefusalCause, ScenarioId, ScenarioStep, ScenarioValue,
};
use crate::witness::{self, Distinction};

/// Expected command inputs, by declared field name.
type Inputs = BTreeMap<String, ScenarioValue>;

/// The steps requiring a branch's events, the events, and what they depend on.
type Consequences = (Vec<ScenarioStep>, Vec<EventRef>, BTreeSet<EssSemanticRef>);

/// One occurrence the suite delivers: its fields and its context.
struct Occurrence {
    payload: BTreeMap<String, Node>,
    context: BTreeMap<String, Node>,
}

/// What each delivery of one binding is made of, shared by its scenarios.
struct Delivered<'ir> {
    ir: &'ir EssIr,
    binding: &'ir ResolvedBinding,
    invoked: &'ir ResolvedCommand,
    context: &'ir ResolvedDeliveryContext,
    event: EventRef,
    first: Occurrence,
    second: Occurrence,
    /// How many witness values each context field's type and invariants admit, by name.
    admitted: BTreeMap<String, usize>,
}

impl Delivered<'_> {
    fn deliver(&self, occurrence: &Occurrence) -> ScenarioStep {
        ScenarioStep::DeliverEvent {
            event: self.event.clone(),
            authority: self.context.authority.to_string(),
            payload: occurrence.payload.clone(),
            context: occurrence.context.clone(),
        }
    }

    fn command(&self) -> CommandRef {
        CommandRef::new(self.invoked.name.clone())
    }

    /// What the invocation for `occurrence` must receive, and the part of it its payload fills.
    fn expected(&self, occurrence: &Occurrence) -> Result<(Inputs, Inputs), BindingGap> {
        let unobservable = |reason: String| BindingGap::AccessorObservation {
            reason: format!("DeliveryContext: {reason}"),
        };
        let mut selecting = BTreeMap::new();
        let mut input = BTreeMap::new();
        for mapped in &self.binding.mapping {
            if mapped.conversion.is_some() {
                return Err(unobservable(format!(
                    "`{}` crosses a declared conversion to `{}`, whose result only the host \
                     computes",
                    mapped.target, mapped.target_type
                )));
            }
            let value = match &mapped.value {
                ResolvedMappingValue::DeliveryContext { field, .. } => {
                    let Some(value) = occurrence.context.get(field) else {
                        return Err(unobservable(format!(
                            "the context field `{field}` has no delivered value"
                        )));
                    };
                    ScenarioValue::literal(value.clone())
                }
                ResolvedMappingValue::EventField { field, .. } => {
                    let Some(value) = occurrence.payload.get(field) else {
                        return Err(unobservable(format!(
                            "the event field `{field}` has no delivered value"
                        )));
                    };
                    let value = ScenarioValue::literal(value.clone());
                    selecting.insert(mapped.target.clone(), value.clone());
                    value
                }
                // As for any binding: the text over text or an enum, the typed value over a
                // `Boolean`, `Integer` or `Decimal` (beyond10x/ess#445).
                ResolvedMappingValue::Literal { value } => ScenarioValue::literal(
                    crate::input::mapping_literal(self.ir, &mapped.target_type, value).ok_or_else(
                        || {
                            unobservable(format!(
                                "the constant `{value}` is not a value of `{}`",
                                mapped.target_type
                            ))
                        },
                    )?,
                ),
                ResolvedMappingValue::EventAccessor { plan, .. } => {
                    return Err(unobservable(format!(
                        "`{}` reads `{}` through a bounded accessor, which a delivered occurrence \
                         is not yet projected through",
                        mapped.target,
                        plan.path()
                    )))
                }
                ResolvedMappingValue::Selection { .. } => {
                    return Err(unobservable(format!(
                        "`{}` reads a local selection, which a delivered occurrence is not yet \
                         projected through",
                        mapped.target
                    )))
                }
                ResolvedMappingValue::HostContext { .. }
                | ResolvedMappingValue::HostRead { .. } => {
                    return Err(unobservable(
                        "a periodic host input on an event binding".to_owned(),
                    ))
                }
            };
            input.insert(mapped.target.clone(), value);
        }
        Ok((selecting, input))
    }

    /// Whether every context field the mapping reads can be told apart from everything a target
    /// could read in its place: the same field in the other occurrence, so that an invocation
    /// shows which context it was filled from, and — within each occurrence — every value at every
    /// depth of every other context field and every payload field, whatever their names and
    /// types, so that a target reading the payload (a flag inside a payload struct, a list
    /// element) or the wrong context field fails. Values are compared at every depth on both
    /// sides ([`leaves`]).
    fn separated(&self) -> Result<(), BindingGap> {
        for mapped in &self.binding.mapping {
            let ResolvedMappingValue::DeliveryContext { field, type_ref } = &mapped.value else {
                continue;
            };
            if let (Some(value), Some(again)) = (
                self.first.context.get(field),
                self.second.context.get(field),
            ) {
                if value == again {
                    let admitted = self.admitted.get(field).copied().unwrap_or(0);
                    return Err(BindingGap::AccessorObservation {
                        reason: format!(
                            "DeliveryContext: the context field `{field}` (`{type_ref}`) carries \
                             `{value}` in both deliveries: its type and invariants admit \
                             {admitted} of the first {} witness values, so two deliveries cannot \
                             be told apart by it",
                            witness::MAX_CANDIDATES
                        ),
                    });
                }
            }
            for (nth, occurrence) in [(1, &self.first), (2, &self.second)] {
                let Some(value) = occurrence.context.get(field) else {
                    continue;
                };
                let others = occurrence
                    .context
                    .iter()
                    .filter(|(name, _)| *name != field)
                    .map(|(name, other)| (format!("context.{name}"), other))
                    .chain(
                        occurrence
                            .payload
                            .iter()
                            .map(|(name, other)| (format!("event.{name}"), other)),
                    );
                for (other, other_value) in others {
                    let Some((mine, theirs, shared)) =
                        collision(&format!("context.{field}"), value, &other, other_value)
                    else {
                        continue;
                    };
                    return Err(BindingGap::AccessorObservation {
                        reason: format!(
                            "DeliveryContext: the context field `{field}` (`{type_ref}`) cannot \
                             be kept apart from `{other}`: in delivery {nth} `{mine}` and \
                             `{theirs}` both carry `{shared}`, and no values the declared types \
                             and invariants admit separate them — there are too few witness \
                             values to keep every field apart, so a target reading `{theirs}` in \
                             place of `{mine}` could not be told apart"
                        ),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Every value inside `node`, at every depth, with the path it sits at under `path`: the value
/// itself, each struct or map member and each list element. An absent value (`null`) is left out,
/// since no target reads one in place of a delivered value.
fn leaves<'n>(path: &str, node: &'n Node, found: &mut Vec<(String, &'n Node)>) {
    match node {
        Node::Null => return,
        Node::Map(members) => {
            for (name, member) in members {
                leaves(&format!("{path}.{name}"), member, found);
            }
        }
        Node::Seq(elements) => {
            for (index, element) in elements.iter().enumerate() {
                leaves(&format!("{path}.{index}"), element, found);
            }
        }
        Node::Bool(_) | Node::Number(_) | Node::Text(_) => {}
    }
    found.push((path.to_owned(), node));
}

/// The first value, at any depth, that `value` (at `path`) and `other` (at `other_path`) share:
/// the two paths and the value.
fn collision(
    path: &str,
    value: &Node,
    other_path: &str,
    other: &Node,
) -> Option<(String, String, Node)> {
    let (mut mine, mut theirs) = (Vec::new(), Vec::new());
    leaves(path, value, &mut mine);
    leaves(other_path, other, &mut theirs);
    mine.iter().find_map(|(at, node)| {
        theirs
            .iter()
            .find(|(_, candidate)| candidate == node)
            .map(|(other_at, _)| (at.clone(), other_at.clone(), (*node).clone()))
    })
}

/// Whether `value` and `other` share a value at any depth.
fn clashes(value: &Node, other: &Node) -> bool {
    collision("", value, "", other).is_some()
}

/// How many assignments [`occurrences`] tries before it gives up on separating and lets
/// [`Delivered::separated`] refuse with the collision. Only the mapped context fields are
/// searched, so a real model spends a handful.
const SEARCH_BUDGET: usize = 4096;

/// One declared field of the delivered event or its context, with every value its type and
/// invariants admit among the first [`witness::MAX_CANDIDATES`] distinctions, in distinction
/// order, without repeats.
struct Slot<'f> {
    field: &'f ResolvedField,
    in_context: bool,
    mapped: bool,
    values: Vec<Node>,
}

impl Slot<'_> {
    /// The order the values are tried in for occurrence `nth` (0 or 1): the second occurrence
    /// starts one value further on, so a field changes between the deliveries wherever it can.
    fn order(&self, nth: usize) -> impl Iterator<Item = usize> + '_ {
        let count = self.values.len();
        (0..count).map(move |index| (index + nth) % count)
    }
}

/// The two occurrences the suite delivers, and how many values each context field admits.
///
/// A witness keeps two fields of one table apart only where it is built from the field's path
/// (`String`, `Uuid`, `Json`); every other primitive and an enum take their value from the
/// distinction alone ([`witness::fields`]), and a struct moves every leaf by one distinction. So
/// values are chosen, not numbered: each field's admitted values are collected by distinction —
/// a value its declared invariants refuse is skipped, never delivered — and the context fields
/// the mapping reads are searched until each differs between the deliveries and, within each
/// delivery, shares no value at any depth with any other context or payload field. Every other
/// field then takes the first value (the second delivery starting one further on) that holds none
/// of those, preferring one no earlier field of the delivery carries. What no choice separates
/// is left to [`Delivered::separated`], which refuses `mapping` and `delivery` with the collision;
/// `flow` and `on-failure` need one delivery only and are synthesized from it.
///
/// # Errors
///
/// The [`witness::WitnessGap`] of a field no value of whose type and invariants is admitted at
/// any of the distinctions tried; nothing can be delivered then.
fn occurrences(
    ir: &EssIr,
    payload: &[ResolvedField],
    context: &[ResolvedField],
    mapped: &BTreeSet<&str>,
) -> Result<(Occurrence, Occurrence, BTreeMap<String, usize>), witness::WitnessGap> {
    let mut slots = Vec::new();
    for (in_context, field) in payload
        .iter()
        .map(|field| (false, field))
        .chain(context.iter().map(|field| (true, field)))
    {
        let mut values: Vec<Node> = Vec::new();
        let mut gap = None;
        let mut absent = false;
        for nth in 0..witness::MAX_CANDIDATES {
            match witness::fields(ir, std::slice::from_ref(field), Distinction::further(nth)) {
                Ok(mut found) => match found.remove(&field.name) {
                    Some(value) if !values.contains(&value) => values.push(value),
                    Some(_) => {}
                    // Left out of the table, as an optional field is: absent from every delivery.
                    None => {
                        absent = values.is_empty();
                        break;
                    }
                },
                Err(refused) => {
                    gap.get_or_insert(refused);
                }
            }
        }
        if absent {
            continue;
        }
        if values.is_empty() {
            return Err(
                gap.expect("a field with no admitted value was refused at some distinction")
            );
        }
        slots.push(Slot {
            field,
            in_context,
            mapped: in_context && mapped.contains(field.name.as_str()),
            values,
        });
    }
    let searched: Vec<usize> = (0..slots.len()).filter(|&at| slots[at].mapped).collect();
    let chosen = search(&slots, &searched).unwrap_or_else(|| {
        // Nothing separates: each field at its first value, and the second delivery one further
        // on, for `separated` to name the collision and for `flow` and `on-failure`.
        [0, 1].map(|nth| {
            slots
                .iter()
                .map(|slot| slot.order(nth).next().expect("every slot has a value"))
                .collect()
        })
    });
    let both = [0, 1].map(|nth| {
        let mut occurrence = Occurrence {
            payload: BTreeMap::new(),
            context: BTreeMap::new(),
        };
        for (slot, &index) in slots.iter().zip(&chosen[nth]) {
            let side = if slot.in_context {
                &mut occurrence.context
            } else {
                &mut occurrence.payload
            };
            side.insert(slot.field.name.clone(), slot.values[index].clone());
        }
        occurrence
    });
    let admitted = slots
        .iter()
        .filter(|slot| slot.in_context)
        .map(|slot| (slot.field.name.clone(), slot.values.len()))
        .collect();
    let [first, second] = both;
    Ok((first, second, admitted))
}

/// The value index of every slot in each occurrence, or `None` when no assignment of the
/// `searched` (mapped context) slots within [`SEARCH_BUDGET`] separates them.
fn search(slots: &[Slot<'_>], searched: &[usize]) -> Option<[Vec<usize>; 2]> {
    // The variables, in order: every searched slot in the first occurrence, then in the second.
    let variables: Vec<(usize, usize)> = [0, 1]
        .into_iter()
        .flat_map(|nth| searched.iter().map(move |&at| (nth, at)))
        .collect();
    let mut assigned: Vec<usize> = Vec::with_capacity(variables.len());
    let mut budget = SEARCH_BUDGET;
    descend(slots, &variables, &mut assigned, &mut budget)
}

/// One step of [`search`]: extend `assigned` by the next variable's value, or complete it.
fn descend(
    slots: &[Slot<'_>],
    variables: &[(usize, usize)],
    assigned: &mut Vec<usize>,
    budget: &mut usize,
) -> Option<[Vec<usize>; 2]> {
    let Some(&(nth, at)) = variables.get(assigned.len()) else {
        return complete(slots, variables, assigned);
    };
    let tried: Vec<usize> = slots[at].order(nth).collect();
    for index in tried {
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        let value = &slots[at].values[index];
        let consistent = variables
            .iter()
            .zip(assigned.iter())
            .all(|(&(was_nth, was_at), &was)| {
                let other = &slots[was_at].values[was];
                if was_at == at {
                    // The same field in the other delivery: the two deliveries differ.
                    return other != value;
                }
                was_nth != nth || !clashes(value, other)
            });
        if !consistent {
            continue;
        }
        assigned.push(index);
        if let Some(found) = descend(slots, variables, assigned, budget) {
            return Some(found);
        }
        assigned.pop();
    }
    None
}

/// Every unsearched slot's value in each occurrence, given the searched ones: the first in the
/// slot's order holding no searched value of that occurrence at any depth, preferring one no
/// earlier slot of the occurrence carries. `None` when some slot has no such value.
fn complete(
    slots: &[Slot<'_>],
    variables: &[(usize, usize)],
    assigned: &[usize],
) -> Option<[Vec<usize>; 2]> {
    let mut chosen = [vec![0; slots.len()], vec![0; slots.len()]];
    for (&(nth, at), &index) in variables.iter().zip(assigned) {
        chosen[nth][at] = index;
    }
    for nth in [0, 1] {
        let fixed: Vec<&Node> = variables
            .iter()
            .zip(assigned)
            .filter(|((was_nth, _), _)| *was_nth == nth)
            .map(|(&(_, at), &index)| &slots[at].values[index])
            .collect();
        let mut taken: Vec<&Node> = fixed.clone();
        for (at, slot) in slots.iter().enumerate() {
            if slot.mapped {
                continue;
            }
            let free: Vec<usize> = slot
                .order(nth)
                .filter(|&index| {
                    fixed
                        .iter()
                        .all(|searched| !clashes(searched, &slot.values[index]))
                })
                .collect();
            let index = free
                .iter()
                .copied()
                .find(|&index| !taken.contains(&&slot.values[index]))
                .or_else(|| free.first().copied())?;
            chosen[nth][at] = index;
            taken.push(&slot.values[index]);
        }
    }
    Some(chosen)
}

/// Every scenario of one binding that declares a delivery context.
#[allow(clippy::too_many_lines)]
pub(super) fn synthesize(
    ir: &EssIr,
    binding: &ResolvedBinding,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let Some(context) = &binding.context else {
        return;
    };
    let subject = BindingRef::new(binding.name.clone());
    let event_handle = binding
        .cause
        .event()
        .expect("a delivery context is on an event cause");
    // A policy selected per refusal (ess/22, beyond10x/ess#269) of a delivered event: each refusal
    // is refused by name, as a bounded retry's failure scenario is here, whatever else follows.
    if let Some(policy) = &binding.refusal_policy {
        super::refusal_policy::refuse_all(
            binding,
            policy,
            &BindingGap::AccessorObservation {
                reason: "DeliveryContext: a failure policy selected per refusal of a delivered \
                         event is not synthesized yet"
                    .into(),
            },
            refusals,
        );
    }
    let event = EventRef::from(event_handle);
    let mapped: BTreeSet<&str> = binding
        .mapping
        .iter()
        .filter_map(|mapped| match &mapped.value {
            ResolvedMappingValue::DeliveryContext { field, .. } => Some(field.as_str()),
            _ => None,
        })
        .collect();
    let (mut first, mut second, admitted) =
        match occurrences(ir, &ir.event(event_handle).fields, &context.fields, &mapped) {
            Ok(chosen) => chosen,
            Err(gap) => {
                refusals.push(Refusal {
                    subject: subject.into(),
                    scenario: None,
                    cause: RefusalCause::NoWitness(gap),
                    stands: false,
                });
                return;
            }
        };
    // An event-payload condition (ess/22, beyond10x/ess#268): the delivered payloads are chosen for
    // it too, varying only the members it reads — made to hold for the four positive aspects, and
    // to fail, or to leave a proved Optional level out, for its two witnesses.
    let witnessed = binding.condition.as_ref().map(|condition| {
        super::binding_condition::delivered(
            ir,
            event_handle,
            condition,
            &[first.payload.clone(), second.payload.clone()],
        )
    });
    let mut unheld = None;
    if let Some(witnessed) = &witnessed {
        match &witnessed.holds {
            Ok(holding) => {
                first.payload = holding[0].clone();
                second.payload = holding[1].clone();
            }
            Err(gap) => unheld = Some(gap.clone()),
        }
    }
    let delivered = Delivered {
        ir,
        binding,
        invoked: ir.command(&binding.command),
        context,
        event,
        first,
        second,
        admitted,
    };
    let source = source(ir, &delivered);

    for aspect in BindingAspect::ALL.map(|(aspect, _)| aspect) {
        let id = ScenarioId::Binding {
            binding: subject.clone(),
            aspect,
        };
        let built = match (aspect, &unheld) {
            (
                BindingAspect::FinalFailure
                | BindingAspect::ConditionFalse
                | BindingAspect::ConditionAbsent,
                _,
            ) => continue,
            (BindingAspect::OnFailure, _) if binding.refusal_policy.is_some() => continue,
            (_, Some(gap)) => Err(gap.clone()),
            (BindingAspect::Flow, None) => flow(ir, &delivered),
            (BindingAspect::Mapping, None) => mapping(&delivered),
            (BindingAspect::Delivery, None) => delivery(ir, &delivered),
            (BindingAspect::OnFailure, None) => on_failure(ir, &delivered),
        };
        let (steps, purpose, extra) = match built {
            Ok(built) => built,
            Err(gap) => {
                refusals.push(Refusal::about(
                    &id,
                    RefusalCause::BindingUnobservable {
                        binding: subject.clone(),
                        gap,
                    },
                ));
                continue;
            }
        };
        let mut depends = source.clone();
        depends.extend(extra);
        insert(
            suite,
            id,
            ConformanceScenario::new(purpose, steps, depends),
            refusals,
        );
    }
    if let Some(witnessed) = witnessed {
        negative_witnesses(ir, &delivered, witnessed, &source, suite, refusals);
    }
}

/// A conditioned external binding's `condition-false` and `condition-absent` scenarios, or the
/// refusal naming why each could not be delivered (ess/22, beyond10x/ess#268).
fn negative_witnesses(
    ir: &EssIr,
    delivered: &Delivered<'_>,
    witnessed: super::binding_condition::Delivered,
    source: &BTreeSet<EssSemanticRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    let subject = BindingRef::new(delivered.binding.name.clone());
    for (aspect, witness) in [
        (
            BindingAspect::ConditionFalse,
            Some(witnessed.fails.map(|payload| vec![payload])),
        ),
        (BindingAspect::ConditionAbsent, witnessed.absent),
    ] {
        let Some(witness) = witness else {
            continue;
        };
        let id = ScenarioId::Binding {
            binding: subject.clone(),
            aspect,
        };
        match witness {
            Ok(payloads) => {
                let (steps, purpose, siblings) = never_invoked(ir, delivered, payloads);
                let mut depends = source.clone();
                depends.extend(siblings.into_iter().map(EssSemanticRef::from));
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

/// A conditioned external binding's negative witness (ess/22, beyond10x/ess#268): each payload,
/// one its condition does not hold for, delivered under the first context, and zero invocations of
/// the binding for the whole eventual window. The suite delivers every occurrence itself, so no
/// other one can arrive to blur it.
fn never_invoked(
    ir: &EssIr,
    delivered: &Delivered<'_>,
    payloads: Vec<BTreeMap<String, Node>>,
) -> (Vec<ScenarioStep>, super::ScenarioPurpose, Vec<BindingRef>) {
    let context = &delivered.first.context;
    // Where the condition is Unknown on a delivered payload, the binding owes its unmet
    // obligation for it, and a target that skips silently fails.
    let unknown = delivered
        .binding
        .condition
        .as_ref()
        .is_some_and(|condition| {
            payloads.iter().any(|payload| {
                condition.plan.evaluate(payload) == Ok(ess_primitives::predicate::Truth::Unknown)
            })
        });
    // `other_binding_still_invokes`, as for an event a command publishes: every binding beside
    // this one that the channel delivers the occurrence to still invokes, before the window.
    let siblings = super::siblings(
        ir,
        delivered.binding,
        &payloads,
        Some((delivered.context.authority.as_str(), context)),
    );
    let mut steps: Vec<ScenarioStep> = payloads
        .into_iter()
        .map(|payload| {
            delivered.deliver(&Occurrence {
                payload,
                context: context.clone(),
            })
        })
        .collect();
    for sibling in &siblings {
        steps.push(ScenarioStep::ExpectInvocation {
            binding: BindingRef::new(sibling.name.clone()),
            command: CommandRef::new(ir.command(&sibling.command).name.clone()),
            input: BTreeMap::new(),
            count: None,
        });
    }
    steps.push(ScenarioStep::ExpectNoInvocation {
        binding: BindingRef::new(delivered.binding.name.clone()),
        command: delivered.command(),
        obligation: unknown.then(|| crate::no_invocation::UNKNOWN_CONDITION.to_owned()),
    });
    let text = format!(
        "`{}` invokes `{}` no times for a delivered `{}` its condition does not hold for",
        delivered.binding.name, delivered.invoked.name, delivered.event
    );
    let siblings = siblings
        .iter()
        .map(|sibling| BindingRef::new(sibling.name.clone()))
        .collect();
    (steps, clipped(&text), siblings)
}

/// What every scenario of the binding rests on: the binding, the event and its field types, the
/// context's field types and the invoked command's input.
fn source(ir: &EssIr, delivered: &Delivered<'_>) -> BTreeSet<EssSemanticRef> {
    let mut source: BTreeSet<EssSemanticRef> = [
        BindingRef::new(delivered.binding.name.clone()).into(),
        delivered.event.clone().into(),
        delivered.command().into(),
    ]
    .into_iter()
    .collect();
    let mut types = BTreeSet::new();
    let event = delivered
        .binding
        .cause
        .event()
        .expect("a delivery context is on an event cause");
    for field in ir
        .event(event)
        .fields
        .iter()
        .chain(&delivered.context.fields)
    {
        reachable_types(ir, &field.type_ref, &mut types);
    }
    source.extend(types.into_iter().map(EssSemanticRef::from));
    source.extend(
        input_types(ir, delivered.invoked)
            .into_iter()
            .map(EssSemanticRef::from),
    );
    if let Some(component) = accepting_component(ir, &delivered.invoked.name) {
        source.insert(component.into());
    }
    source
}

/// The invoked branch's events, required eventually, where the branch is decided and publishes.
fn consequences(ir: &EssIr, delivered: &Delivered<'_>) -> Result<Consequences, BindingGap> {
    let reached = reachable_branch(delivered.invoked)?;
    let published = publishes(delivered.invoked, reached)?;
    let steps = published
        .iter()
        .map(|event| ScenarioStep::EventuallyEvent {
            event: event.clone(),
            payload: BTreeMap::new(),
            shape: payload_shape(ir, event),
        })
        .collect();
    Ok((steps, published, downstream(ir, delivered.invoked, reached)))
}

/// §16: the delivered event invokes the command, which publishes what its branch declares.
fn flow(ir: &EssIr, delivered: &Delivered<'_>) -> Built {
    let (after, published, extra) = consequences(ir, delivered)?;
    let mut steps = vec![delivered.deliver(&delivered.first)];
    steps.extend(after);
    let text = format!(
        "`{}` delivered from `{}` invokes `{}`, which publishes {}",
        delivered.event,
        delivered.context.authority,
        delivered.invoked.name,
        listed(&published)
    );
    Ok((steps, clipped(&text), extra))
}

/// §16, and the issue's own claim: one event delivered under two contexts, each invocation
/// carrying its own delivery's context.
fn mapping(delivered: &Delivered<'_>) -> Built {
    if delivered.binding.mapping.is_empty() {
        return Err(BindingGap::NothingMapped {
            command: delivered.command(),
        });
    }
    delivered.separated()?;
    let binding = BindingRef::new(delivered.binding.name.clone());
    let mut steps = Vec::new();
    for occurrence in [&delivered.first, &delivered.second] {
        let (_, input) = delivered.expected(occurrence)?;
        steps.push(delivered.deliver(occurrence));
        steps.push(ScenarioStep::ExpectInvocation {
            binding: binding.clone(),
            command: delivered.command(),
            input,
            count: None,
        });
    }
    let text = format!(
        "`{}` fills `{}` from each delivery of `{}`, its context included",
        delivered.binding.name, delivered.invoked.name, delivered.event
    );
    Ok((steps, clipped(&text), BTreeSet::new()))
}

/// §17 for a delivered event: a redelivery carries the context of the occurrence it repeats.
///
/// Two occurrences under two contexts, then a redelivery of the event — which repeats the most
/// recent — and every invocation for that occurrence is required to carry its context. Where no
/// payload field reaches the input, the invocations of the two occurrences cannot be told apart,
/// so the scenario delivers the second one alone; the context is still required of every
/// invocation, and the first-context defect is then not observable through this binding.
fn delivery(ir: &EssIr, delivered: &Delivered<'_>) -> Built {
    if matches!(delivered.binding.delivery, Delivery::AtMostOnce) {
        return Err(BindingGap::DeliverySingleAttempt);
    }
    delivered.separated()?;
    let (first_selecting, _) = delivered.expected(&delivered.first)?;
    let (selecting, input) = delivered.expected(&delivered.second)?;
    let mut steps = Vec::new();
    if !selecting.is_empty() && first_selecting != selecting {
        steps.push(delivered.deliver(&delivered.first));
    }
    steps.push(delivered.deliver(&delivered.second));
    match delivered.binding.delivery {
        Delivery::AtLeastOnce => steps.push(ScenarioStep::RedeliverEvent {
            event: delivered.event.clone(),
        }),
        Delivery::AtMostOnce => return Err(BindingGap::DeliverySingleAttempt),
    }
    steps.push(ScenarioStep::ExpectEveryInvocation {
        binding: BindingRef::new(delivered.binding.name.clone()),
        command: delivered.command(),
        selecting,
        input,
    });
    let mut extra = BTreeSet::new();
    if let Ok((after, _, depends)) = consequences(ir, delivered) {
        steps.extend(after);
        extra = depends;
    }
    let text = format!(
        "`{}` redelivered carries the context it was delivered with, and no count is required",
        delivered.event
    );
    Ok((steps, clipped(&text), extra))
}

/// §18 for a delivered event: the declared failure policy, with the failure forced before the
/// delivery.
fn on_failure(ir: &EssIr, delivered: &Delivered<'_>) -> Built {
    let policy = delivered.binding.on_failure();
    if matches!(policy, ResolvedFailure::Drop) {
        return Err(BindingGap::PolicySilent);
    }
    if matches!(
        policy,
        ResolvedFailure::BoundedRetry { .. } | ResolvedFailure::ByRefusal { .. }
    ) {
        return Err(BindingGap::AccessorObservation {
            reason: "DeliveryContext: a bounded retry of a delivered event is not synthesized yet"
                .into(),
        });
    }
    let invoked = delivered.invoked;
    let forced = invoked
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::InjectFault)
        .ok_or_else(|| BindingGap::NoForcibleFailure {
            command: delivered.command(),
        })?;
    if matches!(forced.condition, ResolvedCondition::ExternalWhen { .. }) {
        return Err(BindingGap::AccessorObservation {
            reason: "GuardedExternalEligibility: fault eligibility requires an observation of the binding-mapped input".into(),
        });
    }
    let forced_ref = OutcomeRef::new(delivered.command(), forced.name.clone());
    let mut steps = vec![
        ScenarioStep::ConfigureExternalOutcome {
            force: forced_ref.clone(),
            times: None,
        },
        delivered.deliver(&delivered.first),
    ];
    let mut extra: BTreeSet<EssSemanticRef> = [forced_ref.into()].into_iter().collect();
    let text = match policy {
        ResolvedFailure::Retry => {
            let (after, published, depends) = consequences(ir, delivered)?;
            steps.extend(after);
            extra.extend(depends);
            format!(
                "`{}` retries a failed `{}` until {} is published",
                delivered.binding.name,
                invoked.name,
                listed(&published)
            )
        }
        ResolvedFailure::Escalate { emits } => {
            let escalation = EventRef::from(emits);
            steps.push(ScenarioStep::EventuallyEvent {
                event: escalation.clone(),
                payload: BTreeMap::new(),
                shape: payload_shape(ir, &escalation),
            });
            extra.insert(escalation.clone().into());
            format!(
                "a failed `{}` makes `{}` escalate into `{escalation}`",
                invoked.name, delivered.binding.name
            )
        }
        ResolvedFailure::Drop
        | ResolvedFailure::BoundedRetry { .. }
        | ResolvedFailure::ByRefusal { .. } => {
            unreachable!("refused above, before anything was forced")
        }
    };
    Ok((steps, clipped(&text), extra))
}
