//! Admission at every authored type position, including directly constructed models.

use crate::system::{FormatVersion, SystemSpec};
use crate::{Field, Primitive, Specification, TypeBody, TypeRef, TypeRegistry};
use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};

pub(crate) fn reference(
    ty: &TypeRef,
    format: Option<FormatVersion>,
    at: &str,
    errors: &mut ValidationErrors,
) {
    match ty {
        TypeRef::Primitive(Primitive::Binary64) if format == Some(FormatVersion::V1) => errors
            .push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                at,
                "Binary64 requires specification format ess/2",
            )),
        // An older reader refuses `Json` as an undeclared type, with no version hint.
        TypeRef::Primitive(Primitive::Json)
            if format.is_some_and(|format| format.major() < FormatVersion::V15.major()) =>
        {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                at,
                "Json requires specification format ess/15",
            ));
        }
        TypeRef::Map(key, value) => {
            if *key == Primitive::Binary64 {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    format!("{at}.key"),
                    "Binary64 map keys have no admitted wire spelling",
                ));
            }
            if *key == Primitive::Json {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    format!("{at}.key"),
                    "a Json value is not a map key",
                ));
            }
            reference(value, format, &format!("{at}.value"), errors);
        }
        TypeRef::Optional(of) | TypeRef::List(of) => {
            reference(of, format, &format!("{at}.of"), errors);
        }
        _ => {}
    }
}

fn fields(
    values: &[Field],
    format: FormatVersion,
    types: &TypeRegistry,
    at: &str,
    errors: &mut ValidationErrors,
) {
    for field in values {
        reference(
            &field.type_ref,
            Some(format),
            &format!("{at}.{}.type", field.name),
            errors,
        );
        presence(
            field,
            format,
            types,
            &format!("{at}.{}.presence", field.name),
            errors,
        );
    }
}

/// A field's `presence:` (beyond10x/ess#139): ess/15, and only on an `Optional<T>`, because a
/// required field is always sent and has no absent value to spell.
///
/// Here, beside the type admission, because this walk reaches every field position that takes
/// `presence:` — struct, entity, command input and response, event, error — so no position is gated
/// in one place and forgotten in another. A view's own fields and parameters do not take it: a
/// view field is read as `RawViewField`, which refuses the key, and a view projecting a named
/// struct carries that struct's fields, checked where the struct is declared. The walk still
/// visits view fields, where no policy can be set.
fn presence(
    field: &Field,
    format: FormatVersion,
    types: &TypeRegistry,
    at: &str,
    errors: &mut ValidationErrors,
) {
    let Some(policy) = field.presence() else {
        return;
    };
    // An older reader fails `presence:` as an unknown field with no version hint.
    if format.major() < FormatVersion::V15.major() {
        errors.push(ValidationError::new(
            ValidationCode::UnsupportedFormatVersion,
            at,
            "field presence policies require specification format ess/15",
        ));
    }
    if !field.type_ref.is_optional() {
        errors.push(
            ValidationError::new(
                ValidationCode::TypeMismatch,
                at,
                format!(
                    "`{}` is `{}`, which is always sent, so `presence: {policy}` has no absent \
                     value to spell",
                    field.name, field.type_ref
                ),
            )
            .with_hint("declare the field `Optional<…>`, or delete `presence:`"),
        );
    }
    // `null` is a JSON value (beyond10x/ess#138): an `Optional<Json>` that is never sent as
    // `null` would refuse one of its own values, so the only spelling of its absence left is the
    // explicit `null` of `null_when_absent`.
    if policy == crate::types::Presence::OmittedWhenAbsent
        && types.newtype_layers(field.type_ref.required()).terminal
            == TypeRef::Primitive(Primitive::Json)
    {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at,
                format!(
                    "`{}` is `{}`, and `null` is a JSON value, so `presence: omitted_when_absent` \
                     (never sent as null) would refuse a value the type has",
                    field.name, field.type_ref
                ),
            )
            .with_hint("write `presence: null_when_absent`, or delete `presence:`"),
        );
    }
}

