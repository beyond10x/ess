//! A guard over the rows a `List` of references names (source format `ess/28`,
//! beyond10x/ess#558, `docs/design/generic-language-features.md` § G2, quantifiers).
//!
//! `when_related: {via: input.<list>, exists: false}` over an input typed `List<…>` of one entity's
//! identity is taken when some element names no row of that entity: `not forall(id in input.<list>:
//! exists(E[id]))`. An empty list names no missing row, so it selects no such branch. The one value
//! the branch publishes about it is the first element, in list order, that names no row:
//! `{missing: input.<list>}`, in an error or event payload of that branch, typed as the element.
//!
//! In this cut a list is tested for a missing row alone: a `predicate:` over each named row, a list
//! in `all:`, a stored list field and a command reading another related row beside the list are
//! refused by name. Below `ess/28` the guard keeps its refusal (`when_related` reads the one row an
//! identity names), its hint naming `ess/28`, and `{missing: …}` is the nested mapping it always
//! was.
use std::collections::BTreeMap;

use super::{CommandSpec, OutcomeCondition, PayloadSource, RelatedVia};
use crate::name::QualifiedName;
use crate::spec::Specification;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRef};
use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError, ValidationErrors};

/// The key of the value source: `{missing: input.<list>}`.
pub const MISSING: &str = "missing";

/// The element type of a `List<…>` input a `when_related:` guard may read from `ess/28`; `None`
/// for any other type.
pub fn element(declared: &TypeRef) -> Option<&TypeRef> {
    match declared {
        TypeRef::List(element) => Some(element),
        _ => None,
    }
}

/// Whether `via` is an input the command declares as a `List<…>`.
pub fn is_list(command: &CommandSpec, via: &RelatedVia) -> bool {
    match via {
        RelatedVia::Input(field) => command
            .input_field(field)
            .is_some_and(|read| element(&read.type_ref).is_some()),
        RelatedVia::Subject(_) => false,
    }
}

/// Whether the command guards on the rows a list names.
pub fn uses(command: &CommandSpec) -> bool {
    command.outcomes.iter().any(|outcome| {
        matches!(&outcome.condition, OutcomeCondition::Related { via, also, .. }
            if is_list(command, via) || also.iter().any(|row| is_list(command, &row.via)))
    })
}

/// The refusals of a guard over a list of references: below `ess/28`, with a `predicate:`, in
/// `all:`, and beside another related row. Empty where the command reads no list, or reads one as
/// this cut admits.
pub(super) fn gates(spec: &Specification, command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    if !uses(command) {
        return errors;
    }
    let below = spec.system().format.major() < FormatVersion::V28.major();
    let lists: Vec<&RelatedVia> = super::related_guard::read_vias(command)
        .into_iter()
        .filter(|via| is_list(command, via))
        .collect();
    let others = super::related_guard::read_vias(command).len() - lists.len();
    for outcome in &command.outcomes {
        let OutcomeCondition::Related {
            via, test, also, ..
        } = &outcome.condition
        else {
            continue;
        };
        let at = site(command, outcome);
        let listed = std::iter::once(via)
            .chain(also.iter().map(|row| &row.via))
            .find(|via| is_list(command, via));
        let Some(list) = listed else {
            continue;
        };
        let read = command
            .input_field(list.field())
            .map(|read| read.type_ref.to_string())
            .unwrap_or_default();
        if below {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::TypeMismatch,
                    format!(
                        "`{list}` is `{read}`, and `when_related` reads the one row an identity \
                         names; a guard over every row a list of references names requires \
                         specification format ess/28"
                    ),
                )
                .with_hint(
                    "declare `format: ess/28` to refuse a list naming a missing row with `exists: \
                     false`, or read an input typed as the other entity's identity",
                ),
            );
            continue;
        }
        if !also.is_empty() {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "outcome `{}` lists `{list}`, a list of references, in `all:`; `all:` \
                         requires rows one identity each names",
                        outcome.name
                    ),
                )
                .with_hint("guard the list on a branch of its own: `{via: <list>, exists: false}`"),
            );
            continue;
        }
        if test.predicate().is_some() {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::UnsupportedConstruct,
                    format!(
                        "outcome `{}` tests the rows the list `{list}` names with a `predicate:`; \
                         a list of references is tested for a missing row only in this cut",
                        outcome.name
                    ),
                )
                .with_hint(
                    "write `exists: false`, taken when some element names no row; test each \
                     row's fields with a row-set guard (`entity`, `where`, `forall`)",
                ),
            );
            continue;
        }
        if lists.len() > 1 || others > 0 {
            errors.push(
                ValidationError::at(
                    at,
                    ValidationCode::ConflictingDeclaration,
                    format!(
                        "`{}` guards on the rows the list `{list}` names and on another related \
                         row; a command guarding on a list of references reads no other related \
                         row in this cut",
                        command.name
                    ),
                )
                .with_hint("split the guards across commands, or drop one"),
            );
        }
    }
    errors
}

