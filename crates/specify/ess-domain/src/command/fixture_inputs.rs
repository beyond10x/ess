//! Source-owned names for inputs a conformance fixture supplies before execution.
use super::{CommandSpec, OutcomeName};
use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
use std::fmt;

/// One stable fixture name, bounded and lower-kebab.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(into = "String", try_from = "String")]
pub struct FixtureName(OutcomeName);

impl FixtureName {
    /// Admit one name, reusing the established lower-kebab grammar.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ParseError> {
        let value = value.as_ref();
        if value.len() > 128 {
            return Err(ParseError::identifier(
                "fixture",
                value,
                "must not exceed 128 bytes".into(),
            ));
        }
        OutcomeName::new(value).map(Self)
    }

    /// The source spelling.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for FixtureName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<FixtureName> for String {
    fn from(value: FixtureName) -> Self {
        value.0.into()
    }
}

impl TryFrom<String> for FixtureName {
    type Error = ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::str::FromStr for FixtureName {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl schemars::JsonSchema for FixtureName {
    fn schema_name() -> String {
        "FixtureName".into()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        };
        schema.string().pattern = Some(OutcomeName::PATTERN.to_owned());
        schema.string().max_length = Some(128);
        schema.into()
    }
}

/// A fixture must not override a predicate witness or a scenario-owned subject.
pub(super) fn validate(command: &CommandSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for field in command.fixture_inputs.keys() {
        let at = command.site().key("fixture_inputs").named(field);
        if command.input_field(field).is_none() {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::UndeclaredReference,
                format!("fixture input `{field}` is not a declared command input"),
            ));
        }
        if command.outcomes.iter().any(|outcome| {
            outcome.condition.predicate().is_some_and(|predicate| {
                predicate
                    .fact_paths()
                    .iter()
                    .any(|path| path.namespace() == field)
            })
        }) {
            errors.push(ValidationError::at(
                at.clone(),
                ValidationCode::UnsupportedConstruct,
                "a fixture input cannot replace a field read by an outcome predicate",
            ));
        }
        if command.outcomes.iter().any(|outcome| {
            outcome.subject.as_ref().is_some_and(|subject| {
                subject.surface() == super::InstanceSurface::CommandInput
                    && subject.instance == *field
            })
        }) {
            errors.push(ValidationError::at(
                at,
                ValidationCode::ConflictingDeclaration,
                "a scenario-owned subject identity cannot also be a fixture input",
            ));
        }
    }
    errors
}
