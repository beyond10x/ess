//! `shorthand: <field>` (`ess/27`, beyond10x/ess#594): a struct whose value may be written as one
//! of its fields alone.
//!
//! A data file often lets an entry be either a bare name or an object with options, told apart by
//! JSON type. ESS keeps one union form, adjacently tagged, so such an entry is not a union here: it
//! is one struct with two wire forms. With `shorthand: op`, the scalar `getIssue` is the value
//! `{op: getIssue}`, and the struct's invariants apply to that value. Every writer the model
//! generates writes the long form.
//!
//! The key is admitted on a `kind: struct` type only, and there only where the short form can
//! fill the whole value and cannot be mistaken for the long one:
//!
//! | written | refusal, at `<type>.shorthand` |
//! |---|---|
//! | naming no declared field | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | naming an `Optional` field, or one whose type is not a scalar on the wire | [`TypeMismatch`](ValidationCode::TypeMismatch) |
//! | beside another field that is not `Optional` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration) |
//! | on a newtype, an enum or a union | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | on an entity, an event, an error, a view, an actor or a command | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | under a header older than `ess/27` | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//!
//! A value that is not one field name — a list or a mapping — is refused as the document is read.
//!
//! A scalar on the wire is a primitive other than `Json`, an enum, or a newtype over one of those:
//! a JSON string, number or boolean, never an object, so the short form and the long form differ by
//! JSON type and a reader never guesses.
//!
//! The key is read beside the body, as `undeclared_fields:` is and for the same reason
//! ([`crate::undeclared_fields`]): [`RawSpecFile::parse`](crate::spec::RawSpecFile::parse) takes it
//! out of each declaration ([`Written::take`]) and assembly sets it on the converted struct or
//! refuses it where it stands. The published schema admits it on a struct only.

use std::collections::BTreeMap;

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};

use crate::system::FormatVersion;
use crate::types::{NamedType, Primitive, TypeBody, TypeRef, TypeRegistry};
use crate::undeclared_fields::Section;

/// The key it is written under.
pub const KEY: &str = "shorthand";

/// The `shorthand:` each declaration of one document wrote, by list and position.
///
/// Never part of the document, its schema or its bytes: read by [`Self::take`] beside the typed
/// fields, and spent by assembly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Written {
    declared: BTreeMap<(Section, usize), String>,
}

impl Written {
    /// Takes `shorthand:` out of every declaration of `document` that writes it.
    ///
    /// A value that is not one field name — a list, a mapping, a number — is refused as the
    /// document is read, naming the declaration's position.
    pub(crate) fn take(document: &mut ess_yaml::Value) -> Result<Self, ess_yaml::Error> {
        let mut written = Self::default();
        // A command too, though it reads `undeclared_fields:` as its own field: `shorthand:` on one
        // is refused at its line (review 2026-10-19), not as a field the command does not know.
        for section in Section::ALL.into_iter().chain([Section::Commands]) {
            let Some(items) = document
                .get_mut(section.key())
                .and_then(ess_yaml::Value::as_sequence_mut)
            else {
                continue;
            };
            for (index, item) in items.iter_mut().enumerate() {
                let Some(mapping) = item.as_mapping_mut() else {
                    continue;
                };
                let Some(value) = mapping.shift_remove(KEY) else {
                    continue;
                };
                let Some(field) = value.as_str() else {
                    return Err(<ess_yaml::Error as serde::de::Error>::custom(format!(
                        "{}[{index}].{KEY}: names exactly one field of the struct, as a plain \
                         field name; a list, a mapping or a number names none",
                        section.key()
                    )));
                };
                written.declared.insert((section, index), field.to_owned());
            }
        }
        Ok(written)
    }

    /// Sets what the type at `index` of `types:` wrote on `declared`, a struct, or returns the
    /// refusal of it where `declared` is no struct. Empty where the type wrote nothing.
    pub(crate) fn apply_to_type(&self, index: usize, declared: &mut NamedType) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        let Some(written) = self.declared.get(&(Section::Types, index)) else {
            return errors;
        };
        if let TypeBody::Struct { shorthand, .. } = &mut declared.body {
            *shorthand = Some(written.clone());
            return errors;
        }
        let kind = match &declared.body {
            TypeBody::Newtype { .. } => "newtype",
            TypeBody::Enum { .. } => "enum",
            TypeBody::Union { .. } => "union",
            TypeBody::Struct { .. } => "struct",
        };
        errors.push(
            ValidationError::at(
                at(ConstructKind::Type, &declared.name.to_string()),
                ValidationCode::UnsupportedConstruct,
                format!(
                    "`shorthand:` is admitted on a `kind: struct` type, and `{}` is a {kind}",
                    declared.name
                ),
            )
            .with_hint(
                "drop `shorthand:`: only a record has a field its value could be written as",
            ),
        );
        errors
    }

    /// The refusal of every `shorthand:` `file` wrote outside `types:`, each at the declaration
    /// that wrote it.
    pub(crate) fn refused_outside_types(
        &self,
        file: &crate::spec::RawSpecFile,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        for (section, index) in self
            .declared
            .keys()
            .copied()
            .filter(|(section, _)| *section != Section::Types)
        {
            let name = match section {
                Section::Types => None,
                Section::Entities => file.entities.get(index).map(|it| it.name.to_string()),
                Section::Events => file.events.get(index).map(|it| it.name.to_string()),
                Section::Errors => file.errors.get(index).map(|it| it.name.to_string()),
                Section::Views => file.views.get(index).map(|it| it.name.to_string()),
                Section::Actors => file.actors.get(index).map(|it| it.name.to_string()),
                Section::Commands => file.commands.get(index).map(|it| it.name.to_string()),
            };
            if let Some(name) = name {
                errors.push(
                    ValidationError::at(
                        at(section.kind(), &name),
                        ValidationCode::UnsupportedConstruct,
                        format!(
                            "`shorthand:` is admitted on a `kind: struct` type, not on the {} \
                             `{name}`",
                            section.noun()
                        ),
                    )
                    .with_hint(
                        "drop `shorthand:`, or declare it on the struct type a value is written as",
                    ),
                );
            }
        }
        errors
    }
}

