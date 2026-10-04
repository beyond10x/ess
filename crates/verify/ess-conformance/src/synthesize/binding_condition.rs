//! Witnesses for a binding's event-payload condition (ess/22, beyond10x/ess#268 and
//! beyond10x/ess#194, `docs/design/conditional-binding-failure-policies.md`).
//!
//! A conditioned binding invokes only for an occurrence whose payload makes its condition True, so
//! every scenario that expects it to run needs a trigger that publishes such a payload, and two
//! scenarios prove it does not run otherwise:
//!
//! | aspect | the trigger publishes | required |
//! |---|---|---|
//! | flow, mapping, delivery, on-failure | a payload the condition holds for | as before |
//! | [`ConditionFalse`](BindingAspect::ConditionFalse) | a payload it does not hold for, every Optional member it reads present | the occurrence, then zero invocations for the whole window |
//! | [`ConditionAbsent`](BindingAspect::ConditionAbsent) | one occurrence per Optional level the condition proves present, that level absent | each occurrence, then zero invocations for the whole window |
//!
//! # Where the payload comes from
//!
//! Every branch that publishes the event is a candidate trigger, in the model's order. A member the
//! condition reads is either written by the branch as a literal, which fixes it for that branch, or
//! copied straight from one input (`input.<field>`, no conversion), which this synthesis varies —
//! and only where that is all varying changes: no branch of the publishing command guards on the
//! input, and the branch reads it nowhere else. Any other source rules the branch out. The values
//! tried for an input are finite: the trigger's own value, every literal the condition compares
//! the member with, every variant of an enum leaf, and absence of each Optional member, in that
//! order.
//!
//! The positive aspects use the first branch and value set the condition holds for. Each negative
//! witness differs from that payload in one fact where one does: one compared leaf changed, or one
//! proved level left out. A level left out may leave the condition Unknown rather than False — a
//! comparison reading an absent member — and Unknown invokes nothing too. Where no single fact
//! makes the condition fail, the first branch, in order, that publishes a payload it fails for with
//! every member present is taken.
//!
//! # One occurrence, and only one
//!
//! Zero invocations are required of the binding for the whole scenario, so a negative witness must
//! publish the event exactly once. A witness is not taken where a setup command may publish the
//! event, or where a chain of bindings set off by the scenario's own events may publish it again:
//! another binding on the occurrence whose condition the known payload does not rule out, and every
//! binding on any later event, are followed to every event their commands may publish.
//!
//! Anything else is [`BindingGap::ConditionUnarranged`] naming why, never a guessed payload.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ess_compiler::ir::{
    EssIr, EventHandle, ResolvedBinding, ResolvedBindingCondition, ResolvedCommand, ResolvedField,
    ResolvedOutcome, ResolvedPayloadValue, ResolvedTypeRef,
};
use ess_domain::binding::condition::{ConditionRead, Leaf};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, Truth};

use super::{ActorRef, BindingGap, RefusalCause, Run};
use crate::scenario::{CommandRef, ScenarioStep, ScenarioValue};

/// The most value sets tried for one condition on one branch.
const MAX_TRIALS: usize = 512;

/// The payload members the condition reads.
type Payload = BTreeMap<String, Node>;

/// A branch that publishes the binding's event, and the run that takes it.
pub(super) struct Trigger<'ir> {
    /// The publishing command.
    pub(super) publisher: &'ir ResolvedCommand,
    /// The branch taken.
    pub(super) published_by: &'ir ResolvedOutcome,
    /// The arrangement and invocation.
    pub(super) run: Run,
}

/// The triggers a conditioned binding's scenarios are built from.
pub(super) struct Witnesses<'ir> {
    /// A trigger the condition holds for.
    pub(super) holds: Result<Trigger<'ir>, BindingGap>,
    /// A trigger it does not hold for with every Optional member it reads present.
    pub(super) fails: Result<Trigger<'ir>, BindingGap>,
    /// One trigger per Optional level the condition proves present, outermost first, that level
    /// absent; `None` where it proves no Optional member present.
    pub(super) absent: Option<Result<Vec<Trigger<'ir>>, BindingGap>>,
}

