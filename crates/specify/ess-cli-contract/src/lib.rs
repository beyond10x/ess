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
    globals: AuthoredGlobals,
    callables: BTreeMap<String, CallableDeclaration>,
    commands: Vec<Command>,
}

/// The binding formats this reader admits, each with the plan format it compiles to.
const FORMATS: [(&str, &str); 2] = [
    ("ess-cli/1", "ess-cli-plan/1"),
    ("ess-cli/2", "ess-cli-plan/2"),
];

/// `globals` as authored. Each key is either absent or present with a value, and a present key
/// keeps its null apart from its absence, so `config: null` is refused rather than read as
/// omission or as a flag named `null`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredGlobals {
    #[serde(default, deserialize_with = "present")]
    config: Authored,
    #[serde(default, deserialize_with = "present")]
    state: Authored,
    #[serde(default, deserialize_with = "present")]
    output: Authored,
}

/// One authored global key: omitted, present with a null value, or present with a string.
#[derive(Debug, Default)]
enum Authored {
    #[default]
    Omitted,
    Null,
    Flag(String),
}

/// Called only for a key that is present, so it never yields [`Authored::Omitted`].
fn present<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Authored, D::Error> {
    Option::<String>::deserialize(deserializer)
        .map(|value| value.map_or(Authored::Null, Authored::Flag))
}

impl AuthoredGlobals {
    /// The plan's globals: `state` required in every version, `config` and `output` required in
    /// `ess-cli/1` and optional from `ess-cli/2`; a null or empty value refused, naming the key.
    fn resolve(&self, format: &str) -> Result<Globals, Error> {
        let optional = format != "ess-cli/1";
        Ok(Globals {
            config: global(format, "config", &self.config, optional)?,
            state: global(format, "state", &self.state, false)?
                .expect("a required global is present or refused"),
            output: global(format, "output", &self.output, optional)?,
        })
    }
}

/// One authored global: its flag name, `None` for an admitted omission, or the refusal naming it.
fn global(
    format: &str,
    name: &str,
    value: &Authored,
    may_omit: bool,
) -> Result<Option<String>, Error> {
    match value {
        Authored::Omitted if may_omit => Ok(None),
        Authored::Omitted if name == "state" => Err(Error(
            "globals: missing field `state`; every CLI binding declares its state flag".to_owned(),
        )),
        Authored::Omitted => Err(Error(format!(
            "globals: missing field `{name}`; {format} requires `config`, `state` and `output`, \
             and ess-cli/2 lets a binding omit `config` and `output`"
        ))),
        Authored::Null => Err(Error(format!(
            "globals.{name}: null names no flag; {}",
            absence(name, may_omit)
        ))),
        Authored::Flag(flag) if flag.is_empty() => Err(Error(format!(
            "globals.{name}: an empty string names no flag; {}",
            absence(name, may_omit)
        ))),
        Authored::Flag(flag) => Ok(Some(flag.clone())),
    }
}

/// How a binding says a global is absent, if it can.
fn absence(name: &str, may_omit: bool) -> String {
    if name == "state" {
        "`state` is required".to_owned()
    } else if may_omit {
        format!("omit the `{name}` key to declare no such flag")
    } else {
        format!("ess-cli/2 lets a binding omit the `{name}` key")
    }
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
    #[serde(default)]
    invalid_input: Option<String>,
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
        binding.plan_format()?;
        binding.globals()?;
        Ok(binding)
    }

    /// The plan format this binding compiles to, or the refusal of an unknown binding format.
    fn plan_format(&self) -> Result<&'static str, Error> {
        FORMATS
            .iter()
            .find(|(binding, _)| *binding == self.format)
            .map(|(_, plan)| *plan)
            .ok_or_else(|| {
                Error(format!(
                    "unsupported CLI format `{}`; expected ess-cli/1 or ess-cli/2",
                    self.format
                ))
            })
    }

    /// The declared globals, held to this binding's format.
    fn globals(&self) -> Result<Globals, Error> {
        self.plan_format()?;
        self.globals.resolve(&self.format)
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
