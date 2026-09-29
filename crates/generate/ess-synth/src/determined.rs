//! Which command behaviours the specification fully determines — language-neutral.
//!
//! A command's behaviour is generated when every one of its outcomes is something an emitter can
//! express with the conformance interpreter's semantics (`ess_conformance::interpret::execute`) and
//! the one precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence
//! order"); otherwise the whole command stays an obligation, and [`command`] names the first
//! construct that kept it one. The planner asks this module and nothing else, so the plan and every
//! emitter agree on the line.
//!
//! The paths a guard reads are resolved here too ([`resolve`]), against the declared types, so an
//! emitter renders a path that this module already checked rather than re-deriving which field of
//! which struct a dotted fact names.

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedEntity,
    ResolvedField, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue,
    ResolvedSubject, ResolvedTypeRef,
};
use ess_domain::types::Primitive;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};
use ess_primitives::time::CurrentTime;

/// `Ok` when every outcome of `command` is expressible; `Err` names the construct that keeps the
/// whole command an obligation, in a phrase that reads after "kept an obligation by".
pub(crate) fn command(ir: &EssIr, command: &ResolvedCommand) -> Result<(), String> {
    if !command.response.is_empty() {
        return Err("a typed response (`response:`)".to_owned());
    }
    let guarded = subject_guarded(command);
    let lifecycle = command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectState { .. } | ResolvedCondition::StateChange { .. }
        )
    });
    let external = command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
        )
    });
    if lifecycle && external {
        return Err("`when_subject_state:` beside `external:` in one command".to_owned());
    }
    let defaults = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.condition == ResolvedCondition::Otherwise)
        .count();
    if defaults > 1 {
        return Err("more than one default branch".to_owned());
    }
    let selection = selection_subject(command);
    if guarded {
        let Some(selection) = selection else {
            return Err("a subject guard with no supplied subject to read".to_owned());
        };
        for outcome in &command.outcomes {
            if let Some(subject) = &outcome.subject {
                if matches!(subject.instance, ResolvedInstance::Supplied { .. })
                    && !same_subject(subject, selection)
                {
                    return Err(
                        "subject guards beside a branch addressing another subject".to_owned()
                    );
                }
            }
        }
        let accepting: Vec<&ResolvedEffect> = command
            .outcomes
            .iter()
            .filter_map(|outcome| outcome.subject.as_ref())
            .map(|subject| &subject.effect)
            .collect();
        let moves = accepting
            .iter()
            .any(|effect| matches!(effect, ResolvedEffect::Moves { .. }));
        let updates = accepting
            .iter()
            .any(|effect| matches!(effect, ResolvedEffect::Updates));
        if moves && updates {
            return Err("a subject predicate choosing between a move and an update".to_owned());
        }
    }
    let reads_supplied = guarded
        || command.outcomes.iter().any(|outcome| {
            outcome.subject.as_ref().is_some_and(|subject| {
                matches!(subject.instance, ResolvedInstance::Supplied { .. })
            })
        });
    if reads_supplied && unknown_answer(command).is_none() {
        return Err(
            "an unknown identity, which reaches no declared outcome (neither \
             `unknown_instance:` nor `wrong_state:`)"
                .to_owned(),
        );
    }
    let mut subjects = command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .filter(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }));
    if let Some(first) = subjects.next() {
        if subjects.any(|other| !same_subject(first, other))
            && command.outcomes.iter().any(|outcome| {
                outcome.condition == ResolvedCondition::WrongState
                    && outcome
                        .error
                        .as_ref()
                        .is_some_and(|error| !ir.error(error).fields.is_empty())
            })
        {
            return Err(
                "a `wrong_state:` refusal describing the rows of more than one subject".to_owned(),
            );
        }
    }
    for outcome in &command.outcomes {
        self::outcome(ir, command, outcome, guarded, selection)
            .map_err(|construct| format!("{construct}, in `{}`", outcome.name))?;
    }
    Ok(())
}

