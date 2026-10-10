//! `undeclared_fields: ignored | refused` (`ess/24`, beyond10x/ess#500): a record that admits the
//! fields it does not declare.
//!
//! Several protocols require a reader to ignore members it does not recognise, so a producer can
//! add extension members. A record in ESS is closed, and before `ess/24` that fact could not be
//! stated. The key is admitted in two places:
//!
//! | where | what it governs |
//! |---|---|
//! | a `kind: struct` type | that struct, wherever it is reached |
//! | a command with a non-empty `response:` | the response object only, never the input |
//!
//! Everywhere else it is refused by name, at the key:
//!
//! | written | refusal |
//! |---|---|
//! | on a command without `response:` | [`MissingDeclaration`](ValidationCode::MissingDeclaration) |
//! | on a newtype, an enum or a union | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | on an entity, an event, an error, a view or an actor | [`UnsupportedConstruct`](ValidationCode::UnsupportedConstruct) |
//! | under a header older than `ess/24` | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//!
//! # Why a type's key is read beside its body
//!
//! A command reads the key as a field of [`RawCommandSpec`](crate::command::RawCommandSpec). A
//! type cannot: [`RawNamedType`](crate::types::RawNamedType) and its
//! [`RawTypeBody::Struct`](crate::types::RawTypeBody::Struct) are built field by field by the
//! conformance observers, which hold neither a document nor the key. So
//! [`RawSpecFile::parse`](crate::spec::RawSpecFile::parse) takes the key out of each declaration
//! of the document it reads ([`Written::take`]) — the same place that keeps how each guard was
//! spelled — and assembly sets it on the converted struct or refuses it where it stands. Every
//! other declaration list is read the same way, so the key written on an event is refused at its
//! line rather than stopping the whole file as an unknown field. The published schema admits it on
//! a struct only (`RawNamedType`'s hand-written schema) and nowhere else but a command.

use std::collections::BTreeMap;

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};

use crate::system::FormatVersion;
use crate::types::{TypeBody, UndeclaredFields};

/// A declaration list of a document that [`Written::take`] reads the key out of.
///
/// Commands are not here: a command reads the key as its own field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Section {
    /// `types:`.
    Types,
    /// `entities:`.
    Entities,
    /// `events:`.
    Events,
    /// `errors:`.
    Errors,
    /// `views:`.
    Views,
    /// `actors:`.
    Actors,
}

impl Section {
    /// Every list, in the order a document is read.
    const ALL: [Self; 6] = [
        Self::Types,
        Self::Entities,
        Self::Events,
        Self::Errors,
        Self::Views,
        Self::Actors,
    ];

    /// The key the list is written under.
    fn key(self) -> &'static str {
        match self {
            Self::Types => "types",
            Self::Entities => "entities",
            Self::Events => "events",
            Self::Errors => "errors",
            Self::Views => "views",
            Self::Actors => "actors",
        }
    }

    /// The construct a declaration of this list is.
    fn kind(self) -> ConstructKind {
        match self {
            Self::Types => ConstructKind::Type,
            Self::Entities => ConstructKind::Entity,
            Self::Events => ConstructKind::Event,
            Self::Errors => ConstructKind::Error,
            Self::Views => ConstructKind::View,
            Self::Actors => ConstructKind::Actor,
        }
    }

    /// What a declaration of this list is called in a refusal.
    fn noun(self) -> &'static str {
        match self {
            Self::Types => "type",
            Self::Entities => "entity",
            Self::Events => "event",
            Self::Errors => "error",
            Self::Views => "view",
            Self::Actors => "actor",
        }
    }
}

/// The `undeclared_fields:` each declaration of one document wrote, by list and position.
///
/// Never part of the document, its schema or its bytes: read by [`Self::take`] beside the typed
/// fields, and spent by assembly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Written {
    declared: BTreeMap<(Section, usize), UndeclaredFields>,
}

impl Written {
    /// Takes `undeclared_fields:` out of every declaration of `document` that writes it.
    ///
    /// A value other than `ignored` or `refused` is refused as the document is read, naming the
    /// declaration's position, as a misspelt value of any other closed key is.
    pub(crate) fn take(document: &mut serde_yaml::Value) -> Result<Self, serde_yaml::Error> {
        let mut written = Self::default();
        for section in Section::ALL {
            let Some(items) = document
                .get_mut(section.key())
                .and_then(serde_yaml::Value::as_sequence_mut)
            else {
                continue;
            };
            for (index, item) in items.iter_mut().enumerate() {
                let Some(mapping) = item.as_mapping_mut() else {
                    continue;
                };
                let Some(value) = mapping.shift_remove(UndeclaredFields::KEY) else {
                    continue;
                };
                let value = serde_yaml::from_value::<UndeclaredFields>(value).map_err(|error| {
                    <serde_yaml::Error as serde::de::Error>::custom(format!(
                        "{}[{index}].{}: {error}",
                        section.key(),
                        UndeclaredFields::KEY
                    ))
                })?;
                written.declared.insert((section, index), value);
            }
        }
        Ok(written)
    }

    /// What the declaration at `index` of `section` wrote, if anything.
    pub(crate) fn at(&self, section: Section, index: usize) -> Option<UndeclaredFields> {
        self.declared.get(&(section, index)).copied()
    }

