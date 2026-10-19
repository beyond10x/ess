//! Row sets: branches guarded by the rows a selector selects, and values read from the one row a
//! selector selects (source format `ess/22`, `docs/design/filtered-related-reads.md`;
//! beyond10x/ess#228, beyond10x/ess#299).
//!
//! Every branch such a command decides is witnessed on rows its scenario arranges through the
//! declared creating commands, before the command under test:
//!
//! * one decoy per conjunct of the selector, each refuting that conjunct alone ([`misses`]), so a
//!   target that drops any one conjunct counts a row it must leave out;
//! * the rows the selector selects, as many as the branch needs and — for a `forall` — one of them
//!   refuting the tested predicate where the branch needs it false. The counts are tried in the
//!   order one, two, none, three, so a branch true of a nonempty set is witnessed on one;
//! * where a row is selected, the decoys again after the selected rows, so the selected rows are
//!   neither the oldest nor the newest record of any value they share with a decoy, and a target
//!   keeping one record per such value fails.
//!
//! A command may read several selectors — the two sides of one slot, differing by one conjunct
//! (`docs/design/ordered-related-head.md`, "Several selectors in one command", beyond10x/ess#543).
//! Each is arranged in the order it first appears in the command, its count drawn from
//! [`COUNTS`] for every selector in turn; a decoy of one is never a row another selects, and a
//! selected row's copied value differs from every row arranged for the selectors before it.
//!
//! The rows are arranged before the branch is decided, and the branch the specification takes on
//! them is decided again over the rows the steps actually leave ([`walk`], [`consistent`]): the
//! arrangement is a search, and what it found is checked, never assumed. A value read from the
//! selected row is asserted where exactly one row is selected, and its value differs from every
//! decoy's and from zero, so a target copying the wrong row or nothing fails.
//!
//! The same check runs over every scenario of the suite that sends a row-set command
//! ([`contradictions`]): a family that sends such a command with an arrangement of its own — an
//! existence or a lifecycle scenario, or an arrangement creating a row through it — keeps its
//! scenario only where the rows it leaves decide the branch it requires, and is refused by name
//! otherwise.
//!
//! The rows a selector reads must be the scenario's own: the selector needs a top-level equality
//! between a `String` or `Uuid` field and the input or the addressed subject, or the command is
//! refused as unscoped. A step whose effect on the selector's rows this module does not follow — a
//! set effect, a binding, a direct setup — leaves the decision unknown, and the scenario refused.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue,
    ResolvedPublishedRead, ResolvedRowSelection, ResolvedTypeRef, RowMember, Selected,
};
use ess_domain::command::ordered_head::Ties;
use ess_domain::command::published_operand;
use ess_domain::command::OutcomeName;
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_domain::view::{Direction, Ranking};
use ess_primitives::facts::FactPath;
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Truth};

use super::identity;
use super::ordered_head;
use super::precedence;
use super::set_effects::{input_value, misses, subject_value, written_in};

mod copied;
use super::{
    absorb, arrange_toward_owned, candidates, flatten, instance_name, prepare_in, settled,
    still_unwritten, subject_fact, supply, unwritten_by, ActorRef, Arrangement, ConformanceSuite,
    Decision, Determined, Distinction, EntityRef, InstanceName, Refusal, RefusalCause,
    ScenarioStep, ScenarioValue, Setup, WitnessGap,
};

/// How many rows the selector selects, in the order they are tried.
const COUNTS: [usize; 4] = [1, 2, 0, 3];

/// How many rows a selector whose head the branch reads selects, in the order they are tried
/// (ess/27, beyond10x/ess#543): three, so the head is the middle one inserted, then two.
const HEAD_COUNTS: [usize; 2] = [3, 2];

/// The key a selected row's value is settled under in the arrangement's fields: not a field name
/// — it holds spaces — so nothing reading fields by name finds it.
pub(super) fn key(selection: &ResolvedRowSelection, field: &str) -> String {
    format!(
        "row set {} where {} field {field}",
        selection.entity.name(),
        selection.filter
    )
}

/// Every filtered read of `outcome`, and every read of the head of an ordered selection (ess/27,
/// beyond10x/ess#543), in `sets:`, `payload:` and an error payload, at any depth.
fn reads(outcome: &ResolvedOutcome) -> Vec<&ResolvedPayloadField> {
    fn walk<'a>(field: &'a ResolvedPayloadField, out: &mut Vec<&'a ResolvedPayloadField>) {
        match &field.value {
            ResolvedPayloadValue::RelatedSelection { .. }
            | ResolvedPayloadValue::RelatedHead { .. } => out.push(field),
            ResolvedPayloadValue::Struct { fields } => {
                for leaf in fields {
                    walk(leaf, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    for field in outcome
        .sets
        .iter()
        .chain(&outcome.error_payload)
        .chain(outcome.payload.iter().flat_map(|payload| &payload.fields))
    {
        walk(field, &mut out);
    }
    out
}

/// Whether any branch of the command is guarded by a row set, or reads a selected row.
pub(super) fn uses(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(outcome.condition, ResolvedCondition::RelatedSet { .. })
            || !reads(outcome).is_empty()
    })
}

/// Whether this strategy arranges the scenario for `outcome`: every branch of a row-set command
/// the rows decide — all of them but an input-guarded refusal, which answers before any row is
/// read, and the branches their own families file.
pub(super) fn routes(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    uses(command)
        && rows_decide(command, outcome)
        && outcome.replays.is_none()
        && !(outcome.error.is_some() && matches!(outcome.condition, ResolvedCondition::When { .. }))
        && !matches!(
            outcome.condition,
            ResolvedCondition::External { .. }
                | ResolvedCondition::ExternalWhen { .. }
                | ResolvedCondition::ExistingInstance
                | ResolvedCondition::UnknownInstance
                | ResolvedCondition::WrongState
                | ResolvedCondition::InputAbsent
        )
}

/// Whether the rows a selector selects bear on `outcome` at all: its own guard is a row set, it
/// reads a selected row or the head of an ordered selection, or a branch answered before it under
/// the precedence order is guarded by a row set, so that the rows must pass that branch over.
///
/// A branch none of this holds for is decided without reading a row, so it is witnessed as any
/// other command's branch is, and what the row-set arrangement cannot do for a sibling — a head
/// whose key the target mints, rows a binding changes — does not take it away
/// (<https://github.com/beyond10x/ess/issues/552>).
fn rows_decide(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> bool {
    let guarded =
        |branch: &ResolvedOutcome| matches!(branch.condition, ResolvedCondition::RelatedSet { .. });
    guarded(outcome)
        || !reads(outcome).is_empty()
        || precedence::answers_before(command, outcome)
            .into_iter()
            .any(guarded)
}

/// The refusal for a further witness of a row-set branch, which only its own scenario arranges.
pub(super) fn unarranged() -> RefusalCause {
    gap(
        "a further witness of a row-set branch".into(),
        "arranges no rows of its own: the branch's own scenario witnesses it on the rows its \
         selector selects",
    )
}

/// Why no rows were arranged that take a row-set branch.
pub(super) const ROWS_UNARRANGED: &str = "has no arrangement of rows the declared commands \
     produce on which the rows its selector selects take this branch";

fn gap(path: String, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path,
        type_ref: "row set".into(),
        reason,
    })
}

/// The selector a filtered read or the head of an ordered selection reads, where `value` is one.
fn read_selection(value: &ResolvedPayloadValue) -> Option<&ResolvedRowSelection> {
    match value {
        ResolvedPayloadValue::RelatedSelection { selection, .. }
        | ResolvedPayloadValue::RelatedHead { selection, .. } => Some(selection),
        _ => None,
    }
}

/// The source `outcome`'s payload gives `<event>.<field>`, where it fills that field.
fn source_of<'o>(
    outcome: &'o ResolvedOutcome,
    event: &QualifiedName,
    field: &str,
) -> Option<&'o ResolvedPayloadField> {
    outcome
        .payload
        .iter()
        .filter(|payload| payload.event.name() == event)
        .flat_map(|payload| &payload.fields)
        .find(|source| source.target == field)
}

/// The selectors whose head or one row supply the published values `selection` reads, in any
/// branch of `command` (ess/27, beyond10x/ess#549).
fn producers<'c>(
    command: &'c ResolvedCommand,
    selection: &ResolvedRowSelection,
) -> Vec<&'c ResolvedRowSelection> {
    selection
        .reads
        .iter()
        .flat_map(|read| {
            command.outcomes.iter().filter_map(move |outcome| {
                read_selection(&source_of(outcome, read.event.name(), &read.field)?.value)
            })
        })
        .collect()
}

/// Whether `outcome` publishes every value `selection` reads, so that it reads that selection.
fn publishes(outcome: &ResolvedOutcome, selection: &ResolvedRowSelection) -> bool {
    selection
        .reads
        .iter()
        .all(|read| source_of(outcome, read.event.name(), &read.field).is_some())
}

/// The listed read `path` names, `payload.<event>.<field>[.<member>…]`, by its index in `reads`,
/// and the member path below it; `None` for any other path (ess/27, beyond10x/ess#549).
fn published_route<'p>(
    reads: &[ResolvedPublishedRead],
    path: &'p FactPath,
) -> Option<(usize, &'p [String])> {
    let (root, rest) = path.segments().split_first()?;
    if root != published_operand::ROOT {
        return None;
    }
    reads.iter().enumerate().find_map(|(index, read)| {
        let after = rest.strip_prefix(read.event.name().segments())?;
        let (field, members) = after.split_first()?;
        (*field == read.field).then_some((index, members))
    })
}

/// `filter` with each listed published read `values` holds written in as a literal; a read with
/// no value is kept as written.
fn written_published(
    filter: &Predicate,
    reads: &[ResolvedPublishedRead],
    values: &[Option<Node>],
) -> Predicate {
    written_in(filter, &|path| {
        let (index, members) = published_route(reads, path)?;
        let mut node = values.get(index)?.as_ref()?;
        for member in members {
            let Node::Map(entries) = node else {
                return None;
            };
            node = entries.get(member)?;
        }
        super::fact_value(node)
    })
}

/// Whether `filter` still reads one of the listed published values.
fn reads_published(filter: &Predicate, reads: &[ResolvedPublishedRead]) -> bool {
    filter
        .fact_paths()
        .iter()
        .any(|path| published_route(reads, path).is_some())
}

/// Every selector the command reads, by its guards and its values, each once, in the order they
/// first appear in it (`docs/design/ordered-related-head.md`, "Several selectors in one command",
/// beyond10x/ess#543), a selector reading a published value after the selectors supplying it
/// (beyond10x/ess#549); or a refusal naming the command where it reads none.
fn selections_of(command: &ResolvedCommand) -> Result<Vec<&ResolvedRowSelection>, RefusalCause> {
    let mut found: Vec<&ResolvedRowSelection> = Vec::new();
    for outcome in &command.outcomes {
        if let ResolvedCondition::RelatedSet { selection, .. } = &outcome.condition {
            found.push(selection);
        }
        for read in reads(outcome) {
            if let Some(selection) = read_selection(&read.value) {
                found.push(selection);
            }
        }
    }
    let mut pending: Vec<&ResolvedRowSelection> = Vec::new();
    for selection in found {
        if !pending.contains(&selection) {
            pending.push(selection);
        }
    }
    if pending.is_empty() {
        return Err(gap(
            command.name.to_string(),
            "reads no row set, so no row-set arrangement applies",
        ));
    }
    // Topologically by what each reads, then by first appearance; validation refuses a cycle, and
    // one would be taken in first-appearance order rather than loop.
    let mut out = Vec::with_capacity(pending.len());
    while !pending.is_empty() {
        let ready = pending
            .iter()
            .position(|selection| {
                producers(command, selection)
                    .iter()
                    .all(|producer| producer == selection || !pending.contains(producer))
            })
            .unwrap_or(0);
        out.push(pending.remove(ready));
    }
    Ok(out)
}