pub(crate) fn system(system: &SystemSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for declared in system.types.iter() {
        let at = format!("types.{}", declared.name);
        if declared.reading.is_some() && system.format.major() < 3 {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                format!("{at}.reading"),
                "clock reading contracts require specification format ess/3",
            ));
        }
        match &declared.body {
            TypeBody::Newtype {
                of,
                alphabet,
                prefix,
                ..
            } => {
                reference(of, Some(system.format), &format!("{at}.of"), &mut errors);
                // An older reader fails `alphabet:` as an unknown field with no version hint.
                if alphabet.is_some() && system.format.major() < FormatVersion::V11.major() {
                    errors.push(ValidationError::new(
                        ValidationCode::UnsupportedFormatVersion,
                        format!("{at}.alphabet"),
                        "declared alphabets require specification format ess/11",
                    ));
                }
                // The same for `prefix:` (beyond10x/ess#146).
                if prefix.is_some() && system.format.major() < FormatVersion::V15.major() {
                    errors.push(ValidationError::new(
                        ValidationCode::UnsupportedFormatVersion,
                        format!("{at}.prefix"),
                        "declared prefixes require specification format ess/15",
                    ));
                }
            }
            TypeBody::Struct {
                fields: members, ..
            } => fields(
                members,
                system.format,
                &system.types,
                &format!("{at}.fields"),
                &mut errors,
            ),
            TypeBody::Union { variants, .. } => {
                for (name, ty) in variants {
                    reference(
                        ty,
                        Some(system.format),
                        &format!("{at}.variants.{name}"),
                        &mut errors,
                    );
                }
            }
            TypeBody::Enum { variants } => {
                for variant in variants {
                    if system.format.major() < 5 && !variant.is_bare() {
                        errors.push(ValidationError::new(
                            ValidationCode::UnsupportedFormatVersion,
                            format!("{at}.variants.{}", variant.name()),
                            "declared enum variant naming requires specification format ess/5",
                        ));
                    }
                }
            }
        }
    }
    errors
}

/// The two conditions that read the state a subject already holds, each against its own format.
///
/// Split out of [`specification`] rather than inlined, and two checks rather than one over
/// [`uses`](crate::command::subject_state::uses): the constructs arrived in different source
/// formats, so a document written before either has to be told which one it reached for.
fn held_state_conditions(
    command: &crate::command::CommandSpec,
    format: FormatVersion,
    errors: &mut ValidationErrors,
) {
    if format.major() < 7
        && (command.has_state_refusal()
            || command
                .outcomes
                .iter()
                .any(|outcome| outcome.replays.is_some()))
    {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::UnsupportedFormatVersion,
            "retained results and effect-free state defaults require specification format ess/7",
        ));
    }
    if format.major() < 6
        && command.outcomes.iter().any(|outcome| {
            matches!(
                outcome.condition,
                crate::command::OutcomeCondition::ExternalWhen { .. }
                    | crate::command::OutcomeCondition::SubjectField { .. }
            ) || outcome
                .subject
                .as_ref()
                .is_some_and(|subject| subject.effect == crate::command::Effect::Preserves)
        })
    {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::UnsupportedFormatVersion,
            "guarded external outcomes, subject facts and preservation require specification format ess/6",
        ));
    }
    // A new shape of an existing key's value: an older reader fails it with `unknown field
    // predicate` and no version hint, so the meaning change is a format change of its own
    // (`docs/design/cross-record-and-stored-field-guards.md`). `{field, equals}` keeps ess/6.
    if format.major() < 9 && crate::command::subject_fact::uses_predicate(command) {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::UnsupportedFormatVersion,
            "subject predicates require specification format ess/9",
        ));
    }
    if format.major() < 3 && crate::command::subject_state::uses_subject_state(command) {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::UnsupportedFormatVersion,
            "subject-state outcome guards require specification format ess/3",
        ));
    }
    if format.major() < 4 && crate::command::subject_state::uses_state_changes(command) {
        errors.push(ValidationError::at(
            command.site().key("outcomes"),
            ValidationCode::UnsupportedFormatVersion,
            "state-change outcome guards require specification format ess/4",
        ));
    }
}

