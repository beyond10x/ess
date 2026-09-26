//! Resolved-type adapter for the domain's shared expression policy.

use ess_domain::expression::{
    self, Checked, ExpressionError, Resolution, ScalarKind, Shape, TypeEnvironment,
};
use ess_primitives::error::{ConstructKind, ConstructRef};
use ess_primitives::{facts::FactPath, predicate::Predicate};

use crate::ir::{EssIr, ResolvedBody, ResolvedCondition, ResolvedField, ResolvedTypeRef};

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
    fn is_instant(&self, reference: &ResolvedTypeRef) -> bool {
        matches!(
            reference,
            ResolvedTypeRef::Primitive {
                name: ess_domain::Primitive::Timestamp
            }
        )
    }
    fn is_duration(&self, reference: &ResolvedTypeRef) -> bool {
        matches!(
            reference,
            ResolvedTypeRef::Primitive {
                name: ess_domain::Primitive::Duration
            }
        )
    }
    fn is_string(&self, reference: &ResolvedTypeRef) -> bool {
        matches!(
            reference,
            ResolvedTypeRef::Primitive {
                name: ess_domain::Primitive::String
            }
        )
    }
    fn is_clock_reading(&self, reference: &ResolvedTypeRef) -> bool {
        matches!(reference, ResolvedTypeRef::Declared { name } if self.ir.types().get(name.name()).is_some_and(|declared| declared.reading.is_some()))
    }
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
                ResolvedBody::Enum { variants } => Shape::Enum(
                    variants
                        .iter()
                        .map(|variant| variant.name().to_owned())
                        .collect(),
                ),
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

/// One predicate the IR holds: where it is written, and the owner fields it reads.
#[derive(Debug, Clone)]
pub struct PredicateSite<'ir> {
    /// The construct it is written on, as `ess-domain`'s admission walk names it.
    pub site: ConstructRef,
    /// The predicate.
    pub predicate: &'ir Predicate,
    /// The fields its free paths are rooted in.
    pub fields: Vec<ResolvedField>,
}

impl PredicateSite<'_> {
    /// Checks this predicate against its owner fields: every read, resolved.
    pub fn check(&self, ir: &EssIr) -> Checked<ResolvedTypeRef> {
        check_predicate(ir, &self.fields, self.predicate, &self.site.render())
    }
}

