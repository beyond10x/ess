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
    let format = spec.system().format;
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
