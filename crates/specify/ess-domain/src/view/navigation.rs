//! A view reads the row its source's reference names, one hop, as it is when the view is read
//! (`ess/28`, beyond10x/ess#564, beyond10x/ess#557, `docs/design/generic-language-features.md`
//! § G2, navigation).
//!
//! ```yaml
//! source: incident.Window
//! fields:
//!   - {name: window_id, type: incident.WindowId}
//!   - {name: incident_state, type: Optional<incident.Incident.State>,
//!      field: {related: {via: incident_id, field: state}}}
//! filter: incident_id.state == Resolved
//! ```
//!
//! A reference is a field of the source entity typed as another entity's identity (or `Optional`
//! of one), the entity a `references`/`owns` relation on it names — the rule
//! [`crate::command::related_value::referenced_entity`] states for every `{related: …}` read. A
//! view field with `field: {related: {via, field}}` reads `field` (a declared field, the identity
//! or `state`) of the row `via` names; a filter path `<via>.<field>` reads the same value. Where
//! the reference is absent or names no row the value is absent: a projected field holds nothing,
//! so it is declared `Optional`, and a comparison over it is unknown, so the filter shows no row.
//!
//! This cut navigates one hop, on a view projecting rows: an aggregate view, `paging:`, `absent:`
//! and an `order_by:` key read through a reference are refused by name.

use std::collections::BTreeSet;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::facts::FactPath;

use super::{join, ViewSpec};
use crate::command::related_value::{referenced_entity, Referenced};
use crate::entity::EntitySpec;
use crate::system::FormatVersion;
use crate::types::{Field, TypeRef};
use crate::Specification;

/// What a view field reads where it is not its source's own field of that name (ess/28):
/// `field: {related: {via, field}}`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ViewFieldSource {
    /// A field of the row a reference of the source names.
    pub related: ViewRelated,
}

/// `{related: {via, field}}` on a view field: `field` of the row the source's `via` names.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ViewRelated {
    /// The source field holding the other row's identity.
    pub via: String,
    /// The field of that row: a declared field, its identity, or `state`.
    pub field: String,
}

/// One reference a view's source holds, which its filter and fields may navigate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewReference {
    /// The source field holding the other row's identity.
    pub via: String,
    /// The entity it names.
    pub entity: crate::name::QualifiedName,
    /// That entity's observable fields: its identity, its declared fields and `state`.
    pub fields: Vec<Field>,
}

/// Every reference `view`'s source holds: each field, the identity included, whose type (through an
/// `Optional`) is one entity's identity by [`referenced_entity`]'s rule — the identity only where a
/// relation makes it name another entity's row. Empty where the source is not declared.
pub fn references(spec: &Specification, view: &ViewSpec) -> Vec<ViewReference> {
    source_references(spec, &view.source)
}

/// [`references`] of a view whose source is `source`, for a view that is not in `spec`
/// ([`super::from`]).
pub(crate) fn source_references(
    spec: &Specification,
    source: &crate::name::QualifiedName,
) -> Vec<ViewReference> {
    let Some(source) = spec.entities().get(source) else {
        return Vec::new();
    };
    std::iter::once(&source.identity)
        .chain(&source.fields)
        .filter_map(|field| {
            let identity = match &field.type_ref {
                TypeRef::Optional(of) => of.as_ref(),
                other => other,
            };
            if !matches!(identity, TypeRef::Named(_) | TypeRef::Primitive(_)) {
                return None;
            }
            let Referenced::Entity(target) =
                referenced_entity(spec, identity, Some((source, &field.name)))
            else {
                return None;
            };
            if field.name == source.identity.name
                && crate::command::related_value::identity_reference(source).is_none()
            {
                return None;
            }
            Some(ViewReference {
                via: field.name.clone(),
                entity: target.name.clone(),
                fields: target.observable_fields(),
            })
        })
        .collect()
}

