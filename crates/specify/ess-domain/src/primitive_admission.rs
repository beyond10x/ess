//! Admission at every authored type position, including directly constructed models.

use crate::system::{FormatVersion, SystemSpec};
use crate::{Field, Primitive, Specification, TypeBody, TypeRef};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

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
            TypeBody::Enum { .. } => {}
        }
    }
    errors
}

pub(crate) fn specification(spec: &Specification) -> ValidationErrors {
    let mut errors = system(spec.system());
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
        if format.major() < 3 && crate::command::subject_state::uses(command) {
            errors.push(ValidationError::at(
                command.site().key("outcomes"),
                ValidationCode::UnsupportedFormatVersion,
                "subject-state outcome guards require specification format ess/3",
            ));
        }
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
