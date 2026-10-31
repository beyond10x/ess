//! Source-owned names for inputs a conformance fixture supplies before execution.
use super::{CommandSpec, OutcomeCondition, OutcomeName, RelatedVia};
use ess_primitives::error::{ParseError, ValidationCode, ValidationError, ValidationErrors};
use ess_primitives::predicate::Predicate;
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

/// Whether `predicate` reads the input field `field`: as a root (`when:`, the input guard beside
/// another guard) or under `input.` (a predicate over a row, and from `ess/22` a `when:` too).
///
/// A row whose entity declares a stored field named `input` reads `input.<member>` as its own
/// field; a fixture binding over an input of that member's name is refused all the same, which
/// asks only that the author drop the binding.
fn predicate_reads(predicate: &Predicate, field: &str) -> bool {
    predicate.fact_paths().iter().any(|path| {
        path.namespace() == field
            || (path.namespace() == super::subject_fact::INPUT_NAMESPACE
                && path.segments().get(1).is_some_and(|read| read == field))
    })
}

/// Whether some branch of `command` is selected by the input field `field`: every place a guard
/// reads the input (story:a-fixture-may-be-a-row-the-deployment-already-holds).
///
/// | condition | where it reads the input |
/// |---|---|
/// | every guard | its input guard, [`OutcomeCondition::predicate`] |
/// | `when_related: {via: input.<field>, …}` | the identity naming the row, and an `all:` element's |
/// | `when_related: {…, predicate}` | `input.<field>` in the row's predicate |
/// | `when_related: {entity, where, forall}` | `input.<field>` in the selector or the `forall:` body |
/// | `when_subject: {predicate}`, an `external:` narrowed by one | `input.<field>` in the predicate |
///
/// A fixture over any of them would choose the branch the guard takes, which is the generated
/// witness a fixture must not replace (`docs/design/pre-execution-fixture-values.md`).
fn read_by_a_guard(condition: &OutcomeCondition, field: &str) -> bool {
    if condition
        .predicate()
        .is_some_and(|predicate| predicate_reads(predicate, field))
    {
        return true;
    }
    let via_reads = |via: &RelatedVia| matches!(via, RelatedVia::Input(read) if read == field);
    match condition {
        OutcomeCondition::Related {
            via, test, also, ..
        } => {
            via_reads(via)
                || test
                    .predicate()
                    .is_some_and(|predicate| predicate_reads(predicate, field))
                || also
                    .iter()
                    .any(|other| via_reads(&other.via) || predicate_reads(&other.predicate, field))
        }
        OutcomeCondition::RelatedSet {
            selection, test, ..
        } => {
            predicate_reads(&selection.filter, field)
                || matches!(test, super::row_set::RowSetTest::Forall(body) if predicate_reads(body, field))
        }
        OutcomeCondition::SubjectPredicate { predicate, .. }
        | OutcomeCondition::ExternalSubject { predicate, .. } => predicate_reads(predicate, field),
        _ => false,
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
        if command
            .outcomes
            .iter()
            .any(|outcome| read_by_a_guard(&outcome.condition, field))
        {
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