fn site(command: &CommandSpec, outcome: &super::Outcome) -> ConstructRef {
    command
        .site()
        .key("outcomes")
        .named(outcome.name.as_str())
        .key(super::related_guard::KEY)
}

/// `{missing: input.<list>}` as written.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawMissingSource {
    /// The list input a `when_related: {via: input.<list>, exists: false}` guard of the branch
    /// reads, written `input.<list>`.
    missing: String,
}

impl RawMissingSource {
    /// The mapping `{missing: input.<field>}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, super::RawPayloadSource)]) -> Option<Self> {
        match entries {
            [(key, super::RawPayloadSource::Text(read))]
                if key == MISSING
                    && read
                        .strip_prefix(PayloadSource::INPUT_PREFIX)
                        .is_some_and(crate::binding::is_field_name) =>
            {
                Some(Self {
                    missing: read.clone(),
                })
            }
            _ => None,
        }
    }

    /// The source it is.
    pub(super) fn into_source(self) -> PayloadSource {
        let field = self
            .missing
            .strip_prefix(PayloadSource::INPUT_PREFIX)
            .unwrap_or(&self.missing)
            .to_owned();
        PayloadSource::MissingElement { field }
    }

    /// The written form of a read of `field`.
    pub(super) fn of(field: &str) -> Self {
        Self {
            missing: format!("{}{field}", PayloadSource::INPUT_PREFIX),
        }
    }
}

/// Reads `{missing: input.<list>}` back as the nested mapping it was below `ess/28`.
pub fn read_below_ess_28(
    format: FormatVersion,
    commands: &mut BTreeMap<QualifiedName, CommandSpec>,
) {
    if format.major() >= FormatVersion::V28.major() {
        return;
    }
    for outcome in commands
        .values_mut()
        .flat_map(|command| command.outcomes.iter_mut())
    {
        let affects = outcome
            .set_effects
            .affects
            .iter_mut()
            .flat_map(|affect| affect.sets.values_mut());
        for source in outcome
            .payload
            .values_mut()
            .flat_map(|fields| fields.values_mut())
            .chain(outcome.error_payload.values_mut())
            .chain(outcome.sets.values_mut())
            .chain(affects)
        {
            nested_missing(source);
        }
    }
}

fn nested_missing(source: &mut PayloadSource) {
    match source {
        PayloadSource::MissingElement { field } => {
            *source = PayloadSource::Struct {
                fields: vec![super::PayloadField {
                    target: MISSING.to_owned(),
                    source: PayloadSource::InputField {
                        field: field.clone(),
                    },
                }],
            };
        }
        PayloadSource::Struct { fields } => {
            for leaf in fields {
                nested_missing(&mut leaf.source);
            }
        }
        PayloadSource::InputOrGenerated {
            otherwise: Some(otherwise),
            ..
        } => nested_missing(otherwise),
        _ => {}
    }
}

/// The check of `{missing: input.<field>}` filling `target`: admitted as a whole payload field of a
/// branch guarded by `when_related: {via: input.<field>, exists: false}` over a `List<…>` input,
/// at the element's type; refused by name anywhere else.
pub(super) fn check(
    command: &CommandSpec,
    outcome: &super::Outcome,
    at: &ConstructRef,
    target: &Field,
    field: &str,
    whole_payload_field: bool,
    permits: impl Fn(&TypeRef, &TypeRef) -> bool,
) -> Option<ValidationError> {
    let refuse = |code: ValidationCode, message: String, hint: &str| {
        Some(ValidationError::at(at.clone(), code, message).with_hint(hint.to_owned()))
    };
    if !whole_payload_field {
        return refuse(
            ValidationCode::UnsupportedConstruct,
            format!(
                "`{{missing: input.{field}}}` fills a whole field of a refusal's or an event's \
                 payload, not a `sets:` entry or a field inside a nested mapping"
            ),
            "publish the missing element in a payload field of its own",
        );
    }
    let guards = matches!(&outcome.condition, OutcomeCondition::Related {
        via: RelatedVia::Input(read),
        test: super::related_guard::RelatedTest::Absent,
        ..
    } if read == field);
    let Some(element) = command
        .input_field(field)
        .and_then(|read| element(&read.type_ref))
        .filter(|_| guards)
    else {
        return refuse(
            ValidationCode::UndeclaredReference,
            format!(
                "`{{missing: input.{field}}}` reads the first element of `input.{field}` that names \
                 no row, and outcome `{}` is not guarded by `when_related: {{via: input.{field}, \
                 exists: false}}` over a list of references",
                outcome.name
            ),
            "read it on the branch whose guard refuses the list for a missing row",
        );
    };
    if permits(element, &target.type_ref) {
        return None;
    }
    refuse(
        ValidationCode::TypeMismatch,
        format!(
            "`{{missing: input.{field}}}` is `{element}`, and `{}` is `{}`",
            target.name, target.type_ref
        ),
        "type the field as the list's element",
    )
}
