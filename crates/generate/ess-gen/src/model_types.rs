//! Checked type-root selection using the existing model-to-JSON wire mapping.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{ResolvedBody, ResolvedTypeRef};
use ess_compiler::refs::DeclaredTypeRef;
use ess_compiler::EssIr;
use serde_json::Value;

use super::types;
use crate::provenance::{Provenance, ProvenanceMint};

/// A selection or wire-identity error, before any schema properties are collapsed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelTypeError {
    /// Qualified model type, or the root selection when no type was provided.
    pub name: String,
    /// Stable refusal category.
    pub rule: &'static str,
    /// What prevents the selected model from being realized faithfully.
    pub detail: String,
}

/// A sealed projection of selected types from a resolved model, not an imported service.
#[derive(Debug, Clone)]
pub struct ModelTypes {
    roots: BTreeSet<String>,
    definitions: BTreeMap<String, Value>,
    newtypes: BTreeSet<String>,
    binary64: BTreeSet<String>,
    provenance: Provenance,
}

impl ModelTypes {
    /// Close explicitly selected roots through checked type handles and preserve wire mapping.
    pub fn select(ir: &EssIr, roots: &BTreeSet<String>) -> Result<Self, Vec<ModelTypeError>> {
        let mut errors = BTreeSet::new();
        let mut selected = BTreeMap::new();
        if roots.is_empty() {
            errors.insert(ModelTypeError {
                name: String::new(),
                rule: "empty_roots",
                detail: "select at least one model type".to_owned(),
            });
        }
        for root in roots {
            let Some(declared) = ir
                .types()
                .values()
                .find(|item| item.name.to_string() == *root)
            else {
                errors.insert(ModelTypeError {
                    name: root.clone(),
                    rule: "unknown_root",
                    detail: "no resolved model type has this qualified name".to_owned(),
                });
                continue;
            };
            selected.insert(declared.name.clone(), declared);
            for handle in types::reachable(ir, types::body_leaves(&declared.body)) {
                let reached = ir.named_type(handle);
                selected.insert(reached.name.clone(), reached);
            }
        }
        for declared in selected.values() {
            if let ResolvedBody::Struct { fields, .. } = &declared.body {
                let mut wires = BTreeMap::new();
                for field in fields {
                    let wire = types::wire_name(field);
                    if let Some(previous) = wires.insert(wire, &field.name) {
                        errors.insert(ModelTypeError {
                            name: declared.name.to_string(),
                            rule: "wire_field_collision",
                            detail: format!(
                                "fields {previous:?} and {:?} both use wire key {wire:?}",
                                field.name
                            ),
                        });
                    }
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors.into_iter().collect());
        }
        let provenance = ProvenanceMint::new(ir)
            .of_seeds(
                selected
                    .keys()
                    .map(|name| DeclaredTypeRef::new(name.clone()).into()),
            )
            .provenance;
        let mut binary64 = BTreeSet::new();
        for item in selected.values() {
            body_numeric_locations(item, &mut binary64);
        }
        Ok(Self {
            binary64,
            roots: roots.clone(),
            definitions: selected
                .values()
                .map(|item| {
                    (
                        item.name.to_string(),
                        serde_json::to_value(types::body(item))
                            .expect("typed schema nodes serialize"),
                    )
                })
                .collect(),
            newtypes: selected
                .values()
                .filter(|item| matches!(item.body, ResolvedBody::Newtype { .. }))
                .map(|item| item.name.to_string())
                .collect(),
            provenance,
        })
    }

    /// Exact selected roots, distinct from their transitive closure.
    pub fn roots(&self) -> &BTreeSet<String> {
        &self.roots
    }

    /// The complete reference closure, using the existing JSON Schema wire mapping.
    pub fn definitions(&self) -> &BTreeMap<String, Value> {
        &self.definitions
    }

    /// Nominal model identities that structural host aliases alone cannot enforce.
    pub fn newtypes(&self) -> &BTreeSet<String> {
        &self.newtypes
    }

    /// Exact schema-node locations whose finite Binary64 identity was minted from checked types.
    /// These are private projection metadata, never recovered from serialized annotations.
    pub fn binary64_locations(&self) -> &BTreeSet<String> {
        &self.binary64
    }

    /// Existing model and contract provenance, minted from the resolved input.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Retain the projected definitions and model provenance as a JSON Schema document.
    pub fn to_json(&self) -> String {
        let document = serde_json::json!({
            "$schema": super::DIALECT,
            "$defs": self.definitions,
            "x-ess-provenance": self.provenance,
        });
        format!(
            "{}\n",
            serde_json::to_string_pretty(&document).expect("typed selection serializes")
        )
    }
}

fn pointer(at: &str, key: &str) -> String {
    format!("{at}/{}", key.replace('~', "~0").replace('/', "~1"))
}

fn numeric_locations(reference: &ResolvedTypeRef, at: &str, found: &mut BTreeSet<String>) {
    match reference {
        ResolvedTypeRef::Primitive {
            name: ess_domain::Primitive::Binary64,
        } => {
            found.insert(at.to_owned());
        }
        ResolvedTypeRef::Optional { of } => numeric_locations(of, &format!("{at}/anyOf/0"), found),
        ResolvedTypeRef::List { of } => numeric_locations(of, &format!("{at}/items"), found),
        ResolvedTypeRef::Map { value, .. } => {
            numeric_locations(value, &format!("{at}/additionalProperties"), found);
        }
        ResolvedTypeRef::Declared { .. } | ResolvedTypeRef::Primitive { .. } => {}
    }
}

fn body_numeric_locations(item: &ess_compiler::ir::ResolvedType, found: &mut BTreeSet<String>) {
    let at = pointer("/$defs", &item.name.to_string());
    match &item.body {
        ResolvedBody::Newtype { of, .. } => numeric_locations(of, &at, found),
        ResolvedBody::Struct { fields, .. } => {
            for field in fields {
                numeric_locations(
                    field.type_ref.required(),
                    &pointer(&format!("{at}/properties"), types::wire_name(field)),
                    found,
                );
            }
        }
        ResolvedBody::Union { tag, variants } => {
            let content = types::content_key(tag);
            for (index, (_, payload)) in variants.iter().enumerate() {
                numeric_locations(
                    payload.required(),
                    &pointer(&format!("{at}/oneOf/{index}/properties"), content),
                    found,
                );
            }
        }
        ResolvedBody::Enum { .. } => {}
    }
}