/// Every predicate a specification holds, each with the site it is written at.
///
/// Command outcome conditions (`when`, and the input predicate beside `when_subject_state`,
/// `when_state_changes`, `when_subject` and `external`), entity invariants, newtype and struct
/// invariants, view filters and binding selections. A format gate over predicate vocabulary asks
/// this one walk, so a position cannot be gated in one construct and forgotten in another; a new
/// predicate position is added here, not beside the gate that reads it.
pub fn predicates(
    spec: &Specification,
) -> Vec<(ConstructRef, &ess_primitives::predicate::Predicate)> {
    let mut found = Vec::new();
    for declared in spec.system().types.iter() {
        if let TypeBody::Newtype { invariants, .. } | TypeBody::Struct { invariants, .. } =
            &declared.body
        {
            for (index, invariant) in invariants.iter().enumerate() {
                found.push((
                    ConstructRef::new(ConstructKind::Type, declared.name.to_string())
                        .key("invariants")
                        .index(index),
                    &invariant.predicate,
                ));
            }
        }
    }
    for entity in spec.entities().values() {
        for (index, invariant) in entity.invariants.iter().enumerate() {
            found.push((
                ConstructRef::new(ConstructKind::Entity, entity.name.to_string())
                    .key("invariants")
                    .index(index),
                &invariant.predicate,
            ));
        }
    }
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            if let Some(predicate) = outcome.condition.predicate() {
                found.push((
                    command
                        .site()
                        .key("outcomes")
                        .named(outcome.name.to_string()),
                    predicate,
                ));
            }
            // The stored-field predicate is the same grammar, so a string operator under
            // `when_subject:` needs the format that admits it as much as one under `when:`.
            if let crate::command::OutcomeCondition::SubjectPredicate { predicate, .. } =
                &outcome.condition
            {
                found.push((
                    command
                        .site()
                        .key("outcomes")
                        .named(outcome.name.to_string())
                        .key("when_subject"),
                    predicate,
                ));
            }
        }
    }
    for view in spec.views().values() {
        if let Some(filter) = &view.filter {
            found.push((
                ConstructRef::new(ConstructKind::View, view.name.to_string()).key("filter"),
                filter,
            ));
        }
    }
    for binding in spec.bindings().values() {
        for selection in &binding.selections {
            if let Some(first) = &selection.first {
                found.push((
                    ConstructRef::new(ConstructKind::Binding, binding.name.to_string())
                        .key("selections")
                        .named(selection.name.clone()),
                    &first.predicate,
                ));
            }
        }
    }
    found
}

/// V15 of `docs/design/aggregate-views.md`: an older reader fails `aggregate:` as an unknown field
/// and no version hint, so the construct is a format of its own. A `group_by` with no aggregate is V5
/// at every version and has no `Aggregation` to be gated here.
fn aggregate_view(view: &crate::ViewSpec, format: FormatVersion, errors: &mut ValidationErrors) {
    let Some(aggregation) = &view.aggregation else {
        return;
    };
    if format.major() < FormatVersion::V10.major() {
        let at = if aggregation.group_by.is_empty() {
            "fields"
        } else {
            "group_by"
        };
        errors.push(ValidationError::new(
            ValidationCode::UnsupportedFormatVersion,
            format!("view.{}.{at}", view.name),
            "aggregate views require specification format ess/10",
        ));
    }
}

/// A format, the question that finds its operator in a predicate, and the refusal below it.
type OperatorFormat = (
    FormatVersion,
    fn(&ess_primitives::predicate::Predicate) -> bool,
    &'static str,
);

/// The predicate operators that arrived after `ess/1`, each with the format that admits it:
/// `starts_with`, `ends_with` and `contains` (beyond10x/ess#95) in `ess/8`, and
/// `equals_ignore_case` and `in_ignore_case` (beyond10x/ess#140) in `ess/15`. One walk over every
/// predicate position, so a later operator is gated where the earlier ones are.
const OPERATOR_FORMATS: &[OperatorFormat] = &[
    (
        FormatVersion::V8,
        ess_primitives::predicate::Predicate::uses_text_match,
        "string predicate operators require specification format ess/8",
    ),
    (
        FormatVersion::V15,
        ess_primitives::predicate::Predicate::uses_case_fold,
        "case-insensitive text operators require specification format ess/15",
    ),
];

fn predicate_operators(spec: &Specification, format: FormatVersion, errors: &mut ValidationErrors) {
    for (site, predicate) in predicates(spec) {
        for (admitted, uses, message) in OPERATOR_FORMATS {
            if format.major() < admitted.major() && uses(predicate) {
                errors.push(ValidationError::at(
                    site.clone(),
                    ValidationCode::UnsupportedFormatVersion,
                    *message,
                ));
            }
        }
    }
}