    /// Sets what the type at `index` of `types:` wrote on `declared`, a struct, or returns the
    /// refusal of it where `declared` is no struct. Empty where the type wrote nothing.
    pub(crate) fn apply_to_type(
        &self,
        index: usize,
        declared: &mut crate::types::NamedType,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if let Some(refused) = self
            .at(Section::Types, index)
            .and_then(|written| apply_to_type(declared, written))
        {
            errors.push(refused);
        }
        errors
    }

    /// Every declaration outside `types:` that wrote the key, by list and position. Each is
    /// refused by [`refused_on`].
    pub(crate) fn outside_types(&self) -> impl Iterator<Item = (Section, usize)> + '_ {
        self.declared
            .keys()
            .copied()
            .filter(|(section, _)| *section != Section::Types)
    }
}

/// The refusal of every `undeclared_fields:` `file` wrote outside `types:` and its commands, each
/// at the declaration that wrote it.
pub(crate) fn refused_outside_types(file: &crate::spec::RawSpecFile) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    file.undeclared_fields
        .outside_types()
        .filter_map(|(section, index)| {
            let name = match section {
                Section::Types => None,
                Section::Entities => file.entities.get(index).map(|it| it.name.to_string()),
                Section::Events => file.events.get(index).map(|it| it.name.to_string()),
                Section::Errors => file.errors.get(index).map(|it| it.name.to_string()),
                Section::Views => file.views.get(index).map(|it| it.name.to_string()),
                Section::Actors => file.actors.get(index).map(|it| it.name.to_string()),
            }?;
            Some(refused_on(section, &name))
        })
        .for_each(|refused| errors.push(refused));
    errors
}

/// Sets `written` on the struct `declared`, or refuses it where `declared` is no struct.
pub(crate) fn apply_to_type(
    declared: &mut crate::types::NamedType,
    written: UndeclaredFields,
) -> Option<ValidationError> {
    if let TypeBody::Struct {
        undeclared_fields, ..
    } = &mut declared.body
    {
        *undeclared_fields = Some(written);
        return None;
    }
    let kind = match &declared.body {
        TypeBody::Newtype { .. } => "newtype",
        TypeBody::Enum { .. } => "enum",
        TypeBody::Union { .. } => "union",
        TypeBody::Struct { .. } => "struct",
    };
    Some(
        ValidationError::at(
            at(ConstructKind::Type, &declared.name.to_string()),
            ValidationCode::UnsupportedConstruct,
            format!(
                "`undeclared_fields:` is admitted on a `kind: struct` type and on a command with a \
                 `response:`, and `{}` is a {kind}",
                declared.name
            ),
        )
        .with_hint("drop `undeclared_fields:`: only a record has fields it could leave undeclared"),
    )
}

/// The refusal of `undeclared_fields:` written on the declaration `name` of `section`, a list
/// other than `types:`.
pub(crate) fn refused_on(section: Section, name: &str) -> ValidationError {
    let note = match section {
        Section::Events => {
            "; an event's payload is the closed set of fields it declares, and a reader ignores \
             undeclared members only where a struct type or a command's response says so"
        }
        Section::Errors => "; an error's payload is the closed set of fields it declares",
        _ => "",
    };
    ValidationError::at(
        at(section.kind(), name),
        ValidationCode::UnsupportedConstruct,
        format!(
            "`undeclared_fields:` is admitted on a `kind: struct` type and on a command with a \
             `response:`, not on the {} `{name}`{note}",
            section.noun()
        ),
    )
    .with_hint("drop `undeclared_fields:`, or declare it on the struct type the fields belong to")
}

/// The refusals of the `undeclared_fields:` `command` wrote, under the format of `spec`.
///
/// On a command the key governs the response: it is admitted from `ess/24`, and only where there
/// is a `response:` to govern.
pub(crate) fn on_command(
    spec: &crate::Specification,
    command: &crate::command::CommandSpec,
) -> ValidationErrors {
    let format = spec.system().format;
    let mut errors = ValidationErrors::new();
    if command.undeclared_fields.is_none() {
        return errors;
    }
    let name = command.name.to_string();
    if let Some(refused) = below_format(Some(format), ConstructKind::Command, &name) {
        errors.push(refused);
    }
    if command.response.is_empty() {
        errors.push(without_response(&name));
    }
    errors
}

/// The refusal of `undeclared_fields:` on a command that declares no `response:`.
pub(crate) fn without_response(command: &str) -> ValidationError {
    ValidationError::at(
        at(ConstructKind::Command, command),
        ValidationCode::MissingDeclaration,
        format!(
            "`undeclared_fields:` on a command governs its `response:`, and `{command}` declares \
             none"
        ),
    )
    .with_hint("declare the `response:` it governs, or drop `undeclared_fields:`")
}

/// The refusal of `undeclared_fields:` written on the declaration `name` of `kind` under a header
/// older than `ess/24`, or `None` from `ess/24` on.
pub(crate) fn below_format(
    format: Option<FormatVersion>,
    kind: ConstructKind,
    name: &str,
) -> Option<ValidationError> {
    format
        .is_some_and(|format| format.major() < FormatVersion::V24.major())
        .then(|| {
            ValidationError::at(
                at(kind, name),
                ValidationCode::UnsupportedFormatVersion,
                "`undeclared_fields:` requires specification format ess/24",
            )
            .with_hint("declare `format: ess/24`, or drop `undeclared_fields:`")
        })
}

/// `<kind>.<name>.undeclared_fields`, where every refusal of the key is located.
fn at(kind: ConstructKind, name: &str) -> ConstructRef {
    ConstructRef::new(kind, name).key(UndeclaredFields::KEY)
}