/// Every outcome construct, checked for one branch.
// One exhaustive pass over every construct a branch can carry.
#[allow(clippy::too_many_lines)]
fn outcome(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    guarded: bool,
    selection: Option<&ResolvedSubject>,
) -> Result<(), String> {
    if outcome.replays.is_some() || outcome.retains_result {
        return Err("a retained result (`replays:`)".to_owned());
    }
    if outcome.instances.is_some() {
        return Err("a set outcome over filtered rows (`instances:`)".to_owned());
    }
    if !outcome.affects.is_empty() {
        return Err("effects on filtered rows (`affects:`)".to_owned());
    }
    let selection_entity = selection.map(|subject| ir.entity(&subject.entity));
    match &outcome.condition {
        ResolvedCondition::Related { .. } => return Err("`when_related:`".to_owned()),
        ResolvedCondition::InputAbsent => return Err("`input_absent:`".to_owned()),
        ResolvedCondition::ExistingInstance => return Err("`existing_instance:`".to_owned()),
        ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } => {
            supported(ir, &Env::Input(command), predicate)?;
        }
        ResolvedCondition::SubjectField {
            field, predicate, ..
        } => {
            let entity = selection_entity.ok_or("a subject guard with no subject")?;
            resolve(
                ir,
                &Env::Subject(command, entity),
                &FactPath::new(field).map_err(|error| error.to_string())?,
            )?;
            if let Some(predicate) = predicate {
                supported(ir, &Env::Input(command), predicate)?;
            }
        }
        ResolvedCondition::SubjectPredicate { predicate, input } => {
            let entity = selection_entity.ok_or("a subject guard with no subject")?;
            supported(ir, &Env::Subject(command, entity), predicate)?;
            if let Some(input) = input {
                supported(ir, &Env::Input(command), input)?;
            }
        }
        ResolvedCondition::SubjectState { predicate, .. }
        | ResolvedCondition::StateChange { predicate, .. } => {
            if let Some(predicate) = predicate {
                supported(ir, &Env::Input(command), predicate)?;
            }
        }
        ResolvedCondition::Otherwise
        | ResolvedCondition::External { .. }
        | ResolvedCondition::WrongState
        | ResolvedCondition::UnknownInstance => {}
    }
    let answered_for_subject = matches!(
        outcome.condition,
        ResolvedCondition::WrongState | ResolvedCondition::UnknownInstance
    );
    if answered_for_subject
        && (!outcome.emits.is_empty() || !outcome.sets.is_empty() || outcome.subject.is_some())
    {
        return Err("a `wrong_state:` or `unknown_instance:` branch that acts or emits".to_owned());
    }
    if let Some(error) = &outcome.error {
        let fields = &ir.error(error).fields;
        if !fields.is_empty() {
            // An input-guarded refusal answers before the row is read (step 2), and an unknown
            // identity has no row; every other refusal of a subject-guarded command answers after.
            let reads_held = outcome.condition == ResolvedCondition::WrongState
                || (guarded
                    && outcome.subject.is_none()
                    && !matches!(
                        outcome.condition,
                        ResolvedCondition::When { .. } | ResolvedCondition::UnknownInstance
                    ));
            let entity = if outcome.condition == ResolvedCondition::WrongState {
                wrong_state_entity(ir, command)
            } else {
                selection_entity
            }
            .filter(|_| reads_held);
            // A field the specification gives a source (ess/19, `story:error-payload-sources`)
            // is filled from it, checked as an event payload's source is; any other is read from
            // the held row, where there is one and it holds the field.
            for source in &outcome.error_payload {
                if matches!(source.value, ResolvedPayloadValue::Cleared) {
                    return Err(format!("`{{cleared}}` on the error `{error}`"));
                }
                value(ir, command, source, entity, false)?;
            }
            let unsourced = fields.iter().filter(|field| {
                !outcome
                    .error_payload
                    .iter()
                    .any(|source| source.target == field.name)
            });
            for field in unsourced {
                let Some(entity) = entity else {
                    return Err(format!(
                        "the fields of error `{error}`, which the specification gives no source"
                    ));
                };
                held_field(entity, field).ok_or_else(|| {
                    format!(
                        "the field `{}` of error `{error}`, which neither the specification nor \
                         the held `{}` determines",
                        field.name, entity.name
                    )
                })?;
            }
        }
    }
    let Some(subject) = &outcome.subject else {
        if !outcome.sets.is_empty() {
            return Err("`sets:` without a subject".to_owned());
        }
        return payloads(ir, command, outcome, None);
    };
    let entity = ir.entity(&subject.entity);
    match (&subject.effect, &subject.instance) {
        (ResolvedEffect::Creates, ResolvedInstance::Observed { .. }) => {
            for field in &entity.fields {
                let set = outcome.sets.iter().find(|set| set.target == field.name);
                if set.is_none() && !field.type_ref.is_optional() {
                    return Err(format!(
                        "`creates:` leaving the required field `{}` of `{}` undetermined",
                        field.name, entity.name
                    ));
                }
            }
        }
        (ResolvedEffect::Creates, ResolvedInstance::Supplied { .. }) => {
            return Err("a creation whose identity the caller supplies".to_owned());
        }
        (_, ResolvedInstance::Observed { .. }) => {
            return Err(format!(
                "a `{}` whose identity is observed",
                subject.effect.verb()
            ));
        }
        (_, ResolvedInstance::Supplied { .. }) => {}
    }
    let held = !matches!(subject.effect, ResolvedEffect::Creates);
    for set in &outcome.sets {
        let target = entity
            .fields
            .iter()
            .find(|field| field.name == set.target)
            .ok_or_else(|| format!("a `sets:` of `{}`, not a field of the entity", set.target))?;
        value(ir, command, set, held.then_some(entity), true)?;
        if matches!(set.value, ResolvedPayloadValue::Increment { .. })
            && !integer(ir, &target.type_ref)
        {
            return Err(format!(
                "`{{increment:}}` of `{}`, which is not an `Integer`",
                set.target
            ));
        }
    }
    payloads(ir, command, outcome, held.then_some(entity))
}

