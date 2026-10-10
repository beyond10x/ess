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
    EntityHandle, EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedEntity, ResolvedFallback, ResolvedField, ResolvedInstance, ResolvedOutcome,
    ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedTest, ResolvedRelatedVia,
    ResolvedSubject, ResolvedTypeRef,
};
use ess_domain::types::Primitive;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Derived, Operand, Predicate};
use ess_primitives::time::CurrentTime;

/// `Err` for a command with a refusal that changes its addressed row before answering (ess/22,
/// `compensates: true`, beyond10x/ess#197): no emitter writes a refusal with an effect, and one
/// that answered the error alone would be the service that stopped rolling back. Owed by name.
fn uncompensated(command: &ResolvedCommand) -> Result<(), String> {
    match command.outcomes.iter().find(|outcome| outcome.compensates) {
        Some(outcome) => Err(format!(
            "a refusal that compensates (`compensates: true`), on `{}`",
            outcome.name
        )),
        None => Ok(()),
    }
}

/// `Ok` when every outcome of `command` is expressible; `Err` names the construct that keeps the
/// whole command an obligation, in a phrase that reads after "kept an obligation by".
pub(crate) fn command(ir: &EssIr, command: &ResolvedCommand) -> Result<(), String> {
    if !command.response.is_empty() {
        return Err("a typed response (`response:`)".to_owned());
    }
    uncompensated(command)?;
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
    existence_identity(command)?;
    related_composition(ir, command).and_then(|()| rekey_composition(ir, command, guarded))?;
    for outcome in &command.outcomes {
        self::outcome(ir, command, outcome, guarded, selection)
            .map_err(|construct| format!("{construct}, in `{}`", outcome.name))?;
    }
    Ok(())
}

/// `Err` for a re-key (ess/23, beyond10x/ess#429) beside a guard over the addressed or a related
/// row: a re-key is generated where the addressed row is read by its branch alone, and the
/// collision refusal answers before it.
fn rekey_composition(ir: &EssIr, command: &ResolvedCommand, guarded: bool) -> Result<(), String> {
    if command
        .outcomes
        .iter()
        .any(|outcome| outcome.identity_write(ir).is_some())
        && (guarded || related(command).is_some())
    {
        return Err(
            "a re-key (`updates:` writing the identity) beside a guard over the addressed or a \
             related row"
                .to_owned(),
        );
    }
    Ok(())
}

