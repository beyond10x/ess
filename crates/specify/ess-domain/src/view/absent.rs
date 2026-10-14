//! A view's answer for an identity no row carries (`absent:`, ess/25, beyond10x/ess#533,
//! `docs/design/read-api-view-idioms.md`, "An identity no row carries").
//!
//! ```yaml
//! params: [{name: id, type: demo.status.FlagId}]
//! filter: id == param.id
//! absent:
//!   row: {paused: false}
//! ```
//!
//! A view whose filter is the one comparison `<identity field> == param.<name>`, over a parameter
//! that is not `Optional`, selects one row of its source by identity. Without this block a read for
//! an identity no row carries answers no row; with it, the read answers one default row: the
//! identity field, where the view projects it, holds the requested value, each field `row:` names
//! holds its literal, and every other field is `Optional` and absent. A read for a stored identity
//! answers that row, as before.
//!
//! This cut admits the default row only. An error answer, `absent: {error: <Error>}`, needs a suite
//! expectation a view read does not have, and is the additive next step; until then `error:` is
//! refused as an unknown key.

use std::collections::BTreeMap;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

use super::{join, ViewSpec};
use crate::types::{AttributeLiteral, TypeRegistry};

/// What a view read answers for an identity no row of its source carries.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Absent {
    /// The default row: each projected field it names, with its literal value, ordered by name.
    ///
    /// The identity field is never here: the default row holds the requested value there.
    pub row: BTreeMap<String, AttributeLiteral>,
}

impl ViewSpec {
    /// The parameter this view selects its source's row by, where its filter is the one comparison
    /// `<identity> == param.<name>`, compared by value, in either operand order; `None` otherwise.
    ///
    /// The condition `absent:` is admitted under, and the one the compiler reads the parameter
    /// from.
    pub fn identity_parameter(&self, identity: &str) -> Option<&str> {
        let Some(Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq,
            right: Operand::Fact(right),
            kind: CompareKind::Value,
        }) = &self.filter
        else {
            return None;
        };
        let is_identity =
            |path: &ess_primitives::facts::FactPath| path.segments() == [identity.to_owned()];
        if is_identity(left) {
            param(right)
        } else if is_identity(right) {
            param(left)
        } else {
            None
        }
        .filter(|name| self.params.iter().any(|declared| declared.name == *name))
    }

    /// The source-format gate: `absent:` is refused below `ess/25` at the key the author wrote.
    ///
    /// Beside [`Self::paging_admission`] in `primitive_admission::specification`.
    pub fn absent_admission(&self, format: crate::system::FormatVersion) -> ValidationErrors {
        let mut errors = ValidationErrors::new();
        if self.absent.is_some() && format.major() < crate::system::FormatVersion::V25.major() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    format!("view.{}.absent", self.name),
                    "a view's answer for an identity no row carries (`absent:`) requires \
                     specification format ess/25",
                )
                .with_hint("declare `format: ess/25`, or drop `absent:`"),
            );
        }
        errors
    }
}

/// The parameter name `path` reads, where it is `param.<name>`; `None` otherwise.
fn param(path: &ess_primitives::facts::FactPath) -> Option<&str> {
    match path.segments() {
        [namespace, name] if namespace == ViewSpec::PARAM => Some(name.as_str()),
        _ => None,
    }
}

/// Checks `view`'s `absent:` against its source entity: the filter selects by identity alone, from
/// a parameter that is not `Optional`, no aggregate or paging beside it, and the default row names
/// projected fields other than the identity, gives every required one a value, and holds literals
/// of their types, inside every alphabet and invariant those types declare.
///
/// `entity` is the view's source, `None` where it is not declared, which [`ViewSpec::validate`]
/// already refuses.
pub(crate) fn validate(
    view: &ViewSpec,
    entity: Option<&crate::entity::EntitySpec>,
    types: &TypeRegistry,
    conversions: &crate::types::ConversionRegistry,
) -> ValidationErrors {
    let (Some(absent), Some(entity)) = (&view.absent, entity) else {
        return ValidationErrors::new();
    };
    let mut errors = selection(view, entity);
    errors.extend(row(view, absent, entity, types, conversions));
    errors
}