/// Every emitted event's payload sources on one branch.
fn payloads(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    held: Option<&ResolvedEntity>,
) -> Result<(), String> {
    for payload in &outcome.payload {
        for field in &payload.fields {
            if matches!(field.value, ResolvedPayloadValue::Cleared) {
                return Err("`{cleared}` on an event payload".to_owned());
            }
            value(ir, command, field, held, false)?;
        }
    }
    Ok(())
}

/// One value source, checked against the field it fills.
fn value(
    ir: &EssIr,
    command: &ResolvedCommand,
    field: &ResolvedPayloadField,
    held: Option<&ResolvedEntity>,
    sets: bool,
) -> Result<(), String> {
    if field.conversion.is_some() {
        return Err(format!(
            "a declared conversion into `{}` (`{}`)",
            field.target,
            field.value.describe()
        ));
    }
    let target = &field.target_type;
    match &field.value {
        ResolvedPayloadValue::InputField { type_ref, .. }
        | ResolvedPayloadValue::CallerAttribute { type_ref, .. } => {
            if !assignable(type_ref, target) {
                return Err(format!("a value of another type for `{}`", field.target));
            }
        }
        ResolvedPayloadValue::SubjectField { type_ref, .. } => {
            if held.is_none() {
                return Err(format!(
                    "`{{subject:}}` for `{}` on a branch that holds no subject",
                    field.target
                ));
            }
            if !assignable(type_ref, target) {
                return Err(format!("a value of another type for `{}`", field.target));
            }
        }
        ResolvedPayloadValue::Literal { value } => literal(ir, target, value)?,
        ResolvedPayloadValue::Generated | ResolvedPayloadValue::Cleared => {}
        ResolvedPayloadValue::Increment { by } => {
            if held.is_none() || !sets {
                return Err(format!(
                    "`{{increment:}}` for `{}` with no previous value",
                    field.target
                ));
            }
            by.parse::<i64>()
                .map_err(|_| format!("`{{increment: {by}}}`, which is not a whole number"))?;
        }
        ResolvedPayloadValue::InputOrGenerated {
            type_ref,
            otherwise,
            ..
        } => {
            let inner = type_ref.required();
            if inner != target.required() {
                return Err(format!("a value of another type for `{}`", field.target));
            }
            if let Some(otherwise) = otherwise {
                literal(ir, target, otherwise)?;
            }
        }
        ResolvedPayloadValue::Struct { fields } => {
            let ResolvedTypeRef::Declared { name } = target.required() else {
                return Err(format!("a struct source for `{}`", field.target));
            };
            let ResolvedBody::Struct {
                fields: declared, ..
            } = &ir.named_type(name).body
            else {
                return Err(format!("a struct source for `{}`", field.target));
            };
            for member in declared {
                if !fields.iter().any(|source| source.target == member.name)
                    && !member.type_ref.is_optional()
                {
                    return Err(format!(
                        "a struct source for `{}` leaving `{}` undetermined",
                        field.target, member.name
                    ));
                }
            }
            for source in fields {
                value(ir, command, source, held, sets)?;
            }
        }
        ResolvedPayloadValue::RelatedField { .. } => {
            return Err(format!("`{{related:}}` for `{}`", field.target));
        }
        ResolvedPayloadValue::ChangedCount => {
            return Err(format!("`{{count: changed}}` for `{}`", field.target));
        }
        ResolvedPayloadValue::ResponseField { .. } => {
            return Err(format!("a response field for `{}`", field.target));
        }
    }
    let _ = command;
    Ok(())
}

