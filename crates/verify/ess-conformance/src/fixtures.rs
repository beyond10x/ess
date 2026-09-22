//! Typed values supplied independently before any target scenario activity.
use crate::scenario::{ConformanceSuite, EventRef, ScenarioId, ScenarioStep, ScenarioValue};
use crate::selection::Declaration;
use ess_compiler::ir::EssIr;
use ess_compiler::ir::{ResolvedOutcome, ResolvedPayloadValue};
use ess_domain::{command::fixture_inputs::FixtureName, Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Source-owned authority for the finite fixture values a scenario needs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawContract")]
pub struct Contract {
    /// Names and declared types; these are not supplied by the fixture provider.
    pub fields: Vec<FixtureField>,
    /// Exactly the reachable nominal declarations.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
}

/// A fixture key has a lower-kebab identity, independent of generated code field names.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureField {
    /// The stable provider key.
    pub name: FixtureName,
    /// The source-owned type of the value.
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContract {
    fields: Vec<FixtureField>,
    declarations: BTreeMap<QualifiedName, Declaration>,
}

impl TryFrom<RawContract> for Contract {
    type Error = String;

    fn try_from(raw: RawContract) -> Result<Self, String> {
        let value = Self {
            fields: raw.fields,
            declarations: raw.declarations,
        };
        value.validate()?;
        Ok(value)
    }
}

impl Contract {
    /// Compile the declared types without asking a target what its values should be.
    pub fn of(ir: &EssIr, fields: BTreeMap<FixtureName, TypeRef>) -> Result<Self, String> {
        let fields: Vec<_> = fields
            .into_iter()
            .map(|(name, type_ref)| FixtureField { name, type_ref })
            .collect();
        let typed: Vec<_> = fields
            .iter()
            .map(|field| Field::new(field.name.as_str(), field.type_ref.clone()))
            .collect();
        let declarations = crate::typed_fields::declarations(ir, &typed)?;
        let value = Self {
            fields,
            declarations,
        };
        value.validate()?;
        Ok(value)
    }

    /// Admit the source type authority before invoking the fixture provider.
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() || self.fields.len() > 256 || self.declarations.len() > 4096 {
            return Err("fixture field/declaration bound".into());
        }
        let typed: Vec<_> = self
            .fields
            .iter()
            .map(|field| Field::new(field.name.as_str(), field.type_ref.clone()))
            .collect();
        crate::typed_fields::validate([typed.as_slice()], &self.declarations)?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 1_048_576 {
            return Err("fixture contract byte limit".into());
        }
        Ok(())
    }

    /// Check the provider's complete result without coercion or target activity.
    pub fn validate_values(&self, values: &BTreeMap<String, Node>) -> Result<(), String> {
        self.validate()?;
        let expected: BTreeSet<_> = self
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        if values.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
            return Err("fixture result must contain exactly the declared names".into());
        }
        let mut bytes = 0;
        for field in &self.fields {
            crate::selection::validate_response_value(
                &field.type_ref,
                values.get(field.name.as_str()),
                &self.declarations,
                &mut bytes,
            )
            .map_err(|e| format!("fixture {}: {e}", field.name))?;
        }
        if bytes > 1_048_576 {
            return Err("fixture value byte limit".into());
        }
        Ok(())
    }
}

/// Retain equality for fixture-derived event fields instead of dropping it as non-literal.
pub(crate) fn event_values(
    outcome: &ResolvedOutcome,
    event: &EventRef,
    supplied: &BTreeMap<String, ScenarioValue>,
) -> BTreeMap<String, ScenarioValue> {
    let mut values = BTreeMap::new();
    if let Some(payload) = outcome
        .payload
        .iter()
        .find(|p| EventRef::from(&p.event) == *event)
    {
        for field in &payload.fields {
            if let ResolvedPayloadValue::InputField { field: source, .. } = &field.value {
                if let Some(value @ ScenarioValue::Fixture { .. }) = supplied.get(source) {
                    values.insert(field.target.clone(), value.clone());
                }
            }
        }
    }
    values
}