/// Whether `type_ref` is a `String` or a `Uuid`, through `Optional` and newtypes.
fn scopable(ir: &EssIr, type_ref: &ResolvedTypeRef) -> bool {
    match type_ref {
        ResolvedTypeRef::Primitive { name } => matches!(name, Primitive::String | Primitive::Uuid),
        ResolvedTypeRef::Optional { of } => scopable(ir, of),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => scopable(ir, of),
            _ => false,
        },
        ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => false,
    }
}

/// Whether the selector selects only rows the scenario's own values name: a top-level equality
/// between a `String` or `Uuid` field of the row and the input or the addressed subject. From
/// `ess/23` a `String` or `Uuid` member of a struct identity counts as such a field
/// (beyond10x/ess#463). From `ess/27` a value the outcome publishes counts as the input does where
/// it is the head or the one row of a selector that is itself scoped, transitively: it then names a
/// row the scenario arranged (beyond10x/ess#549).
fn scoped(ir: &EssIr, command: &ResolvedCommand, selection: &ResolvedRowSelection) -> bool {
    scoped_at(ir, command, selection, 0)
}

/// How deep a chain of published scopes is followed: validation refuses a cycle, so this bounds
/// only a malformed model.
const SCOPE_DEPTH: usize = 16;

fn scoped_at(
    ir: &EssIr,
    command: &ResolvedCommand,
    selection: &ResolvedRowSelection,
    depth: usize,
) -> bool {
    let entity = ir.entity(&selection.entity);
    let conjuncts: Vec<&Predicate> = match &selection.filter {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let members = subject_fact::identity_selectors(ir);
    let field = |path: &FactPath| {
        let [name] = path.segments() else {
            return members
                && path.namespace() == entity.identity.name
                && identity::member_type(ir, &selection.entity, &path.segments().join("."))
                    .is_some_and(|member| scopable(ir, &member));
        };
        std::iter::once(&entity.identity)
            .chain(&entity.fields)
            .any(|held| held.name == *name && scopable(ir, &held.type_ref))
    };
    let named = |path: &FactPath| {
        path.segments().len() > 1
            && [
                ess_domain::command::subject_fact::INPUT_NAMESPACE,
                ess_domain::command::set_effects::SUBJECT_NAMESPACE,
            ]
            .contains(&path.namespace())
    };
    let published = |path: &FactPath| {
        depth < SCOPE_DEPTH
            && published_route(&selection.reads, path).is_some_and(|(index, _)| {
                let read = &selection.reads[index];
                command.outcomes.iter().any(|outcome| {
                    source_of(outcome, read.event.name(), &read.field)
                        .and_then(|source| read_selection(&source.value))
                        .is_some_and(|producer| scoped_at(ir, command, producer, depth + 1))
                })
            })
    };
    let reads_any = |path: &FactPath| named(path) || published(path);
    conjuncts.iter().any(|conjunct| {
        matches!(
            conjunct,
            Predicate::Compare {
                left: Operand::Fact(left),
                op: CompareOp::Eq,
                right: Operand::Fact(right),
                ..
            } if (field(left) && reads_any(right)) || (field(right) && reads_any(left))
        )
    })
}

/// Whether the selector reads a member of a struct identity no declared creation publishes from
/// its input (ess/23, beyond10x/ess#463): every row then carries its identity only as an observed
/// value, and a struct held by reference has no member projection.
fn unprojected_identity(ir: &EssIr, selection: &ResolvedRowSelection) -> bool {
    let identity = &ir.entity(&selection.entity).identity.name;
    subject_fact::identity_selectors(ir)
        && selection.filter.fact_paths().iter().any(|path| {
            path.segments().len() > 1
                && path.namespace() == identity
                && identity::member_type(ir, &selection.entity, &path.segments().join("."))
                    .is_some()
        })
        && !ir.drivers().get(&selection.entity).is_some_and(|drivers| {
            drivers.iter().any(|driver| {
                matches!(driver.effect, ResolvedEffect::Creates)
                    && super::published_identity_input(driver.outcome).is_some()
            })
        })
}

/// Every member path below `at`, a value of type `type_ref`, down to the members that are not
/// structs: `at` itself where it is not one.
fn identity_members(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    at: Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    if let ResolvedTypeRef::Declared { name } = type_ref.required() {
        if let ResolvedBody::Struct { fields, .. } = &ir.named_type(name).body {
            for field in fields {
                let mut member = at.clone();
                member.push(field.name.clone());
                identity_members(ir, &field.type_ref, member, out);
            }
            return;
        }
    }
    out.push(at);
}

/// Whether `filter`, its input and subject written in, pins every member of `entity`'s identity to
/// a literal by a top-level equality: every row it selects is then the one record of that
/// identity (beyond10x/ess#463).
fn pins_identity(ir: &EssIr, entity: &EntityHandle, filter: &Predicate) -> bool {
    let identity = &ir.entity(entity).identity;
    let conjuncts: Vec<&Predicate> = match filter {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let pinned = |member: &[String]| {
        conjuncts.iter().any(|conjunct| match conjunct {
            Predicate::Compare {
                left,
                op: CompareOp::Eq,
                right,
                ..
            } => [(left, right), (right, left)].iter().any(|(fact, value)| {
                matches!(fact, Operand::Fact(path) if path.segments() == member)
                    && matches!(value, Operand::Literal(_))
            }),
            _ => false,
        })
    };
    let mut members = Vec::new();
    identity_members(
        ir,
        &identity.type_ref,
        vec![identity.name.clone()],
        &mut members,
    );
    !members.is_empty() && members.iter().all(|member| pinned(member))
}

/// Whether `other`, read before a branch, answers whatever the input and the rows hold, so that
/// no witness of that branch passes it over: the default. Neither search refutes it — the input
/// search refutes `when:` guards ([`input_for`]) and [`consistent`] row sets — and under the
/// precedence order it is read last, so this answers only for an order the plan's test seam
/// exchanged. Every other condition is refuted by a search, answered by the arrangement
/// (existence, held state, an absent input), or passes the request on (`external:`, whose
/// provider is configured not to take it).
fn unrefutable(other: &ResolvedOutcome) -> bool {
    match &other.condition {
        ResolvedCondition::Otherwise => true,
        ResolvedCondition::SubjectField { .. }
        | ResolvedCondition::SubjectPredicate { .. }
        | ResolvedCondition::Related { .. }
        | ResolvedCondition::RelatedSet { .. }
        | ResolvedCondition::When { .. }
        | ResolvedCondition::SubjectState { .. }
        | ResolvedCondition::StateChange { .. }
        | ResolvedCondition::ExternalWhen { .. }
        | ResolvedCondition::External { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance
        | ResolvedCondition::InputAbsent
        | ResolvedCondition::ExistingInstance => false,
    }
}

/// The `when:` of `other`, where its condition is one.
fn when(other: &ResolvedOutcome) -> Option<&Predicate> {
    match &other.condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        _ => None,
    }
}

/// An input reaching `outcome` as far as its input decides it: its own input guard true, and the
/// `when:` of every branch the precedence plan answers before it false
/// ([`precedence::answers_before`]). Under the precedence order those are every input-guarded
/// refusal and, for an accepting branch, every accepting `when:` branch declared before it; for
/// the default, every other `when:`.
pub(super) fn input_for(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, RefusalCause> {
    let own: Option<&Predicate> = match &outcome.condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        ResolvedCondition::RelatedSet { input, .. } => input.as_ref(),
        _ => None,
    };
    let refute: Vec<&Predicate> = precedence::answers_before(command, outcome)
        .into_iter()
        .filter_map(when)
        .collect();
    let mut guards: Vec<&Predicate> = own.into_iter().collect();
    guards.extend(refute.iter().copied());
    let inputs = candidates(ir, command, &guards, distinction).map_err(RefusalCause::NoWitness)?;
    let tried = inputs.len();
    for input in inputs {
        let facts = flatten(ir, command, &input).map_err(RefusalCause::WitnessRejected)?;
        let holds = own.is_none_or(|guard| facts.decide(guard) == Decision::Satisfied);
        let refuted = refute
            .iter()
            .all(|guard| matches!(facts.decide(guard), Decision::Refuted(_)));
        if holds && refuted {
            return Ok(input);
        }
    }
    Err(RefusalCause::GuardUnsatisfiable {
        predicate: guards
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        tried,
    })
}

// ---- the rows a scenario's steps leave ----------------------------------------------------------

/// One row a scenario's steps created, as far as they determine it.
#[derive(Debug, Clone)]
pub(super) struct Row {
    entity: EntityHandle,
    instance: Option<InstanceName>,
    identity: Option<ScenarioValue>,
    state: StateName,
    settled: BTreeMap<String, Determined>,
    unwritten: BTreeSet<String>,
}

/// Every row a scenario's steps left, and the entities whose rows a step changed in a way not
/// followed here.
#[derive(Debug, Clone, Default)]
pub(super) struct Rows {
    held: Vec<Row>,
    opaque: BTreeSet<EntityHandle>,
    all_opaque: bool,
}

impl Rows {
    fn of(&self, entity: &EntityHandle) -> Result<Vec<&Row>, &'static str> {
        if self.all_opaque || self.opaque.contains(entity) {
            return Err(
                "a step of the scenario changes rows of the selector's entity in a way synthesis \
                 does not follow (a set effect, a binding or a direct setup)",
            );
        }
        Ok(self
            .held
            .iter()
            .filter(|row| row.entity == *entity)
            .collect())
    }

    /// The row a supplied identity names: the arranged instance, or the literal identity.
    fn named(&mut self, entity: &EntityHandle, named: &ScenarioValue) -> Option<&mut Row> {
        self.held.iter_mut().find(|row| {
            row.entity == *entity
                && match named {
                    ScenarioValue::Instance { instance } => row.instance.as_ref() == Some(instance),
                    other => row.identity.as_ref() == Some(other),
                }
        })
    }
}

/// The literal values of a sent input, and of an input naming an arranged instance whose identity
/// the steps named by a literal: the own-identity collision of a re-key (ess/23,
/// beyond10x/ess#429) sends the addressed row's own instance as the new identity.
fn resolved_literals(
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
) -> BTreeMap<String, Node> {
    let mut out = literals(supplied);
    for (field, value) in supplied {
        let ScenarioValue::Instance { instance } = value else {
            continue;
        };
        let named = rows
            .held
            .iter()
            .filter(|row| row.instance.as_ref() == Some(instance))
            .find_map(|row| row.identity.as_ref().and_then(ScenarioValue::as_literal));
        if let Some(identity) = named {
            out.insert(field.clone(), identity.clone());
        }
    }
    out
}

/// The literal values of a sent input, as the predicates over it read them.
fn literals(supplied: &BTreeMap<String, ScenarioValue>) -> BTreeMap<String, Node> {
    supplied
        .iter()
        .filter_map(|(field, value)| {
            value
                .as_literal()
                .map(|value| (field.clone(), value.clone()))
        })
        .collect()
}

/// The supplied identity of the existing subject the command addresses, and its entity.
fn addressed<'a>(
    command: &'a ResolvedCommand,
    supplied: &'a BTreeMap<String, ScenarioValue>,
) -> Option<(&'a EntityHandle, &'a ScenarioValue)> {
    command.outcomes.iter().find_map(|outcome| {
        let subject = outcome.subject.as_ref()?;
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, _) | (_, ResolvedInstance::Observed { .. }) => None,
            (_, ResolvedInstance::Supplied { field }) => supplied
                .get(&field.name)
                .map(|value| (&subject.entity, value)),
        }
    })
}

