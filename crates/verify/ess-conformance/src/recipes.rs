//! Typed, acyclic fixture expansion against original pinned component models.
//!
//! A recipe contains commands and other recipes, never assertions or executable snippets.
//! Parameters take their type from an admitted command field. Outputs refer to actual
//! response captures; expansion never invents backend identities. This module produces a
//! borrowed native expansion, not an independently executable or persisted suite format.

use crate::authored::Source;
use crate::input::{bind, Completeness};
use crate::models::{Models, Owned};
use crate::scenario::{CommandRef, InstanceName};
use ess_compiler::ir::{ResolvedCommand, ResolvedField, ResolvedTypeRef};
use ess_composition::ServiceKey;
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Native fixture source format. Its output still needs scenario compilation and admission.
pub const FORMAT: &str = "ess-fixture/1";
const MAX_DEPTH: usize = 32;
const MAX_COMMANDS: usize = 4096;

/// One recipe source, with a closed setup-only vocabulary.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    /// Must equal [`FORMAT`].
    #[serde(rename = "type")]
    pub format: String,
    /// Library key used by a given fixture invocation.
    pub recipe: InstanceName,
    /// Each parameter borrows a declared field's type, including its transitive declarations.
    #[serde(default)]
    pub parameters: BTreeMap<InstanceName, Parameter>,
    /// Setup operations, in authored order.
    pub steps: Vec<Step>,
    /// Public output names mapped to locally captured response values.
    #[serde(default)]
    pub outputs: BTreeMap<InstanceName, String>,
}

/// The typed home of a recipe parameter. No parallel type declaration is introduced.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    /// Pinned service that admits the command.
    pub service: ServiceKey,
    /// Existing command declaration.
    pub command: CommandRef,
    /// Existing input field whose type the parameter takes.
    pub field: String,
}

/// A fixture supplies values, invokes admitted commands, or composes another fixture.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "do", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    /// Invoke a real setup command and optionally retain fields from its actual response.
    Command {
        /// Original pinned service owner.
        service: ServiceKey,
        /// Admitted command.
        command: CommandRef,
        /// Complete declared input.
        #[serde(default)]
        input: BTreeMap<String, Value>,
        /// Local capture name to declared response field.
        #[serde(default)]
        capture: BTreeMap<InstanceName, String>,
    },
    /// Expand an acyclic dependency, exposing only its declared outputs.
    Use {
        /// Recipe library key.
        recipe: InstanceName,
        /// Local namespace for the dependency's outputs.
        #[serde(rename = "as")]
        alias: InstanceName,
        /// Complete values for the dependency's declared parameters.
        #[serde(default)]
        with: BTreeMap<InstanceName, Value>,
    },
}

/// Literal input, a recipe parameter, or an earlier actual response capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An exact authored value.
    Literal(Node),
    /// A parameter in this recipe's lexical scope.
    Parameter(InstanceName),
    /// A preceding local capture or a dependency output such as `alice.id`.
    Capture(String),
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let node = <Node as serde::Deserialize>::deserialize(deserializer)?;
        if let Node::Map(fields) = &node {
            if fields.keys().any(|key| key.starts_with('$')) {
                if fields.len() != 1 {
                    return Err(serde::de::Error::custom(
                        "a fixture reference has exactly one sigil",
                    ));
                }
                if let Some(Node::Text(name)) = fields.get("$parameter") {
                    return InstanceName::new(name)
                        .map(Self::Parameter)
                        .map_err(serde::de::Error::custom);
                }
                if let Some(Node::Text(name)) = fields.get("$capture") {
                    if !name.is_empty() {
                        return Ok(Self::Capture(name.clone()));
                    }
                }
                return Err(serde::de::Error::custom(
                    "expected {$parameter: name} or {$capture: name}",
                ));
            }
        }
        Ok(Self::Literal(node))
    }
}

/// Exact source identity retained independently of expansion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIdentity {
    /// Original source path, for diagnostics and archival.
    pub origin: String,
    /// SHA-256 over original UTF-8 source bytes, not reconstructed YAML.
    pub digest: String,
}

/// One authored location in a complete expansion stack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Location {
    /// Original file name.
    pub origin: String,
    /// Structural source pointer, such as `/steps/2`.
    pub pointer: String,
}

