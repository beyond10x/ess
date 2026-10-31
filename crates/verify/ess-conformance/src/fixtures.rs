//! Typed values supplied independently before any target scenario activity.
//!
//! `docs/design/pre-execution-fixture-values.md` is the binding design. The format authority lives
//! here: a fixture prelude or a resolved event-value assertion is carried by ordinary
//! suite/[`ORDINARY`] and coverage suite/[`COVERAGE`], and each implies every major below it. A
//! fixture naming a row the deployment provides, [`FixtureField::entity`], is
//! [`crate::entity_fixtures`]' vocabulary.
use crate::scenario::{
    ConformanceScenario, ConformanceSuite, EventRef, ScenarioId, ScenarioStep, ScenarioValue,
};
use crate::selection::Declaration;
use ess_compiler::ir::EssIr;
use ess_compiler::ir::{ResolvedOutcome, ResolvedPayloadValue};
use ess_domain::{command::fixture_inputs::FixtureName, Field, QualifiedName, StateName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// The first ordinary suite major that carries fixture vocabulary.
pub const ORDINARY: u32 = 18;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = 19;

const REQUIRES: &str = "fixture values require suite/18 or /19";

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
    /// The entity whose row the deployment provides (suite/[`crate::entity_fixtures::ORDINARY`],
    /// `provided: true`): the value is the identity of a row of it resting in [`Self::state`],
    /// and [`Self::type_ref`] is the entity's identity type. `None` for a type-only fixture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<QualifiedName>,
    /// The lifecycle state the provided row rests in; present exactly with [`Self::entity`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<StateName>,
}

impl FixtureField {
    /// A type-only fixture: any value of `type_ref`.
    pub fn typed(name: FixtureName, type_ref: TypeRef) -> Self {
        Self {
            name,
            type_ref,
            entity: None,
            state: None,
        }
    }

