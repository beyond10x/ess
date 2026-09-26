//! Admission at every authored type position, including directly constructed models.

use crate::system::{FormatVersion, SystemSpec};
use crate::{Field, Primitive, Specification, TypeBody, TypeRef};
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
        TypeRef::Map(key, value) => {
            if *key == Primitive::Binary64 {
                errors.push(ValidationError::new(
                    ValidationCode::TypeMismatch,
                    format!("{at}.key"),
                    "Binary64 map keys have no admitted wire spelling",
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

fn fields(values: &[Field], format: FormatVersion, at: &str, errors: &mut ValidationErrors) {
    for field in values {
        reference(
            &field.type_ref,
            Some(format),
            &format!("{at}.{}.type", field.name),
            errors,
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
            TypeBody::Newtype { of, .. } => {
                reference(of, Some(system.format), &format!("{at}.of"), &mut errors);
            }
            TypeBody::Struct {
                fields: members, ..
            } => fields(members, system.format, &format!("{at}.fields"), &mut errors),
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
pub(crate) fn predicates(
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

/// `starts_with`, `ends_with` and `contains` (beyond10x/ess#95) arrived in `ess/8`.
fn string_operators(spec: &Specification, format: FormatVersion, errors: &mut ValidationErrors) {
    if format.major() >= FormatVersion::V8.major() {
        return;
    }
    for (site, predicate) in predicates(spec) {
        if predicate.uses_text_match() {
            errors.push(ValidationError::at(
                site,
                ValidationCode::UnsupportedFormatVersion,
                "string predicate operators require specification format ess/8",
            ));
        }
    }
}

pub(crate) fn specification(spec: &Specification) -> ValidationErrors {
    let mut errors = system(spec.system());
    string_operators(spec, spec.system().format, &mut errors);
    errors.extend(crate::command::validate_response_contracts(spec));
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
            &format!("entity {}.fields", entity.name),
            &mut errors,
        );
    }
    for command in spec.commands().values() {
        held_state_conditions(command, format, &mut errors);
        fields(
            &command.input,
            format,
            &format!("command.{}.input", command.name),
            &mut errors,
        );
    }
    for event in spec.events().values() {
        fields(
            &event.fields,
            format,
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
            &format!("error.{}.fields", error.name),
            &mut errors,
        );
    }
    for view in spec.views().values() {
        aggregate_view(view, format, &mut errors);
        if let Some(members) = view.projected_fields(&spec.system().types) {
            fields(
                members,
                format,
                &format!("view.{}.fields", view.name),
                &mut errors,
            );
        }
        fields(
            &view.params,
            format,
            &format!("view.{}.params", view.name),
            &mut errors,
        );
    }
    errors
}