/// Add one prelude from source-declared command input types after assembling each scenario.
pub(crate) fn install(ir: &EssIr, suite: &mut ConformanceSuite) -> Vec<(ScenarioId, String)> {
    let mut refused = Vec::new();
    for (id, scenario) in &mut suite.scenarios {
        let result = (|| {
            let mut fields = BTreeMap::new();
            for step in &scenario.steps {
                let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
                    continue;
                };
                let declared = ir
                    .commands()
                    .get(command.name())
                    .ok_or("fixture command is undeclared")?;
                for outcome in &declared.outcomes {
                    for payload in &outcome.payload {
                        for field in &payload.fields {
                            if let ResolvedPayloadValue::InputField { field: source, .. } =
                                &field.value
                            {
                                if field.conversion.is_some()
                                    && matches!(
                                        input.get(source),
                                        Some(ScenarioValue::Fixture { .. })
                                    )
                                {
                                    return Err(format!("fixture input {source} needs an unsupported payload conversion"));
                                }
                            }
                        }
                    }
                }
                for field in declared.fixture_inputs.keys() {
                    if matches!(input.get(field), Some(ScenarioValue::Instance { .. })) {
                        return Err(format!(
                            "fixture input {field} conflicts with a scenario-owned instance"
                        ));
                    }
                }
                for (field, value) in input {
                    let ScenarioValue::Fixture { fixture } = value else {
                        continue;
                    };
                    let ty = declared
                        .input
                        .iter()
                        .find(|f| f.name == *field)
                        .ok_or("fixture input field is undeclared")?;
                    let ty = crate::accessor::unresolve(&ty.type_ref);
                    if let Some(prior) = fields.insert(fixture.clone(), ty.clone()) {
                        if prior != ty {
                            return Err(format!(
                                "fixture {fixture} has incompatible declared types"
                            ));
                        }
                    }
                }
            }
            if fields.is_empty() {
                return Ok(None);
            }
            Contract::of(ir, fields).map(Some)
        })();
        match result {
            Ok(Some(fixtures)) => scenario
                .steps
                .insert(0, ScenarioStep::ResolveFixtures { fixtures }),
            Ok(None) => {}
            Err(reason) => refused.push((id.clone(), reason)),
        }
    }
    refused
}

/// Whether the suite needs the pre-execution fixture vocabulary.
pub(crate) fn used_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::ResolveFixtures { .. } | ScenarioStep::ExpectEventValues { .. }
            )
        })
    })
}

/// Reject unresolved or misplaced fixture vocabulary before any target callback.
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), String> {
    for (id, scenario) in &suite.scenarios {
        let mut declared = BTreeSet::new();
        let mut referenced = BTreeSet::new();
        for (index, step) in scenario.steps.iter().enumerate() {
            if let ScenarioStep::ResolveFixtures { fixtures } = step {
                if index != 0 {
                    return Err(format!(
                        "{id}: fixture resolution must be the first and only prelude"
                    ));
                }
                fixtures.validate()?;
                declared.extend(fixtures.fields.iter().map(|field| field.name.as_str()));
            }
            if let ScenarioStep::ExpectEventValues { payload, .. } = step {
                if payload.is_empty() {
                    return Err(format!(
                        "{id}: an event value assertion must compare at least one field"
                    ));
                }
            }
            for value in values(step) {
                if let ScenarioValue::Fixture { fixture } = value {
                    referenced.insert(fixture.as_str());
                }
            }
        }
        if declared != referenced {
            return Err(format!(
                "{id}: fixture declarations must exactly match the referenced names"
            ));
        }
    }
    Ok(())
}

fn values(step: &ScenarioStep) -> Vec<&ScenarioValue> {
    use crate::scenario::ViewExpectation;
    let mut result = Vec::new();
    match step {
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => result.extend(input.values()),
        ScenarioStep::ExpectEventValues { payload, .. } => result.extend(payload.values()),
        ScenarioStep::SnapshotSubject { subject, .. } => result.extend(subject.values()),
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::EventuallyView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => result.extend(params.values()),
        _ => {}
    }
    if let ScenarioStep::ExpectView { expectation, .. }
    | ScenarioStep::EventuallyView { expectation, .. } = step
    {
        match expectation {
            ViewExpectation::Contains { fields }
            | ViewExpectation::Excludes { fields }
            | ViewExpectation::At { fields, .. } => result.extend(fields.values()),
            _ => {}
        }
    }
    result
}

/// A fixture may widen into Optional, without an undeclared conversion or nominal coercion.
pub(crate) fn assignable(from: &TypeRef, to: &TypeRef) -> bool {
    from == to || matches!(to, TypeRef::Optional(inner) if assignable(from, inner))
}