    /// The identity of a row of `entity` the deployment provides, resting in `state`.
    pub fn provided(
        ir: &EssIr,
        name: FixtureName,
        entity: &QualifiedName,
        state: StateName,
    ) -> Self {
        let identity = &ir.entities()[entity].identity;
        Self {
            name,
            type_ref: crate::accessor::unresolve(&identity.type_ref),
            entity: Some(entity.clone()),
            state: Some(state),
        }
    }
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
        Self::of_fields(
            ir,
            fields
                .into_iter()
                .map(|(name, type_ref)| FixtureField::typed(name, type_ref))
                .collect(),
        )
    }

    /// [`Self::of`] over fields that may name a row the deployment provides, in name order.
    pub fn of_fields(ir: &EssIr, mut fields: Vec<FixtureField>) -> Result<Self, String> {
        fields.sort_by(|left, right| left.name.cmp(&right.name));
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
    ///
    /// A type that reaches itself only behind `Optional`, `List` or `Map` has finite values and is
    /// admitted; one with no such boundary has none and is refused, naming the fixture input and
    /// the members that close the cycle (beyond10x/ess#416).
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() || self.fields.len() > 256 || self.declarations.len() > 4096 {
            return Err("fixture field/declaration bound".into());
        }
        if let Some(field) = self
            .fields
            .iter()
            .find(|field| field.entity.is_some() != field.state.is_some())
        {
            return Err(format!(
                "fixture {} names a provided row by its entity and its state together",
                field.name
            ));
        }
        let typed: Vec<_> = self
            .fields
            .iter()
            .map(|field| Field::new(field.name.as_str(), field.type_ref.clone()))
            .collect();
        crate::typed_fields::validate_input([typed.as_slice()], &self.declarations)?;
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
            crate::selection::validate_fixture_value(
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
///
/// A row the deployment provides was arranged as an inline [`ScenarioStep::ResolveFixtures`] of
/// entity fixtures where the scenario needed it ([`provided_rows`]); those are lifted into the one
/// prelude here, beside the type-only fixtures the inputs name.
pub(crate) fn install(ir: &EssIr, suite: &mut ConformanceSuite) -> Vec<(ScenarioId, String)> {
    let mut refused = Vec::new();
    for (id, scenario) in &mut suite.scenarios {
        match prelude(ir, scenario) {
            Ok(Some((mut steps, fixtures))) => {
                steps.insert(0, ScenarioStep::ResolveFixtures { fixtures });
                scenario.steps = steps;
            }
            Ok(None) => {}
            Err(reason) => refused.push((id.clone(), reason)),
        }
    }
    refused
}

/// The scenario's steps with every provided row it names sent as its fixture, and the one
/// fixture prelude they need; `None` where they need none.
fn prelude(
    ir: &EssIr,
    scenario: &ConformanceScenario,
) -> Result<Option<(Vec<ScenarioStep>, Contract)>, String> {
    let (steps, provided) = provided_rows(scenario)?;
    let mut fields: BTreeMap<FixtureName, FixtureField> = provided
        .into_iter()
        .map(|field| (field.name.clone(), field))
        .collect();
    for step in &steps {
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
                    if let ResolvedPayloadValue::InputField { field: source, .. } = &field.value {
                        if field.conversion.is_some()
                            && matches!(input.get(source), Some(ScenarioValue::Fixture { .. }))
                        {
                            return Err(format!(
                                "fixture input {source} needs an unsupported payload conversion"
                            ));
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
            match fields.get(fixture) {
                // A provided row's identity is sent wherever the input takes it, `Optional` too.
                Some(held) if held.entity.is_some() => {
                    if !assignable(&held.type_ref, &ty) {
                        return Err(format!(
                            "fixture {fixture}, a provided row's identity, is sent as {field}, \
                             which does not take its type"
                        ));
                    }
                }
                Some(held) => {
                    if held.type_ref != ty {
                        return Err(format!("fixture {fixture} has incompatible declared types"));
                    }
                }
                None => {
                    fields.insert(fixture.clone(), FixtureField::typed(fixture.clone(), ty));
                }
            }
        }
    }
    let referenced: BTreeSet<&str> = steps
        .iter()
        .flat_map(values)
        .filter_map(|value| match value {
            ScenarioValue::Fixture { fixture } => Some(fixture.as_str()),
            _ => None,
        })
        .collect();
    // A provided row no value names is not needed by the scenario, and a prelude declares
    // exactly the names it references.
    fields.retain(|name, _| referenced.contains(name.as_str()));
    if fields.is_empty() {
        return Ok(None);
    }
    let contract = Contract::of_fields(ir, fields.into_values().collect())?;
    Ok(Some((steps, contract)))
}

/// The scenario's steps without the rows the deployment provides that its arrangement placed
/// inline, each value naming one of them as an instance rewritten to name its fixture, and those
/// rows as entity fixtures.
///
/// A row is named by its fixture wherever [`values`] reads a value: the positions a fixture may
/// stand in. A provided row named anywhere else (inside a list or a struct, or a view expectation
/// keyed by identity) refuses the scenario, naming the row, as nothing would bind its name.
fn provided_rows(
    scenario: &ConformanceScenario,
) -> Result<(Vec<ScenarioStep>, Vec<FixtureField>), String> {
    let mut rows: BTreeMap<FixtureName, FixtureField> = BTreeMap::new();
    let mut steps = Vec::with_capacity(scenario.steps.len());
    for step in &scenario.steps {
        match step {
            ScenarioStep::ResolveFixtures { fixtures }
                if fixtures.fields.iter().all(|field| field.entity.is_some()) =>
            {
                for field in &fixtures.fields {
                    if rows.get(&field.name).is_some_and(|held| held != field) {
                        return Err(format!(
                            "fixture {} names two different provided rows",
                            field.name
                        ));
                    }
                    rows.insert(field.name.clone(), field.clone());
                }
            }
            other => steps.push(other.clone()),
        }
    }
    if rows.is_empty() {
        return Ok((steps, Vec::new()));
    }
    let names: BTreeSet<&str> = rows.keys().map(FixtureName::as_str).collect();
    for step in &mut steps {
        for value in values_mut(step) {
            let ScenarioValue::Instance { instance } = value else {
                continue;
            };
            if names.contains(instance.as_str()) {
                let fixture = FixtureName::new(instance.as_str()).map_err(|e| e.to_string())?;
                *value = ScenarioValue::Fixture { fixture };
            }
        }
    }
    if let Some(name) = steps.iter().find_map(|step| names_instance(step, &names)) {
        return Err(format!(
            "the provided row `{name}` is named where no fixture value is read"
        ));
    }
    Ok((steps, rows.into_values().collect()))
}

/// The first instance in `names` a step still names, at any depth.
fn names_instance(step: &ScenarioStep, names: &BTreeSet<&str>) -> Option<String> {
    fn scan(value: &serde_json::Value, names: &BTreeSet<&str>) -> Option<String> {
        match value {
            serde_json::Value::Object(map) => {
                let instance_kind =
                    map.get("kind").and_then(serde_json::Value::as_str) == Some("instance");
                instance_kind
                    .then(|| map.get("instance").and_then(serde_json::Value::as_str))
                    .flatten()
                    .filter(|name| names.contains(name))
                    .map(str::to_owned)
                    .or_else(|| map.values().find_map(|inner| scan(inner, names)))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|inner| scan(inner, names)),
            _ => None,
        }
    }
    serde_json::to_value(step)
        .ok()
        .and_then(|value| scan(&value, names))
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

/// Whether the suite carries a value only an independently provisioned fixture supplies.
///
/// Narrower than [`used_by`]: an `expect_event_values` step comparing captured identities and
/// literals (beyond10x/ess#273) needs suite/18 but no fixture provider, so a reader that cannot
/// resolve fixture values can still read it.
pub(crate) fn provisioned_by(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ResolveFixtures { .. })
                || values(step)
                    .into_iter()
                    .any(|value| matches!(value, ScenarioValue::Fixture { .. }))
        })
    })
}