/// What the rows decide for one send of `command`: the members of each of its selectors, read
/// with the input and subject the send names.
struct Reading<'a> {
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    supplied: BTreeMap<String, ScenarioValue>,
    input: BTreeMap<String, Node>,
    subject: Option<BTreeMap<String, Determined>>,
    rows: &'a Rows,
    /// The branch whose values a selector reading a published value reads (ess/27,
    /// beyond10x/ess#549); `None` where only guards are read.
    outcome: Option<&'a ResolvedOutcome>,
    /// How many published values are being resolved around the current one.
    depth: std::cell::Cell<usize>,
}

impl<'a> Reading<'a> {
    /// The same reading, a selector's published operands read from what `outcome` publishes.
    fn for_outcome(mut self, outcome: &'a ResolvedOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// The value the branch publishes in `read`, computed over the rows as the selector supplying
    /// it reads them: the identity or field of the head or the one row, or a literal input. A value
    /// the target mints, or one the steps leave undetermined, is no literal to scope by.
    fn published_value(&self, read: &ResolvedPublishedRead) -> Result<Node, String> {
        let named = || format!("`{}.{}`", read.event.name(), read.field);
        let outcome = self
            .outcome
            .ok_or_else(|| format!("{} is read where no branch publishes it", named()))?;
        let source = source_of(outcome, read.event.name(), &read.field)
            .ok_or_else(|| format!("{} is not published by `{}`", named(), outcome.name))?;
        if self.depth.get() >= SCOPE_DEPTH {
            return Err(format!(
                "{} reads published values that read each other",
                named()
            ));
        }
        self.depth.set(self.depth.get() + 1);
        let value = match &source.value {
            ResolvedPayloadValue::InputField { field, .. } => self
                .input
                .get(field)
                .cloned()
                .ok_or_else(|| format!("{} copies an input the send leaves out", named())),
            value => match self.read(value) {
                Some(Ok(row)) => read_value(self.ir, &row)
                    .and_then(|value| value.as_literal().cloned())
                    .ok_or_else(|| {
                        format!(
                            "{} is not a literal the steps name: a value the target mints",
                            named()
                        )
                    }),
                Some(Err(why)) => Err(format!("{}: {why}", named())),
                None => Err(format!(
                    "{} has a source synthesis does not follow",
                    named()
                )),
            },
        };
        self.depth.set(self.depth.get() - 1);
        value
    }

    /// `selection`'s filter with the published values it reads and the send's input and subject
    /// written in.
    fn closed_selection(&self, selection: &ResolvedRowSelection) -> Result<Predicate, String> {
        if selection.reads.is_empty() {
            return self.closed(&selection.filter);
        }
        let values = selection
            .reads
            .iter()
            .map(|read| self.published_value(read).map(Some))
            .collect::<Result<Vec<_>, _>>()?;
        self.closed(&written_published(
            &selection.filter,
            &selection.reads,
            &values,
        ))
    }

    fn new(
        ir: &'a EssIr,
        command: &'a ResolvedCommand,
        supplied: &BTreeMap<String, ScenarioValue>,
        rows: &'a Rows,
    ) -> Result<Self, String> {
        // Every selector's rows are followed, or the send is not decided: checked once, here.
        for selection in selections_of(command).map_err(|cause| cause.to_string())? {
            rows.of(&selection.entity)?;
        }
        let subject = addressed(command, supplied).and_then(|(entity, named)| {
            rows.held
                .iter()
                .find(|row| {
                    row.entity == *entity
                        && match named {
                            ScenarioValue::Instance { instance } => {
                                row.instance.as_ref() == Some(instance)
                            }
                            other => row.identity.as_ref() == Some(other),
                        }
                })
                .map(|row| row.settled.clone())
        });
        Ok(Self {
            ir,
            command,
            supplied: supplied.clone(),
            input: if subject_fact::identity_selectors(ir) {
                resolved_literals(supplied, rows)
            } else {
                literals(supplied)
            },
            subject,
            rows,
            outcome: None,
            depth: std::cell::Cell::new(0),
        })
    }

    /// The rows of `selection`'s entity the steps left, each a candidate member.
    fn candidates(&self, selection: &ResolvedRowSelection) -> Result<Vec<&'a Row>, String> {
        self.rows.of(&selection.entity).map_err(str::to_owned)
    }

    /// `predicate` with the subject's values written in, where it reads any.
    fn closed(&self, predicate: &Predicate) -> Result<Predicate, String> {
        let empty = BTreeMap::new();
        let subject = self.subject.as_ref().unwrap_or(&empty);
        // The input the send names is written in where it is literal, as the arrangement reads it: an
        // input naming an arranged instance — the addressed row of an `updates:` (ess/23,
        // beyond10x/ess#429) — leaves the rest of the input unflattened, and the literal it compares
        // with would read nothing. From `ess/23` only: an older document keeps the suite it had.
        let literal_input = subject_fact::identity_selectors(self.ir);
        let written = written_in(predicate, &|path| {
            subject_value(path, subject).or_else(|| {
                literal_input
                    .then(|| input_value(path, &self.input))
                    .flatten()
            })
        });
        if written.fact_paths().iter().any(|path| {
            path.segments().len() > 1
                && path.namespace() == ess_domain::command::set_effects::SUBJECT_NAMESPACE
        }) {
            return Err(format!(
                "`{predicate}` reads a subject field the steps leave undetermined"
            ));
        }
        Ok(written)
    }

    fn truth(&self, row: &Row, predicate: &Predicate) -> Truth {
        // The row's own identity, where the steps named it by a literal: a selector over the
        // identity (ess/23, beyond10x/ess#429) reads the key the row is stored under.
        // Only where the predicate reads the identity: every other predicate binds what it bound.
        // A member of a struct identity (beyond10x/ess#463) reads the same literal, member by member.
        let identity = &self.ir.entity(&row.entity).identity;
        let reads_identity = subject_fact::identity_selectors(self.ir)
            && subject_fact::reads_key(predicate, &identity.name);
        let mut settled = std::borrow::Cow::Borrowed(&row.settled);
        if let Some(value) = row
            .identity
            .as_ref()
            .filter(|value| reads_identity && value.as_literal().is_some())
        {
            settled
                .to_mut()
                .entry(identity.name.clone())
                .or_insert_with(|| Determined {
                    value: value.clone(),
                    type_ref: identity.type_ref.clone(),
                });
        }
        let predicate = instances_decided(predicate, &settled, &self.supplied);
        subject_fact::row_truth_with(
            self.ir,
            &row.entity,
            &settled,
            &row.unwritten,
            Some(&row.state),
            &predicate,
            Some((self.command, &self.input)),
        )
    }

    fn members(
        &self,
        selection: &ResolvedRowSelection,
        satisfies: Option<&Predicate>,
    ) -> Result<Vec<RowMember>, String> {
        let filter = self.closed_selection(selection)?;
        let satisfies = satisfies
            .map(|predicate| self.closed(predicate))
            .transpose()?;
        Ok(self
            .candidates(selection)?
            .iter()
            .map(|row| {
                let selected = self.truth(row, &filter);
                RowMember {
                    selected,
                    satisfies: match (&satisfies, selected) {
                        (Some(predicate), Truth::True | Truth::Unknown) => {
                            self.truth(row, predicate)
                        }
                        _ => Truth::True,
                    },
                }
            })
            .collect())
    }

    /// Whether `outcome`'s row-set guard and input guard select it, where it has one.
    fn takes(&self, outcome: &ResolvedOutcome) -> Result<Option<Truth>, String> {
        let ResolvedCondition::RelatedSet {
            selection,
            test,
            input,
        } = &outcome.condition
        else {
            return Ok(None);
        };
        let tested = test.decide(&self.members(selection, test.predicate())?);
        let guarded = match input {
            None => Truth::True,
            Some(guard) => {
                let facts = crate::input::replay_facts(self.ir, self.command, &self.supplied)
                    .map_err(|error| error.to_string())?;
                match facts.decide(guard) {
                    Decision::Satisfied => Truth::True,
                    Decision::Refuted(_) => Truth::False,
                    Decision::Unevaluable(_) => Truth::Unknown,
                }
            }
        };
        Ok(Some(tested.and(guarded)))
    }

    /// The one row `selection` selects, or why there is none to read.
    fn one(&self, selection: &ResolvedRowSelection) -> Result<&'a Row, String> {
        let members = self.members(selection, None)?;
        match Selected::of(&members) {
            Selected::One(index) => Ok(self.candidates(selection)?[index]),
            Selected::None => Err("no row is selected, and a read needs one".into()),
            Selected::Several => Err("several rows are selected, and a read needs one".into()),
            Selected::Unknown => Err("which row is selected is not decided".into()),
        }
    }

    /// The first row `selection` selects by `order_by` (ess/27, beyond10x/ess#543), or why there is
    /// none to read: no row selected, a row whose membership is unknown, or a ranking the rows'
    /// known values leave undecided. Under `ties: unordered` no two selected rows may tie on every
    /// key, so a scenario never states an order the model leaves to the implementation.
    fn head(
        &self,
        selection: &ResolvedRowSelection,
        order_by: &[Ranking],
        ties: Ties,
    ) -> Result<&'a Row, String> {
        let members = self.members(selection, None)?;
        if members
            .iter()
            .any(|member| member.selected == Truth::Unknown)
        {
            return Err("which rows are selected is not decided".into());
        }
        let selected: Vec<&'a Row> = self
            .candidates(selection)?
            .into_iter()
            .zip(&members)
            .filter(|(_, member)| member.selected == Truth::True)
            .map(|(row, _)| row)
            .collect();
        if selected.is_empty() {
            return Err("no row is selected, and a head needs one".into());
        }
        let value = |row: &&'a Row, name: &str| row_key(self.ir, row, name);
        if ties == Ties::Unordered
            && ordered_head::tied(self.ir, &selection.entity, order_by, &value, &selected)
        {
            return Err(
                "two selected rows tie on every key, and the order leaves ties unordered".into(),
            );
        }
        ordered_head::first(self.ir, &selection.entity, order_by, &value, &selected)
            .map(|at| selected[at])
            .ok_or_else(|| "which selected row ranks first is not decided".into())
    }

    /// The row a filtered read or a head reads, the field it reads, that field's type, and the key
    /// its value is settled under; or why there is no row to read.
    fn read(&self, value: &'a ResolvedPayloadValue) -> Option<Result<ReadRow<'a>, String>> {
        match value {
            ResolvedPayloadValue::RelatedSelection {
                selection,
                field,
                type_ref,
            } => Some(self.one(selection).map(|row| ReadRow {
                row,
                field,
                type_ref,
                settled_as: key(selection, field),
            })),
            ResolvedPayloadValue::RelatedHead {
                selection,
                order_by,
                ties,
                field,
                type_ref,
            } => Some(self.head(selection, order_by, *ties).map(|row| ReadRow {
                row,
                field,
                type_ref,
                settled_as: ordered_head::key(selection, order_by, field),
            })),
            _ => None,
        }
    }
}

/// The row a filtered read or a head reads, the field it reads, that field's type, and the key its
/// value is settled under in the arrangement's fields.
struct ReadRow<'a> {
    row: &'a Row,
    field: &'a str,
    type_ref: &'a ResolvedTypeRef,
    settled_as: String,
}

/// The value the walked `row` holds for the ranking key `name`: its identity where the steps named
/// it by a literal, a stored field the steps settled, or its held state.
fn row_key(ir: &EssIr, row: &Row, name: &str) -> Option<Node> {
    if ordered_head::is_identity(ir, &row.entity, name) {
        return row.identity.as_ref()?.as_literal().cloned();
    }
    if name == ess_domain::entity::EntitySpec::STATE
        && !ir
            .entity(&row.entity)
            .fields
            .iter()
            .any(|field| field.name == name)
    {
        return Some(Node::Text(row.state.to_string()));
    }
    row.settled.get(name)?.value.as_literal().cloned()
}