/// The refusals of the `shorthand:` the struct `declared` wrote, against the complete `registry`.
/// Empty where it wrote none.
pub(crate) fn validate(declared: &NamedType, registry: &TypeRegistry) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let TypeBody::Struct {
        fields,
        shorthand: Some(shorthand),
        ..
    } = &declared.body
    else {
        return errors;
    };
    let name = declared.name.to_string();
    if registry
        .format()
        .is_some_and(|format| format.major() < FormatVersion::V27.major())
    {
        errors.push(
            ValidationError::at(
                at(ConstructKind::Type, &name),
                ValidationCode::UnsupportedFormatVersion,
                "`shorthand:` requires specification format ess/27",
            )
            .with_hint("declare `format: ess/27`, or drop `shorthand:`"),
        );
    }
    let Some(field) = fields.iter().find(|field| field.name == *shorthand) else {
        let declared_fields: Vec<&str> = fields.iter().map(|field| field.name.as_str()).collect();
        errors.push(
            ValidationError::at(
                at(ConstructKind::Type, &name),
                ValidationCode::UndeclaredReference,
                format!(
                    "`shorthand: {shorthand}` names `{shorthand}`, which `{name}` does not declare"
                ),
            )
            .with_hint(format!(
                "name one of its fields: {}",
                declared_fields.join(", ")
            )),
        );
        return errors;
    };
    if let Some(reason) = not_scalar(&field.type_ref, registry) {
        errors.push(
            ValidationError::at(
                at(ConstructKind::Type, &name),
                ValidationCode::TypeMismatch,
                format!(
                    "`shorthand: {shorthand}` names `{shorthand}`, of type `{}`, which {reason}; \
                     the short form is a JSON string, number or boolean standing for the whole \
                     value",
                    field.type_ref
                ),
            )
            .with_hint(
                "name a required field whose type is a primitive other than `Json`, an enum, or a \
                 newtype over one of those",
            ),
        );
    }
    let required: Vec<&str> = fields
        .iter()
        .filter(|other| other.name != *shorthand && !other.type_ref.is_optional())
        .map(|other| other.name.as_str())
        .collect();
    if !required.is_empty() {
        let listed: Vec<String> = required.iter().map(|it| format!("`{it}`")).collect();
        errors.push(
            ValidationError::at(
                at(ConstructKind::Type, &name),
                ValidationCode::ConflictingDeclaration,
                format!(
                    "`shorthand: {shorthand}` writes a value of `{name}` as `{shorthand}` alone, \
                     and the short form fills nothing else, but {} {} required",
                    listed.join(", "),
                    if required.len() == 1 { "is" } else { "are" }
                ),
            )
            .with_hint("declare every other field `Optional<…>`, or drop `shorthand:`"),
        );
    }
    errors
}

/// Why `reference` is not a scalar on the wire, or `None` where it is one.
fn not_scalar(reference: &TypeRef, registry: &TypeRegistry) -> Option<String> {
    if reference.is_optional() {
        return Some("is `Optional`, and the short form always carries it".to_owned());
    }
    let layers = registry.newtype_layers(reference);
    match &layers.terminal {
        TypeRef::Primitive(Primitive::Json) => Some("may be any JSON value".to_owned()),
        TypeRef::Primitive(_) if !layers.optional => None,
        TypeRef::Named(inner) => match registry.get(inner).map(|it| &it.body) {
            Some(TypeBody::Enum { .. }) if !layers.optional => None,
            Some(TypeBody::Struct { .. }) => Some(format!("is the struct `{inner}`")),
            Some(TypeBody::Union { .. }) => Some(format!("is the union `{inner}`")),
            // An unknown name is refused where it is resolved.
            None => None,
            Some(_) => Some("is not a scalar on the wire".to_owned()),
        },
        TypeRef::List(_) => Some("is a `List`".to_owned()),
        TypeRef::Map(..) => Some("is a `Map`".to_owned()),
        TypeRef::Optional(_) | TypeRef::Primitive(_) => {
            Some("is `Optional` through a newtype, and the short form always carries it".to_owned())
        }
    }
}

/// `type <name>.shorthand`, where every refusal of the key is located.
fn at(kind: ConstructKind, name: &str) -> ConstructRef {
    ConstructRef::new(kind, name).key(KEY)
}
