//! Closed CLI presentation bindings resolved against an ESS model.

mod resolve;
pub mod wire;

use serde::Deserialize;
use std::collections::BTreeMap;
use wire::{Command, Globals, Plan, Target};

pub use resolve::compile;

/// Authored CLI presentation. The closed reader owns construction.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    format: String,
    binary: String,
    about: String,
    globals: Globals,
    callables: BTreeMap<String, CallableDeclaration>,
    commands: Vec<Command>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallableDeclaration {
    target: Target,
    #[serde(deserialize_with = "required_input")]
    input: Option<String>,
    result: String,
    #[serde(default)]
    errors: BTreeMap<String, String>,
}

fn required_input<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Option::deserialize(deserializer)
}

/// Resolved presentation, constructible only by compiling a binding.
#[derive(Debug)]
pub struct CompiledBinding(Plan);

/// A binding refusal.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

impl Binding {
    /// Read an authored YAML document.
    pub fn from_yaml(text: &str) -> Result<Self, Error> {
        let binding: Self = serde_yaml::from_str(text).map_err(|e| Error(e.to_string()))?;
        if binding.format != "ess-cli/1" {
            return Err(Error(format!(
                "unsupported CLI format `{}`; expected ess-cli/1",
                binding.format
            )));
        }
        Ok(binding)
    }
}

impl CompiledBinding {
    /// Deterministic serialized output.
    pub fn to_canonical_json(&self) -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&self.0).expect("the resolved plan is serializable")
        )
    }

    /// Read-only projection input, with no unresolved model type references.
    pub fn plan(&self) -> &Plan {
        &self.0
    }
}