/// The paths of `view`'s filter that navigate one of `references`: `<via>.<member>` where `<via>`
/// is a reference and `<member>` an observable field of the row it names, and no struct member of
/// `<via>`'s own type is called that.
pub fn navigated_paths<'v>(
    spec: &Specification,
    view: &'v ViewSpec,
    references: &[ViewReference],
) -> Vec<&'v FactPath> {
    let Some(filter) = &view.filter else {
        return Vec::new();
    };
    let Some(source) = spec.entities().get(&view.source) else {
        return Vec::new();
    };
    filter
        .fact_paths()
        .into_iter()
        .filter(|path| {
            let [root, member, ..] = path.segments() else {
                return false;
            };
            references.iter().any(|reference| {
                reference.via == *root
                    && reference.fields.iter().any(|field| field.name == *member)
                    && !struct_member(spec, source, root, member)
            })
        })
        .collect()
}

/// Whether the source field `root` is a struct (through `Optional` and newtypes) with a member
/// `member`, which a path reads as it always did (beyond10x/ess#428).
fn struct_member(spec: &Specification, source: &EntitySpec, root: &str, member: &str) -> bool {
    let Some(field) = std::iter::once(&source.identity)
        .chain(&source.fields)
        .find(|field| field.name == root)
    else {
        return false;
    };
    let mut held = &field.type_ref;
    for _ in 0..=crate::types::MAX_TYPE_DEPTH {
        match held {
            TypeRef::Optional(of) => held = of,
            TypeRef::Named(name) => match spec.system().types.get(name).map(|t| &t.body) {
                Some(crate::types::TypeBody::Newtype { of, .. }) => held = of,
                Some(crate::types::TypeBody::Struct { fields, .. }) => {
                    return fields.iter().any(|field| field.name == member)
                }
                _ => return false,
            },
            _ => return false,
        }
    }
    false
}

impl ViewSpec {
    /// Whether this view reads through a reference: a field declares `field: {related: …}`.
    pub fn reads_related(&self) -> bool {
        !self.related.is_empty()
    }

    /// The source-format gate: a view field reading a referenced row and a filter path navigating a
    /// reference are refused below `ess/28`, each at the key the author wrote.
    pub fn navigation_admission(
        &self,
        spec: &Specification,
        format: FormatVersion,
    ) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if format.major() >= FormatVersion::V28.major() {
            return errors;
        }
        for name in self.related.keys() {
            let index = self
                .fields
                .iter()
                .position(|field| &field.name == name)
                .unwrap_or_default();
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("view.{}.fields[{index}].field", self.name),
                    format!(
                        "`{name}` reads the row a reference names (`field: {{related: …}}`), \
                         which requires specification format ess/28"
                    ),
                )
                .with_hint(
                    "declare `format: ess/28`, or copy the value onto the row when it is written",
                ),
            );
        }
        let references = references(spec, self);
        if let Some(path) = navigated_paths(spec, self, &references).first() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("view.{}.filter", self.name),
                    format!(
                        "`{path}` reads the row a reference names, which requires specification \
                         format ess/28"
                    ),
                )
                .with_hint("declare `format: ess/28`, or filter on a value copied onto the row"),
            );
        }
        errors
    }
}

/// Checks what `view` reads through its source's references: each related field's reference, the
/// field it reads and the type it is declared at, and where navigation stands beside an aggregate,
/// `paging:`, `absent:` or an order. Nothing for a view that navigates nothing.
pub(crate) fn validate(spec: &Specification, view: &ViewSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(source) = spec.entities().get(&view.source) else {
        return errors;
    };
    let references = references(spec, view);
    let navigated = navigated_paths(spec, view, &references);
    if view.related.is_empty() && navigated.is_empty() {
        return errors;
    }
    let at = |suffix: &str| format!("view.{}.{suffix}", view.name);
    let observable = source.observable_fields();
    for (name, read) in &view.related {
        let index = view
            .fields
            .iter()
            .position(|field| &field.name == name)
            .unwrap_or_default();
        let field_at = at(&format!("fields[{index}].field"));
        if observable.iter().any(|field| &field.name == name) {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    field_at.clone(),
                    format!(
                        "`{name}` is a field of `{}` and reads `{}` of the row `{}` names; a view \
                         field reads one of the two",
                        view.source, read.field, read.via
                    ),
                )
                .with_hint("give the field read through the reference a name of its own"),
            );
            continue;
        }
        if let Some(refused) = related_field(
            spec,
            view,
            (source, &observable),
            &references,
            (name, read),
            &field_at,
        ) {
            errors.push(refused);
        }
    }
    let read_through = view.related.keys().cloned().collect::<BTreeSet<_>>();
    let refuse = |key: &str, what: &str| {
        ValidationError::new(
            ValidationCode::UnsupportedConstruct,
            at(key),
            format!(
                "`{}` reads through a reference of `{}`, and {what} is not read through one in \
                 this cut",
                view.name, view.source
            ),
        )
        .with_hint("read the referenced row in a view of its own, or drop the navigation")
    };
    if view.is_aggregate() {
        errors.push(refuse("fields", "an aggregate view"));
    }
    if view.paging.is_some() {
        errors.push(refuse("paging", "a paged view"));
    }
    if view.absent.is_some() {
        errors.push(refuse(
            "absent",
            "a view's answer for an identity no row carries",
        ));
    }
    for (index, key) in view.order_by.iter().enumerate() {
        if read_through.contains(&key.field) {
            errors.push(refuse(&format!("order_by[{index}]"), "an order key"));
        }
    }
    errors
}