fn gap(why: impl Into<String>) -> BindingGap {
    BindingGap::ConditionUnarranged { why: why.into() }
}

/// Where a member the condition reads comes from on one branch.
enum Source {
    /// Copied from this trigger input, which may be varied.
    Input(String),
    /// Written by the branch, and fixed.
    Literal,
}

/// One member of the event the condition reads.
struct Member {
    /// The event member.
    name: String,
    /// What fills it.
    source: Source,
}

/// What one read path may be set to in a trial.
#[derive(Clone)]
enum Choice {
    /// Whatever the base value holds.
    Base,
    /// A present text leaf.
    Text(String),
    /// The Optional member at this many members into the path is absent.
    Absent(usize),
}

/// One publishing branch, ready to be varied.
struct Candidate<'ir> {
    trigger: Trigger<'ir>,
    members: Vec<Member>,
    base: Payload,
    choices: Vec<Vec<Choice>>,
}

/// What the witnesses are searched with: the binding, its condition and its reads.
struct Search<'a> {
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
    condition: &'a ResolvedBindingCondition,
    event: &'a EventHandle,
    reads: Vec<&'a ConditionRead>,
}

/// Builds the triggers for `binding`'s condition from every branch that publishes its event. The
/// error is a refusal of the whole binding: nothing publishes the event, or no branch can be run.
pub(super) fn witnesses<'ir>(
    ir: &'ir EssIr,
    binding: &'ir ResolvedBinding,
    condition: &'ir ResolvedBindingCondition,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Witnesses<'ir>, RefusalCause> {
    let event = binding
        .cause
        .event()
        .expect("a conditioned binding has an event cause");
    let search = Search {
        ir,
        binding,
        condition,
        event,
        reads: condition.plan.reads.values().collect(),
    };
    let mut ruled_out = Vec::new();
    let mut first_failure = None;
    let mut published = false;
    let mut candidates = Vec::new();
    for publisher in ir.commands().values() {
        for published_by in publisher
            .outcomes
            .iter()
            .filter(|o| o.emits.contains(event))
        {
            published = true;
            let run = match super::run(ir, publisher, published_by, actors) {
                Ok(run) => run,
                Err(cause) => {
                    first_failure.get_or_insert(cause);
                    continue;
                }
            };
            let trigger = Trigger {
                publisher,
                published_by,
                run,
            };
            match search.candidate(trigger) {
                Ok(candidate) => candidates.push(candidate),
                Err(why) => ruled_out.push(why),
            }
        }
    }
    if !published {
        return Err(RefusalCause::BindingUnobservable {
            binding: crate::scenario::BindingRef::new(binding.name.clone()),
            gap: BindingGap::NothingPublishes {
                event: crate::scenario::EventRef::from(event),
            },
        });
    }
    if candidates.is_empty() && ruled_out.is_empty() {
        return Err(first_failure.expect("a publishing branch either runs or fails"));
    }
    Ok(search.witnesses(&candidates, &ruled_out))
}