/// Whether `expected` is the branch the rows decide for this send of `command`, as far as the rows
/// decide it: its row set holds, every row-set branch answered before it does not, and every value
/// it reads from a selected row has exactly one to read. What the input alone decides is the input
/// search's, and what existence and held state decide is the arrangement's.
fn consistent(
    ir: &EssIr,
    command: &ResolvedCommand,
    expected: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
) -> Result<(), String> {
    if matches!(
        expected.condition,
        ResolvedCondition::ExistingInstance
            | ResolvedCondition::UnknownInstance
            | ResolvedCondition::WrongState
            | ResolvedCondition::InputAbsent
    ) || (expected.error.is_some()
        && matches!(expected.condition, ResolvedCondition::When { .. }))
    {
        return Ok(());
    }
    // The command's own identity is read before any row set: a creation sent with the identity of
    // a row the steps left is `existing_instance:`'s, never this branch's. From `ess/23` whatever
    // the creator declares: a target refuses a second record under one identity, so no scenario
    // expects this branch to create one a row already carries (beyond10x/ess#463).
    if subject_fact::identity_selectors(ir)
        || command
            .outcomes
            .iter()
            .any(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
    {
        if let Some(subject) = expected
            .subject
            .as_ref()
            .filter(|subject| subject.effect == ResolvedEffect::Creates)
        {
            if let Some(identity) = created_identity(expected, supplied) {
                let by_instance = subject_fact::identity_selectors(ir);
                if rows.held.iter().any(|row| {
                    row.entity == subject.entity
                        && (row.identity.as_ref() == Some(&identity)
                            || matches!(&identity, ScenarioValue::Instance { instance }
                                if by_instance && row.instance.as_ref() == Some(instance)))
                }) {
                    return Err("the creation names an identity a row already carries".into());
                }
            }
        }
    }
    // A branch no row bears on is decided without reading one (beyond10x/ess#552).
    if !rows_decide(command, expected) {
        return Ok(());
    }
    let reading = Reading::new(ir, command, supplied, rows)?.for_outcome(expected);
    // Which row-set branches answer before `expected` is the precedence plan's: under the
    // precedence order the row-set refusals in declaration order, then the accepting branches in
    // declaration order, then the default. Every row set is still read, in declaration order, so
    // a row set the rows cannot decide is the error it always was.
    let before: BTreeSet<&OutcomeName> = precedence::answers_before(command, expected)
        .into_iter()
        .map(|other| &other.name)
        .collect();
    for other in &command.outcomes {
        if before.contains(&other.name) && unrefutable(other) {
            return Err(format!(
                "`{}` is not decidedly passed over: it answers every request it is read for",
                other.name
            ));
        }
        let Some(taken) = reading.takes(other)? else {
            continue;
        };
        if before.contains(&other.name) && taken != Truth::False {
            return Err(format!(
                "`{}` is not decidedly passed over: its row set is {taken:?}",
                other.name
            ));
        }
    }
    if let Some(taken) = reading.takes(expected)? {
        if taken != Truth::True {
            return Err(format!("its own row set is {taken:?}"));
        }
    }
    for read in reads(expected) {
        if let Some(read) = reading.read(&read.value) {
            read?;
        }
    }
    Ok(())
}

/// Whether the creation `outcome` names a literal identity a row the steps left already carries,
/// where a row-set selector reads that entity's identity (ess/23, beyond10x/ess#463): such a
/// selector counts records by their key, and a target refuses a second record under one key
/// whatever the creator declares. Every other creation, and every older document, is left as it
/// was.
fn recreates(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
) -> Result<(), String> {
    if !subject_fact::identity_selectors(ir) || outcome.error.is_some() {
        return Ok(());
    }
    let Some(subject) = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)
    else {
        return Ok(());
    };
    let identity = &ir.entity(&subject.entity).identity.name;
    let keyed = ir
        .commands()
        .values()
        .filter(|command| uses(command))
        .filter_map(|command| selections_of(command).ok())
        .flatten()
        .any(|selection| {
            selection.entity == subject.entity
                && subject_fact::reads_key(&selection.filter, identity)
        });
    let Some(created) =
        created_identity(outcome, supplied).filter(|created| created.as_literal().is_some())
    else {
        return Ok(());
    };
    if keyed
        && rows
            .held
            .iter()
            .any(|row| row.entity == subject.entity && row.identity.as_ref() == Some(&created))
    {
        return Err("creates an identity a row the steps left already carries".into());
    }
    Ok(())
}

/// The identity a creation names, where the input supplies it.
fn created_identity(
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Option<ScenarioValue> {
    let subject = outcome.subject.as_ref()?;
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        return None;
    };
    let source = outcome
        .payload
        .iter()
        .filter(|payload| payload.event == *event)
        .flat_map(|payload| &payload.fields)
        .find(|source| source.target == field.name)?;
    match &source.value {
        ResolvedPayloadValue::InputField { field, .. } => supplied.get(field).cloned(),
        _ => None,
    }
}

/// The values `outcome`'s filtered reads take on `rows`, under their settled keys.
fn selected_values(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    rows: &Rows,
) -> BTreeMap<String, Determined> {
    let mut out = BTreeMap::new();
    let Ok(reading) = Reading::new(ir, command, supplied, rows) else {
        return out;
    };
    let reading = reading.for_outcome(outcome);
    for read in reads(outcome) {
        let Some(Ok(row)) = reading.read(&read.value) else {
            continue;
        };
        if let Some(value) = read_value(ir, &row) {
            out.insert(
                row.settled_as,
                Determined {
                    value,
                    type_ref: row.type_ref.clone(),
                },
            );
        }
    }
    out
}

/// The value a filtered read or a head takes from the row it reads: the identity the steps named,
/// a field they settled, or null for a field the creation left unwritten.
fn read_value(ir: &EssIr, read: &ReadRow<'_>) -> Option<ScenarioValue> {
    let row = read.row;
    let identity = &ir.entity(&row.entity).identity.name;
    if read.field == identity.as_str() {
        row.identity.clone()
    } else if let Some(held) = row.settled.get(read.field) {
        Some(held.value.clone())
    } else if row.unwritten.contains(read.field) {
        Some(ScenarioValue::Literal { value: Node::Null })
    } else {
        None
    }
}

/// The rows `steps` leave, deciding again every send of a row-set command among them where `check`
/// is set: the first whose required branch the rows do not decide is the error.
pub(super) fn walk(ir: &EssIr, steps: &[ScenarioStep], check: bool) -> Result<Rows, String> {
    let mut rows = Rows {
        all_opaque: !ir.bindings().is_empty(),
        ..Rows::default()
    };
    for (at, step) in steps.iter().enumerate() {
        match step {
            ScenarioStep::ExecuteCommand {
                command: invoked,
                input,
                ..
            } => {
                let Some(command) = ir.commands().get(invoked.name()) else {
                    rows.all_opaque = true;
                    continue;
                };
                let expected = match steps.get(at + 1) {
                    Some(ScenarioStep::ExpectOutcome { outcome }) => command
                        .outcomes
                        .iter()
                        .find(|candidate| candidate.name == outcome.outcome),
                    _ => None,
                };
                let Some(expected) = expected else {
                    // An answer the scenario does not require: what it changed is not followed.
                    if command
                        .outcomes
                        .iter()
                        .any(|outcome| outcome.error.is_none())
                    {
                        rows.all_opaque = true;
                    }
                    continue;
                };
                if check {
                    recreates(ir, expected, input, &rows)
                        .map_err(|why| format!("`{}/{}` {why}", command.name, expected.name))?;
                }
                if check && uses(command) {
                    consistent(ir, command, expected, input, &rows)
                        .map_err(|why| format!("`{}/{}` {why}", command.name, expected.name))?;
                }
                let captured = steps[at + 2..]
                    .iter()
                    .take_while(|step| !matches!(step, ScenarioStep::ExecuteCommand { .. }))
                    .find_map(|step| match step {
                        ScenarioStep::CaptureInstance {
                            instance, entity, ..
                        } => Some((instance.clone(), entity.clone())),
                        _ => None,
                    });
                apply(ir, command, expected, input, captured, &mut rows);
            }
            ScenarioStep::EstablishEntity { .. }
            | ScenarioStep::DeliverEvent { .. }
            | ScenarioStep::RedeliverEvent { .. } => rows.all_opaque = true,
            _ => {}
        }
    }
    Ok(rows)
}

/// What one taken branch does to the rows.
fn apply(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    supplied: &BTreeMap<String, ScenarioValue>,
    captured: Option<(InstanceName, EntityRef)>,
    rows: &mut Rows,
) {
    if outcome.error.is_some() {
        return;
    }
    for affect in &outcome.affects {
        rows.opaque.insert(affect.entity.clone());
    }
    if let Some(set) = &outcome.instances {
        rows.opaque.insert(set.entity.clone());
    }
    let Some(subject) = &outcome.subject else {
        return;
    };
    let entity = &subject.entity;
    match &subject.effect {
        ResolvedEffect::Creates => {
            let mut before = if uses(command) {
                selected_values(ir, command, outcome, supplied, rows)
            } else {
                BTreeMap::new()
            };
            before.extend(copied::values(
                ir,
                outcome,
                supplied,
                &BTreeMap::new(),
                rows,
            ));
            let settled = settled(ir, outcome, supplied, &before);
            let instance = captured
                .filter(|(_, captured)| *captured == EntityRef::from(entity))
                .map(|(instance, _)| instance);
            rows.held.push(Row {
                entity: entity.clone(),
                instance,
                identity: created_identity(outcome, supplied),
                state: subject
                    .into
                    .clone()
                    .unwrap_or_else(|| ir.entity(entity).lifecycle.initial.clone()),
                settled,
                unwritten: unwritten_by(ir, entity, outcome),
            });
        }
        ResolvedEffect::Moves { .. } | ResolvedEffect::Updates | ResolvedEffect::Preserves => {
            let ResolvedInstance::Supplied { field } = &subject.instance else {
                rows.opaque.insert(entity.clone());
                return;
            };
            let Some(named) = supplied.get(&field.name).cloned() else {
                rows.opaque.insert(entity.clone());
                return;
            };
            let mut before = rows
                .named(entity, &named)
                .map(|row| row.settled.clone())
                .unwrap_or_default();
            let read = copied::values(ir, outcome, supplied, &before, rows);
            before.extend(read);
            let determined = settled(ir, outcome, supplied, &before);
            let Some(row) = rows.named(entity, &named) else {
                rows.opaque.insert(entity.clone());
                return;
            };
            absorb(&mut row.settled, outcome, determined);
            row.unwritten = still_unwritten(&row.unwritten, outcome);
            // A re-key (ess/23, beyond10x/ess#429): the row answers to the identity written, and
            // the instance name the arrangement captured names the identity it left.
            if let Some(write) = outcome.identity_write(ir) {
                row.identity = row
                    .settled
                    .get(&write.target)
                    .map(|held| held.value.clone());
                row.instance = None;
            }
            if let ResolvedEffect::Moves { transition } = &subject.effect {
                row.state = transition.to.clone();
            }
        }
        ResolvedEffect::Deletes => {
            let ResolvedInstance::Supplied { field } = &subject.instance else {
                rows.opaque.insert(entity.clone());
                return;
            };
            let Some(named) = supplied.get(&field.name).cloned() else {
                rows.opaque.insert(entity.clone());
                return;
            };
            let before = rows.held.len();
            rows.held.retain(|row| {
                !(row.entity == *entity
                    && match &named {
                        ScenarioValue::Instance { instance } => {
                            row.instance.as_ref() == Some(instance)
                        }
                        other => row.identity.as_ref() == Some(other),
                    })
            });
            if rows.held.len() == before {
                rows.opaque.insert(entity.clone());
            }
        }
    }
}

