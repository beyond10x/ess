//! Outcome-local disclosure authority; no name-based inference across outcomes.
use super::{CommandSpec, Outcome, PayloadSource};
use crate::{
    types::{Primitive, TypeBody},
    Specification, TypeRef,
};
use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};
use std::collections::BTreeSet;

pub(super) fn present_fields<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error> {
    <Vec<String> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

pub(super) fn validate(
    spec: &Specification,
    command: &CommandSpec,
    outcome: &Outcome,
    at: &ConstructRef,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if outcome.one_time_response.is_empty() {
        return errors;
    }
    let at = at.clone().key("one_time_response");
    if spec.system().format.major() < 21 {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::UnsupportedFormatVersion,
            "one_time_response requires specification format ess/21",
        ));
    }
    if !outcome.returns
        || outcome.error.is_some()
        || outcome.accepts_nothing
        || outcome.replays.is_some()
    {
        errors.push(ValidationError::at(
            at.clone(),
            ValidationCode::ConflictingDeclaration,
            "one_time_response requires a successful returns outcome without replay",
        ));
    }
    let mut seen = BTreeSet::new();
    for name in &outcome.one_time_response {
        if !seen.insert(name) {
            errors.push(ValidationError::at(
                at.clone().named(name),
                ValidationCode::DuplicateDeclaration,
                "one_time_response contains a duplicate field",
            ));
        }
        let Some(field) = command.response.iter().find(|field| &field.name == name) else {
            errors.push(ValidationError::at(
                at.clone().named(name),
                ValidationCode::UndeclaredReference,
                "one_time_response names an undeclared response field",
            ));
            continue;
        };
        if !required_string(spec, &field.type_ref) {
            errors.push(ValidationError::at(at.clone().named(name), ValidationCode::TypeMismatch,
                "one_time_response requires String or a finite transparent String newtype without an opaque reading"));
        }
    }
    for source in outcome
        .payload
        .values()
        .flat_map(|fields| fields.values())
        .chain(outcome.sets.values())
        .chain(outcome.error_payload.values())
    {
        if leaks(source, &seen) {
            errors.push(ValidationError::at(at.clone(), ValidationCode::ConflictingDeclaration,
                "one_time_response plaintext cannot flow into a declared payload or persistent field"));
        }
    }
    errors
}

fn required_string<'a>(spec: &'a Specification, mut ty: &'a TypeRef) -> bool {
    let mut seen = BTreeSet::new();
    loop {
        match ty {
            TypeRef::Primitive(Primitive::String) => return true,
            TypeRef::Named(name) if seen.insert(name) => {
                let Some(declared) = spec.system().types.get(name) else {
                    return false;
                };
                if declared.reading.is_some() {
                    return false;
                }
                let TypeBody::Newtype { of, .. } = &declared.body else {
                    return false;
                };
                ty = of;
            }
            _ => return false,
        }
    }
}

fn leaks(source: &PayloadSource, marked: &BTreeSet<&String>) -> bool {
    match source {
        PayloadSource::ResponseField { field } => marked.contains(field),
        PayloadSource::Struct { fields } => fields.iter().any(|field| leaks(&field.source, marked)),
        _ => false,
    }
}