impl Search<'_> {
    fn witnesses<'ir>(
        &self,
        candidates: &[Candidate<'ir>],
        ruled_out: &[String],
    ) -> Witnesses<'ir> {
        let unarranged = |side: &str| {
            let mut why = format!(
                "no publishing branch and value set this synthesis tries makes it {side} (`{}`)",
                self.condition.plan.predicate
            );
            for reason in ruled_out {
                why.push_str("; ");
                why.push_str(reason);
            }
            gap(why)
        };
        let holds = candidates.iter().find_map(|candidate| {
            self.search(candidate, |truth, complete, _| {
                truth == Truth::True && complete
            })
            .or_else(|| self.search(candidate, |truth, _, _| truth == Truth::True))
            .map(|payload| (candidate, payload))
        });
        let Some((holding, payload)) = holds else {
            return Witnesses {
                holds: Err(unarranged("hold")),
                fails: Err(unarranged("hold, so no single fact can make it fail")),
                absent: self
                    .proves_optional()
                    .then(|| Err(unarranged("hold, so no proved member can be left out"))),
            };
        };
        let fails = self
            .one_leaf(holding, &payload, true)
            .map(|payload| (holding, payload))
            .or_else(|| {
                candidates.iter().find_map(|candidate| {
                    self.search(candidate, |truth, complete, payload| {
                        truth == Truth::False
                            && complete
                            && self.republishes(candidate, payload).is_none()
                    })
                    .map(|payload| (candidate, payload))
                })
            });
        Witnesses {
            holds: varied(holding, self.event, &payload),
            fails: fails.map_or_else(
                || {
                    Err(self
                        .republication(candidates, holding, &payload)
                        .map_or_else(
                            || unarranged("fail with every member it reads present"),
                            |why| {
                                gap(format!(
                                "every payload it fails for with every member present publishes \
                                 the event again: {why}"
                            ))
                            },
                        ))
                },
                |(candidate, payload)| varied(candidate, self.event, &payload),
            ),
            absent: self
                .proves_optional()
                .then(|| self.levels(holding, &payload)),
        }
    }

    /// Why the first payload the condition fails for with every member present — one leaf changed
    /// from `holds`, else the first branch's — publishes the event again, where it does.
    fn republication(
        &self,
        candidates: &[Candidate<'_>],
        holding: &Candidate<'_>,
        holds: &Payload,
    ) -> Option<String> {
        self.one_leaf(holding, holds, false)
            .map(|payload| (holding, payload))
            .or_else(|| {
                candidates.iter().find_map(|candidate| {
                    self.search(candidate, |truth, complete, _| {
                        truth == Truth::False && complete
                    })
                    .map(|payload| (candidate, payload))
                })
            })
            .and_then(|(candidate, payload)| self.republishes(candidate, &payload))
    }

    /// Whether the condition proves any Optional member present.
    fn proves_optional(&self) -> bool {
        !self.proved_levels().is_empty()
    }

    /// Every Optional level the condition proves present, outermost first.
    fn proved_levels(&self) -> Vec<Vec<String>> {
        let mut levels = BTreeSet::new();
        for read in &self.reads {
            for (depth, optional) in read.optional.iter().enumerate() {
                let prefix = &read.members[..=depth];
                if *optional && self.condition.proves(prefix) {
                    levels.insert((depth, prefix.to_vec()));
                }
            }
        }
        levels.into_iter().map(|(_, prefix)| prefix).collect()
    }

    /// Prepares one publishing branch, or says why it cannot be varied.
    fn candidate<'ir>(&self, trigger: Trigger<'ir>) -> Result<Candidate<'ir>, String> {
        let members = self.members(&trigger)?;
        unguarded(&trigger, &members)?;
        let mut base = Payload::new();
        for member in &members {
            let value = match &member.source {
                Source::Literal => literal(&trigger, self.event, &member.name)
                    .ok_or_else(|| format!("`{}` writes no literal", trigger.published_by.name))?,
                Source::Input(input) => match sent(&trigger, input) {
                    Some(ScenarioValue::Literal { value }) if !matches!(value, Node::Null) => {
                        value.clone()
                    }
                    Some(ScenarioValue::Literal { .. }) | None => {
                        present_witness(self.ir, trigger.publisher, input)?
                    }
                    Some(_) => {
                        return Err(format!(
                            "`{}` sends `{input}` as a value it does not choose",
                            trigger.publisher.name
                        ))
                    }
                },
            };
            base.insert(member.name.clone(), value);
        }
        let choices: Vec<Vec<Choice>> = self
            .reads
            .iter()
            .map(|read| {
                let fixed = members.iter().any(|member| {
                    member.name == read.members[0] && matches!(member.source, Source::Literal)
                });
                if fixed {
                    vec![Choice::Base]
                } else {
                    choices(read, &self.condition.plan.predicate)
                }
            })
            .collect();
        let trials = choices.iter().map(Vec::len).product::<usize>();
        if trials > MAX_TRIALS {
            return Err(format!(
                "`{}` needs {trials} value sets tried, more than the {MAX_TRIALS} this synthesis \
                 tries",
                trigger.published_by.name
            ));
        }
        Ok(Candidate {
            trigger,
            members,
            base,
            choices,
        })
    }

    /// Every event member the condition reads, with what fills it on this branch.
    fn members(&self, trigger: &Trigger<'_>) -> Result<Vec<Member>, String> {
        let roots: BTreeSet<&String> = self.reads.iter().map(|read| &read.members[0]).collect();
        roots
            .into_iter()
            .map(|name| {
                let source = trigger
                    .published_by
                    .payload
                    .iter()
                    .filter(|payload| &payload.event == self.event)
                    .flat_map(|payload| &payload.fields)
                    .find(|field| &field.target == name);
                let source = match source {
                    Some(field) if field.conversion.is_none() => match &field.value {
                        ResolvedPayloadValue::InputField { field: input, .. } => {
                            Source::Input(input.clone())
                        }
                        ResolvedPayloadValue::Literal { .. } => Source::Literal,
                        _ => {
                            return Err(format!(
                                "`{}` fills `event.{name}` from neither an input nor a literal",
                                trigger.published_by.name
                            ))
                        }
                    },
                    _ => {
                        return Err(format!(
                            "`{}` does not fill `event.{name}` unconverted",
                            trigger.published_by.name
                        ))
                    }
                };
                Ok(Member {
                    name: name.clone(),
                    source,
                })
            })
            .collect()
    }

    /// The first value set on `candidate`, in trial order, `wanted` accepts given the condition's
    /// truth, whether every member it reads is present, and the payload.
    fn search(
        &self,
        candidate: &Candidate<'_>,
        wanted: impl Fn(Truth, bool, &Payload) -> bool,
    ) -> Option<Payload> {
        let trials = candidate.choices.iter().map(Vec::len).product::<usize>();
        (0..trials).find_map(|trial| {
            let mut payload = candidate.base.clone();
            let mut index = trial;
            let mut absences = Vec::new();
            for (read, options) in self.reads.iter().zip(&candidate.choices) {
                let choice = &options[index % options.len()];
                index /= options.len();
                match choice {
                    Choice::Base => {}
                    Choice::Text(text) => {
                        set_leaf(&mut payload, &read.members, Node::Text(text.clone()));
                    }
                    Choice::Absent(depth) => absences.push(read.members[..=*depth].to_vec()),
                }
            }
            for path in &absences {
                clear(&mut payload, path);
            }
            let truth = self.condition.plan.evaluate(&payload).ok()?;
            let complete = self
                .reads
                .iter()
                .all(|read| first_absent(&payload, read).is_none());
            wanted(truth, complete, &payload).then_some(payload)
        })
    }

    /// The holding payload with one compared leaf changed so the condition fails, every member
    /// still present and the event published once.
    fn one_leaf(&self, candidate: &Candidate<'_>, holds: &Payload, once: bool) -> Option<Payload> {
        self.reads
            .iter()
            .zip(&candidate.choices)
            .find_map(|(read, options)| {
                options.iter().find_map(|choice| {
                    let Choice::Text(text) = choice else {
                        return None;
                    };
                    let mut payload = holds.clone();
                    (set_leaf(&mut payload, &read.members, Node::Text(text.clone()))
                        && self.condition.plan.evaluate(&payload) == Ok(Truth::False)
                        && self
                            .reads
                            .iter()
                            .all(|read| first_absent(&payload, read).is_none())
                        && (!once || self.republishes(candidate, &payload).is_none()))
                    .then_some(payload)
                })
            })
    }

    /// One trigger per proved Optional level, each the holding payload with that level left out.
    /// Leaving a level out makes the condition False, or Unknown where a comparison reads it; both
    /// invoke nothing.
    fn levels<'ir>(
        &self,
        candidate: &Candidate<'ir>,
        holds: &Payload,
    ) -> Result<Vec<Trigger<'ir>>, BindingGap> {
        self.proved_levels()
            .into_iter()
            .map(|level| {
                let path = format!("event.{}", level.join("."));
                let mut payload = holds.clone();
                clear(&mut payload, &level);
                if self.condition.plan.evaluate(&payload) == Ok(Truth::True) {
                    return Err(gap(format!(
                        "leaving `{path}` out still makes its condition hold"
                    )));
                }
                if let Some(why) = self.republishes(candidate, &payload) {
                    return Err(gap(format!("with `{path}` left out, {why}")));
                }
                varied(candidate, self.event, &payload)
            })
            .collect()
    }

    /// Why a scenario triggered by `candidate` publishing `payload` may publish the event more than
    /// once, or `None`. Setup commands that may publish it count, and so does every chain of
    /// bindings the scenario's own events set off: another binding on the occurrence counts unless
    /// the known payload makes its condition fail, and every binding on any later event counts.
    fn republishes(&self, candidate: &Candidate<'_>, payload: &Payload) -> Option<String> {
        let known: BTreeSet<&String> = candidate
            .members
            .iter()
            .map(|member| &member.name)
            .collect();
        let mut seen: BTreeSet<&EventHandle> = BTreeSet::new();
        let mut pending: VecDeque<(&EventHandle, bool)> = VecDeque::new();
        for step in &candidate.trigger.run.setup {
            let (ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. }) = step
            else {
                continue;
            };
            let declared = self
                .ir
                .commands()
                .values()
                .find(|declared| &declared.name == command.name());
            for emitted in declared
                .into_iter()
                .flat_map(|d| &d.outcomes)
                .flat_map(|o| &o.emits)
            {
                if emitted == self.event {
                    return Some(format!(
                        "arranging its trigger runs `{command}`, which may publish `{}`",
                        self.event
                    ));
                }
                if seen.insert(emitted) {
                    pending.push_back((emitted, false));
                }
            }
        }
        for emitted in &candidate.trigger.published_by.emits {
            pending.push_back((emitted, emitted == self.event));
        }
        while let Some((occurred, known_payload)) = pending.pop_front() {
            for other in self.ir.bindings().values() {
                if other.context.is_some()
                    || other
                        .cause
                        .event()
                        .is_none_or(|cause| cause.name() != occurred.name())
                    || (known_payload && other.name == self.binding.name)
                {
                    continue;
                }
                let ruled_out = known_payload
                    && other.condition.as_ref().is_some_and(|condition| {
                        condition
                            .plan
                            .reads
                            .values()
                            .all(|read| known.contains(&read.members[0]))
                            && condition.plan.evaluate(payload) == Ok(Truth::False)
                    });
                if ruled_out {
                    continue;
                }
                let invoked = self.ir.command(&other.command);
                for emitted in invoked.outcomes.iter().flat_map(|outcome| &outcome.emits) {
                    if emitted == self.event {
                        return Some(format!(
                            "`{}` may publish `{}` again through `{}`",
                            other.name, self.event, invoked.name
                        ));
                    }
                    if seen.insert(emitted) {
                        pending.push_back((emitted, false));
                    }
                }
            }
        }
        None
    }
}