// ---- the scenario's own arrangement -------------------------------------------------------------

/// Every instance name `steps` capture.
fn captured(steps: &[ScenarioStep]) -> BTreeSet<InstanceName> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CaptureInstance { instance, .. } => Some(instance.clone()),
            _ => None,
        })
        .collect()
}

/// The lowest further distinction whose instance name nothing in the scenario holds yet, claimed.
fn next(ir: &EssIr, entity: &EntityHandle, taken: &mut BTreeSet<InstanceName>) -> Distinction {
    let name = &ir.entity(entity).name;
    (1..=taken.len() + 64)
        .map(Distinction::further)
        .find(|distinction| taken.insert(instance_name(name, *distinction)))
        .unwrap_or(Distinction::UNKNOWN)
}

/// The value `row` holds for `field`, where it is a literal.
fn held<'r>(row: &'r Arrangement, field: &str) -> Option<&'r Node> {
    row.settled
        .get(field)
        .and_then(|held| held.value.as_literal())
}

/// Whether `value` is the zero of its kind: the value a target copying nothing would write.
fn zero(value: &Node) -> bool {
    match value {
        Node::Number(number) => number
            .exact_text()
            .trim_start_matches('-')
            .chars()
            .all(|digit| matches!(digit, '0' | '.')),
        Node::Text(text) => text.is_empty(),
        Node::Bool(flag) => !flag,
        Node::Null => true,
        _ => false,
    }
}

/// The other selectors of the command, and the rows the scenario already arranged for them: a
/// decoy of this selector is a row none of `filters` selects, and a selected row's copied value
/// differs from every row of `arranged` (beyond10x/ess#543).
#[derive(Clone, Copy, Default)]
struct Siblings<'s> {
    filters: &'s [Predicate],
    arranged: &'s [Arrangement],
}

/// The rows one attempt arranges: a decoy per conjunct of `filter`, then `count` rows it selects,
/// the first of them refuting `foralls[violate]` where `violate` names one, then — where `count`
/// is not zero — a decoy per conjunct again. Each selected row's value of every field in `copied`
/// differs from zero, from every decoy's and from every row `siblings` arranged; no decoy is a row
/// a sibling selector selects.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn arrange_rows(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
    foralls: &[Predicate],
    (count, violate): (usize, Option<usize>),
    copied: &[&str],
    siblings: Siblings<'_>,
    head: &mut Head<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    (before, owner): (&[ScenarioStep], Option<&InstanceName>),
) -> Option<Vec<Arrangement>> {
    // Whether the steps so far, and `row`'s after them, leave every row-set command they send
    // taking the branch it is required to: a row arranged through such a command is kept only
    // where the rows before it decide that command's branch.
    let follows = |arranged: &[Arrangement], row: &Arrangement| {
        let mut steps = before.to_vec();
        for earlier in arranged {
            steps.extend(earlier.steps.iter().cloned());
        }
        steps.extend(row.steps.iter().cloned());
        walk(ir, &steps, true).is_ok()
    };
    let truth = |row: &Arrangement, predicate: &Predicate| {
        subject_fact::row_truth_with(
            ir,
            entity,
            &row.settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            None,
        )
    };
    let conjuncts: Vec<Predicate> = match filter {
        Predicate::All(children) if children.len() > 1 => children.clone(),
        other => vec![other.clone()],
    };
    // Where the selector pins every member of the identity, a decoy refuting any other conjunct
    // carries the selected row's identity: it is that same record, so a branch needing a selected
    // row arranges none, and the empty branch arranges it (ess/23, beyond10x/ess#463).
    let identity = &ir.entity(entity).identity.name;
    let one_record =
        count > 0 && subject_fact::identity_selectors(ir) && pins_identity(ir, entity, filter);
    // One decoy per conjunct, refuting that conjunct alone, each holding a copied value no row of
    // `selected` holds.
    let place_decoys = |arranged: &mut Vec<Arrangement>,
                        taken: &mut BTreeSet<InstanceName>,
                        selected: &[Arrangement]| {
        for (missed, miss) in misses(filter).into_iter().enumerate() {
            if one_record && !subject_fact::reads_key(&conjuncts[missed], identity) {
                continue;
            }
            // Decidedly not selected, by this conjunct: it is false, and no other is. A conjunct
            // that reads what the missed one says is absent stays unknown, and is no other's
            // refutation.
            // Nor is it a row another selector of the command selects: there it would be that
            // selector's row, not a decoy (two sides of one slot differ by one conjunct).
            let accept = |row: &Arrangement| {
                truth(row, filter) == Truth::False
                    && conjuncts.iter().enumerate().all(|(at, conjunct)| {
                        (truth(row, conjunct) == Truth::False) == (at == missed)
                    })
                    && siblings
                        .filters
                        .iter()
                        .all(|other| truth(row, other) == Truth::False)
                    && copied.iter().all(|field| {
                        selected
                            .iter()
                            .all(|chosen| held(row, field) != held(chosen, field))
                    })
            };
            let distinction = next(ir, entity, taken);
            // A decoy no creating command arranges with the rows before it decided is left out:
            // it would catch a target ignoring that conjunct, and the scenario stands without it.
            if let Some(row) = arrange_row(ir, entity, &miss, actors, (distinction, owner), &accept)
            {
                if follows(arranged, &row) {
                    arranged.push(row);
                }
            }
        }
    };
    let mut arranged: Vec<Arrangement> = Vec::new();
    place_decoys(&mut arranged, taken, &[]);
    let decoys = arranged.clone();
    let goal_for = |nth: usize| {
        let mut goal = vec![filter.clone()];
        for (index, forall) in foralls.iter().enumerate() {
            goal.push(if nth == 0 && violate == Some(index) {
                Predicate::Not(Box::new(forall.clone()))
            } else {
                forall.clone()
            });
        }
        Predicate::All(goal)
    };
    // Every copied value of `row` differs from zero, from every decoy's, from every row a sibling
    // selector arranged, and from every row of `chosen`.
    // A head read (ess/27) also reads the identity the arrangement sent, which no settled field
    // holds; a filtered read keeps reading what it read.
    let order_by = head.order_by;
    let copy = |row: &Arrangement, field: &str| {
        if order_by.is_some() {
            arranged_key(ir, entity, row, field)
        } else {
            held(row, field).cloned()
        }
    };
    let fresh = |row: &Arrangement, chosen: &[Arrangement]| {
        copied.iter().all(|field| {
            copy(row, field).is_some_and(|value| {
                !zero(&value)
                    && decoys
                        .iter()
                        .chain(siblings.arranged)
                        .chain(chosen)
                        .all(|decoy| copy(decoy, field).as_ref() != Some(&value))
            })
        })
    };
    if let Some(order_by) = order_by.filter(|_| count >= 2) {
        let placed = |rows: &[Arrangement]| {
            let mut steps = before.to_vec();
            for row in decoys.iter().chain(rows) {
                steps.extend(row.steps.iter().cloned());
            }
            walk(ir, &steps, true).is_ok()
        };
        let holds = |row: &Arrangement, goal: &Predicate| truth(row, goal) == Truth::True;
        arranged.extend(arrange_head(
            ir,
            entity,
            order_by,
            count,
            (&goal_for, &holds, &fresh, &placed),
            (actors, owner),
            taken,
            &mut *head.gaps,
        )?);
    } else {
        for nth in 0..count {
            let goal = goal_for(nth);
            let accept = |row: &Arrangement| truth(row, &goal) == Truth::True && fresh(row, &[]);
            let distinction = next(ir, entity, taken);
            let row = arrange_row(ir, entity, &goal, actors, (distinction, owner), &accept)?;
            if !follows(&arranged, &row) {
                return None;
            }
            arranged.push(row);
        }
    }
    // The decoys again after the selected rows. A decoy holds every value the selector reads but
    // the one its conjunct refutes, so the selected rows are then neither the oldest nor the newest
    // record of any value they share with one, and a target keeping one record per value — the
    // newest or the oldest replacing the rest — answers from a decoy.
    if count > 0 {
        let selected = arranged[decoys.len()..].to_vec();
        place_decoys(&mut arranged, taken, &selected);
    }
    Some(arranged)
}

/// The order of the head a branch reads from a selector, where it reads one (ess/27,
/// beyond10x/ess#543), and where the witness gaps the arrangement could not close are recorded.
struct Head<'h> {
    order_by: Option<&'h [Ranking]>,
    gaps: &'h mut Vec<RefusalCause>,
}

/// What [`arrange_head`] reads of its caller: the goal of the `nth` selected row, whether a row
/// satisfies a goal, whether a row's copied values are fresh beside some chosen rows, and whether a
/// sequence of rows placed after the decoys leaves every row-set command it sends decided.
type HeadChecks<'c> = (
    &'c dyn Fn(usize) -> Predicate,
    &'c dyn Fn(&Arrangement, &Predicate) -> bool,
    &'c dyn Fn(&Arrangement, &[Arrangement]) -> bool,
    &'c dyn Fn(&[Arrangement]) -> bool,
);

/// The value `row` holds for the ranking key `name`: its identity where the arrangement sent it
/// as a literal input, its held state, or a stored field it settled.
fn arranged_key(ir: &EssIr, entity: &EntityHandle, row: &Arrangement, name: &str) -> Option<Node> {
    if ordered_head::is_identity(ir, entity, name) {
        if let Some(held) = held(row, name) {
            return Some(held.clone());
        }
        return row.steps.iter().find_map(|step| {
            let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                return None;
            };
            let command = ir.commands().get(command.name())?;
            command.outcomes.iter().find_map(|outcome| {
                let creates = outcome.subject.as_ref().is_some_and(|subject| {
                    subject.entity == *entity && subject.effect == ResolvedEffect::Creates
                });
                let field = super::published_identity_input(outcome).filter(|_| creates)?;
                input.get(field)?.as_literal().cloned()
            })
        });
    }
    if name == ess_domain::entity::EntitySpec::STATE
        && !ir
            .entity(entity)
            .fields
            .iter()
            .any(|field| field.name == name)
    {
        return Some(Node::Text(row.state.to_string()));
    }
    held(row, name).cloned()
}

/// `field` compared with `value` by `op`, as a predicate over the candidate row.
fn compared(field: &str, op: CompareOp, value: &Node) -> Option<Predicate> {
    Some(Predicate::Compare {
        left: Operand::Fact(FactPath::from_segments([field.to_owned()])),
        op,
        right: Operand::Literal(super::fact_value(value)?),
        kind: ess_primitives::predicate::CompareKind::Value,
    })
}