/// A response slot keeps the original field and model owner until native suite lowering.
#[derive(Debug, Clone)]
pub struct Capture<'a> {
    /// Unique deterministic slot, never a backend identity.
    pub slot: String,
    /// Original service owner.
    pub service: ServiceKey,
    /// Original admitted command, which owns the field's transitive type handles.
    pub command: Owned<'a, ResolvedCommand>,
    /// Original response field.
    pub field: &'a ResolvedField,
}

/// A compiled input remains literal or explicitly bound to an actual response slot.
#[derive(Debug, Clone)]
pub enum Bound<'a> {
    /// Exact authored or parameter value, checked against each destination field.
    Literal(Node),
    /// Runtime value captured from the declared response field.
    Capture(Capture<'a>),
}

/// One admitted setup invocation. It contains no assertion or manufactured response value.
#[derive(Debug)]
pub struct Invocation<'a> {
    /// Pinned service owner.
    pub service: ServiceKey,
    /// Original command and type authority.
    pub command: Owned<'a, ResolvedCommand>,
    /// Complete checked input values.
    pub input: BTreeMap<String, Bound<'a>>,
    /// Slots to populate from the actual returned response.
    pub captures: Vec<Capture<'a>>,
    /// Complete outer-to-inner recipe expansion stack.
    pub locations: Vec<Location>,
}

/// Deterministic native expansion awaiting scenario lowering and execution admission.
#[derive(Debug)]
pub struct Expansion<'a> {
    /// Commands in fixture order.
    pub commands: Vec<Invocation<'a>>,
    /// Declared root outputs backed by actual response captures.
    pub outputs: BTreeMap<InstanceName, Capture<'a>>,
    /// Every recipe actually used, including transitive dependencies.
    pub sources: BTreeMap<InstanceName, SourceIdentity>,
}

/// An immutable, parsed fixture library with a checked dependency graph.
#[derive(Debug)]
pub struct Library {
    documents: BTreeMap<InstanceName, (Document, SourceIdentity)>,
    originals: Vec<Source>,
}

impl Library {
    /// Parse original source bytes and refuse duplicate names, unknown verbs, missing
    /// dependencies, cycles and excessively deep expansions before invoking any target.
    pub fn parse(sources: &[Source]) -> Result<Self, String> {
        if sources.len() > MAX_COMMANDS {
            return Err("fixture source count limit".into());
        }
        let mut documents = BTreeMap::new();
        let mut origins = BTreeSet::new();
        for source in sources {
            if !origins.insert(&source.origin) {
                return Err(format!("duplicate fixture origin {}", source.origin));
            }
            let document: Document = serde_yaml::from_str(&source.text)
                .map_err(|error| format!("{}: {error}", source.origin))?;
            if document.format != FORMAT {
                return Err(format!(
                    "{}: unsupported fixture format {}",
                    source.origin, document.format
                ));
            }
            if document.steps.is_empty() || document.steps.len() > MAX_COMMANDS {
                return Err(format!(
                    "{}: fixture requires 1..={MAX_COMMANDS} setup steps",
                    source.origin
                ));
            }
            let identity = SourceIdentity {
                origin: source.origin.clone(),
                digest: crate::coverage::digest(&source.text),
            };
            let name = document.recipe.clone();
            if documents
                .insert(name.clone(), (document, identity))
                .is_some()
            {
                return Err(format!("duplicate fixture recipe {name}"));
            }
        }
        let library = Self {
            documents,
            originals: sources.to_vec(),
        };
        let mut heights = BTreeMap::new();
        for name in library.documents.keys() {
            library.visit(name, &mut Vec::new(), &mut heights)?;
        }
        Ok(library)
    }

    /// Original bytes admitted as fixture input, including unused library members.
    pub fn sources(&self) -> &[Source] {
        &self.originals
    }

    fn visit(
        &self,
        name: &InstanceName,
        stack: &mut Vec<InstanceName>,
        heights: &mut BTreeMap<InstanceName, usize>,
    ) -> Result<usize, String> {
        if stack.contains(name) {
            return Err(format!("fixture dependency cycle: {stack:?} -> {name}"));
        }
        if stack.len() >= MAX_DEPTH {
            return Err("fixture dependency depth limit".into());
        }
        if let Some(height) = heights.get(name) {
            return Ok(*height);
        }
        let (document, _) = self
            .documents
            .get(name)
            .ok_or_else(|| format!("unknown fixture {name}"))?;
        stack.push(name.clone());
        let mut height = 1;
        for step in &document.steps {
            if let Step::Use { recipe, .. } = step {
                height = height.max(1 + self.visit(recipe, stack, heights)?);
            }
        }
        stack.pop();
        if height > MAX_DEPTH {
            return Err("fixture dependency depth limit".into());
        }
        heights.insert(name.clone(), height);
        Ok(height)
    }