/// `true` where a value of `source` fills a field of `target` as it is, or wrapped as present.
pub(crate) fn assignable(source: &ResolvedTypeRef, target: &ResolvedTypeRef) -> bool {
    source == target || matches!(target, ResolvedTypeRef::Optional { of } if of.as_ref() == source)
}

/// `true` where the type is an `Integer` under any newtypes.
fn integer(ir: &EssIr, type_ref: &ResolvedTypeRef) -> bool {
    matches!(
        representation(ir, type_ref),
        Some(Leaf::Primitive(Primitive::Integer))
    )
}

/// What a literal written in the model is, read against the field it fills.
pub(crate) fn literal(ir: &EssIr, target: &ResolvedTypeRef, text: &str) -> Result<(), String> {
    let refuse = || Err(format!("the literal `{text}` for `{target}`"));
    match representation(ir, target.required()) {
        Some(Leaf::Primitive(Primitive::Integer)) => {
            if text.parse::<i64>().is_err() {
                return refuse();
            }
        }
        Some(Leaf::Primitive(Primitive::Boolean)) => {
            if text != "true" && text != "false" {
                return refuse();
            }
        }
        Some(Leaf::Primitive(
            Primitive::String
            | Primitive::Uuid
            | Primitive::Timestamp
            | Primitive::Duration
            | Primitive::Decimal,
        )) => {}
        Some(Leaf::Enum(variants)) => {
            if !variants.iter().any(|variant| variant == text) {
                return refuse();
            }
        }
        _ => return refuse(),
    }
    Ok(())
}

/// What a type is at its leaf, under every `Optional` and newtype, where it is a scalar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Leaf {
    /// A primitive.
    Primitive(Primitive),
    /// A closed set of names.
    Enum(Vec<String>),
}

/// The leaf of `type_ref`, or `None` for a struct, union, list or map.
pub(crate) fn representation(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<Leaf> {
    let mut current = type_ref;
    for _ in 0..=ess_domain::types::MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Optional { of } => current = of,
            ResolvedTypeRef::Primitive { name } => return Some(Leaf::Primitive(*name)),
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                ResolvedBody::Enum { variants } => return Some(Leaf::Enum(names(variants))),
                _ => return None,
            },
            ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => return None,
        }
    }
    None
}

/// The names of an enum's variants, as the model spells them.
fn names(variants: &[ess_domain::types::EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

/// A branch's subject guards read the addressed row before selection (step 3 of the precedence
/// order): `when_subject`, `when_subject_state`, `when_state_changes`.
pub(crate) fn subject_guarded(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::SubjectField { .. }
                | ResolvedCondition::SubjectPredicate { .. }
                | ResolvedCondition::SubjectState { .. }
                | ResolvedCondition::StateChange { .. }
        )
    })
}

/// The one supplied subject a subject-guarded command reads: the first branch that names one.
pub(crate) fn selection_subject(command: &ResolvedCommand) -> Option<&ResolvedSubject> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }))
}

/// Two subjects naming one entity through one input field.
fn same_subject(left: &ResolvedSubject, right: &ResolvedSubject) -> bool {
    left.entity == right.entity && left.instance.field().name == right.instance.field().name
}