/// The selected rows of a branch reading the head by `order_by` (`docs/design/ordered-related-head.md`,
/// "Synthesis"): S1, S2 and S3 (or S1 and S2 for `count` two) created in that order, the first key
/// steered so that S2 ranks first, S1 second and S3 last; then, for each key after the first, a
/// tie row created before S2 that agrees with S2 on every key before it and loses on that one. A
/// tie no creating input steers is left out and recorded in `gaps`, and the scenario stands.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn arrange_head(
    ir: &EssIr,
    entity: &EntityHandle,
    order_by: &[Ranking],
    count: usize,
    (goal_for, holds, fresh, placed): HeadChecks<'_>,
    (actors, owner): (&BTreeMap<QualifiedName, ActorRef>, Option<&InstanceName>),
    taken: &mut BTreeSet<InstanceName>,
    gaps: &mut Vec<RefusalCause>,
) -> Option<Vec<Arrangement>> {
    use std::cmp::Ordering;
    let value = |row: &Arrangement, name: &str| arranged_key(ir, entity, row, name);
    let on = |ranking: &Ranking, left: &Arrangement, right: &Arrangement| {
        ordered_head::on(
            ir,
            entity,
            ranking,
            value(left, &ranking.field).as_ref(),
            value(right, &ranking.field).as_ref(),
        )
    };
    let first = &order_by[0];
    let by_identity = ordered_head::is_identity(ir, entity, &first.field);
    let last = order_by.len() - 1;
    let tie_breaks_up = order_by[last].direction == Direction::Ascending;
    // Distinctions, claimed in ascending order and dealt so that the identities a further
    // distinction moves rank as the arrangement needs them: by identity alone, S2 first; with a key
    // before the identity, the tie row T loses to S2 on it. `accept` checks; the dealing proposes.
    let pool: Vec<Distinction> = (0..=count).map(|_| next(ir, entity, taken)).collect();
    let (rows_at, tie_at): (Vec<Distinction>, Distinction) = if by_identity {
        let mut ranked: Vec<Distinction> = pool[..count].to_vec();
        if first.direction == Direction::Descending {
            ranked.reverse();
        }
        // S1 second, S2 first, S3 third.
        let mut dealt = vec![ranked[1], ranked[0]];
        dealt.extend(ranked.get(2).copied());
        (dealt, pool[count])
    } else if tie_breaks_up {
        (pool[..count].to_vec(), pool[count])
    } else {
        (pool[1..].to_vec(), pool[0])
    };
    let (better, worse) = if first.direction == Direction::Descending {
        (CompareOp::Gt, CompareOp::Lt)
    } else {
        (CompareOp::Lt, CompareOp::Gt)
    };
    let mut selected: Vec<Arrangement> = Vec::new();
    for (nth, distinction) in rows_at.iter().enumerate() {
        let mut goal = goal_for(nth);
        let wanted = match nth {
            0 => None,
            1 => Some(Ordering::Less),
            _ => Some(Ordering::Greater),
        };
        if let (Some(wanted), false) = (wanted, by_identity) {
            let op = if wanted == Ordering::Less {
                better
            } else {
                worse
            };
            if let Some(steer) =
                value(&selected[0], &first.field).and_then(|held| compared(&first.field, op, &held))
            {
                goal = Predicate::All(vec![goal, steer]);
            }
        }
        let accept = |row: &Arrangement| {
            holds(row, &goal)
                && fresh(row, &selected)
                && wanted.is_none_or(|wanted| on(first, row, &selected[0]) == Some(wanted))
                && (nth < 2 || on(first, row, &selected[1]) == Some(Ordering::Greater))
        };
        selected.push(arrange_row(
            ir,
            entity,
            &goal,
            actors,
            (*distinction, owner),
            &accept,
        )?);
    }
    if !placed(&selected) {
        return None;
    }
    // The tie rows, each created before S2.
    let mut ties: Vec<Arrangement> = Vec::new();
    for at in 1..order_by.len() {
        if by_identity {
            break;
        }
        let ranking = &order_by[at];
        // The last key of a total order; never one under `ties: unordered`, which holds none.
        let identity = ordered_head::is_identity(ir, entity, &ranking.field);
        if identity && value(&selected[1], &ranking.field).is_none() {
            gaps.push(gap(
                format!("{}: `{}`", entity.name(), ranking.field),
                ordered_head::MINTED,
            ));
            continue;
        }
        let mut goal = vec![goal_for(1)];
        for earlier in &order_by[..at] {
            if let Some(pinned) = value(&selected[1], &earlier.field)
                .and_then(|held| compared(&earlier.field, CompareOp::Eq, &held))
            {
                goal.push(pinned);
            }
        }
        if !identity {
            let op = if ranking.direction == Direction::Descending {
                CompareOp::Lt
            } else {
                CompareOp::Gt
            };
            if let Some(steer) = value(&selected[1], &ranking.field)
                .and_then(|held| compared(&ranking.field, op, &held))
            {
                goal.push(steer);
            }
        }
        let goal = Predicate::All(goal);
        let chosen: Vec<Arrangement> = selected.iter().chain(&ties).cloned().collect();
        let accept = |row: &Arrangement| {
            holds(row, &goal)
                && fresh(row, &chosen)
                && order_by[..at]
                    .iter()
                    .all(|earlier| on(earlier, row, &selected[1]) == Some(Ordering::Equal))
                && on(ranking, row, &selected[1]) == Some(Ordering::Greater)
        };
        let distinction = if identity {
            tie_at
        } else {
            next(ir, entity, taken)
        };
        let Some(tie) = arrange_row(ir, entity, &goal, actors, (distinction, owner), &accept)
        else {
            gaps.push(gap(
                format!("{}: `{}`", entity.name(), ranking.field),
                ordered_head::UNSTEERED,
            ));
            continue;
        };
        ties.push(tie);
    }
    let mut placed_rows = vec![selected[0].clone()];
    placed_rows.extend(ties.iter().cloned());
    placed_rows.extend(selected[1..].iter().cloned());
    // Ties placed before S2 that leave a row-set command the steps send undecided are left out,
    // named, and the scenario stands on S1, S2 and S3.
    if !ties.is_empty() && !placed(&placed_rows) {
        gaps.push(gap(
            format!("{}: a tie created before the head", entity.name()),
            ordered_head::UNSTEERED,
        ));
        return Some(selected);
    }
    Some(placed_rows)
}

/// One row `accept` takes, its creating input steered by `goal`; where no creator's input maps every
/// field `goal` reads — an `Optional` field one creator leaves unwritten — by the conjuncts over the
/// required fields alone, and then by nothing. `accept` decides; the steering only proposes.
fn arrange_row(
    ir: &EssIr,
    entity: &EntityHandle,
    goal: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    (distinction, owner): (Distinction, Option<&InstanceName>),
    accept: &dyn Fn(&Arrangement) -> bool,
) -> Option<Arrangement> {
    let optional: BTreeSet<&str> = ir
        .entity(entity)
        .fields
        .iter()
        .filter(|field| field.type_ref.is_optional())
        .map(|field| field.name.as_str())
        .collect();
    let required = |predicate: &Predicate| {
        !predicate
            .fact_paths()
            .iter()
            .any(|path| optional.contains(path.namespace()))
    };
    let relaxed = match goal {
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .filter(|child| required(child))
                .cloned()
                .collect(),
        ),
        other if required(other) => other.clone(),
        _ => Predicate::Always,
    };
    copied::steered(ir, entity, goal, actors, (distinction, owner), accept).or_else(|| {
        [goal.clone(), relaxed, Predicate::Always]
            .iter()
            .find_map(|steer| {
                arrange_toward_owned(ir, entity, steer, actors, distinction, owner, accept)
            })
    })
}

/// What one branch's arrangement reads per selector of its command, in the order the selectors
/// first appear: the filter with the input and subject written in, the `forall` predicates of the
/// row-set guards over it, and the fields the branch copies from the rows it selects.
struct Plan<'p> {
    command: &'p ResolvedCommand,
    outcome: &'p ResolvedOutcome,
    selections: &'p [&'p ResolvedRowSelection],
    filters: Vec<Predicate>,
    /// The subject each selector's rows are arranged under, by their owner link (beyond10x/ess#553).
    owners: Vec<Option<&'p InstanceName>>,
    foralls: Vec<Vec<Predicate>>,
    copied: Vec<Vec<&'p str>>,
    /// The order of the head the branch reads from each selector, where it reads one (ess/27).
    heads: Vec<Option<&'p [Ranking]>>,
}

/// The selections of a branch and, beside each, the owner row arranged under it (beyond10x/ess#553).
type Selections<'p> = (
    &'p [&'p ResolvedRowSelection],
    &'p [Option<(Predicate, Arrangement)>],
);

