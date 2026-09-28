//! Complete typed return observations, independent of events, subjects and views.
use crate::scenario::{CommandRef, OutcomeRef};
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_domain::{types::Presence, Field, QualifiedName};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

/// First ordinary suite with direct return observations.
pub const ORDINARY: u32 = 28;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = 29;

/// Declaration authority and independently authored literals for one actual invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawObservation")]
pub struct Observation {
    /// Exact command whose most recent invocation returned the value.
    pub command: CommandRef,
    /// Selected outcome, when the scenario names it.
    pub outcome: Option<OutcomeRef>,
    /// Complete closed response schema.
    pub fields: Vec<Field>,
    /// Exactly the finite nominal declarations reachable from the response.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
    /// Partial field-value assertions; every named value is compared completely.
    pub expected: BTreeMap<String, Node>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    command: CommandRef,
    outcome: Option<OutcomeRef>,
    fields: Vec<Field>,
    declarations: BTreeMap<QualifiedName, Declaration>,
    expected: BTreeMap<String, Node>,
}

impl TryFrom<RawObservation> for Observation {
    type Error = String;
    fn try_from(raw: RawObservation) -> Result<Self, String> {
        let value = Self {
            command: raw.command,
            outcome: raw.outcome,
            fields: raw.fields,
            declarations: raw.declarations,
            expected: raw.expected,
        };
        value.validate()?;
        Ok(value)
    }
}

impl Observation {
    /// Derive response authority from the compiled command, never from a target's result.
    pub fn of(
        ir: &EssIr,
        command: &ResolvedCommand,
        outcome: Option<OutcomeRef>,
        expected: BTreeMap<String, Node>,
    ) -> Result<Self, String> {
        let fields: Vec<_> = command
            .response
            .iter()
            .map(|field| {
                let mut result =
                    Field::new(&field.name, crate::accessor::unresolve(&field.type_ref));
                result.naming.presence = field.naming.presence;
                result
            })
            .collect();
        let result = Self {
            command: CommandRef::new(command.name.clone()),
            outcome,
            declarations: crate::typed_fields::direct_response_declarations(ir, &fields)?,
            fields,
            expected,
        };
        result.validate()?;
        Ok(result)
    }

    /// Admit the complete finite schema and the literal assertions before any target effects.
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() || self.fields.len() > 256 || self.declarations.len() > 4096 {
            return Err("direct response contract field/declaration bound".into());
        }
        if self
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.command != self.command)
        {
            return Err("direct response outcome belongs to another command".into());
        }
        crate::typed_fields::validate([self.fields.as_slice()], &self.declarations)?;
        let mut bytes = 0;
        for (name, value) in &self.expected {
            let field = self
                .fields
                .iter()
                .find(|field| &field.name == name)
                .ok_or_else(|| format!("undeclared response field {name}"))?;
            crate::selection::validate_direct_response_value(
                &field.type_ref,
                Some(value),
                &self.declarations,
                &mut bytes,
            )
            .map_err(|reason| format!("response literal {name}: {reason}"))?;
            presence(field, Some(value))?;
        }
        if serde_json::to_vec(self)
            .map_err(|error| error.to_string())?
            .len()
            > 1_048_576
        {
            return Err("direct response contract byte limit".into());
        }
        Ok(())
    }

    /// Check actual complete return shape, then compare the independent literal expectations.
    pub fn compare(&self, actual: Option<&BTreeMap<String, Node>>) -> Result<(), String> {
        self.validate()?;
        let actual = actual.ok_or("command returned no response")?;
        if actual
            .keys()
            .any(|name| !self.fields.iter().any(|field| &field.name == name))
        {
            return Err("response has an undeclared field".into());
        }
        let mut bytes = 0;
        for field in &self.fields {
            crate::selection::validate_direct_response_value(
                &field.type_ref,
                actual.get(&field.name),
                &self.declarations,
                &mut bytes,
            )
            .map_err(|reason| format!("response field {}: {reason}", field.name))?;
            presence(field, actual.get(&field.name))?;
        }
        if serde_json::to_vec(actual)
            .map_err(|error| error.to_string())?
            .len()
            > 1_048_576
        {
            return Err("direct response resource byte limit".into());
        }
        for (name, expected) in &self.expected {
            if actual.get(name) != Some(expected) {
                return Err(format!(
                    "response field {name} differs from its declared literal"
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn presence(field: &Field, actual: Option<&Node>) -> Result<(), String> {
    match (field.presence(), actual) {
        (Some(Presence::NullWhenAbsent), None) => Err(format!(
            "response field {} requires null_when_absent",
            field.name
        )),
        (Some(Presence::OmittedWhenAbsent), Some(Node::Null)) => Err(format!(
            "response field {} requires omitted_when_absent",
            field.name
        )),
        _ => Ok(()),
    }
}

/// Whether this suite requires direct return observation semantics.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| matches!(step, crate::ScenarioStep::ExpectDirectResponse { .. }))
    })
}

pub(crate) fn admit(suite: &crate::ConformanceSuite) -> Result<(), crate::AdmissionError> {
    for scenario in suite.scenarios.values() {
        let mut command = None;
        for step in &scenario.steps {
            if let crate::ScenarioStep::ExecuteCommand {
                command: called, ..
            }
            | crate::ScenarioStep::ExecuteCommandWithoutInput {
                command: called, ..
            } = step
            {
                command = Some(called);
            }
            if let crate::ScenarioStep::ExpectDirectResponse { response } = step {
                if suite.provenance.suite_version.major() < ORDINARY {
                    return Err(crate::AdmissionError::new(
                        "UnsupportedVocabulary",
                        "$suite",
                        "direct responses require suite/28 or /29",
                    ));
                }
                if command != Some(&response.command) {
                    return Err(crate::AdmissionError::new(
                        "InvalidResponse",
                        "$suite",
                        "direct response does not name the preceding invocation",
                    ));
                }
                response.validate().map_err(|reason| {
                    crate::AdmissionError::new("InvalidResponse", "$suite", reason)
                })?;
            }
        }
    }
    Ok(())
}

pub(crate) fn refuse_generation(
    suite: &crate::ConformanceSuite,
    target: &str,
) -> Result<(), crate::AdmissionError> {
    if used_by(suite) {
        return Err(crate::AdmissionError::new(
            "UnsupportedTarget",
            "$suite",
            format!("{target} does not execute direct response observations; use the Rust runner"),
        ));
    }
    Ok(())
}
