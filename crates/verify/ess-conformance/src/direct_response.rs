//! Complete typed return observations, independent of events, subjects and views.
use crate::one_time_response::StringConstraints;
use crate::scenario::{CommandRef, OutcomeRef};
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_domain::types::UndeclaredFields;
use ess_domain::{types::Presence, Field, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// First ordinary suite with direct return observations.
pub const ORDINARY: u32 = 28;
/// The corresponding inventory-bearing suite.
pub const COVERAGE: u32 = 29;

/// First ordinary suite whose response observations carry String-newtype `constraints`
/// (beyond10x/ess#499), cumulative over every major below it.
pub const CONSTRAINED_ORDINARY: u32 = 46;
/// The corresponding inventory-bearing suite.
pub const CONSTRAINED_COVERAGE: u32 = 47;
/// The suite majors the constrained-response pair introduces.
pub const CONSTRAINED_ADMITTED: [u32; 2] = [CONSTRAINED_ORDINARY, CONSTRAINED_COVERAGE];

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
    /// String-newtype rules of the constrained declarations, keyed by the nominal type they
    /// constrain (suite/46 and /47, beyond10x/ess#499). Empty, and then absent from the bytes, when
    /// no reachable type is constrained.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub constraints: BTreeMap<QualifiedName, StringConstraints>,
    /// `ignored` where the command declares `undeclared_fields: ignored` (suite/48 and /49,
    /// beyond10x/ess#500): the response object admits keys it does not declare. Absent from the
    /// bytes when `refused`, the closed default.
    #[serde(default, skip_serializing_if = "UndeclaredFields::is_refused")]
    pub undeclared_fields: UndeclaredFields,
    /// The struct declarations that admit keys they do not declare, wherever the response reaches
    /// them (suite/48 and /49). Empty, and then absent from the bytes, when every one is closed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub undeclared_fields_ignored: BTreeSet<QualifiedName>,
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
    #[serde(default, deserialize_with = "constraints_present")]
    constraints: Option<BTreeMap<QualifiedName, StringConstraints>>,
    #[serde(default, deserialize_with = "crate::undeclared_fields::root_present")]
    undeclared_fields: UndeclaredFields,
    #[serde(default, deserialize_with = "crate::undeclared_fields::opened_present")]
    undeclared_fields_ignored: BTreeSet<QualifiedName>,
    expected: BTreeMap<String, Node>,
}

/// A `constraints` member that is present is never empty: an empty one would be a second spelling
/// of the bytes an unconstrained observation already has.
pub(crate) fn constraints_present<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<BTreeMap<QualifiedName, StringConstraints>>, D::Error> {
    let value = <BTreeMap<QualifiedName, StringConstraints> as serde::Deserialize>::deserialize(d)?;
    if value.is_empty() {
        return Err(serde::de::Error::custom(
            "response constraints, when present, name at least one type",
        ));
    }
    Ok(Some(value))
}