    /// Expand one given fixture. Parameter values are validated in their original models;
    /// captures remain slots and cannot become fabricated identities during compilation.
    pub fn expand<'a>(
        &self,
        models: &Models<'a>,
        recipe: &InstanceName,
        arguments: BTreeMap<InstanceName, Node>,
    ) -> Result<Expansion<'a>, String> {
        self.expand_with_bindings(
            models,
            recipe,
            recipe,
            &arguments
                .into_iter()
                .map(|(name, value)| (name, Bound::Literal(value)))
                .collect(),
        )
    }

    /// Expand a named given instance using literal values or captures from earlier givens.
    /// The caller supplies a unique alias so repeated uses cannot share response slots.
    pub fn expand_with_bindings<'a>(
        &self,
        models: &Models<'a>,
        recipe: &InstanceName,
        alias: &InstanceName,
        arguments: &BTreeMap<InstanceName, Bound<'a>>,
    ) -> Result<Expansion<'a>, String> {
        let mut expansion = Expansion {
            commands: Vec::new(),
            outputs: BTreeMap::new(),
            sources: BTreeMap::new(),
        };
        expansion.outputs = self.expand_into(
            models,
            recipe,
            arguments,
            alias.as_str(),
            &[],
            &mut expansion,
        )?;
        Ok(expansion)
    }

    // Keep one lexical-scope pass: commands bind local captures and dependencies expose
    // only their declared outputs. Splitting the two paths must not split that ownership.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn expand_into<'a>(
        &self,
        models: &Models<'a>,
        name: &InstanceName,
        arguments: &BTreeMap<InstanceName, Bound<'a>>,
        scope: &str,
        parents: &[Location],
        expansion: &mut Expansion<'a>,
    ) -> Result<BTreeMap<InstanceName, Capture<'a>>, String> {
        let (document, identity) = self
            .documents
            .get(name)
            .ok_or_else(|| format!("unknown fixture {name}"))?;
        if arguments.keys().ne(document.parameters.keys()) {
            return Err(format!(
                "{}: fixture {name} needs exactly its declared parameters",
                identity.origin
            ));
        }
        for (parameter, contract) in &document.parameters {
            let owner = models
                .command(&contract.service, &contract.command)
                .ok_or_else(|| {
                    format!(
                        "{}: unadmitted parameter command {}",
                        identity.origin, contract.command
                    )
                })?;
            let field = owner
                .declaration()
                .input
                .iter()
                .find(|field| field.name == contract.field)
                .ok_or_else(|| {
                    format!(
                        "{}: unknown parameter field {}",
                        identity.origin, contract.field
                    )
                })?;
            check_value(&contract.service, &owner, field, &arguments[parameter])?;
        }
        expansion.sources.insert(name.clone(), identity.clone());
        let mut captures = BTreeMap::<String, Capture<'a>>::new();
        let mut aliases = BTreeSet::new();
        for (index, step) in document.steps.iter().enumerate() {
            let mut locations = parents.to_vec();
            locations.push(Location {
                origin: identity.origin.clone(),
                pointer: format!("/steps/{index}"),
            });
            match step {
                Step::Command {
                    service,
                    command,
                    input,
                    capture,
                } => {
                    if expansion.commands.len() >= MAX_COMMANDS {
                        return Err("fixture expanded command limit".into());
                    }
                    let owner = models.command(service, command).ok_or_else(|| {
                        format!(
                            "{} /steps/{index}: unadmitted command {service}:{command}",
                            identity.origin
                        )
                    })?;
                    let fields = &owner.declaration().input;
                    let mut values = BTreeMap::new();
                    for (key, value) in input {
                        let field = fields
                            .iter()
                            .find(|field| &field.name == key)
                            .ok_or_else(|| format!("{command}: undeclared input {key}"))?;
                        let value = resolve(value, arguments, &captures)?;
                        check_value(service, &owner, field, &value)?;
                        values.insert(key.clone(), value);
                    }
                    if let Some(field) = fields.iter().find(|field| {
                        !field.type_ref.is_optional() && !values.contains_key(&field.name)
                    }) {
                        return Err(format!("{command}: missing input {}", field.name));
                    }
                    let mut slots = Vec::new();
                    for (local, field_name) in capture {
                        let field = owner
                            .declaration()
                            .response
                            .iter()
                            .find(|field| &field.name == field_name)
                            .ok_or_else(|| {
                                format!("{command}: undeclared response capture {field_name}")
                            })?;
                        if field.type_ref.is_optional() {
                            return Err(format!("{command}: optional response {field_name} cannot guarantee a capture"));
                        }
                        let slot = Capture {
                            slot: format!("{scope}/{index}/{local}"),
                            service: service.clone(),
                            command: owner.clone(),
                            field,
                        };
                        if captures.insert(local.to_string(), slot.clone()).is_some() {
                            return Err(format!("fixture {name}: duplicate capture {local}"));
                        }
                        slots.push(slot);
                    }
                    expansion.commands.push(Invocation {
                        service: service.clone(),
                        command: owner,
                        input: values,
                        captures: slots,
                        locations,
                    });
                }
                Step::Use {
                    recipe,
                    alias,
                    with,
                } => {
                    if !aliases.insert(alias.clone()) {
                        return Err(format!(
                            "fixture {name}: duplicate dependency alias {alias}"
                        ));
                    }
                    let values = with
                        .iter()
                        .map(|(key, value)| {
                            Ok((key.clone(), resolve(value, arguments, &captures)?))
                        })
                        .collect::<Result<_, String>>()?;
                    let outputs = self.expand_into(
                        models,
                        recipe,
                        &values,
                        &format!("{scope}/{index}/{alias}"),
                        &locations,
                        expansion,
                    )?;
                    for (output, slot) in outputs {
                        let exposed = format!("{alias}.{output}");
                        if captures.insert(exposed.clone(), slot).is_some() {
                            return Err(format!("duplicate dependency output {exposed}"));
                        }
                    }
                }
            }
        }
        document
            .outputs
            .iter()
            .map(|(output, local)| {
                captures
                    .get(local)
                    .cloned()
                    .map(|slot| (output.clone(), slot))
                    .ok_or_else(|| {
                        format!(
                            "fixture {name}: output {output} is not a preceding capture: {local}"
                        )
                    })
            })
            .collect()
    }
}

