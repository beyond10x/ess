//! Wire namespaces are checked before any projection can collapse fields into a map.

use std::collections::BTreeMap;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use crate::{EntitySpec, Field, Specification, TypeBody};

pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for declared in spec.system().types.iter() {
        if let TypeBody::Struct { fields, .. } = &declared.body {
            check_fields(
                fields,
                &format!("types.{}.fields", declared.name),
                &mut errors,
            );
        }
    }
    for entity in spec.entities().values() {
        let at = format!("entity {}", entity.name);
        let mut namespace = Namespace::default();
        namespace.insert(EntitySpec::STATE, format!("{at}.lifecycle"), &mut errors);
        namespace.field(&entity.identity, format!("{at}.identity"), &mut errors);
        namespace.fields(&entity.fields, &format!("{at}.fields"), &mut errors);
    }
    for command in spec.commands().values() {
        check_fields(
            &command.input,
            &format!("command.{}.input", command.name),
            &mut errors,
        );
    }
    for event in spec.events().values() {
        check_fields(
            &event.fields,
            &format!("event.{}.fields", event.name),
            &mut errors,
        );
    }
    for error in spec.errors().values() {
        check_fields(
            &error.fields,
            &format!("error.{}.fields", error.name),
            &mut errors,
        );
    }
    for view in spec.views().values() {
        if let Some(fields) = view.projected_fields(&spec.system().types) {
            let member = if view.shape.is_some() {
                "shape.fields"
            } else {
                "fields"
            };
            check_fields(fields, &format!("view.{}.{member}", view.name), &mut errors);
        }
        check_fields(
            &view.params,
            &format!("view.{}.params", view.name),
            &mut errors,
        );
    }
    errors
}

fn check_fields(fields: &[Field], at: &str, errors: &mut ValidationErrors) {
    Namespace::default().fields(fields, at, errors);
}

#[derive(Default)]
struct Namespace<'a>(BTreeMap<&'a str, String>);

impl<'a> Namespace<'a> {
    fn fields(&mut self, fields: &'a [Field], at: &str, errors: &mut ValidationErrors) {
        for (index, field) in fields.iter().enumerate() {
            self.field(field, format!("{at}[{index}]"), errors);
        }
    }

    fn field(&mut self, field: &'a Field, at: String, errors: &mut ValidationErrors) {
        self.insert(
            field.naming.wire.as_deref().unwrap_or(&field.name),
            at,
            errors,
        );
    }

    fn insert(&mut self, wire: &'a str, at: String, errors: &mut ValidationErrors) {
        if let Some(first) = self.0.get(wire) {
            errors.push(
                ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    at,
                    format!("wire field {wire:?} is already used at {first}"),
                )
                .with_hint("give each field in this object a distinct effective wire name"),
            );
        } else {
            self.0.insert(wire, at);
        }
    }
}