/// Every text value a JSON tree holds, split into identifier tokens.
fn tokens(value: &serde_json::Value, into: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::String(text) => into.extend(
            text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|token| !token.is_empty())
                .map(str::to_owned),
        ),
        serde_json::Value::Array(items) => items.iter().for_each(|item| tokens(item, into)),
        serde_json::Value::Object(entries) => entries.values().for_each(|item| tokens(item, into)),
        _ => {}
    }
}

/// How many times a JSON tree reads `input` as an input field.
fn input_reads(value: &serde_json::Value, input: &str) -> usize {
    match value {
        serde_json::Value::Object(entries) => {
            let here = usize::from(
                matches!(
                    entries.get("kind").and_then(serde_json::Value::as_str),
                    Some("input_field" | "input_or_generated")
                ) && entries.get("field").and_then(serde_json::Value::as_str) == Some(input),
            );
            here + entries
                .values()
                .map(|item| input_reads(item, input))
                .sum::<usize>()
        }
        serde_json::Value::Array(items) => items.iter().map(|item| input_reads(item, input)).sum(),
        _ => 0,
    }
}

/// Varying an input changes only the members the condition reads: no branch guards on it, and
/// the trigger's branch reads it nowhere else.
fn unguarded(trigger: &Trigger<'_>, members: &[Member]) -> Result<(), String> {
    let mut guarded = BTreeSet::new();
    for outcome in &trigger.publisher.outcomes {
        let condition = serde_json::to_value(&outcome.condition).unwrap_or_default();
        tokens(&condition, &mut guarded);
    }
    let branch = serde_json::to_value(trigger.published_by).unwrap_or_default();
    for member in members {
        let Source::Input(input) = &member.source else {
            continue;
        };
        if guarded.contains(input) {
            return Err(format!(
                "`{}` may guard on `{input}`, which fills `event.{}`, so varying it may change \
                 its branch",
                trigger.publisher.name, member.name
            ));
        }
        let copies = members
            .iter()
            .filter(|m| matches!(&m.source, Source::Input(other) if other == input))
            .count();
        if input_reads(&branch, input) != copies {
            return Err(format!(
                "`{}` reads `input.{input}` somewhere besides `event.{}`",
                trigger.published_by.name, member.name
            ));
        }
    }
    Ok(())
}