fn resolve<'a>(
    value: &Value,
    parameters: &BTreeMap<InstanceName, Bound<'a>>,
    captures: &BTreeMap<String, Capture<'a>>,
) -> Result<Bound<'a>, String> {
    match value {
        Value::Literal(value) => Ok(Bound::Literal(value.clone())),
        Value::Parameter(name) => parameters
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown fixture parameter {name}")),
        Value::Capture(name) => captures
            .get(name)
            .cloned()
            .map(Bound::Capture)
            .ok_or_else(|| format!("capture {name} is not available before this command")),
    }
}

pub(crate) fn check_value(
    service: &ServiceKey,
    owner: &Owned<'_, ResolvedCommand>,
    field: &ResolvedField,
    value: &Bound<'_>,
) -> Result<(), String> {
    match value {
        Bound::Literal(value) => {
            bind(
                owner.model(),
                std::slice::from_ref(field),
                &BTreeMap::from([(field.name.clone(), value.clone())]),
                Completeness::Total,
            )
            .map_err(|error| error.to_string())?;
            Ok(())
        }
        Bound::Capture(capture) => {
            // Nominal references retain their original owner. Equal spellings in different
            // models are not interchangeable; only the shared primitive constructors are.
            let primitives = matches!(
                (&capture.field.type_ref, &field.type_ref),
                (
                    ResolvedTypeRef::Primitive { .. },
                    ResolvedTypeRef::Primitive { .. }
                )
            );
            if capture.field.type_ref != field.type_ref
                || (!primitives && &capture.service != service)
            {
                return Err(format!(
                    "capture {} has type {} from {}, expected {} from {service}",
                    capture.slot, capture.field.type_ref, capture.service, field.type_ref
                ));
            }
            Ok(())
        }
    }
}