/// Whether `outcome` is the collision refusal of a re-key `command` declares (ess/23,
/// beyond10x/ess#429): the one row-set guard an emitter answers, by looking the written identity up
/// through the entity's storage port.
pub(crate) fn collision_answer(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
) -> bool {
    command.outcomes.iter().any(|other| {
        command
            .collision_answer(ir, other)
            .is_some_and(|answer| answer.name == outcome.name)
    })
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
        // The collision refusal of a re-key (ess/23, beyond10x/ess#429) selects the rows whose
        // identity is the one written: one lookup by identity through the storage port, which the
        // behaviour makes.
        ResolvedCondition::RelatedSet { .. } if collision_answer(ir, command, outcome) => {}
        // Generated storage enumerates no rows by a selector in this cut (ess/22, beyond10x/ess#228,
        // #299): the command stays an obligation, named.
        ResolvedCondition::RelatedSet { .. } => {
            return Err(
                "a guard over the rows a selector selects (`when_related: {entity, where, \
                 exists | count | forall}`)"
                    .to_owned(),
            )
        }
        ResolvedCondition::Related {
            entity,
            test,
            input,
            ..
        } => {
            // The predicate reads the related row as a `when_subject` predicate reads the
            // addressed one: its stored fields and `state`, the input under `input.`, the caller.
            if let ResolvedRelatedTest::Holds { predicate } = test {
                supported(ir, &Env::Subject(command, ir.entity(entity)), predicate)?;
            }
            if let Some(input) = input {
                supported(ir, &Env::Input(command), input)?;
            }
        }
        ResolvedCondition::InputAbsent => return Err("`input_absent:`".to_owned()),
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
        | ResolvedCondition::UnknownInstance
        // Selected by the storage lookup before any branch is taken (beyond10x/ess#310).
        | ResolvedCondition::ExistingInstance => {}
    }
    // The creating half of create-or-update acts: it is the creation an unknown identity takes.
    let answered_for_subject = matches!(
        outcome.condition,
        ResolvedCondition::WrongState | ResolvedCondition::UnknownInstance
    ) && !creates_unknown(outcome);
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
    // The identity is written by a re-key (ess/23, beyond10x/ess#429), and by nothing else.
    let rekey = outcome
        .identity_write(ir)
        .map(|write| write.target.as_str());
    for set in &outcome.sets {
        if rekey != Some(set.target.as_str()) {
            entity
                .fields
                .iter()
                .find(|field| field.name == set.target)
                .ok_or_else(|| {
                    format!("a `sets:` of `{}`, not a field of the entity", set.target)
                })?;
        }
        value(ir, command, set, held.then_some(entity), true)?;
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
// One arm per value source a branch can write.
#[allow(clippy::too_many_lines)]
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
        // The held lifecycle state (ess/23, beyond10x/ess#458), read as the row is.
        ResolvedPayloadValue::SubjectState { type_ref } => {
            if held.is_none() {
                return Err(format!(
                    "`{{subject: state}}` for `{}` on a branch that holds no subject",
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
            if !integer(ir, target) {
                return Err(format!(
                    "`{{increment:}}` of `{}`, which is not an `Integer`",
                    field.target
                ));
            }
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
            fallback(ir, field, otherwise.as_ref())?;
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
        // Every `{related:}` value stays owed; an ess/22 form (beyond10x/ess#285) says which, so a
        // reader knows the generated behaviour would have to follow two rows or an absent one.
        ResolvedPayloadValue::RelatedField { via, through, .. } => {
            let form = if !through.is_empty() {
                " across two references"
            } else if via.type_ref().is_optional() {
                " through an Optional reference"
            } else {
                ""
            };
            return Err(format!("`{{related:}}`{form} for `{}`", field.target));
        }
        ResolvedPayloadValue::ChangedCount => {
            return Err(format!("`{{count: changed}}` for `{}`", field.target));
        }
        // Generated storage enumerates no rows by a selector in this cut (ess/22, beyond10x/ess#299).
        ResolvedPayloadValue::RelatedSelection { .. } => {
            return Err(format!(
                "a read of the one row a selector selects (`{{related: {{entity, where, field}}}}`) \
                 for `{}`",
                field.target
            ));
        }
        ResolvedPayloadValue::ResponseField { .. } => {
            return Err(format!("a response field for `{}`", field.target));
        }
    }
    let _ = command;
    Ok(())
}

/// `true` where a value of `source` fills a field of `target` as it is, or wrapped as present.
/// Whether what stands in for an absent input after `else:` fills `field`: a literal its type
/// reads, or (ess/22, A4) another input of its type.
fn fallback(
    ir: &EssIr,
    field: &ResolvedPayloadField,
    otherwise: Option<&ResolvedFallback>,
) -> Result<(), String> {
    let target = &field.target_type;
    match otherwise {
        Some(ResolvedFallback::Literal(otherwise)) => literal(ir, target, otherwise),
        Some(ResolvedFallback::Input { input })
            if !assignable(&input.type_ref, target.required()) =>
        {
            Err(format!("a fallback of another type for `{}`", field.target))
        }
        Some(ResolvedFallback::Input { .. }) | None => Ok(()),
    }
}

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
    /// One stored row of an entity, as a view's `filter:` reads it: its fields and `state`, and
    /// nothing else — no input, no caller.
    Row(&'a ResolvedEntity),
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
    /// A parameter of the view whose query reads it, compared with by a string operator
    /// (beyond10x/ess#200).
    Param(String),
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
    /// How many bytes the UTF-8 encoding of a `String` takes (`docs/design/expression-family-source22.md`,
    /// decision 11): `str::len` in Rust, `len` of a string that is UTF-8 in Go.
    Utf8Bytes,
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
    /// A `Timestamp`, compared with another by the instant each names
    /// (`docs/design/expression-family-source22.md`, decision 2).
    Instant,
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
        (Env::Row(entity), [first, rest @ ..]) => {
            if first == "state" {
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
    walk(ir, path, root, root_type, rest)
}

/// A view parameter a string operator compares with (beyond10x/ess#200), resolved as a fact path
/// rooted at the generated query's argument of that name.
pub(crate) fn resolve_param(ir: &EssIr, param: &ResolvedField) -> Result<Resolved, String> {
    let path = FactPath::from_segments([ess_domain::view::ViewSpec::PARAM, param.name.as_str()]);
    walk(
        ir,
        &path,
        Root::Param(param.name.clone()),
        param.type_ref.clone(),
        &[],
    )
}

/// The steps from `root`, of type `root_type`, along the segments `rest` of `path`.
// One walk of a dotted path through every type shape it can cross, kept together so the rules
// an emitter renders are read in one place.
#[allow(clippy::too_many_lines)]
fn walk(
    ir: &EssIr,
    path: &FactPath,
    root: Root,
    root_type: ResolvedTypeRef,
    rest: &[String],
) -> Result<Resolved, String> {
    let segments = path.segments();
    let unknown = || format!("the guard path `{path}`");
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
                    // From ess/22 a `Timestamp` compares by its instant (decision 2); below it, it
                    // stays the value no guard compares, with every reason it had.
                    Primitive::Timestamp
                        if ir.format().major()
                            >= ess_domain::system::FormatVersion::V22.major() =>
                    {
                        Kind::Instant
                    }
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

/// Resolves a derived operand (`docs/design/expression-family-source22.md`, decision 11): the UTF-8
/// byte length of a `String` read at its parent, one [`Step::Utf8Bytes`] past it, compared as an
/// `Integer`. A view filter's is refused by name: no generated query measures text.
pub(crate) fn resolve_derived(
    ir: &EssIr,
    env: &Env<'_>,
    derived: &Derived,
) -> Result<Resolved, String> {
    if matches!(env, Env::Row(_)) {
        return Err(format!(
            "`{derived}`, a byte length the generated view query does not compare"
        ));
    }
    let Derived::Utf8Bytes(parent) = derived;
    let mut resolved = resolve(ir, env, parent)?;
    if resolved.kind != Kind::Text || leaf_primitive(ir, &resolved) != Some(Primitive::String) {
        return Err(format!(
            "`{derived}`, the byte length of a value that is no String"
        ));
    }
    resolved.steps.push(Step::Utf8Bytes);
    resolved.kind = Kind::Number(Primitive::Integer);
    Ok(resolved)
}

/// Whether any predicate of the model compares the UTF-8 byte length of a text, `{utf8_bytes: …}`
/// (decision 11). Read off the canonical IR, where that mapping is the one `utf8_bytes` key whose
/// value is a path: a member named `utf8_bytes` is written inside a path or as a field's name.
pub(crate) fn reads_utf8_bytes(ir: &EssIr) -> bool {
    serde_json::to_string(ir).is_ok_and(|text| text.contains("{\"utf8_bytes\":\""))
}

/// The primitive a resolved path ends in, under its newtypes and `Optional`s.
pub(crate) fn leaf_primitive(ir: &EssIr, resolved: &Resolved) -> Option<Primitive> {
    let mut current = resolved.root_type.clone();
    for step in &resolved.steps {
        current = match (step, &current) {
            (Step::Optional, ResolvedTypeRef::Optional { of }) => (**of).clone(),
            (Step::Newtype, ResolvedTypeRef::Declared { name }) => {
                match &ir.named_type(name).body {
                    ResolvedBody::Newtype { of, .. } => of.clone(),
                    _ => return None,
                }
            }
            (Step::Field(field), ResolvedTypeRef::Declared { name }) => {
                match &ir.named_type(name).body {
                    ResolvedBody::Struct { fields, .. } => fields
                        .iter()
                        .find(|member| &member.name == field)?
                        .type_ref
                        .clone(),
                    _ => return None,
                }
            }
            _ => return None,
        };
    }
    match current {
        ResolvedTypeRef::Primitive { name } => Some(name),
        _ => None,
    }
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
        Predicate::Compare {
            right: Operand::Offset(offset),
            left,
            ..
        } => offset_supported(ir, env, predicate, left, offset),
        Predicate::Compare {
            left: Operand::Offset(_),
            ..
        } => Err(format!("`{predicate}`, an offset on the left")),
        Predicate::Compare {
            left, op, right, ..
        } => {
            let kind = |operand: &Operand| match operand {
                Operand::Fact(path) => resolve(ir, env, path).map(|it| Some(it.kind)),
                Operand::Derived(derived) => {
                    resolve_derived(ir, env, derived).map(|it| Some(it.kind))
                }
                Operand::Literal(_) | Operand::Offset(_) => Ok(None),
            };
            let (left_kind, right_kind) = (kind(left)?, kind(right)?);
            let compared = match (&left_kind, &right_kind) {
                // An instant (ess/22, see `resolve`) is ordered against the current time on its
                // right, read from the decision's one instant ([`reads_clock`], family F A3), or
                // compared with another fact; a literal instant has no generated reading here.
                (Some(Kind::Instant), None) if current_time(right, *op).is_some() => return Ok(()),
                (Some(Kind::Instant), None) | (None, Some(Kind::Instant)) => {
                    return Err(format!("`{predicate}`, over a value no guard compares"))
                }
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
                (Kind::Number(_) | Kind::Instant, _)
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
        Predicate::TextMatch { path, value, .. } => {
            text_match_supported(ir, env, predicate, path, value)
        }
        // No generated guard, filter or selection compares keys across a list's elements (ess/22,
        // `docs/design/expression-family-source22.md`, `distinct`): owed by name, never decided.
        Predicate::Distinct(_) => Err(format!(
            "`{predicate}`, distinct list members no generated behaviour compares"
        )),
        // No generated behaviour evaluates a calendar window (`docs/design/calendar-window-guards.md`):
        // the command stays owed, naming it, rather than rendered without it.
        Predicate::Window(_) => Err(format!(
            "`{predicate}`, a calendar window no generated guard evaluates"
        )),
        _ => Err(format!("the guard `{predicate}`")),
    }
}

/// `Ok` where a string operator tests a text against a text literal, or against an operand the
/// generated code reads where the predicate sits (beyond10x/ess#200): a command's input in its
/// guards, a view's parameter in its query's filter.
fn text_match_supported(
    ir: &EssIr,
    env: &Env<'_>,
    predicate: &Predicate,
    path: &FactPath,
    value: &ess_primitives::predicate::TextOperand,
) -> Result<(), String> {
    use ess_primitives::predicate::{TextNamespace, TextOperand};
    let not_text = || format!("`{predicate}`, a text test over a value that is not text");
    if resolve(ir, env, path)?.kind != Kind::Text {
        return Err(not_text());
    }
    match (value, env) {
        // A view's parameter: the query takes it as an argument, and `view_query` holds it to a
        // text the query string carries.
        (TextOperand::Literal(FactValue::Text(_)), _)
        | (
            TextOperand::Fact {
                namespace: TextNamespace::Param,
                ..
            },
            Env::Row(_),
        ) => Ok(()),
        (TextOperand::Literal(_), _) => Err(not_text()),
        // A command's input, compared with as it is read.
        (
            TextOperand::Fact {
                namespace: TextNamespace::Input,
                path: read,
                ..
            },
            Env::Input(_) | Env::Subject(..),
        ) => match resolve(ir, env, read)?.kind {
            Kind::Text => Ok(()),
            _ => Err(not_text()),
        },
        (TextOperand::Fact { .. }, _) => Err(format!(
            "`{predicate}`, a text test against an operand the generated code does not read here"
        )),
    }
}

/// `Ok` where a guard compares a fact with one constant offset of another the way a generated
/// behaviour decides it (`docs/design/expression-family-source22.md`, A2): an Integer magnitude
/// between two `Integer` reads, compared exactly; an elapsed one between two `Timestamp` reads, as
/// instants. A view filter's offset is refused by name: no generated query compares one.
fn offset_supported(
    ir: &EssIr,
    env: &Env<'_>,
    predicate: &Predicate,
    left: &Operand,
    offset: &ess_primitives::predicate::OffsetOperand,
) -> Result<(), String> {
    use ess_primitives::predicate::OffsetMagnitude;
    if matches!(env, Env::Row(_)) {
        return Err(format!(
            "`{predicate}`, an offset the generated view query does not compare"
        ));
    }
    let Operand::Fact(path) = left else {
        return Err(format!("`{predicate}`, an offset compared with a literal"));
    };
    let (left, base) = (
        resolve(ir, env, path)?.kind,
        resolve(ir, env, &offset.base)?.kind,
    );
    let integer = Kind::Number(Primitive::Integer);
    match offset.magnitude {
        OffsetMagnitude::Integer(_) if left == integer && base == integer => Ok(()),
        OffsetMagnitude::ElapsedSeconds { .. }
            if left == Kind::Instant && base == Kind::Instant =>
        {
            Ok(())
        }
        _ => Err(format!(
            "`{predicate}`, an offset over values no generated guard moves"
        )),
    }
}

/// The current-time operand an ordering's right-hand literal is, where it is one: `now`, moved by
/// a whole number of seconds (ess/16 in an input guard; ess/22 in a stored row's predicate too).
pub(crate) fn current_time(operand: &Operand, op: CompareOp) -> Option<CurrentTime> {
    match operand {
        Operand::Literal(FactValue::Text(text)) if op.needs_ordering() => CurrentTime::parse(text),
        _ => None,
    }
}

/// Whether a guard reads the current time anywhere in it.
fn predicate_reads_clock(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().any(predicate_reads_clock)
        }
        Predicate::Not(child) => predicate_reads_clock(child),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            predicate_reads_clock(&quantified.body)
        }
        Predicate::Compare { op, right, .. } => current_time(right, *op).is_some(),
        _ => false,
    }
}

/// `true` where a generated behaviour of `command` reads the decision's one instant: some guard of
/// it — an input guard, a `when_subject:` predicate or a `when_related:` predicate — orders an
/// instant against the current time (`docs/design/expression-family-source22.md`, A3). The
/// behaviour reads the command clock once, before any guard, and every such guard reads that one
/// instant.
pub(crate) fn reads_clock(command: &ResolvedCommand) -> bool {
    command.outcomes.iter().any(|outcome| {
        let (stored, input) = match &outcome.condition {
            ResolvedCondition::When { predicate }
            | ResolvedCondition::ExternalWhen { predicate, .. } => (None, Some(predicate)),
            ResolvedCondition::SubjectState { predicate, .. }
            | ResolvedCondition::StateChange { predicate, .. }
            | ResolvedCondition::SubjectField { predicate, .. } => (None, predicate.as_ref()),
            ResolvedCondition::SubjectPredicate { predicate, input } => {
                (Some(predicate), input.as_ref())
            }
            ResolvedCondition::Related { test, input, .. } => (
                match test {
                    ResolvedRelatedTest::Holds { predicate } => Some(predicate),
                    ResolvedRelatedTest::Absent => None,
                },
                input.as_ref(),
            ),
            _ => (None, None),
        };
        stored.into_iter().chain(input).any(predicate_reads_clock)
    })
}

/// The literal an operand is, where it is one.
fn literal_of(operand: &Operand) -> Option<&FactValue> {
    match operand {
        Operand::Literal(value) => Some(value),
        Operand::Fact(_) | Operand::Offset(_) | Operand::Derived(_) => None,
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
        | (Kind::Instant, Kind::Instant)
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

// ---- selection by existence (ess/16, beyond10x/ess#164, #310) -----------------------------------

/// `true` for the creating half of create-or-update: a `creates:` branch marked
/// `unknown_instance: true`.
pub(crate) fn creates_unknown(outcome: &ResolvedOutcome) -> bool {
    outcome.condition == ResolvedCondition::UnknownInstance
        && outcome
            .subject
            .as_ref()
            .is_some_and(|subject| subject.effect == ResolvedEffect::Creates)
}

/// The command's `existing_instance:` refusal, where it declares one.
pub(crate) fn existing_instance(command: &ResolvedCommand) -> Option<&ResolvedOutcome> {
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.condition == ResolvedCondition::ExistingInstance)
}

/// The payload source of the event field a creation publishes its identity in, or `None` where
/// the outcome creates nothing or declares no source for it.
pub(crate) fn identity_source(outcome: &ResolvedOutcome) -> Option<&ResolvedPayloadField> {
    let subject = outcome
        .subject
        .as_ref()
        .filter(|subject| subject.effect == ResolvedEffect::Creates)?;
    let ResolvedInstance::Observed { event, field } = &subject.instance else {
        return None;
    };
    outcome
        .payload
        .iter()
        .filter(|payload| &payload.event == event)
        .flat_map(|payload| &payload.fields)
        .find(|entry| entry.target == field.name)
}

/// The input field a creation selected by existence takes its identity from, and whether the
/// field is optional (`{input: f, else: {generated: true}}`, where an absent `f` names a new
/// identity no record carries). `None` where the identity is not read from the input.
pub(crate) fn identity_input(outcome: &ResolvedOutcome) -> Option<(&str, bool)> {
    match &identity_source(outcome)?.value {
        ResolvedPayloadValue::InputField { field, type_ref } if !type_ref.is_optional() => {
            Some((field.as_str(), false))
        }
        ResolvedPayloadValue::InputOrGenerated {
            field,
            otherwise: None,
            ..
        } => Some((field.as_str(), true)),
        _ => None,
    }
}

/// Every creation the existence lookup decides, checked: on a command with an `existing_instance:`
/// refusal, every creation reads its identity from one input field, the one the lookup reads; the
/// creating half of create-or-update reads it from a required input field. Anything else keeps the
/// command an obligation, since the lookup would read an identity the creation does not take.
fn existence_identity(command: &ResolvedCommand) -> Result<(), String> {
    // ess/22 (A4): the generated lookup reads a top-level input; an identity read through a path
    // keeps the command an obligation, by name.
    let decided =
        existing_instance(command).is_some() || command.outcomes.iter().any(creates_unknown);
    if let Some(outcome) = command.outcomes.iter().find(|outcome| {
        decided
            && identity_input(outcome)
                .is_some_and(|(field, _)| ess_domain::command::input_path::is_path(field))
    }) {
        return Err(format!(
            "a creation selected by existence whose identity is read through the input path \
             `{}`, in `{}`",
            identity_input(outcome).map_or_else(String::new, |(field, _)| field.to_owned()),
            outcome.name
        ));
    }
    for outcome in command.outcomes.iter().filter(|it| creates_unknown(it)) {
        if !matches!(identity_input(outcome), Some((_, false))) {
            return Err(format!(
                "a creation selected by existence whose identity is not a required input field, \
                 in `{}`",
                outcome.name
            ));
        }
    }
    if existing_instance(command).is_some() {
        let mut read: Option<(&str, bool)> = None;
        let mut entity = None;
        for outcome in &command.outcomes {
            let Some(subject) = outcome
                .subject
                .as_ref()
                .filter(|subject| subject.effect == ResolvedEffect::Creates)
            else {
                continue;
            };
            // The generated lookup reads one entity's storage, so every creation it decides
            // creates that entity (beyond10x/ess#310).
            if entity.is_some_and(|first| first != &subject.entity) {
                return Err(
                    "creations of different entities beside `existing_instance:`".to_owned(),
                );
            }
            entity = Some(&subject.entity);
            let Some(field) = identity_input(outcome) else {
                return Err(format!(
                    "a creation beside `existing_instance:` whose identity is not read from the \
                     input, in `{}`",
                    outcome.name
                ));
            };
            if read.is_some_and(|first| first != field) {
                return Err(
                    "creations beside `existing_instance:` reading their identity from different \
                     input fields"
                        .to_owned(),
                );
            }
            read = Some(field);
        }
    }
    Ok(())
}

// ---- a related row (`when_related:`, ess/18, ess/22; beyond10x/ess#319) -------------------------

/// The one related row a command reads: where its identity is named and whose row it is. A
/// generated command reads one; one reading several (ess/22, beyond10x/ess#283) stays owed
/// ([`related_composition`]), so the first branch naming it names it for all.
pub(crate) fn related(command: &ResolvedCommand) -> Option<(&ResolvedRelatedVia, &EntityHandle)> {
    command
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related { via, entity, .. } => Some((via, entity)),
            _ => None,
        })
}

/// Whether the command's `when_related:` branches read more than one row (ess/22,
/// beyond10x/ess#283): two `via` fields.
pub(crate) fn several_rows(command: &ResolvedCommand) -> bool {
    let mut fields = command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related { via, .. } => Some(via.field()),
            _ => None,
        });
    fields
        .next()
        .is_some_and(|first| fields.any(|other| other != first))
}

/// The subject a stored reference is read from: the first branch addressing an existing row.
pub(crate) fn addressed_subject(command: &ResolvedCommand) -> Option<&ResolvedSubject> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| outcome.subject.as_ref())
        .find(|subject| subject.effect != ResolvedEffect::Creates)
}