/// Every predicate the IR holds, each with its site and owner fields.
///
/// Type, entity and struct invariants, outcome guards (the `when`, and the input predicate beside a
/// subject or external condition), `when_subject:` predicates, view filters and binding
/// selections — the IR-side mirror of `ess_domain::primitive_admission::predicates`, which a test
/// holds this to on every model under `examples/`. A consumer that must refuse a construct wherever
/// a predicate reads it asks this walk, so a position cannot be forgotten in one consumer and
/// remembered in another (`docs/design/string-alphabet-and-length.md`, section 6).
pub fn predicate_sites(ir: &EssIr) -> Vec<PredicateSite<'_>> {
    let mut found = Vec::new();
    for declared in ir.types().values() {
        let (fields, invariants) = match &declared.body {
            ResolvedBody::Newtype { of, invariants, .. } => (
                vec![owner_field(ess_domain::NamedType::VALUE, of.clone())],
                invariants,
            ),
            ResolvedBody::Struct { fields, invariants } => (fields.clone(), invariants),
            ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => continue,
        };
        for (index, invariant) in invariants.iter().enumerate() {
            found.push(PredicateSite {
                site: ConstructRef::new(ConstructKind::Type, declared.name.to_string())
                    .key("invariants")
                    .index(index),
                predicate: &invariant.predicate,
                fields: fields.clone(),
            });
        }
    }
    for entity in ir.entities().values() {
        let fields = entity_fields(entity);
        for (index, invariant) in entity.invariants.iter().enumerate() {
            found.push(PredicateSite {
                site: ConstructRef::new(ConstructKind::Entity, entity.name.to_string())
                    .key("invariants")
                    .index(index),
                predicate: &invariant.predicate,
                fields: fields.clone(),
            });
        }
    }
    for command in ir.commands().values() {
        for outcome in &command.outcomes {
            let at = ConstructRef::new(ConstructKind::Command, command.name.to_string())
                .key("outcomes")
                .named(outcome.name.to_string());
            if let Some(predicate) = input_predicate(&outcome.condition) {
                found.push(PredicateSite {
                    site: at.clone(),
                    predicate,
                    fields: command.input.clone(),
                });
            }
            if let ResolvedCondition::SubjectPredicate { predicate, .. } = &outcome.condition {
                let fields = command
                    .selection_subject(outcome)
                    .map(|subject| ir.entity(&subject.entity).fields.clone())
                    .unwrap_or_default();
                found.push(PredicateSite {
                    site: at.key("when_subject"),
                    predicate,
                    fields,
                });
            }
        }
    }
    for view in ir.views().values() {
        if let Some(filter) = &view.filter {
            found.push(PredicateSite {
                site: ConstructRef::new(ConstructKind::View, view.name.to_string()).key("filter"),
                predicate: filter,
                fields: entity_fields(ir.entity(&view.source)),
            });
        }
    }
    for binding in ir.bindings().values() {
        let Some(selection) = &binding.selection else {
            continue;
        };
        for selector in &selection.plan.selectors {
            if let ess_domain::selection::SelectionOperation::First { predicate, .. } =
                &selector.operation
            {
                let item = selection
                    .plan
                    .inputs
                    .get(selector.input)
                    .and_then(|input| resolved(&input.item_type, &selection.types));
                found.push(PredicateSite {
                    site: ConstructRef::new(ConstructKind::Binding, binding.name.to_string())
                        .key("selections")
                        .named(selector.name.clone()),
                    predicate,
                    fields: item
                        .map(|item| vec![owner_field("item", item)])
                        .unwrap_or_default(),
                });
            }
        }
    }
    found
}

/// The input predicate an outcome condition tests, as `OutcomeCondition::predicate` reads it.
fn input_predicate(condition: &ResolvedCondition) -> Option<&Predicate> {
    match condition {
        ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
        ResolvedCondition::SubjectState { predicate, .. }
        | ResolvedCondition::StateChange { predicate, .. }
        | ResolvedCondition::SubjectField { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. } => input.as_ref(),
        ResolvedCondition::Otherwise
        | ResolvedCondition::External { .. }
        | ResolvedCondition::WrongState => None,
    }
}

/// What an entity invariant or a view filter reads: the identity and the declared fields.
fn entity_fields(entity: &crate::ir::ResolvedEntity) -> Vec<ResolvedField> {
    std::iter::once(entity.identity.clone())
        .chain(entity.fields.iter().cloned())
        .collect()
}

fn owner_field(name: &str, type_ref: ResolvedTypeRef) -> ResolvedField {
    ResolvedField {
        name: name.to_owned(),
        type_ref,
        naming: ess_domain::name::Naming::default(),
    }
}

/// A selection's declared item type, through the handles the plan minted.
fn resolved(
    reference: &ess_domain::TypeRef,
    types: &std::collections::BTreeMap<ess_domain::QualifiedName, crate::ir::TypeHandle>,
) -> Option<ResolvedTypeRef> {
    use ess_domain::TypeRef;
    Some(match reference {
        TypeRef::Primitive(name) => ResolvedTypeRef::Primitive { name: *name },
        TypeRef::Named(name) => ResolvedTypeRef::Declared {
            name: types.get(name)?.clone(),
        },
        TypeRef::Optional(of) => ResolvedTypeRef::Optional {
            of: Box::new(resolved(of, types)?),
        },
        TypeRef::List(of) => ResolvedTypeRef::List {
            of: Box::new(resolved(of, types)?),
        },
        TypeRef::Map(key, value) => ResolvedTypeRef::Map {
            key: *key,
            value: Box::new(resolved(value, types)?),
        },
    })
}
