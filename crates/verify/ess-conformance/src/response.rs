//! Typed response observations bound to one command invocation and its emitted event.
use crate::scenario::{CommandRef, EventRef, OutcomeRef};
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedOutcome, ResolvedPayloadValue};
use ess_domain::{Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

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
            let declarations =
                crate::typed_fields::declarations(ir, fields.iter().chain(&targets))?;
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
        crate::typed_fields::validate(
            [self.fields.as_slice(), self.targets.as_slice()],
            &self.declarations,
        )?;
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
