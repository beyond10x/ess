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
//!   order one, two, none, three, so a branch true of a nonempty set is witnessed on one.
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
    ResolvedRowSelection, ResolvedTypeRef, RowMember, Selected,
};
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_primitives::facts::FactPath;
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Truth};

use super::set_effects::{input_value, misses, subject_value, written_in};
use super::{
    absorb, arrange_toward_filter, candidates, flatten, instance_name, prepare_in, settled,
    still_unwritten, subject_fact, supply, unwritten_by, ActorRef, Arrangement, ConformanceSuite,
    Decision, Determined, Distinction, EntityRef, InstanceName, Refusal, RefusalCause,
    ScenarioStep, ScenarioValue, Setup, WitnessGap,
};

/// How many rows the selector selects, in the order they are tried.
const COUNTS: [usize; 4] = [1, 2, 0, 3];

/// The key a selected row's value is settled under in the arrangement's fields: not a field name
/// — it holds spaces — so nothing reading fields by name finds it.
pub(super) fn key(selection: &ResolvedRowSelection, field: &str) -> String {
    format!(
        "row set {} where {} field {field}",
        selection.entity.name(),
        selection.filter
    )
}

/// Every filtered read of `outcome`, in `sets:`, `payload:` and an error payload, at any depth.
fn reads(outcome: &ResolvedOutcome) -> Vec<&ResolvedPayloadField> {
    fn walk<'a>(field: &'a ResolvedPayloadField, out: &mut Vec<&'a ResolvedPayloadField>) {
        match &field.value {
            ResolvedPayloadValue::RelatedSelection { .. } => out.push(field),
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

/// The refusal for a further witness of a row-set branch, which only its own scenario arranges.
pub(super) fn unarranged() -> RefusalCause {
    gap(
        "a further witness of a row-set branch".into(),
        "arranges no rows of its own: the branch's own scenario witnesses it on the rows its \
         selector selects",
    )
}

fn gap(path: String, reason: &'static str) -> RefusalCause {
    RefusalCause::NoWitness(WitnessGap {
        path,
        type_ref: "row set".into(),
        reason,
    })
}

/// The one selector the command reads, by its guards and its values; or a refusal naming the
/// command where it reads more than one.
fn selection_of(command: &ResolvedCommand) -> Result<&ResolvedRowSelection, RefusalCause> {
    let mut found: Vec<&ResolvedRowSelection> = Vec::new();
    for outcome in &command.outcomes {
        if let ResolvedCondition::RelatedSet { selection, .. } = &outcome.condition {
            found.push(selection);
        }
        for read in reads(outcome) {
            if let ResolvedPayloadValue::RelatedSelection { selection, .. } = &read.value {
                found.push(selection);
            }
        }
    }
    let Some(first) = found.first() else {
        return Err(gap(
            command.name.to_string(),
            "reads no row set, so no row-set arrangement applies",
        ));
    };
    if found.iter().any(|other| *other != *first) {
        return Err(gap(
            command.name.to_string(),
            "reads more than one row set: a scenario arranges the rows of one selector per \
             command in this cut",
        ));
    }
    Ok(first)
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
/// between a `String` or `Uuid` field of the row and the input or the addressed subject.
fn scoped(ir: &EssIr, selection: &ResolvedRowSelection) -> bool {
    let entity = ir.entity(&selection.entity);
    let conjuncts: Vec<&Predicate> = match &selection.filter {
        Predicate::All(children) => children.iter().collect(),
        other => vec![other],
    };
    let field = |path: &FactPath| {
        let [name] = path.segments() else {
            return false;
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
    conjuncts.iter().any(|conjunct| {
        matches!(
            conjunct,
            Predicate::Compare {
                left: Operand::Fact(left),
                op: CompareOp::Eq,
                right: Operand::Fact(right),
                ..
            } if (field(left) && named(right)) || (field(right) && named(left))
        )
    })
}

/// The `when:` of `other`, where its condition is one.
fn when(other: &ResolvedOutcome) -> Option<&Predicate> {
    match &other.condition {
        ResolvedCondition::When { predicate } => Some(predicate),
        _ => None,
    }
}

/// An input reaching `outcome` as far as its input decides it: its own input guard true, every
/// input-guarded refusal false, and — for an accepting branch — every accepting `when:` branch
/// declared before it false; for the default, every other `when:` false.
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
    let default = matches!(outcome.condition, ResolvedCondition::Otherwise);
    let refute: Vec<&Predicate> = command
        .outcomes
        .iter()
        .filter(|other| other.name != outcome.name)
        .filter(|other| {
            other.error.is_some()
                || default
                || (outcome.error.is_none()
                    && command
                        .outcomes
                        .iter()
                        .position(|candidate| candidate.name == other.name)
                        < command
                            .outcomes
                            .iter()
                            .position(|candidate| candidate.name == outcome.name))
        })
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

/// What the rows decide for one send of `command`: the members of its selector, read with the
/// input and subject the send names.
struct Reading<'a> {
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    selection: &'a ResolvedRowSelection,
    supplied: BTreeMap<String, ScenarioValue>,
    input: BTreeMap<String, Node>,
    subject: Option<BTreeMap<String, Determined>>,
    rows: Vec<&'a Row>,
}

impl<'a> Reading<'a> {
    fn new(
        ir: &'a EssIr,
        command: &'a ResolvedCommand,
        supplied: &BTreeMap<String, ScenarioValue>,
        rows: &'a Rows,
    ) -> Result<Self, String> {
        let selection = selection_of(command).map_err(|cause| cause.to_string())?;
        let candidates = rows.of(&selection.entity)?;
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
            selection,
            supplied: supplied.clone(),
            input: resolved_literals(supplied, rows),
            subject,
            rows: candidates,
        })
    }

    /// `predicate` with the subject's values written in, where it reads any.
    fn closed(&self, predicate: &Predicate) -> Result<Predicate, String> {
        let empty = BTreeMap::new();
        let subject = self.subject.as_ref().unwrap_or(&empty);
        // The input the send names is written in where it is literal, as the arrangement reads it: an
        // input naming an arranged instance — the addressed row of an `updates:` (ess/23,
        // beyond10x/ess#429) — leaves the rest of the input unflattened, and the literal it compares
        // with would read nothing.
        let written = written_in(predicate, &|path| {
            subject_value(path, subject).or_else(|| input_value(path, &self.input))
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
        let identity = &self.ir.entity(&row.entity).identity;
        let reads_identity = predicate
            .fact_paths()
            .iter()
            .any(|path| path.segments().len() == 1 && path.namespace() == identity.name);
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
        subject_fact::row_truth_with(
            self.ir,
            &row.entity,
            &settled,
            &row.unwritten,
            Some(&row.state),
            predicate,
            Some((self.command, &self.input)),
        )
    }

    fn members(&self, satisfies: Option<&Predicate>) -> Result<Vec<RowMember>, String> {
        let filter = self.closed(&self.selection.filter)?;
        let satisfies = satisfies
            .map(|predicate| self.closed(predicate))
            .transpose()?;
        Ok(self
            .rows
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
        let ResolvedCondition::RelatedSet { test, input, .. } = &outcome.condition else {
            return Ok(None);
        };
        let tested = test.decide(&self.members(test.predicate())?);
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

    /// The one row the selector selects, or why there is none to read.
    fn one(&self) -> Result<&'a Row, String> {
        let members = self.members(None)?;
        match Selected::of(&members) {
            Selected::One(index) => Ok(self.rows[index]),
            Selected::None => Err("no row is selected, and a read needs one".into()),
            Selected::Several => Err("several rows are selected, and a read needs one".into()),
            Selected::Unknown => Err("which row is selected is not decided".into()),
        }
    }
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
    // a row the steps left is `existing_instance:`'s, never this branch's.
    if command
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
                if rows.held.iter().any(|row| {
                    row.entity == subject.entity && row.identity.as_ref() == Some(&identity)
                }) {
                    return Err("the creation names an identity a row already carries".into());
                }
            }
        }
    }
    let reading = Reading::new(ir, command, supplied, rows)?;
    let position = |outcome: &ResolvedOutcome| {
        command
            .outcomes
            .iter()
            .position(|candidate| candidate.name == outcome.name)
            .unwrap_or(usize::MAX)
    };
    let expected_at = position(expected);
    let default = matches!(expected.condition, ResolvedCondition::Otherwise);
    for other in &command.outcomes {
        let Some(taken) = reading.takes(other)? else {
            continue;
        };
        let answers_first = if other.name == expected.name {
            continue;
        } else if other.error.is_some() {
            // Row-set refusals answer before every accepting branch and the default, in
            // declaration order.
            default || expected.error.is_none() || position(other) < expected_at
        } else {
            default || (expected.error.is_none() && position(other) < expected_at)
        };
        if answers_first && taken != Truth::False {
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
    if !reads(expected).is_empty() {
        reading.one()?;
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
    let Ok(row) = reading.one() else {
        return out;
    };
    for read in reads(outcome) {
        let ResolvedPayloadValue::RelatedSelection {
            selection,
            field,
            type_ref,
        } = &read.value
        else {
            continue;
        };
        let identity = &ir.entity(&row.entity).identity.name;
        let value = if field == identity {
            row.identity.clone()
        } else if let Some(held) = row.settled.get(field) {
            Some(held.value.clone())
        } else if row.unwritten.contains(field) {
            Some(ScenarioValue::Literal { value: Node::Null })
        } else {
            None
        };
        if let Some(value) = value {
            out.insert(
                key(selection, field),
                Determined {
                    value,
                    type_ref: type_ref.clone(),
                },
            );
        }
    }
    out
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
            let before = if uses(command) {
                selected_values(ir, command, outcome, supplied, rows)
            } else {
                BTreeMap::new()
            };
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
            let before = rows
                .named(entity, &named)
                .map(|row| row.settled.clone())
                .unwrap_or_default();
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

/// The rows one attempt arranges: a decoy per conjunct of `filter`, then `count` rows it selects,
/// the first of them refuting `foralls[violate]` where `violate` names one. Each selected row's
/// value of every field in `copied` differs from zero and from every decoy's.
#[allow(clippy::too_many_arguments)]
fn arrange_rows(
    ir: &EssIr,
    entity: &EntityHandle,
    filter: &Predicate,
    foralls: &[Predicate],
    (count, violate): (usize, Option<usize>),
    copied: &[&str],
    actors: &BTreeMap<QualifiedName, ActorRef>,
    taken: &mut BTreeSet<InstanceName>,
    before: &[ScenarioStep],
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
    let mut arranged: Vec<Arrangement> = Vec::new();
    for (missed, miss) in misses(filter).into_iter().enumerate() {
        // Decidedly not selected, by this conjunct: it is false, and no other is. A conjunct that
        // reads what the missed one says is absent stays unknown, and is no other's refutation.
        let accept = |row: &Arrangement| {
            truth(row, filter) == Truth::False
                && conjuncts
                    .iter()
                    .enumerate()
                    .all(|(at, conjunct)| (truth(row, conjunct) == Truth::False) == (at == missed))
        };
        let distinction = next(ir, entity, taken);
        // A decoy no creating command arranges with the rows before it decided is left out: it
        // would catch a target ignoring that conjunct, and the scenario stands without it.
        if let Some(row) = arrange_row(ir, entity, &miss, actors, distinction, &accept) {
            if follows(&arranged, &row) {
                arranged.push(row);
            }
        }
    }
    let decoys = arranged.clone();
    for nth in 0..count {
        let mut goal = vec![filter.clone()];
        for (index, forall) in foralls.iter().enumerate() {
            goal.push(if nth == 0 && violate == Some(index) {
                Predicate::Not(Box::new(forall.clone()))
            } else {
                forall.clone()
            });
        }
        let goal = Predicate::All(goal);
        let accept = |row: &Arrangement| {
            truth(row, &goal) == Truth::True
                && copied.iter().all(|field| {
                    held(row, field).is_some_and(|value| {
                        !zero(value) && decoys.iter().all(|decoy| held(decoy, field) != Some(value))
                    })
                })
        };
        let distinction = next(ir, entity, taken);
        let row = arrange_row(ir, entity, &goal, actors, distinction, &accept)?;
        if !follows(&arranged, &row) {
            return None;
        }
        arranged.push(row);
    }
    Some(arranged)
}

/// One row `accept` takes, its creating input steered by `goal`; where no creator's input maps every
/// field `goal` reads — an `Optional` field one creator leaves unwritten — by the conjuncts over the
/// required fields alone, and then by nothing. `accept` decides; the steering only proposes.
fn arrange_row(
    ir: &EssIr,
    entity: &EntityHandle,
    goal: &Predicate,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
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
    [goal.clone(), relaxed, Predicate::Always]
        .iter()
        .find_map(|steer| arrange_toward_filter(ir, entity, steer, actors, distinction, accept))
}

/// The rows a branch of a row-set command is witnessed on, the subject it addresses where it
/// addresses one, and the input it is sent: the first arrangement — of the counts [`COUNTS`] names,
/// and for a `forall`, with or without a refuting row — on which the rows its steps leave select
/// this branch and no branch answered before it.
pub(super) fn prepare(
    models: &super::caller::InvocationModels<'_>,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<(Setup, BTreeMap<String, Node>), RefusalCause> {
    let ir = models.arrangement;
    let selection = selection_of(command)?;
    let at = || format!("{}/{}", command.name, outcome.name);
    if !scoped(ir, selection) {
        return Err(gap(
            at(),
            "selects rows by no equality between a String or Uuid field and the input or the \
             addressed subject, so the rows it counts are not only the scenario's own",
        ));
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
    let filter = closed(&selection.filter);
    let foralls: Vec<Predicate> = command
        .outcomes
        .iter()
        .filter_map(|other| match &other.condition {
            ResolvedCondition::RelatedSet { test, .. } => test.predicate().map(closed),
            _ => None,
        })
        .collect();
    let copied = copied(outcome);
    let variants: Vec<Option<usize>> = std::iter::once(None)
        .chain((0..foralls.len()).map(Some))
        .collect();
    let mut first_failure: Option<String> = None;
    for count in COUNTS {
        for violate in &variants {
            if violate.is_some() && count == 0 {
                continue;
            }
            let mut taken = captured(&setup.steps);
            let Some(arranged) = arrange_rows(
                ir,
                &selection.entity,
                &filter,
                &foralls,
                (count, *violate),
                &copied,
                actors,
                &mut taken,
                &setup.steps,
            ) else {
                continue;
            };
            let mut steps = setup.steps.clone();
            for row in &arranged {
                steps.extend(row.steps.iter().cloned());
            }
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
            let mut found = setup;
            found.steps = steps;
            for row in &arranged {
                found.source.extend(row.source.iter().cloned());
            }
            found
                .source
                .insert(EntityRef::from(&selection.entity).into());
            found
                .settled
                .extend(selected_values(ir, command, outcome, &supplied, &rows));
            return Ok((found, input));
        }
    }
    Err(gap(
        first_failure.map_or_else(at, |why| format!("{}: {why}", at())),
        "has no arrangement of rows the declared commands produce on which the rows its selector \
         selects take this branch",
    ))
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
    let addressing = if own {
        Some(outcome)
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

/// Every field `outcome` copies from a selected row, once each, in name order.
fn copied(outcome: &ResolvedOutcome) -> Vec<&str> {
    reads(outcome)
        .into_iter()
        .filter_map(|read| match &read.value {
            ResolvedPayloadValue::RelatedSelection { field, .. } => Some(field.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