/// The literal the branch writes into `member` of the event.
fn literal(trigger: &Trigger<'_>, event: &EventHandle, member: &str) -> Option<Node> {
    trigger
        .published_by
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find_map(|field| match &field.value {
            ResolvedPayloadValue::Literal { value } if field.target == member => {
                Some(Node::Text(value.clone()))
            }
            _ => None,
        })
}

/// The trigger's own input value for `name`.
fn sent<'a>(trigger: &'a Trigger<'_>, name: &str) -> Option<&'a ScenarioValue> {
    trigger.run.invoke.iter().find_map(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. }
            if command.name() == &trigger.publisher.name =>
        {
            input.get(name)
        }
        _ => None,
    })
}

fn present_witness(ir: &EssIr, publisher: &ResolvedCommand, input: &str) -> Result<Node, String> {
    let declared = publisher
        .input
        .iter()
        .find(|field| field.name == input)
        .ok_or_else(|| format!("`{}` declares no input `{input}`", publisher.name))?;
    let mut type_ref = declared.type_ref.clone();
    while let ResolvedTypeRef::Optional { of } = type_ref {
        type_ref = *of;
    }
    let field = ResolvedField {
        type_ref,
        ..declared.clone()
    };
    crate::witness::fields(
        ir,
        std::slice::from_ref(&field),
        crate::witness::Distinction::PLAIN,
    )
    .ok()
    .and_then(|mut values| values.remove(input))
    .ok_or_else(|| {
        format!(
            "no present value of `{}.{input}` can be built",
            publisher.name
        )
    })
}

