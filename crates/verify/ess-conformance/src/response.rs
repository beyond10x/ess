//! Typed response observations bound to one command invocation and its emitted event.
use crate::scenario::{CommandRef, EventRef, OutcomeRef};
use crate::selection::Declaration;
use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedOutcome, ResolvedPayloadValue,
};
use ess_domain::{Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Standalone declaration authority for a response-derived event payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawObservation")]
pub struct Observation {
    /// Exact command whose last result must be observed.
    pub command: CommandRef,
    /// Exact branch whose response is being read.
    pub outcome: OutcomeRef,
    /// Event emitted by that same invocation.
    pub event: EventRef,
    /// Ordered closed response fields.
    pub fields: Vec<Field>,
    /// Reachable nominal types; no runtime-supplied schema.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
    /// Target event fields and their source response field names.
    pub mappings: BTreeMap<String, String>,
    /// Declared mapped event field types for independent compatibility admission.
    pub targets: Vec<Field>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    command: CommandRef,
    outcome: OutcomeRef,
    event: EventRef,
    fields: Vec<Field>,
    declarations: BTreeMap<QualifiedName, Declaration>,
    mappings: BTreeMap<String, String>,
    targets: Vec<Field>,
}
impl TryFrom<RawObservation> for Observation {
    type Error = String;
    fn try_from(r: RawObservation) -> Result<Self, String> {
        let result = Self {
            command: r.command,
            outcome: r.outcome,
            event: r.event,
            fields: r.fields,
            declarations: r.declarations,
            mappings: r.mappings,
            targets: r.targets,
        };
        result.validate()?;
        Ok(result)
    }
}
impl Observation {
    /// Derive only response-mapped event observations from resolved source.
    pub fn of(
        ir: &EssIr,
        command: &ResolvedCommand,
        outcome: &ResolvedOutcome,
    ) -> Result<Vec<Self>, String> {
        let mut result = Vec::new();
        for payload in &outcome.payload {
            let mut mappings = BTreeMap::new();
            let mut targets = Vec::new();
            for field in &payload.fields {
                if let ResolvedPayloadValue::ResponseField { field: source, .. } = &field.value {
                    if field.conversion.is_some() {
                        return Err("response conversion requires an executable adapter".into());
                    }
                    mappings.insert(field.target.clone(), source.clone());
                    targets.push(Field::new(
                        &field.target,
                        crate::accessor::unresolve(&field.target_type),
                    ));
                }
            }
            if mappings.is_empty() {
                continue;
            }
            let fields: Vec<_> = command
                .response
                .iter()
                .map(|f| Field::new(&f.name, crate::accessor::unresolve(&f.type_ref)))
                .collect();
            let mut declarations = BTreeMap::new();
            let mut pending: Vec<_> = fields
                .iter()
                .chain(&targets)
                .flat_map(|f| f.type_ref.named_dependencies().into_iter().cloned())
                .collect();
            while let Some(name) = pending.pop() {
                if declarations.contains_key(&name) {
                    continue;
                }
                if declarations.len() >= 4096 {
                    return Err("response declaration resource limit".into());
                }
                let ty = ir.types().get(&name).ok_or("response type is absent")?;
                if ty.reading.is_some()
                    || matches!(&ty.body, ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } if !invariants.is_empty())
                {
                    return Err(
                        "response constrained type needs an executable invariant/reading observer"
                            .into(),
                    );
                }
                let body = match &ty.body {
                    ResolvedBody::Newtype { of, .. } => Declaration::Newtype {
                        of: crate::accessor::unresolve(of),
                    },
                    ResolvedBody::Struct { fields, .. } => Declaration::Struct {
                        fields: fields
                            .iter()
                            .map(|f| Field::new(&f.name, crate::accessor::unresolve(&f.type_ref)))
                            .collect(),
                    },
                    ResolvedBody::Enum { variants } => Declaration::Enum {
                        variants: variants.clone(),
                    },
                    ResolvedBody::Union { tag, variants } => Declaration::Union {
                        tag: tag.clone(),
                        variants: variants
                            .iter()
                            .map(|(tag, ty)| (tag.clone(), crate::accessor::unresolve(ty)))
                            .collect(),
                    },
                };
                for ty in body.references() {
                    pending.extend(ty.named_dependencies().into_iter().cloned());
                }
                declarations.insert(name, body);
            }
            let observation = Self {
                command: CommandRef::new(command.name.clone()),
                outcome: OutcomeRef::new(
                    CommandRef::new(command.name.clone()),
                    outcome.name.clone(),
                ),
                event: EventRef::from(&payload.event),
                fields,
                declarations,
                mappings,
                targets,
            };
            observation.validate()?;
            result.push(observation);
        }
        Ok(result)
    }
    /// Admit the finite schema and every source-target relation before target callbacks.
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty()
            || self.fields.len() > 256
            || self.targets.len() > 256
            || self.declarations.len() > 4096
            || self.mappings.is_empty()
            || self.mappings.len() != self.targets.len()
        {
            return Err("response contract field/declaration bound".into());
        }
        if self.outcome.command != self.command {
            return Err("response outcome belongs to a different command".into());
        }
        let mut registry = ess_domain::TypeRegistry::new();
        for (name, body) in &self.declarations {
            let declared = ess_domain::NamedType::try_from(ess_domain::types::RawNamedType {
                name: name.clone(),
                body: body.body(),
                naming: ess_domain::Naming::default(),
                reading: None,
            })
            .map_err(|e| e.to_string())?;
            registry.insert(declared).map_err(|e| e.to_string())?;
        }
        let mut used = BTreeSet::new();
        for fields in [&self.fields, &self.targets] {
            let mut seen = BTreeSet::new();
            for field in fields {
                if field.name.is_empty() || !seen.insert(&field.name) {
                    return Err("duplicate response contract field".into());
                }
                registry
                    .resolve(&field.type_ref, "response")
                    .into_result(())
                    .map_err(|e| e.to_string())?;
                self.check_type(&field.type_ref, &mut used, &mut BTreeSet::new(), 0)?;
            }
        }
        if used.len() != self.declarations.len() {
            return Err("response contract carries unrelated type declarations".into());
        }
        for target in &self.targets {
            let source = self
                .mappings
                .get(&target.name)
                .and_then(|name| self.fields.iter().find(|f| &f.name == name))
                .ok_or("response mapping has no declared source/target")?;
            if !ess_domain::types::is_assignable(&source.type_ref, &target.type_ref) {
                return Err("response mapping type mismatch".into());
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 1_048_576 {
            return Err("response contract byte limit".into());
        }
        Ok(())
    }
    fn check_type(
        &self,
        ty: &TypeRef,
        used: &mut BTreeSet<QualifiedName>,
        stack: &mut BTreeSet<QualifiedName>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 128 {
            return Err("response type depth limit".into());
        }
        match ty {
            TypeRef::Named(name) => {
                if !stack.insert(name.clone()) {
                    return Err("recursive response type cannot be finitely admitted".into());
                }
                if used.contains(name) {
                    stack.remove(name);
                    return Ok(());
                }
                for child in self
                    .declarations
                    .get(name)
                    .ok_or("missing response type")?
                    .references()
                {
                    self.check_type(child, used, stack, depth + 1)?;
                }
                used.insert(name.clone());
                stack.remove(name);
            }
            TypeRef::Optional(of) | TypeRef::List(of) => {
                self.check_type(of, used, stack, depth + 1)?;
            }
            TypeRef::Map(key, value) => {
                if *key != ess_domain::Primitive::String {
                    return Err("response map keys require String".into());
                }
                self.check_type(value, used, stack, depth + 1)?;
            }
            TypeRef::Primitive(ess_domain::Primitive::Binary64) => {
                return Err("response Binary64 observation is not admitted".into())
            }
            TypeRef::Primitive(_) => {}
        }
        Ok(())
    }
    /// Compare the same invocation's actual response and event; no expected value is sent to a target.
    pub fn compare(
        &self,
        response: Option<&BTreeMap<String, Node>>,
        payload: &BTreeMap<String, Node>,
    ) -> Result<(), String> {
        self.validate()?;
        let response = response.ok_or("command returned no response")?;
        if response
            .keys()
            .any(|name| !self.fields.iter().any(|f| &f.name == name))
        {
            return Err("response has an undeclared field".into());
        }
        let mut bytes = 0;
        for field in &self.fields {
            crate::selection::validate_response_value(
                &field.type_ref,
                response.get(&field.name),
                &self.declarations,
                &mut bytes,
            )
            .map_err(|e| format!("response field {}: {e}", field.name))?;
        }
        if bytes > 1_048_576 {
            return Err("response byte limit".into());
        }
        for (target, source) in &self.mappings {
            let actual = response.get(source);
            let optional = self
                .fields
                .iter()
                .any(|f| &f.name == source && matches!(f.type_ref, TypeRef::Optional(_)));
            if optional
                && (actual.is_none() || matches!(actual, Some(Node::Null)))
                && (payload.get(target).is_none()
                    || matches!(payload.get(target), Some(Node::Null)))
            {
                continue;
            }
            let actual = actual.ok_or_else(|| format!("response source {source} is absent"))?;
            if payload.get(target) != Some(actual) {
                return Err(format!(
                    "event field {target} differs from actual response field {source}"
                ));
            }
        }
        Ok(())
    }
}
/// Whether a suite contains the response observation vocabulary.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|s| {
        s.steps
            .iter()
            .any(|step| matches!(step, crate::ScenarioStep::ExpectResponsePayload { .. }))
    })
}

/// Response-mapped fields are checked as complete typed values by the response observation.
/// Retain ordinary flattened assertions for every other event field, including generated ownership.
pub(crate) fn event_shape(
    ir: &EssIr,
    event: &EventRef,
    outcome: &ResolvedOutcome,
) -> crate::scenario::PayloadShape {
    let mapped: Vec<_> = outcome
        .payload
        .iter()
        .filter(|p| EventRef::from(&p.event) == *event)
        .flat_map(|p| &p.fields)
        .filter(|f| matches!(f.value, ResolvedPayloadValue::ResponseField { .. }))
        .map(|f| f.target.as_str())
        .collect();
    let mut shape = crate::scenario::PayloadShape::new();
    for (path, leaf) in crate::synthesize::payload_shape(ir, event).leaves() {
        if !mapped.iter().any(|field| {
            path == field
                || path
                    .strip_prefix(field)
                    .is_some_and(|tail| tail.starts_with('.'))
        }) {
            shape.insert(path, leaf.clone());
        }
    }
    shape
}