/// The compositions a generated related read answers as the interpreter does: the row is stored
/// where every component accepting the command stores it, no `external:` branch is asked beside
/// it, and a stored reference is read from a subject the input names.
fn related_composition(ir: &EssIr, command: &ResolvedCommand) -> Result<(), String> {
    let Some((via, entity)) = related(command) else {
        return Ok(());
    };
    // The generated read names one row per command; several rows, each with its own
    // `exists: false`, answered in the order the interpreter applies (ess/22, beyond10x/ess#283),
    // stay owed.
    if several_rows(command) {
        return Err(
            "`when_related:` reading several related rows in one command (beyond10x/ess#283)"
                .to_owned(),
        );
    }
    if command.outcomes.iter().any(|outcome| {
        matches!(
            outcome.condition,
            ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
        )
    }) {
        return Err("`external:` beside `when_related:` in one command".to_owned());
    }
    let related = ir.entity(entity);
    let unstored = ir.components().values().any(|component| {
        component
            .accepts
            .iter()
            .any(|accepted| accepted.name() == &command.name)
            && !component.owns.contains(&related.domain)
    });
    if unstored {
        return Err(format!(
            "a `when_related:` row of `{}`, which no component accepting the command stores",
            related.name
        ));
    }
    if let ResolvedRelatedVia::Subject { .. } = via {
        let supplied = addressed_subject(command)
            .is_some_and(|subject| matches!(subject.instance, ResolvedInstance::Supplied { .. }));
        if !supplied {
            return Err(
                "a stored `when_related:` reference on no subject the input names".to_owned(),
            );
        }
    }
    Ok(())
}