/// The entity a `wrong_state:` branch answers for: the one every move of the command addresses.
pub(crate) fn wrong_state_entity<'ir>(
    ir: &'ir EssIr,
    command: &ResolvedCommand,
) -> Option<&'ir ResolvedEntity> {
    selection_subject(command).map(|subject| ir.entity(&subject.entity))
}

/// The branch that answers an identity no record carries, as the interpreter picks it: the
/// `unknown_instance:` branch, else the `wrong_state:` branch, else none.
pub(crate) fn unknown_answer(command: &ResolvedCommand) -> Option<&ResolvedOutcome> {
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::UnknownInstance)
        .or_else(|| {
            command
                .outcomes
                .iter()
                .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
        })
}

/// Where an error field is read from the held instance: its lifecycle state for a field of the
/// entity's state type, else the entity field (or identity) of the same name and type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeldField<'e> {
    /// The state the instance rests in.
    State,
    /// A stored field, the identity included.
    Field(&'e ResolvedField),
}

/// The held source of one error field, or `None` where the held instance does not determine it.
pub(crate) fn held_field<'e>(
    entity: &'e ResolvedEntity,
    field: &ResolvedField,
) -> Option<HeldField<'e>> {
    if matches!(&field.type_ref, ResolvedTypeRef::Declared { name } if *name == entity.state_type) {
        return Some(HeldField::State);
    }
    std::iter::once(&entity.identity)
        .chain(&entity.fields)
        .find(|stored| stored.name == field.name && stored.type_ref == field.type_ref)
        .map(HeldField::Field)
}

// ---- guards --------------------------------------------------------------------------------------

/// What a guard's paths are read against.
pub(crate) enum Env<'a> {
    /// A command's input, and `caller.` attributes.
    Input(&'a ResolvedCommand),
    /// The addressed row's stored fields and `state`, with the input under `input.` and the caller
    /// under `caller.`.
    Subject(&'a ResolvedCommand, &'a ResolvedEntity),
}

/// Where a resolved path starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Root {
    /// An input field.
    Input(String),
    /// A stored field or the identity of the addressed row.
    Stored(String),
    /// The lifecycle state the addressed row rests in.
    State,
    /// An attribute of the authenticated caller.
    Caller(String),
}

/// One step from a value to the next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Step {
    /// Through an `Optional`: absent is Unknown.
    Optional,
    /// Into a newtype's representation.
    Newtype,
    /// A struct member.
    Field(String),
    /// How many elements a list or map holds.
    Count,
}

/// What a path reads, as a guard compares it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    /// An `Integer` or `Decimal` (or a count), compared by value.
    Number(Primitive),
    /// A `String` or `Uuid`, compared for equality by its text.
    Text,
    /// An enum, compared for equality by its variant name.
    Enum(ResolvedTypeRef, Vec<String>),
    /// A lifecycle state.
    State,
    /// A `Boolean`.
    Bool,
    /// Anything else: only `defined()` reads it.
    Opaque,
}

/// A path, resolved against the types it walks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Resolved {
    /// Where it starts.
    pub root: Root,
    /// The type at the root.
    pub root_type: ResolvedTypeRef,
    /// Every step after the root.
    pub steps: Vec<Step>,
    /// What it reads.
    pub kind: Kind,
}