/// The view's shape around `absent:`: its filter selects `entity` by identity alone, with no
/// aggregate or paging beside it.
fn selection(view: &ViewSpec, entity: &crate::entity::EntitySpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = format!("view.{}.absent", view.name);
    let identity = entity.identity.name.as_str();
    // The identity the default row holds is the one the read requested, so it must be given: a
    // read that leaves an `Optional` parameter out requests no identity, which is not "an identity
    // no row carries" (adversary pass 1 of beyond10x/ess#533). Refused rather than answered with
    // no row, which keeps the key's surface narrow and leaves admitting it a later, additive choice.
    if let Some(param) = view
        .identity_parameter(identity)
        .and_then(|name| view.params.iter().find(|param| param.name == name))
        .filter(|param| param.type_ref.is_optional())
    {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at.clone(),
                format!(
                    "`{}` declares `absent:`, and its identity parameter `{}` is `{}`: a read \
                     leaving it out requests no identity, so the default row has none to hold",
                    view.name, param.name, param.type_ref
                ),
            )
            .with_hint(format!(
                "declare `{}` without `Optional`, or drop `absent:`",
                param.name
            )),
        );
    }
    if view.identity_parameter(identity).is_none() {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at.clone(),
                format!(
                    "`{}` declares `absent:`, the answer for an identity no row carries, and its \
                     filter does not select `{}` by its identity field `{identity}`: the filter \
                     must be `{identity} == param.<name>` and nothing else",
                    view.name, entity.name
                ),
            )
            .with_hint(format!(
                "write `filter: {identity} == param.<name>`, or drop `absent:`"
            )),
        );
    }
    for (beside, why) in [
        (
            view.aggregation.is_some(),
            "an aggregate (`group_by:` or an `aggregate:` field), whose row is computed over rows",
        ),
        (
            view.paging.is_some(),
            "`paging:`, whose answer is a page of rows",
        ),
    ] {
        if beside {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    at.clone(),
                    format!(
                        "`{}` declares `absent:` beside {why}; a default row is one projected \
                         row",
                        view.name
                    ),
                )
                .with_hint("drop `absent:`"),
            );
        }
    }
    errors
}

/// The default row: it names projected fields other than the identity, gives every required one a
/// value, and holds literals of their types.
fn row(
    view: &ViewSpec,
    absent: &Absent,
    entity: &crate::entity::EntitySpec,
    types: &TypeRegistry,
    conversions: &crate::types::ConversionRegistry,
) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let at = format!("view.{}.absent", view.name);
    let identity = entity.identity.name.as_str();
    let Some(projected) = view.projected_fields(types) else {
        return errors;
    };
    let inhabitation = crate::system::Inhabitation::of(types);
    for (name, literal) in &absent.row {
        let field_at = format!("{at}.row.{name}");
        let Some(field) = projected.iter().find(|field| &field.name == name) else {
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    field_at,
                    format!(
                        "`{}` does not project `{name}`, so its default row cannot hold it",
                        view.name
                    ),
                )
                .with_hint(format!(
                    "projected fields: {}",
                    join(projected.iter().map(|field| &field.name))
                )),
            );
            continue;
        };
        if name == identity {
            errors.push(
                ValidationError::new(
                    ValidationCode::ConflictingDeclaration,
                    field_at,
                    format!(
                        "`{name}` is the identity `{}` is read by, and the default row holds the \
                         requested value there",
                        view.name
                    ),
                )
                .with_hint(format!("drop `{name}` from `row:`")),
            );
            continue;
        }
        if let Some((reason, hint)) = crate::command::literal_refusal(
            (&view.name, name),
            field,
            (&literal.value, literal.scalar),
            "view's default row (`absent: {row: …}`)",
            &crate::command::DefaultRowOf(&view.name),
            (types, conversions, &inhabitation),
        ) {
            errors.push(
                ValidationError::new(ValidationCode::TypeMismatch, field_at, reason)
                    .with_hint(hint),
            );
        }
    }
    for field in projected {
        if field.name == identity
            || field.type_ref.is_optional()
            || absent.row.contains_key(&field.name)
        {
            continue;
        }
        errors.push(
            ValidationError::new(
                ValidationCode::MissingDeclaration,
                format!("{at}.row"),
                format!(
                    "the default row of `{}` gives no value for `{}`, which is not `Optional`",
                    view.name, field.name
                ),
            )
            .with_hint(format!("add `{}: <literal>` to `row:`", field.name)),
        );
    }
    errors
}