impl<'p> Plan<'p> {
    /// The plan for `outcome` of `command` over `selections`, each predicate `closed`.
    fn new(
        command: &'p ResolvedCommand,
        outcome: &'p ResolvedOutcome,
        (selections, unders): Selections<'p>,
        closed: &dyn Fn(&Predicate) -> Predicate,
    ) -> Self {
        let foralls = selections
            .iter()
            .map(|selection| {
                command
                    .outcomes
                    .iter()
                    .filter_map(|other| match &other.condition {
                        ResolvedCondition::RelatedSet {
                            selection: guarded,
                            test,
                            ..
                        } if guarded == *selection => test.predicate().map(closed),
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        Self {
            command,
            outcome,
            selections,
            filters: selections
                .iter()
                .zip(unders)
                .map(|(selection, under)| {
                    closed(&under.as_ref().map_or_else(
                        || selection.filter.clone(),
                        |(link, _)| without_link(&selection.filter, link),
                    ))
                })
                .collect(),
            owners: unders
                .iter()
                .map(|under| under.as_ref().map(|(_, row)| &row.instance))
                .collect(),
            foralls,
            copied: selections
                .iter()
                .map(|selection| copied(outcome, selection))
                .collect(),
            heads: selections
                .iter()
                .map(|selection| head_order(outcome, selection))
                .collect(),
        }
    }

    /// No refuting row, then one refuting each `forall`, named `(selector, index)`.
    fn variants(&self) -> Vec<Option<(usize, usize)>> {
        std::iter::once(None)
            .chain(
                self.foralls
                    .iter()
                    .enumerate()
                    .flat_map(|(at, tests)| (0..tests.len()).map(move |index| Some((at, index)))),
            )
            .collect()
    }

    /// The steps after `before` that arrange `counts[i]` rows of selector `i`, each selector's
    /// after the one before it, the `forall` `violate` names refuted; and the rows arranged. `None`
    /// where some selector's rows cannot be arranged.
    fn arrange(
        &self,
        ir: &EssIr,
        counts: &[usize],
        violate: Option<(usize, usize)>,
        actors: &BTreeMap<QualifiedName, ActorRef>,
        before: &[ScenarioStep],
        supplied: &BTreeMap<String, ScenarioValue>,
    ) -> Option<(Vec<ScenarioStep>, Vec<Arrangement>, Vec<RefusalCause>)> {
        let mut taken = captured(before);
        let mut gaps: Vec<RefusalCause> = Vec::new();
        let mut steps = before.to_vec();
        let mut arranged: Vec<Arrangement> = Vec::new();
        for (at, selection) in self.selections.iter().enumerate() {
            // A selector reading values this branch does not publish is another branch's to read.
            if !publishes(self.outcome, selection) {
                continue;
            }
            // Its published operands, as the rows arranged so far supply them (ess/27,
            // beyond10x/ess#549): the selectors supplying them come first.
            let filters = self.resolved(ir, supplied, &steps);
            if reads_published(&filters[at], &selection.reads) {
                return None;
            }
            let others: Vec<Predicate> = self
                .selections
                .iter()
                .zip(&filters)
                .enumerate()
                .filter(|(other, (sibling, _))| {
                    *other != at
                        && sibling.entity == selection.entity
                        && publishes(self.outcome, sibling)
                })
                .map(|(_, (_, filter))| filter.clone())
                .collect();
            let rows = arrange_rows(
                ir,
                &selection.entity,
                &filters[at],
                &self.foralls[at],
                (
                    counts[at],
                    violate.filter(|(of, _)| *of == at).map(|(_, index)| index),
                ),
                &self.copied[at],
                Siblings {
                    filters: &others,
                    arranged: &arranged,
                },
                &mut Head {
                    order_by: self.heads[at],
                    gaps: &mut gaps,
                },
                actors,
                &mut taken,
                (&steps, self.owners[at]),
            )?;
            for row in &rows {
                steps.extend(row.steps.iter().cloned());
            }
            arranged.extend(rows);
            if let Some(order_by) = self.heads[at].filter(|_| counts[at] >= 2) {
                if !selection.reads.is_empty() {
                    let decoys = self.cross_decoys(
                        ir,
                        (at, order_by),
                        supplied,
                        (&filters[at], &others),
                        actors,
                        &mut taken,
                        (&mut steps, &arranged),
                        &mut gaps,
                    );
                    arranged.extend(decoys);
                }
            }
        }
        Some((steps, arranged, gaps))
    }

    /// Every filter with the published values it reads written in, as the rows `steps` leave
    /// supply them; a value no row supplies yet is kept as written.
    fn resolved(
        &self,
        ir: &EssIr,
        supplied: &BTreeMap<String, ScenarioValue>,
        steps: &[ScenarioStep],
    ) -> Vec<Predicate> {
        if self
            .selections
            .iter()
            .all(|selection| selection.reads.is_empty())
        {
            return self.filters.clone();
        }
        let Ok(rows) = walk(ir, steps, false) else {
            return self.filters.clone();
        };
        let Ok(reading) = Reading::new(ir, self.command, supplied, &rows) else {
            return self.filters.clone();
        };
        let reading = reading.for_outcome(self.outcome);
        self.selections
            .iter()
            .zip(&self.filters)
            .map(|(selection, filter)| {
                if selection.reads.is_empty() || !publishes(self.outcome, selection) {
                    return filter.clone();
                }
                let values: Vec<Option<Node>> = selection
                    .reads
                    .iter()
                    .map(|read| reading.published_value(read).ok())
                    .collect();
                written_published(filter, &selection.reads, &values)
            })
            .collect()
    }

    /// The cross decoys of selector `at`, whose head the branch reads by `order_by` and whose
    /// filter reads a published value (`docs/design/published-value-operand.md`, "Synthesis",
    /// item 5): for each other selected row of the selector supplying that value, one row meeting
    /// every conjunct but the operand's, carrying that other row's value there, ranked above the
    /// inner head on the first key, with a copied value of its own. A target scoping the inner read
    /// by the wrong outer row, or by none, then answers a cross decoy. One no creating input places
    /// is left out and named in `gaps`; the scenario stands.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn cross_decoys(
        &self,
        ir: &EssIr,
        (at, order_by): (usize, &[Ranking]),
        supplied: &BTreeMap<String, ScenarioValue>,
        (resolved, siblings): (&Predicate, &[Predicate]),
        actors: &BTreeMap<QualifiedName, ActorRef>,
        taken: &mut BTreeSet<InstanceName>,
        (steps, arranged): (&mut Vec<ScenarioStep>, &[Arrangement]),
        gaps: &mut Vec<RefusalCause>,
    ) -> Vec<Arrangement> {
        use std::cmp::Ordering;
        let selection = self.selections[at];
        let entity = &selection.entity;
        let Some(first) = order_by.first() else {
            return Vec::new();
        };
        let Ok(rows) = walk(ir, steps, false) else {
            return Vec::new();
        };
        let Ok(reading) = Reading::new(ir, self.command, supplied, &rows) else {
            return Vec::new();
        };
        let reading = reading.for_outcome(self.outcome);
        let ties = reads(self.outcome)
            .into_iter()
            .find_map(|read| match &read.value {
                ResolvedPayloadValue::RelatedHead {
                    selection: read,
                    ties,
                    ..
                } if read == selection => Some(*ties),
                _ => None,
            })
            .unwrap_or(Ties::Total);
        let Ok(head) = reading.head(selection, order_by, ties) else {
            return Vec::new();
        };
        let head_first = row_key(ir, head, &first.field);
        let copied = &self.copied[at];
        let head_copied: Vec<Option<Node>> = copied
            .iter()
            .map(|field| row_key(ir, head, field))
            .collect();
        let values: Vec<Option<Node>> = selection
            .reads
            .iter()
            .map(|read| reading.published_value(read).ok())
            .collect();
        let better = if first.direction == Direction::Descending {
            CompareOp::Gt
        } else {
            CompareOp::Lt
        };
        let truth = |row: &Arrangement, predicate: &Predicate| {
            subject_fact::row_truth_with(
                ir,
                entity,
                &row.settled,
                &row.unwritten,
                Some(&row.state),
                predicate,
                None,
            )
        };
        let mut out: Vec<Arrangement> = Vec::new();
        for (index, read) in selection.reads.iter().enumerate() {
            let Some(source) = source_of(self.outcome, read.event.name(), &read.field) else {
                continue;
            };
            // Only a value read from another selector's rows has other rows to scope by.
            let Some(outer) = read_selection(&source.value) else {
                continue;
            };
            let Some(Ok(outer_head)) = reading.read(&source.value) else {
                continue;
            };
            let (Ok(candidates), Ok(members)) =
                (reading.candidates(outer), reading.members(outer, None))
            else {
                continue;
            };
            let others: Vec<Node> = candidates
                .iter()
                .zip(&members)
                .filter(|(row, member)| {
                    member.selected == Truth::True && !std::ptr::eq(**row, outer_head.row)
                })
                .filter_map(|(row, _)| {
                    let other = ReadRow {
                        row,
                        field: outer_head.field,
                        type_ref: outer_head.type_ref,
                        settled_as: String::new(),
                    };
                    read_value(ir, &other)?.as_literal().cloned()
                })
                .filter(|other| Some(other) != values[index].as_ref())
                .collect();
            for other in others {
                let mut moved = values.clone();
                moved[index] = Some(other);
                let mut goal = vec![written_published(
                    &self.filters[at],
                    &selection.reads,
                    &moved,
                )];
                if let Some(steer) = head_first
                    .as_ref()
                    .and_then(|held| compared(&first.field, better, held))
                {
                    goal.push(steer);
                }
                let goal = Predicate::All(goal);
                let accept = |row: &Arrangement| {
                    truth(row, resolved) == Truth::False
                        && truth(row, &goal) == Truth::True
                        && siblings
                            .iter()
                            .all(|sibling| truth(row, sibling) == Truth::False)
                        && ordered_head::on(
                            ir,
                            entity,
                            first,
                            arranged_key(ir, entity, row, &first.field).as_ref(),
                            head_first.as_ref(),
                        ) == Some(Ordering::Less)
                        && copied.iter().zip(&head_copied).all(|(field, held)| {
                            arranged_key(ir, entity, row, field).is_some_and(|value| {
                                !zero(&value)
                                    && Some(&value) != held.as_ref()
                                    && arranged.iter().chain(&out).all(|earlier| {
                                        arranged_key(ir, entity, earlier, field).as_ref()
                                            != Some(&value)
                                    })
                            })
                        })
                };
                let distinction = next(ir, entity, taken);
                let placed = arrange_row(ir, entity, &goal, actors, (distinction, None), &accept)
                    .filter(|row| {
                        let mut after = steps.clone();
                        after.extend(row.steps.iter().cloned());
                        walk(ir, &after, true).is_ok()
                    });
                match placed {
                    Some(row) => {
                        steps.extend(row.steps.iter().cloned());
                        out.push(row);
                    }
                    None => gaps.push(gap(
                        format!(
                            "{}: `{}` scoped by another `{}`",
                            self.command.name,
                            entity.name(),
                            outer.entity.name()
                        ),
                        ordered_head::CROSS_UNSTEERED,
                    )),
                }
            }
        }
        out
    }
}

/// The order of the first head `outcome` reads from `selection`, where it reads one.
fn head_order<'o>(
    outcome: &'o ResolvedOutcome,
    selection: &ResolvedRowSelection,
) -> Option<&'o [Ranking]> {
    reads(outcome)
        .into_iter()
        .find_map(|read| match &read.value {
            ResolvedPayloadValue::RelatedHead {
                selection: read,
                order_by,
                ..
            } if read == selection => Some(order_by.as_slice()),
            _ => None,
        })
}

/// Why no scenario can decide the rows `selection` selects, where none can: it is unscoped, or it
/// reads a member of a struct identity no row holds as a literal.
fn decidable(
    ir: &EssIr,
    command: &ResolvedCommand,
    selection: &ResolvedRowSelection,
) -> Result<(), &'static str> {
    if !scoped(ir, command, selection) {
        return Err(
            "selects rows by no equality between a String or Uuid field and the input or the \
             addressed subject, so the rows it counts are not only the scenario's own",
        );
    }
    if unprojected_identity(ir, selection) {
        return Err(
            "reads a member of a struct identity every declared creation publishes only as an \
             observed value, never from its input: a struct held by reference has no member \
             projection, so the rows it selects are never decided",
        );
    }
    Ok(())
}

/// What [`prepare`] arranges: the subject's setup with the rows, the input sent, and the witness
/// gaps the scenario stands with.
pub(super) type Prepared = (Setup, BTreeMap<String, Node>, Vec<RefusalCause>);