/// Refuse an explicitly pinned older format before serialization or target effects.
pub(crate) fn admit_format(
    suite: &ConformanceSuite,
) -> Result<(), crate::admission::AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
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
        ScenarioStep::SnapshotSubject { subject, .. }
        | ScenarioStep::SnapshotCompleteSubject { subject, .. } => result.extend(subject.values()),
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

/// [`values`], to write: the positions a fixture value may stand in.
fn values_mut(step: &mut ScenarioStep) -> Vec<&mut ScenarioValue> {
    use crate::scenario::ViewExpectation;
    let mut result = Vec::new();
    match step {
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => result.extend(input.values_mut()),
        ScenarioStep::ExpectEventValues { payload, .. } => result.extend(payload.values_mut()),
        ScenarioStep::SnapshotSubject { subject, .. }
        | ScenarioStep::SnapshotCompleteSubject { subject, .. } => {
            result.extend(subject.values_mut());
        }
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => result.extend(params.values_mut()),
        ScenarioStep::EventuallyView {
            params,
            expectation,
            ..
        } => {
            result.extend(params.values_mut());
            if let ViewExpectation::Contains { fields }
            | ViewExpectation::Excludes { fields }
            | ViewExpectation::At { fields, .. } = expectation
            {
                result.extend(fields.values_mut());
            }
        }
        ScenarioStep::ExpectView {
            expectation:
                ViewExpectation::Contains { fields }
                | ViewExpectation::Excludes { fields }
                | ViewExpectation::At { fields, .. },
            ..
        } => result.extend(fields.values_mut()),
        _ => {}
    }
    result
}

/// A fixture may widen into Optional, without an undeclared conversion or nominal coercion.
pub(crate) fn assignable(from: &TypeRef, to: &TypeRef) -> bool {
    from == to || matches!(to, TypeRef::Optional(inner) if assignable(from, inner))
}
