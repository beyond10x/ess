//! Resolved-type adapter for the domain's shared expression policy.

use ess_domain::expression::{
    self, Checked, ExpressionError, Resolution, ScalarKind, Shape, TypeEnvironment,
};
use ess_primitives::{facts::FactPath, predicate::Predicate};

use crate::ir::{EssIr, ResolvedBody, ResolvedField, ResolvedTypeRef};

/// Read-only access to an IR owner without rebuilding domain declarations.
pub struct Environment<'a> {
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
}

impl<'a> Environment<'a> {
    /// Observe exactly these declared fields.
    pub fn new(ir: &'a EssIr, fields: &'a [ResolvedField]) -> Self {
        Self { ir, fields }
    }
}

impl TypeEnvironment for Environment<'_> {
    type Type = ResolvedTypeRef;

    fn root(&self, name: &str) -> Option<Self::Type> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
    fn cardinality_type(&self) -> Self::Type {
        ResolvedTypeRef::Primitive {
            name: ess_domain::Primitive::Integer,
        }
    }
    fn shape(&self, reference: &Self::Type) -> Result<Shape<Self::Type>, String> {
        Ok(match reference {
            ResolvedTypeRef::Primitive { name } => Shape::Scalar(ScalarKind::of(*name)),
            ResolvedTypeRef::Optional { of } => Shape::Optional((**of).clone()),
            ResolvedTypeRef::List { of } => Shape::List((**of).clone()),
            ResolvedTypeRef::Map { value, .. } => Shape::Map((**value).clone()),
            ResolvedTypeRef::Declared { name } => match &self
                .ir
                .types()
                .get(name.name())
                .ok_or_else(|| name.to_string())?
                .body
            {
                ResolvedBody::Newtype { of, .. } => Shape::Alias(of.clone()),
                ResolvedBody::Struct { .. } => Shape::Struct,
                ResolvedBody::Enum { variants } => Shape::Enum(variants.clone()),
                ResolvedBody::Union { .. } => Shape::Union,
            },
        })
    }
    fn member(&self, reference: &Self::Type, name: &str) -> Option<Self::Type> {
        let ResolvedTypeRef::Declared { name: declared } = reference else {
            return None;
        };
        let ResolvedBody::Struct { fields, .. } = &self.ir.types().get(declared.name())?.body
        else {
            return None;
        };
        fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.type_ref.clone())
    }
}

/// Complete path typing, independent of which producer can supply its facts.
pub fn resolve_path(
    ir: &EssIr,
    fields: &[ResolvedField],
    path: &FactPath,
    owner: &str,
) -> Result<Resolution<ResolvedTypeRef>, ExpressionError> {
    expression::resolve_path(&Environment::new(ir, fields), path, owner)
}

/// Check all operands, scopes and collection accesses using the domain policy.
pub fn check_predicate(
    ir: &EssIr,
    fields: &[ResolvedField],
    predicate: &Predicate,
    owner: &str,
) -> Checked<ResolvedTypeRef> {
    expression::check_predicate(&Environment::new(ir, fields), predicate, owner)
}