/// The rows a branch of a row-set command is witnessed on, the subject it addresses where it
/// addresses one, the input it is sent, and the witness gaps it stands with: the first
/// arrangement — of the counts [`COUNTS`] names per selector, [`HEAD_COUNTS`] for a selector whose
/// head the branch reads, and for a `forall`, with or without a refuting row — on which the rows
/// its steps leave select this branch and no branch answered before it.
#[allow(clippy::too_many_lines)]
pub(super) fn prepare(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Prepared, RefusalCause> {
    let ir = models.arrangement;
    let selections = selections_of(command)?;
    let at = || format!("{}/{}", command.name, outcome.name);
    for selection in &selections {
        decidable(ir, command, selection).map_err(|reason| gap(at(), reason))?;
    }
    let input = input_for(ir, command, outcome, Distinction::PLAIN)?;
    let setup = subject_setup(ir, command, outcome, actors)?;
    let subject_ref = reading(command, outcome);
    let supplied = supply(
        ir,
        command,
        &input,
        subject_ref,
        setup.instance.as_ref(),
        &setup.bound,
    );
    let closed = |predicate: &Predicate| {
        written_in(predicate, &|path| {
            input_value(path, &input).or_else(|| subject_value(path, &setup.settled))
        })
    };
    // Rows a selector reads under the addressed subject, by their owner link, are arranged under
    // that subject, which their own commands then read as the steps leave it (beyond10x/ess#553):
    // the reviews a reopen counts, filed under the incident it reopens and approved there.
    let unders: Vec<Option<(Predicate, Arrangement)>> = selections
        .iter()
        .map(|selection| under_subject(ir, command, selection, &setup, &supplied))
        .collect();
    // The addressed subject is held while those rows are arranged, so their own commands read it
    // as the steps leave it; every such selector names the same subject.
    let _held = unders
        .iter()
        .flatten()
        .next()
        .and_then(|(_, row)| addressed(command, &supplied).map(|(entity, _)| (entity, row)))
        .map(|(entity, row)| subject_fact::holding(ir, entity, row));
    let plan = Plan::new(command, outcome, (&selections, &unders), &closed);
    // A head is arranged only where a creating command steers its first key: a field set from the
    // input, or an identity the input names (ess/27, beyond10x/ess#543).
    for (selection, order_by) in selections.iter().zip(&plan.heads) {
        let Some(first) = order_by.and_then(|order_by| order_by.first()) else {
            continue;
        };
        if !ordered_head::steerable(ir, &selection.entity, &first.field) {
            let reason = if ordered_head::is_identity(ir, &selection.entity, &first.field) {
                ordered_head::MINTED
            } else if ordered_head::minted(ir, &selection.entity, &first.field) {
                ordered_head::MINTED_KEY
            } else {
                ordered_head::UNSTEERED
            };
            return Err(gap(format!("{}: `{}`", at(), first.field), reason));
        }
    }
    // A selector scoped by a value the target mints reads rows no scenario can arrange against it
    // (ess/27, beyond10x/ess#549): the branch alone is refused.
    for selection in selections
        .iter()
        .filter(|selection| publishes(outcome, selection))
    {
        for read in &selection.reads {
            let Some(source) = source_of(outcome, read.event.name(), &read.field) else {
                continue;
            };
            let (ResolvedPayloadValue::RelatedSelection {
                selection: outer,
                field,
                ..
            }
            | ResolvedPayloadValue::RelatedHead {
                selection: outer,
                field,
                ..
            }) = &source.value
            else {
                continue;
            };
            let mints = if ordered_head::is_identity(ir, &outer.entity, field) {
                !ordered_head::steerable(ir, &outer.entity, field)
            } else {
                ordered_head::minted(ir, &outer.entity, field)
            };
            if mints {
                return Err(gap(
                    format!(
                        "{}: `{}` where {}",
                        at(),
                        selection.entity.name(),
                        selection.filter
                    ),
                    ordered_head::SCOPE_MINTED,
                ));
            }
        }
    }
    let mut first_failure: Option<String> = None;
    let mut search = |heads: &[usize]| {
        let lists: Vec<&[usize]> = plan
            .heads
            .iter()
            .map(|head| if head.is_some() { heads } else { &COUNTS[..] })
            .collect();
        for counts in count_tuples(&lists) {
            for violate in &plan.variants() {
                if violate.is_some_and(|(at, _)| counts[at] == 0) {
                    continue;
                }
                let Some((steps, arranged, gaps)) =
                    plan.arrange(ir, &counts, *violate, actors, &setup.steps, &supplied)
                else {
                    continue;
                };
                let rows = match walk(ir, &steps, true) {
                    Ok(rows) => rows,
                    Err(why) => {
                        first_failure.get_or_insert(why);
                        continue;
                    }
                };
                if let Err(why) = consistent(ir, command, outcome, &supplied, &rows) {
                    first_failure.get_or_insert(why);
                    continue;
                }
                return Some((steps, arranged, gaps, rows));
            }
        }
        None
    };
    if let Some((steps, arranged, gaps, rows)) = search(&HEAD_COUNTS) {
        let mut found = setup;
        found.steps = steps;
        for row in &arranged {
            found.source.extend(row.source.iter().cloned());
        }
        for selection in &selections {
            found
                .source
                .insert(EntityRef::from(&selection.entity).into());
        }
        found
            .settled
            .extend(selected_values(ir, command, outcome, &supplied, &rows));
        return Ok((found, input, gaps));
    }
    // A branch whose rows take it with one selected row and never more never observes its order.
    if plan.heads.iter().any(Option::is_some) && search(&[1]).is_some() {
        return Err(gap(at(), ordered_head::UNOBSERVABLE));
    }
    // A selector reading a field no declared command writes is refused by that field
    // (downstream report, 2026-10-18); of several selectors, the first that reads one.
    Err(selections
        .iter()
        .find_map(|selection| {
            super::set_effects::unwritten(ir, &selection.entity, &selection.filter)
        })
        .unwrap_or_else(|| {
            gap(
                first_failure.map_or_else(at, |why| format!("{}: {why}", at())),
                ROWS_UNARRANGED,
            )
        }))
}

/// The conjunct of a selector comparing its rows' owner link with the input naming the subject the
/// send addresses (beyond10x/ess#553), and that subject as the setup arranged it: the selected
/// entity is owned by the subject's, and the input names the arranged row. `None` for every other
/// selector, which is arranged as it was.
fn under_subject(
    ir: &EssIr,
    command: &ResolvedCommand,
    selection: &ResolvedRowSelection,
    setup: &Setup,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Option<(Predicate, Arrangement)> {
    let instance = setup.instance.as_ref()?;
    let (entity, named) = addressed(command, supplied)?;
    let owned = ir.owner_of(&selection.entity)?;
    if owned.owner != *entity
        || !matches!(named, ScenarioValue::Instance { instance: held } if held == instance)
    {
        return None;
    }
    let conjuncts = match &selection.filter {
        Predicate::All(children) => children.as_slice(),
        other => std::slice::from_ref(other),
    };
    let link = conjuncts.iter().find(|conjunct| {
        let Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq,
            right: Operand::Fact(right),
            ..
        } = conjunct
        else {
            return false;
        };
        let matches = |row: &FactPath, sent: &FactPath| {
            row.segments() == [owned.via.to_owned()]
                && input_field(sent).is_some_and(|field| supplied.get(field) == Some(named))
        };
        matches(left, right) || matches(right, left)
    })?;
    // The subject as the setup's steps leave it, its unwritten `Optional` fields included.
    let walked = walk(ir, &setup.steps, false).ok()?;
    let left = walked
        .held
        .iter()
        .find(|row| row.entity == *entity && row.instance.as_ref() == Some(instance))?;
    let row = Arrangement {
        instance: instance.clone(),
        state: left.state.clone(),
        steps: setup.steps.clone(),
        source: setup.source.clone(),
        settled: left.settled.clone(),
        unwritten: left.unwritten.clone(),
        unobserved: BTreeSet::new(),
    };
    Some((link.clone(), row))
}

/// `filter` without the conjunct `link`, which arranging the rows under the subject holds.
fn without_link(filter: &Predicate, link: &Predicate) -> Predicate {
    match filter {
        Predicate::All(children) => {
            let kept: Vec<Predicate> = children
                .iter()
                .filter(|child| *child != link)
                .cloned()
                .collect();
            match <[Predicate; 1]>::try_from(kept) {
                Ok([only]) => only,
                Err(kept) => Predicate::All(kept),
            }
        }
        other if other == link => Predicate::Always,
        other => other.clone(),
    }
}

/// The top-level input field `path` names, as `input.<field>`.
fn input_field(path: &FactPath) -> Option<&str> {
    match path.segments() {
        [root, field] if root == ess_domain::command::subject_fact::INPUT_NAMESPACE => {
            Some(field.as_str())
        }
        _ => None,
    }
}

/// `predicate` with each equality between a row's field and an input field decided where both name
/// an arranged instance (beyond10x/ess#553): a review filed under the incident a reopen names. Every
/// other leaf is left as it is.
fn instances_decided(
    predicate: &Predicate,
    settled: &BTreeMap<String, Determined>,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> Predicate {
    let decide = |child: &Predicate| instances_decided(child, settled, supplied);
    match predicate {
        Predicate::All(children) => Predicate::All(children.iter().map(decide).collect()),
        Predicate::Any(children) => Predicate::Any(children.iter().map(decide).collect()),
        Predicate::Not(inner) => Predicate::Not(Box::new(decide(inner))),
        Predicate::Compare {
            left: Operand::Fact(left),
            op: op @ (CompareOp::Eq | CompareOp::Ne),
            right: Operand::Fact(right),
            ..
        } => {
            let instance = |row: &FactPath, sent: &FactPath| {
                let [field] = row.segments() else {
                    return None;
                };
                let ScenarioValue::Instance { instance: held } = &settled.get(field)?.value else {
                    return None;
                };
                let ScenarioValue::Instance { instance: named } =
                    supplied.get(input_field(sent)?)?
                else {
                    return None;
                };
                Some(held == named)
            };
            match instance(left, right).or_else(|| instance(right, left)) {
                Some(same) if same == (*op == CompareOp::Eq) => Predicate::Always,
                Some(_) => Predicate::Never,
                None => predicate.clone(),
            }
        }
        other => other.clone(),
    }
}

/// Every scenario of `suite` sending a row-set command whose required branch the rows its steps
/// leave do not decide, taken out of the suite and refused naming the send.
pub(super) fn contradictions(
    ir: &EssIr,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    if !ir.commands().values().any(uses) {
        return;
    }
    let refused: Vec<(crate::scenario::ScenarioId, String)> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if ir.commands().get(command.name()).is_some_and(uses))
            })
        })
        .filter_map(|(id, scenario)| {
            walk(ir, &scenario.steps, true)
                .err()
                .map(|why| (id.clone(), why))
        })
        .collect();
    for (id, why) in refused {
        suite.scenarios.remove(&id);
        refusals.push(Refusal::about(
            &id,
            RefusalCause::NoWitness(WitnessGap {
                path: why,
                type_ref: "row set".into(),
                reason: "sends a row-set command whose required branch the rows the scenario \
                         arranges do not decide",
            }),
        ));
    }
}

/// The subject a branch of a row-set command is sent for: its own, or — for a refusal naming none —
/// the existing one its siblings address through the input, which its selector's `subject.` reads.
pub(super) fn reading<'c>(
    command: &'c ResolvedCommand,
    outcome: &'c ResolvedOutcome,
) -> Option<&'c ess_compiler::ir::ResolvedSubject> {
    outcome.subject.as_ref().or_else(|| {
        command
            .outcomes
            .iter()
            .filter_map(|other| other.subject.as_ref())
            .find(|subject| {
                subject.effect != ResolvedEffect::Creates
                    && matches!(subject.instance, ResolvedInstance::Supplied { .. })
            })
    })
}

/// The subject a branch of a row-set command is sent for, arranged: the branch's own; for a
/// refusal naming none, the one its siblings address through the input, left where it was
/// arranged; nothing for a creation, beyond what its own arrangement needs (an owner).
fn subject_setup(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Setup, RefusalCause> {
    let own = outcome
        .subject
        .as_ref()
        .is_some_and(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }));
    // A creation addresses no existing record: arranging the one its siblings address would arrange
    // the very identity it is sent to create (ess/23, beyond10x/ess#463).
    let creates = outcome
        .subject
        .as_ref()
        .is_some_and(|subject| subject.effect == ResolvedEffect::Creates);
    let addressing = if own {
        Some(outcome)
    } else if creates && subject_fact::identity_selectors(ir) {
        None
    } else {
        command.outcomes.iter().find(|other| {
            other.subject.as_ref().is_some_and(|subject| {
                subject.effect != ResolvedEffect::Creates
                    && matches!(subject.instance, ResolvedInstance::Supplied { .. })
            })
        })
    };
    let mut setup = match (addressing, &outcome.subject) {
        (Some(branch), _) => prepare_in(ir, branch, actors, None, Distinction::PLAIN)?,
        (None, Some(_)) => prepare_in(ir, outcome, actors, None, Distinction::PLAIN)?,
        (None, None) => Setup::none(),
    };
    if !own && addressing.is_some() {
        // A refusal changes nothing: the subject rests where it was arranged.
        setup.after.clone_from(&setup.before);
    }
    Ok(setup)
}

/// Every top-level input field `predicate` reads under `input.`.
pub(super) fn input_reads(predicate: &Predicate) -> impl Iterator<Item = &str> {
    predicate.fact_paths().into_iter().filter_map(|path| {
        let (root, rest) = path.segments().split_first()?;
        (root == ess_domain::command::subject_fact::INPUT_NAMESPACE)
            .then(|| rest.first().map(String::as_str))
            .flatten()
    })
}

/// Every field `outcome` copies from a row `selection` selects, once each, in name order.
fn copied<'o>(outcome: &'o ResolvedOutcome, selection: &ResolvedRowSelection) -> Vec<&'o str> {
    reads(outcome)
        .into_iter()
        .filter_map(|read| match &read.value {
            ResolvedPayloadValue::RelatedSelection {
                selection: read,
                field,
                ..
            }
            | ResolvedPayloadValue::RelatedHead {
                selection: read,
                field,
                ..
            } if read == selection => Some(field.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Every combination of row counts, one per selector, each drawn from its own list in order, the
/// first selector's count varying slowest: for one selector, its list itself.
fn count_tuples(lists: &[&[usize]]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = vec![Vec::new()];
    for list in lists {
        out = out
            .into_iter()
            .flat_map(|prefix| {
                list.iter().map(move |count| {
                    let mut next = prefix.clone();
                    next.push(*count);
                    next
                })
            })
            .collect();
    }
    out
}
