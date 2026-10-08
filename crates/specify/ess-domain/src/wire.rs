//! Wire namespaces are checked before any projection can collapse fields into a map.

use std::collections::BTreeMap;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use crate::{EntitySpec, Field, QualifiedName, Specification, TypeBody};

pub(crate) fn validate(spec: &Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    path_segments(spec, &mut errors);
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

/// Wire names a generated path segment reads: `/{domain}/commands/{command}` and
/// `/{domain}/views/{view}` (`ess_gen::http::routes`, which every HTTP target builds from).
///
/// A view is checked whether or not a network component serves it, so whether a specification
/// validates never depends on how it is composed. A name that falls back to the declaration's own
/// local name is an identifier and cannot hold either shape.
fn path_segments(spec: &Specification, errors: &mut ValidationErrors) {
    for domain in &spec.system().domains {
        path_segment(
            "domain",
            &domain.name,
            domain.naming.wire.as_deref(),
            errors,
        );
    }
    for command in spec.commands().values() {
        path_segment(
            "command",
            &command.name,
            command.naming.wire.as_deref(),
            errors,
        );
    }
    for view in spec.views().values() {
        path_segment("view", &view.name, view.naming.wire.as_deref(), errors);
    }
}

fn path_segment(
    kind: &str,
    name: &QualifiedName,
    wire: Option<&str>,
    errors: &mut ValidationErrors,
) {
    let Some(wire) = wire else { return };
    if !(wire.contains('/') || wire == "." || wire == "..") {
        return;
    }
    errors.push(
        ValidationError::new(
            ValidationCode::PathSegmentWireName,
            format!("{kind}.{name}.naming.wire"),
            format!(
                "{kind} {name} has the wire name {wire:?}, which a generated path segment reads; \
                 a `/` in it, or a name that is `.` or `..`, would address another route"
            ),
        )
        .with_hint("spell the wire name without `/`, and not as `.` or `..`"),
    );
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
