//! Checked type-root selection using the existing model-to-JSON wire mapping.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::ResolvedBody;
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
        Ok(Self {
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
