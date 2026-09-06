//! Checked model acquisition and wire validation; imported annotations are never authority.

use std::collections::{BTreeMap, BTreeSet};

use ess_gen::schema::ModelTypes;
use serde_json::{json, Value};

use super::{finding, path, Bundle, Finding, ModelIdentity, Plan, Refused, Root, Types};
use crate::bundle::source_digest;

impl ModelIdentity {
    pub(super) fn digest(&self) -> String {
        source_digest(&serde_json::to_string(self).expect("typed model identity serializes"))
    }

    /// Mint the full identity of an already checked model selection.
    pub fn pin(model: &ModelTypes) -> Self {
        let provenance = model.provenance();
        Self {
            system: provenance.system.clone(),
            specification_version: provenance.specification_version.clone(),
            source_digest: provenance.source_digest.clone(),
            contract_digest: provenance.contract_digest.clone(),
            projection_digest: source_digest(&model.to_json()),
            roots: model.roots().clone(),
        }
    }
}

impl Root {
    /// Exact source-owned root name.
    pub fn name(&self) -> &str {
        match self {
            Self::Bundle { root, .. } | Self::Model { root, .. } => root,
        }
    }

    /// Pin a root explicitly selected from a checked model, not merely reachable in it.
    pub fn pin_model(model: &ModelTypes, root: impl Into<String>) -> Result<Self, Refused> {
        let root = root.into();
        if !model.roots().contains(&root) {
            return Err(Refused(vec![finding(
                "/roots",
                "unselected_root",
                "model root was not explicitly selected",
            )]));
        }
        Ok(Self::Model {
            model: ModelIdentity::pin(model),
            root,
        })
    }
}

pub(super) fn selection(
    root: &Root,
    bundles: &BTreeMap<String, Bundle>,
    models: &BTreeMap<ModelIdentity, ModelTypes>,
    at: &str,
    found: &mut Vec<Finding>,
) -> Option<Types> {
    let selected = match root {
        Root::Bundle {
            bundle_digest,
            root,
        } => {
            let Some(bundle) = bundles.get(bundle_digest) else {
                found.push(finding(
                    at,
                    "unknown_bundle",
                    "no supplied checked bundle has this canonical digest",
                ));
                return None;
            };
            Types::from_bundle(bundle, &BTreeSet::from([root.clone()]))
        }
        Root::Model { model, root } => {
            let Some(selected) = models.get(model) else {
                found.push(finding(
                    at,
                    "unknown_model",
                    "no supplied checked model selection has this complete identity",
                ));
                return None;
            };
            if !selected.roots().contains(root) {
                found.push(finding(
                    at,
                    "unselected_root",
                    "model root was not explicitly selected",
                ));
                return None;
            }
            Types::from_model(selected)
        }
    };
    match selected {
        Ok(plan) => {
            for obligation in plan
                .obligations
                .iter()
                .filter(|item| item.rule == "model_invariants")
            {
                found.push(finding(
                    &format!("{at}{}", obligation.pointer),
                    "model_invariants",
                    "normalization cannot execute the selected model invariant statements",
                ));
            }
            Some(plan)
        }
        Err(errors) => {
            found.extend(errors.0.into_iter().map(|error| {
                finding(
                    at,
                    "root_selection",
                    &format!("{}: {}", error.pointer, error.detail),
                )
            }));
            None
        }
    }
}

pub(super) fn model_schema(plan: &Plan, root: &Root) -> Value {
    let Root::Model { model, root } = root else {
        unreachable!("model root required")
    };
    let selected = &plan.models[model];
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": format!("urn:ess:normalization:model-{}:{}", model.digest(), source_digest(root)),
        "$ref": format!("#{}", path("/$defs", root)),
        "$defs": selected.definitions(),
        "x-ess-provenance": selected.provenance(),
    })
}

pub(super) fn validate_model(
    plan: &Plan,
    root: &Root,
    value: &Value,
    at: &str,
) -> Result<(), Refused> {
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(false)
        .build(&model_schema(plan, root))
        .map_err(|error| Refused(vec![finding(at, "schema_validation", &error.to_string())]))?;
    let found = validator
        .iter_errors(value)
        .map(|error| {
            finding(
                &format!("{at}{}", error.instance_path()),
                "schema_validation",
                "value does not satisfy the selected stage contract",
            )
        })
        .collect::<Vec<_>>();
    if found.is_empty() {
        Ok(())
    } else {
        Err(Refused(found))
    }
}