/// What a read path may be set to: the base value, each compared literal or enum variant, and
/// absence of each Optional member on it.
fn choices(read: &ConditionRead, predicate: &Predicate) -> Vec<Choice> {
    let mut options = vec![Choice::Base];
    let mut texts: Vec<String> = Vec::new();
    compared(predicate, &read.path(), &mut texts);
    if let Leaf::Enum { variants } = &read.leaf {
        texts.extend(variants.iter().cloned());
    }
    let mut seen = BTreeSet::new();
    options.extend(
        texts
            .into_iter()
            .filter(|text| seen.insert(text.clone()))
            .map(Choice::Text),
    );
    options.extend(
        read.optional
            .iter()
            .enumerate()
            .filter(|(_, optional)| **optional)
            .map(|(depth, _)| Choice::Absent(depth)),
    );
    options
}

fn compared(predicate: &Predicate, path: &str, into: &mut Vec<String>) {
    match predicate {
        Predicate::Compare {
            left: Operand::Fact(read),
            right: Operand::Literal(ess_primitives::facts::FactValue::Text(text)),
            ..
        } if read.to_string() == path => into.push(text.clone()),
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                compared(child, path, into);
            }
        }
        Predicate::Not(child) => compared(child, path, into),
        _ => {}
    }
}

/// Sets the leaf at `members`, through members that are already present objects.
fn set_leaf(payload: &mut Payload, members: &[String], value: Node) -> bool {
    let (last, parents) = members.split_last().expect("a nonempty path");
    if parents.is_empty() {
        payload.insert(last.clone(), value);
        return true;
    }
    let Some(mut node) = payload.get_mut(&parents[0]) else {
        return false;
    };
    for member in &parents[1..] {
        let Node::Map(fields) = node else {
            return false;
        };
        let Some(next) = fields.get_mut(member) else {
            return false;
        };
        node = next;
    }
    let Node::Map(fields) = node else {
        return false;
    };
    fields.insert(last.clone(), value);
    true
}