impl TryFrom<RawObservation> for Observation {
    type Error = String;
    fn try_from(raw: RawObservation) -> Result<Self, String> {
        let value = Self {
            command: raw.command,
            outcome: raw.outcome,
            fields: raw.fields,
            declarations: raw.declarations,
            constraints: raw.constraints.unwrap_or_default(),
            undeclared_fields: raw.undeclared_fields,
            undeclared_fields_ignored: raw.undeclared_fields_ignored,
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
        let protected = outcome.as_ref().is_some_and(|selected| {
            command.outcomes.iter().any(|branch| {
                branch.name == selected.outcome && !branch.one_time_response.is_empty()
            })
        });
        // A one-time outcome's trace checks the String rules on the same value, so its direct
        // observation keeps the bytes it had.
        let (declarations, constraints) = if protected {
            (
                crate::one_time_response::Response::of(ir, command)?.declarations,
                BTreeMap::new(),
            )
        } else {
            let declarations = crate::typed_fields::direct_response_declarations(ir, &fields)?;
            let constraints = response_constraints(ir, &declarations)?;
            (declarations, constraints)
        };
        // `undeclared_fields: ignored` (ess/24) opens the response object and each struct that says
        // so; a one-time outcome never reaches here opened, because its trace refuses it.
        let undeclared_fields_ignored = crate::undeclared_fields::opened(ir, &declarations);
        let result = Self {
            command: CommandRef::new(command.name.clone()),
            outcome,
            declarations,
            constraints,
            undeclared_fields: command.undeclared_fields,
            undeclared_fields_ignored,
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
        validate_response_constraints(&self.declarations, &self.constraints)?;
        crate::undeclared_fields::validate(&self.declarations, &self.undeclared_fields_ignored)?;
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
                &self.undeclared_fields_ignored,
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
        // An opened response (`undeclared_fields: ignored`) admits keys it does not declare and reads
        // nothing of them; every declared field is still checked below.
        if !self.undeclared_fields.is_ignored()
            && actual
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
                &self.undeclared_fields_ignored,
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
        check_constraints(&self.fields, &self.declarations, &self.constraints, actual)?;
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
    if constrained_used_by(suite) && suite.provenance.suite_version.major() < CONSTRAINED_ORDINARY {
        return Err(crate::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "response constraints require suite/46 or /47",
        ));
    }
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

/// The String-newtype rules a response observation carries for `declarations`: each constrained
/// declaration must be a newtype of `String`, and each of its invariants must decide over a lone
/// `value` text fact. Anything else is refused by name, before a scenario is written.
pub(crate) fn response_constraints(
    ir: &EssIr,
    declarations: &BTreeMap<QualifiedName, Declaration>,
) -> Result<BTreeMap<QualifiedName, StringConstraints>, String> {
    let constraints = crate::one_time_response::string_constraints(ir, declarations, "response")?;
    for (name, rules) in &constraints {
        if !crate::one_time_response::is_string(declarations, &TypeRef::Named(name.clone())) {
            return Err(format!(
                "response type `{name}` constrains a newtype of something other than String; only \
                 String-newtype constraints are checked on a returned value"
            ));
        }
        refuse_undecided(name, rules)?;
    }
    Ok(constraints)
}

fn refuse_undecided(name: &QualifiedName, rules: &StringConstraints) -> Result<(), String> {
    if let Some(predicate) = rules
        .invariants
        .iter()
        .find(|predicate| !crate::one_time_response::decides_over_value(predicate))
    {
        return Err(format!(
            "invariant `{predicate}` on response type `{name}` does not decide over its value alone"
        ));
    }
    Ok(())
}

/// Admit carried String-newtype rules against the declarations they travel with.
pub(crate) fn validate_response_constraints(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    constraints: &BTreeMap<QualifiedName, StringConstraints>,
) -> Result<(), String> {
    if constraints.is_empty() {
        return Ok(());
    }
    crate::one_time_response::validate_constraints(declarations, constraints, "response")?;
    for (name, rules) in constraints {
        refuse_undecided(name, rules)?;
    }
    Ok(())
}

/// Hold every actual value of `fields` in `actual` to the carried String-newtype rules.
pub(crate) fn check_constraints(
    fields: &[Field],
    declarations: &BTreeMap<QualifiedName, Declaration>,
    constraints: &BTreeMap<QualifiedName, StringConstraints>,
    actual: &BTreeMap<String, Node>,
) -> Result<(), String> {
    if constraints.is_empty() {
        return Ok(());
    }
    for field in fields {
        if let Some(value) = actual.get(&field.name) {
            crate::one_time_response::check(declarations, constraints, &field.type_ref, value)
                .map_err(|breach| match breach {
                    crate::one_time_response::Breach::Payload => format!(
                        "response field {} breaks a String rule its type declares",
                        field.name
                    ),
                    crate::one_time_response::Breach::Undecided => format!(
                        "response field {} could not be decided against its type's invariants",
                        field.name
                    ),
                })?;
        }
    }
    Ok(())
}

/// Whether a suite carries String-newtype rules on a response observation (suite/46 and /47).
pub fn constrained_used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            crate::ScenarioStep::ExpectDirectResponse { response } => {
                !response.constraints.is_empty()
            }
            crate::ScenarioStep::ExpectResponsePayload { response } => {
                !response.constraints.is_empty()
            }
            _ => false,
        })
    })
}

/// The ordinary major a fresh suite needs at least: [`CONSTRAINED_ORDINARY`] when
/// [`constrained_used_by`].
pub fn constrained_ordinary_floor(suite: &crate::ConformanceSuite) -> Option<u32> {
    constrained_used_by(suite).then_some(CONSTRAINED_ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`CONSTRAINED_COVERAGE`] when
/// [`constrained_used_by`].
pub fn constrained_coverage_floor(suite: &crate::ConformanceSuite) -> Option<u32> {
    constrained_used_by(suite).then_some(CONSTRAINED_COVERAGE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ess_primitives::predicate::Predicate;

    fn rules(invariant: &str) -> StringConstraints {
        StringConstraints {
            alphabet: None,
            prefix: None,
            invariants: vec![Predicate::parse_expression(invariant).expect("parses")],
        }
    }

    #[test]
    fn an_invariant_over_the_value_alone_is_admitted() {
        let name: QualifiedName = "catalog.items.Code".parse().unwrap();
        for invariant in [
            "value.count >= 4",
            "value != 'none'",
            "value.count < 9 and value != ''",
        ] {
            assert_eq!(
                refuse_undecided(&name, &rules(invariant)),
                Ok(()),
                "{invariant}"
            );
        }
    }

    #[test]
    fn an_invariant_reading_anything_else_is_refused_by_name() {
        let name: QualifiedName = "catalog.items.Code".parse().unwrap();
        let refused = refuse_undecided(&name, &rules("label.count >= 4")).unwrap_err();
        assert!(
            refused.contains(
                "on response type `catalog.items.Code` does not decide over its value alone"
            ) && refused.contains("label.count"),
            "{refused}"
        );
    }
}