/// The refusal of one view field reading `read` through a reference, where it is refused.
fn related_field(
    spec: &Specification,
    view: &ViewSpec,
    (source, observable): (&EntitySpec, &[Field]),
    references: &[ViewReference],
    (name, read): (&String, &ViewRelated),
    at: &str,
) -> Option<ValidationError> {
    let Some(via) = observable.iter().find(|field| field.name == read.via) else {
        return Some(
            ValidationError::new(
                ValidationCode::UndeclaredReference,
                at.to_owned(),
                format!(
                    "`{name}` reads through `{}`, which is not a field of `{}`",
                    read.via, view.source
                ),
            )
            .with_hint(format!(
                "fields of `{}`: {}",
                view.source,
                join(observable.iter().map(|field| field.name.clone()))
            )),
        );
    };
    let Some(reference) = references
        .iter()
        .find(|reference| reference.via == read.via)
    else {
        return Some(
            ValidationError::new(
                ValidationCode::TypeMismatch,
                at.to_owned(),
                format!(
                    "`{name}` reads through `{}.{}`, which is `{}` and names no one entity's row",
                    source.name, read.via, via.type_ref
                ),
            )
            .with_hint(
                "read through a field typed as another entity's identity, or `Optional` of one, \
                 that a `references` relation names",
            ),
        );
    };
    let Some(far) = reference
        .fields
        .iter()
        .find(|field| field.name == read.field)
    else {
        return Some(
            ValidationError::new(
                ValidationCode::UndeclaredReference,
                at.to_owned(),
                format!(
                    "`{}` is not a field of `{}`, the entity `{}` names",
                    read.field, reference.entity, read.via
                ),
            )
            .with_hint(format!(
                "`{}` holds: {}",
                reference.entity,
                join(reference.fields.iter().map(|field| field.name.clone()))
            )),
        );
    };
    let declared = view.fields.iter().find(|field| &field.name == name)?;
    let absent = match &far.type_ref {
        TypeRef::Optional(_) => far.type_ref.clone(),
        other => TypeRef::Optional(Box::new(other.clone())),
    };
    if !matches!(declared.type_ref, TypeRef::Optional(_)) {
        return Some(
            ValidationError::new(
                ValidationCode::TypeMismatch,
                at.to_owned(),
                format!(
                    "`{name}` reads `{}.{}` through `{}`, which is absent where the reference is \
                     or names no row; `{name}` is `{}`, which cannot be left absent",
                    reference.entity, read.field, read.via, declared.type_ref
                ),
            )
            .with_hint(format!("declare `{name}` as `{absent}`")),
        );
    }
    if !crate::types::is_assignable(&absent, &declared.type_ref) {
        return Some(ValidationError::new(
            ValidationCode::TypeMismatch,
            at.to_owned(),
            format!(
                "`{name}` is `{}`, and `{}.{}` read through `{}` is `{absent}`",
                declared.type_ref, reference.entity, read.field, read.via
            ),
        ));
    }
    let _ = spec;
    None
}