/// Resolves one fact path against `env`.
// One walk of a dotted path through every type shape it can cross, kept together so the rules
// an emitter renders are read in one place.
#[allow(clippy::too_many_lines)]
pub(crate) fn resolve(ir: &EssIr, env: &Env<'_>, path: &FactPath) -> Result<Resolved, String> {
    let segments = path.segments();
    let unknown = || format!("the guard path `{path}`");
    let caller_type = |attribute: &str| {
        ir.actors()
            .values()
            .flat_map(|actor| &actor.attributes)
            .find(|declared| declared.name == attribute)
            .map(|declared| declared.type_ref.clone())
    };
    let input_type = |command: &ResolvedCommand, name: &str| {
        command
            .input
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    };
    let (root, root_type, rest) = match (env, segments) {
        (_, [first, attribute, rest @ ..]) if first == "caller" => (
            Root::Caller(attribute.clone()),
            caller_type(attribute).ok_or_else(unknown)?,
            rest,
        ),
        (Env::Input(command), [first, rest @ ..]) => match input_type(command, first) {
            Some(type_ref) => (Root::Input(first.clone()), type_ref, rest),
            None => match (first.as_str(), rest) {
                ("input", [field, rest @ ..]) => (
                    Root::Input(field.clone()),
                    input_type(command, field).ok_or_else(unknown)?,
                    rest,
                ),
                _ => return Err(unknown()),
            },
        },
        (Env::Subject(command, entity), [first, rest @ ..]) => {
            if first == "input" {
                let [field, rest @ ..] = rest else {
                    return Err(unknown());
                };
                (
                    Root::Input(field.clone()),
                    input_type(command, field).ok_or_else(unknown)?,
                    rest,
                )
            } else if first == "state" {
                (
                    Root::State,
                    ResolvedTypeRef::Declared {
                        name: entity.state_type.clone(),
                    },
                    rest,
                )
            } else {
                let stored = std::iter::once(&entity.identity)
                    .chain(&entity.fields)
                    .find(|field| &field.name == first)
                    .ok_or_else(unknown)?;
                (Root::Stored(first.clone()), stored.type_ref.clone(), rest)
            }
        }
        (_, []) => return Err(unknown()),
    };
    let mut steps = Vec::new();
    let mut current = root_type.clone();
    let mut remaining = rest.iter();
    let mut pending = remaining.next();
    for _ in 0..=(ess_domain::types::MAX_TYPE_DEPTH + segments.len()) {
        match &current {
            ResolvedTypeRef::Optional { of } => {
                steps.push(Step::Optional);
                current = (**of).clone();
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    steps.push(Step::Newtype);
                    current = of.clone();
                }
                ResolvedBody::Struct { fields, .. } => {
                    let Some(segment) = pending else {
                        return Ok(Resolved {
                            root,
                            root_type,
                            steps,
                            kind: Kind::Opaque,
                        });
                    };
                    let member = fields
                        .iter()
                        .find(|field| &field.name == segment)
                        .ok_or_else(unknown)?;
                    steps.push(Step::Field(member.name.clone()));
                    current = member.type_ref.clone();
                    pending = remaining.next();
                }
                ResolvedBody::Enum { variants } => {
                    if pending.is_some() {
                        return Err(unknown());
                    }
                    let kind = if root == Root::State && steps.is_empty() {
                        Kind::State
                    } else {
                        Kind::Enum(current.clone(), names(variants))
                    };
                    return Ok(Resolved {
                        root,
                        root_type,
                        steps,
                        kind,
                    });
                }
                ResolvedBody::Union { .. } => {
                    if pending.is_some() {
                        return Err(format!("a guard reading into a union (`{path}`)"));
                    }
                    return Ok(Resolved {
                        root,
                        root_type,
                        steps,
                        kind: Kind::Opaque,
                    });
                }
            },
            ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. } => match pending {
                Some(segment) if segment == "count" && remaining.clone().next().is_none() => {
                    steps.push(Step::Count);
                    return Ok(Resolved {
                        root,
                        root_type,
                        steps,
                        kind: Kind::Number(Primitive::Integer),
                    });
                }
                Some(_) => return Err(format!("a guard reading a collection element (`{path}`)")),
                None => {
                    return Ok(Resolved {
                        root,
                        root_type,
                        steps,
                        kind: Kind::Opaque,
                    })
                }
            },
            ResolvedTypeRef::Primitive { name } => {
                if pending.is_some() {
                    return Err(unknown());
                }
                let kind = match name {
                    Primitive::Integer | Primitive::Decimal => Kind::Number(*name),
                    Primitive::String | Primitive::Uuid => Kind::Text,
                    Primitive::Boolean => Kind::Bool,
                    _ => Kind::Opaque,
                };
                return Ok(Resolved {
                    root,
                    root_type,
                    steps,
                    kind,
                });
            }
        }
    }
    Err(unknown())
}