/// As for `alphabet:`, an older reader fails `example:` as an unknown field with no version hint,
/// so an input example is refused below ess/11 at the key the author wrote.
fn input_examples(
    command: &crate::command::CommandSpec,
    format: FormatVersion,
    errors: &mut ValidationErrors,
) {
    if format.major() >= FormatVersion::V11.major() {
        return;
    }
    for field in command.examples.keys() {
        errors.push(ValidationError::at(
            command
                .site()
                .key("input")
                .named(field.clone())
                .key("example"),
            ValidationCode::UnsupportedFormatVersion,
            "input examples require specification format ess/11",
        ));
    }
}

/// A command's input fields, and the presence policy of its response fields: a response field's
/// type was never gated here, so only its `presence:` is.
fn command_fields(
    command: &crate::command::CommandSpec,
    format: FormatVersion,
    types: &TypeRegistry,
    errors: &mut ValidationErrors,
) {
    fields(
        &command.input,
        format,
        types,
        &format!("command.{}.input", command.name),
        errors,
    );
    for field in &command.response {
        presence(
            field,
            format,
            types,
            &format!("command.{}.response.{}.presence", command.name, field.name),
            errors,
        );
    }
}

pub(crate) fn specification(spec: &Specification) -> ValidationErrors {
    let mut errors = system(spec.system());
    predicate_operators(spec, spec.system().format, &mut errors);
    errors.extend(crate::command::validate_response_contracts(spec));
    errors.extend(crate::command::value_expression::validate(spec));
    errors.extend(crate::command::outcome_shapes::validate(spec));
    errors.extend(crate::command::absent_input::validate(spec));
    let format = spec.system().format;
    for binding in spec.bindings().values() {
        if format.major() < 3
            && (!binding.selection_inputs.is_empty()
                || !binding.selections.is_empty()
                || binding.mapping.values().any(|source| {
                    matches!(source, crate::binding::MappingSource::Selection { .. })
                }))
        {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                format!("binding.{}.selections", binding.name),
                "bounded list selection requires specification format ess/3",
            ));
        }
        for (target, source) in &binding.mapping {
            if matches!(source, crate::binding::MappingSource::EventAccessor { .. })
                && format.major() < 3
            {
                errors.push(ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("binding.{}.mapping.{target}", binding.name),
                    "bounded event accessors require specification format ess/3",
                ));
            }
        }
    }
    for entity in spec.entities().values() {
        reference(
            &entity.identity.type_ref,
            Some(format),
            &format!("entity {}.identity.type", entity.name),
            &mut errors,
        );
        fields(
            &entity.fields,
            format,
            &spec.system().types,
            &format!("entity {}.fields", entity.name),
            &mut errors,
        );
    }
    for command in spec.commands().values() {
        held_state_conditions(command, format, &mut errors);
        input_examples(command, format, &mut errors);
        command_fields(command, format, &spec.system().types, &mut errors);
    }
    for event in spec.events().values() {
        fields(
            &event.fields,
            format,
            &spec.system().types,
            &format!("event.{}.fields", event.name),
            &mut errors,
        );
    }
    for error in spec.errors().values() {
        if format.major() < 4 && !error.naming.is_empty() {
            errors.push(ValidationError::new(
                ValidationCode::UnsupportedFormatVersion,
                format!("error.{}.naming", error.name),
                "declared error naming requires specification format ess/4",
            ));
        }
        fields(
            &error.fields,
            format,
            &spec.system().types,
            &format!("error.{}.fields", error.name),
            &mut errors,
        );
    }
    for view in spec.views().values() {
        aggregate_view(view, format, &mut errors);
        errors.extend(view.absent_value_admission(format, &spec.system().types));
        if let Some(members) = view.projected_fields(&spec.system().types) {
            fields(
                members,
                format,
                &spec.system().types,
                &format!("view.{}.fields", view.name),
                &mut errors,
            );
        }
        fields(
            &view.params,
            format,
            &spec.system().types,
            &format!("view.{}.params", view.name),
            &mut errors,
        );
    }
    errors
}
