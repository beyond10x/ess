//! Checked type-root selection using the existing model-to-JSON wire mapping.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{ResolvedBody, ResolvedField, ResolvedTypeRef};
use ess_compiler::refs::{DeclaredTypeRef, EventRef};
use ess_compiler::EssIr;
use ess_domain::name::QualifiedName;
use serde::Serialize;
use serde_json::Value;

use super::types;
use crate::provenance::{Provenance, ProvenanceMint};

/// The declared construct whose existing wire record is selected for realization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "name", rename_all = "snake_case")]
pub enum ModelRoot {
    /// A named model type, as selected by the original type-only API.
    Type(QualifiedName),
    /// An event's declared payload record, without transport or envelope behavior.
    Event(QualifiedName),
}

impl ModelRoot {
    /// Qualified identity, which is also the selected schema-definition key.
    pub fn name(&self) -> &QualifiedName {
        match self {
            Self::Type(name) | Self::Event(name) => name,
        }
    }
}

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
    model_roots: BTreeSet<ModelRoot>,
    definitions: BTreeMap<String, Value>,
    newtypes: BTreeSet<String>,
    binary64: BTreeSet<String>,
    provenance: Provenance,
}

impl ModelTypes {
    /// Close explicitly selected roots through checked type handles and preserve wire mapping.
    pub fn select(ir: &EssIr, roots: &BTreeSet<String>) -> Result<Self, Vec<ModelTypeError>> {
        let mut selected = BTreeSet::new();
        let mut errors = Vec::new();
        for root in roots {
            match root.parse() {
                Ok(name) => {
                    selected.insert(ModelRoot::Type(name));
                }
                Err(_) => errors.push(unknown_root(root, "type")),
            }
        }
        // Preserve every original unknown-root diagnostic, including malformed names.
        match Self::select_roots(ir, &selected) {
            Ok(value) if errors.is_empty() => Ok(value),
            Ok(_) => Err(errors),
            Err(found) => {
                errors.extend(
                    found
                        .into_iter()
                        .filter(|error| roots.is_empty() || error.rule != "empty_roots"),
                );
                errors.sort();
                Err(errors)
            }
        }
    }

    /// Select explicit type and event roots through one checked reference closure.
    pub fn select_roots(
        ir: &EssIr,
        roots: &BTreeSet<ModelRoot>,
    ) -> Result<Self, Vec<ModelTypeError>> {
        let mut errors = BTreeSet::new();
        let mut selected = BTreeMap::new();
        let mut events = BTreeMap::new();
        if roots.is_empty() {
            errors.insert(ModelTypeError {
                name: String::new(),
                rule: "empty_roots",
                detail: "select at least one model type".to_owned(),
            });
        }
        for root in roots {
            let leaves = match root {
                ModelRoot::Type(name) => {
                    let Some(declared) = ir.types().get(name) else {
                        errors.insert(unknown_root(&name.to_string(), "type"));
                        continue;
                    };
                    selected.insert(declared.name.clone(), declared);
                    types::body_leaves(&declared.body)
                }
                ModelRoot::Event(name) => {
                    let Some(event) = ir.events().get(name) else {
                        errors.insert(unknown_root(&name.to_string(), "event"));
                        continue;
                    };
                    events.insert(event.name.clone(), event);
                    types::field_leaves(&event.fields)
                }
            };
            for handle in types::reachable(ir, leaves) {
                let reached = ir.named_type(handle);
                selected.insert(reached.name.clone(), reached);
            }
        }
        for declared in selected.values() {
            if let ResolvedBody::Struct { fields, .. } = &declared.body {
                errors.extend(wire_collisions(&declared.name, fields));
            }
        }
        for event in events.values() {
            errors.extend(wire_collisions(&event.name, &event.fields));
            if selected.contains_key(&event.name) {
                errors.insert(ModelTypeError {
                    name: event.name.to_string(),
                    rule: "root_collision",
                    detail: "an event and a type cannot share a schema definition".into(),
                });
            }
        }
        if !errors.is_empty() {
            return Err(errors.into_iter().collect());
        }
        Ok(Self::project(ir, roots, &selected, &events))
    }

    fn project(
        ir: &EssIr,
        roots: &BTreeSet<ModelRoot>,
        selected: &BTreeMap<QualifiedName, &ess_compiler::ir::ResolvedType>,
        events: &BTreeMap<QualifiedName, &ess_compiler::ir::ResolvedEvent>,
    ) -> Self {
        let provenance = ProvenanceMint::new(ir)
            .of_seeds(
                selected
                    .keys()
                    .map(|name| DeclaredTypeRef::new(name.clone()).into())
                    .chain(events.keys().map(|name| EventRef::new(name.clone()).into())),
            )
            .provenance;
        let mut binary64 = BTreeSet::new();
        for item in selected.values() {
            body_numeric_locations(item, &mut binary64);
        }
        for event in events.values() {
            field_numeric_locations(&event.name, &event.fields, &mut binary64);
        }
        Self {
            binary64,
            roots: roots.iter().map(|root| root.name().to_string()).collect(),
            model_roots: roots.clone(),
            definitions: selected
                .values()
                .map(|item| {
                    (
                        item.name.to_string(),
                        serde_json::to_value(types::body(item))
                            .expect("typed schema nodes serialize"),
                    )
                })
                .chain(events.values().map(|event| {
                    (
                        event.name.to_string(),
                        serde_json::to_value(types::message(&types::Message::of_event(event)))
                            .expect("typed event schema serializes"),
                    )
                }))
                .collect(),
            newtypes: selected
                .values()
                .filter(|item| matches!(item.body, ResolvedBody::Newtype { .. }))
                .map(|item| item.name.to_string())
                .collect(),
            provenance,
        }
    }

    /// Exact selected roots, distinct from their transitive closure.
    pub fn roots(&self) -> &BTreeSet<String> {
        &self.roots
    }

    /// Kind-preserving selected roots, distinct from their reachable type closure.
    pub fn model_roots(&self) -> &BTreeSet<ModelRoot> {
        &self.model_roots
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

fn unknown_root(name: &str, kind: &str) -> ModelTypeError {
    ModelTypeError {
        name: name.into(),
        rule: "unknown_root",
        detail: format!("no resolved model {kind} has this qualified name"),
    }
}

fn wire_collisions(name: &QualifiedName, fields: &[ResolvedField]) -> Vec<ModelTypeError> {
    let mut wires = BTreeMap::new();
    let mut errors = Vec::new();
    for field in fields {
        let wire = types::wire_name(field);
        if let Some(previous) = wires.insert(wire, &field.name) {
            errors.push(ModelTypeError {
                name: name.to_string(),
                rule: "wire_field_collision",
                detail: format!(
                    "fields {previous:?} and {:?} both use wire key {wire:?}",
                    field.name
                ),
            });
        }
    }
    errors
}

fn field_numeric_locations(
    name: &QualifiedName,
    fields: &[ResolvedField],
    found: &mut BTreeSet<String>,
) {
    let at = pointer("/$defs", &name.to_string());
    for field in fields {
        numeric_locations(
            field.type_ref.required(),
            &pointer(&format!("{at}/properties"), types::wire_name(field)),
            found,
        );
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
            field_numeric_locations(&item.name, fields, found);
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