/// `Ok` where every part of `predicate` is something an emitter decides with the evaluator's
/// three-valued semantics; `Err` names the first part that is not.
pub(crate) fn supported(ir: &EssIr, env: &Env<'_>, predicate: &Predicate) -> Result<(), String> {
    match predicate {
        Predicate::Always | Predicate::Never => Ok(()),
        Predicate::All(children) | Predicate::Any(children) => children
            .iter()
            .try_for_each(|child| supported(ir, env, child)),
        Predicate::Not(child) => supported(ir, env, child),
        Predicate::Defined(path) => resolve(ir, env, path).map(|_| ()),
        Predicate::Truthy(path) => match resolve(ir, env, path)?.kind {
            Kind::Bool => Ok(()),
            _ => Err(format!(
                "a truthiness test of `{path}`, which is not a `Boolean`"
            )),
        },
        Predicate::Compare { left, op, right } => {
            let kind = |operand: &Operand| match operand {
                Operand::Fact(path) => resolve(ir, env, path).map(|it| Some(it.kind)),
                Operand::Literal(_) => Ok(None),
            };
            let (left_kind, right_kind) = (kind(left)?, kind(right)?);
            let compared = match (&left_kind, &right_kind) {
                (Some(kind), None) => {
                    literal_matches(kind, literal_of(right))?;
                    kind
                }
                (None, Some(kind)) => {
                    literal_matches(kind, literal_of(left))?;
                    kind
                }
                (Some(left), Some(right)) => {
                    if !comparable(left, right) {
                        return Err(format!("`{predicate}`, comparing two kinds of value"));
                    }
                    left
                }
                (None, None) => return Err(format!("`{predicate}`, comparing two literals")),
            };
            match (compared, op) {
                (Kind::Number(_), _)
                | (
                    Kind::Text | Kind::Enum(..) | Kind::State | Kind::Bool,
                    CompareOp::Eq | CompareOp::Ne,
                ) => Ok(()),
                (Kind::Opaque, _) => Err(format!("`{predicate}`, over a value no guard compares")),
                _ => Err(format!("`{predicate}`, an ordering over text")),
            }
        }
        Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
            let kind = resolve(ir, env, path)?.kind;
            values
                .iter()
                .try_for_each(|value| literal_matches(&kind, Some(value)))
        }
        Predicate::TextMatch { path, value, .. } => match (resolve(ir, env, path)?.kind, value) {
            (Kind::Text, FactValue::Text(_)) => Ok(()),
            _ => Err(format!(
                "`{predicate}`, a text test over a value that is not text"
            )),
        },
        _ => Err(format!("the guard `{predicate}`")),
    }
}

/// The literal an operand is, where it is one.
fn literal_of(operand: &Operand) -> Option<&FactValue> {
    match operand {
        Operand::Literal(value) => Some(value),
        Operand::Fact(_) => None,
    }
}

/// `Ok` where a literal is a value of what a path reads.
fn literal_matches(kind: &Kind, value: Option<&FactValue>) -> Result<(), String> {
    match (kind, value) {
        (Kind::Number(_), Some(FactValue::Number(_))) | (Kind::Bool, Some(FactValue::Bool(_))) => {
            Ok(())
        }
        (Kind::Text | Kind::Enum(..) | Kind::State, Some(FactValue::Text(text))) => {
            if CurrentTime::parse(text).is_some() {
                Err(format!("a guard reading the current time (`{text}`)"))
            } else {
                Ok(())
            }
        }
        (Kind::Enum(_, variants), Some(FactValue::Number(_) | FactValue::Bool(_))) => Err(format!(
            "a literal no variant of {} spells",
            variants.join(", ")
        )),
        _ => Err("a literal of another kind than the value it is compared with".to_owned()),
    }
}

/// Two paths a comparison may put side by side.
fn comparable(left: &Kind, right: &Kind) -> bool {
    match (left, right) {
        (Kind::Number(_), Kind::Number(_))
        | (Kind::Text, Kind::Text)
        | (Kind::Bool, Kind::Bool)
        | (Kind::State, Kind::State) => true,
        (Kind::Enum(left, _), Kind::Enum(right, _)) => left == right,
        _ => false,
    }
}

/// `true` where the plan marks this command's behaviour generated.
pub(crate) fn generated(ir: &EssIr, command: &ResolvedCommand) -> bool {
    self::command(ir, command).is_ok()
}

/// `true` where any command of the model has a generated behaviour.
pub(crate) fn any_generated(ir: &EssIr) -> bool {
    ir.commands().values().any(|command| generated(ir, command))
}