/// Leaves the member at `members` absent.
fn clear(payload: &mut Payload, members: &[String]) {
    let (last, parents) = members.split_last().expect("a nonempty path");
    if parents.is_empty() {
        payload.remove(last);
        return;
    }
    let mut node = payload.get_mut(&parents[0]);
    for member in &parents[1..] {
        node = match node {
            Some(Node::Map(fields)) => fields.get_mut(member),
            _ => None,
        };
    }
    if let Some(Node::Map(fields)) = node {
        fields.remove(last);
    }
}

/// The first Optional member along `read` that the payload leaves absent, as its path.
fn first_absent(payload: &Payload, read: &ConditionRead) -> Option<Vec<String>> {
    let mut value = payload.get(&read.members[0]);
    for (depth, optional) in read.optional.iter().enumerate() {
        if depth > 0 {
            value = match value {
                Some(Node::Map(fields)) => fields.get(&read.members[depth]),
                _ => None,
            };
        }
        if *optional && matches!(value, None | Some(Node::Null)) {
            return Some(read.members[..=depth].to_vec());
        }
    }
    None
}

/// `candidate`'s trigger, sending the inputs that publish `payload` for the members the condition
/// reads, with every expectation of those members on the event adjusted to what it now carries.
/// A literal member is never varied: the payload holds the branch's own literal there.
fn varied<'ir>(
    candidate: &Candidate<'ir>,
    event: &EventHandle,
    payload: &Payload,
) -> Result<Trigger<'ir>, BindingGap> {
    let trigger = &candidate.trigger;
    let mut run = trigger.run.clone();
    let publishing = CommandRef::new(trigger.publisher.name.clone());
    for member in &candidate.members {
        let Source::Input(input) = &member.source else {
            continue;
        };
        let value = payload.get(&member.name).cloned();
        let required = trigger
            .publisher
            .input
            .iter()
            .any(|field| &field.name == input && !field.type_ref.is_optional());
        if value.is_none() && required {
            return Err(gap(format!(
                "leaving `event.{}` absent needs the required input `{input}` of `{}` left out",
                member.name, trigger.publisher.name
            )));
        }
        let assign = |map: &mut BTreeMap<String, ScenarioValue>| match &value {
            Some(value) => {
                map.insert(
                    input.clone(),
                    ScenarioValue::Literal {
                        value: value.clone(),
                    },
                );
            }
            None => {
                map.remove(input);
            }
        };
        assign(&mut run.input);
        for step in run.invoke.iter_mut().chain(run.after_steps.iter_mut()) {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } if *command == publishing => {
                    assign(input);
                }
                ScenarioStep::ExpectEvent {
                    event: expected,
                    payload,
                    ..
                }
                | ScenarioStep::EventuallyEvent {
                    event: expected,
                    payload,
                    ..
                } if expected.name() == event.name() && payload.contains_key(&member.name) => {
                    match &value {
                        Some(value) => {
                            payload.insert(member.name.clone(), value.clone());
                        }
                        None => {
                            payload.remove(&member.name);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(Trigger {
        publisher: trigger.publisher,
        published_by: trigger.published_by,
        run,
    })
}
