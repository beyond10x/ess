//! Closed paired browser projection. Pairing does not authenticate or reconstruct the full model.
use crate::{
    count_json::Json,
    coverage::{self, AdmittedInput, SuiteInputDocument, SuiteReference},
    AdmissionError,
};
use ess_compiler::EssIr;
use serde::{Deserialize, Serialize};

/// Paired browser document version.
pub const REPLAY_FORMAT: &str = "ess-conformance-replay/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Model {
    system: String,
    version: String,
    spec_digest: String,
    contract_digest: String,
    entities: Vec<Entity>,
    commands: Vec<Command>,
    views: Vec<View>,
    actors: Vec<Actor>,
    bindings: Vec<Binding>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entity {
    name: String,
    display: String,
    identity: String,
    initial: String,
    states: Vec<String>,
    terminal: Vec<String>,
    fields: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    name: String,
    display: String,
    outcomes: Vec<Outcome>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    name: String,
    refuses: bool,
    subject: Option<Subject>,
    emits: Vec<String>,
    sets: Vec<Assignment>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Subject {
    entity: String,
    kind: Effect,
    transition: Option<String>,
    from: Vec<String>,
    to: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Effect {
    Creates,
    Updates,
    Moves,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Assignment {
    target: String,
    from: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct View {
    name: String,
    display: String,
    entity: String,
    consistency: Consistency,
    filter: Option<String>,
    fields: Vec<String>,
    params: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Consistency {
    ReadYourWrites,
    Eventual,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    name: String,
    display: String,
    may: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    name: String,
    event: String,
    command: String,
    delivery: Delivery,
    failure: Failure,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Delivery {
    AtLeastOnce,
    AtMostOnce,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Failure {
    Retry,
    Drop,
    Escalate,
}
#[derive(Debug, Clone, Serialize)]
struct Wire {
    format: &'static str,
    model: Model,
    suite: SuiteReference,
    input: SuiteInputDocument,
}
/// An immutable model/input pair with the complete original suite lineage admitted.
#[derive(Debug, Clone)]
pub struct AdmittedReplay {
    wire: Wire,
    input: AdmittedInput,
}
impl AdmittedReplay {
    /// Pair a projection derived from the actual model with an admitted suite input.
    pub fn new(ir: &EssIr, input: &AdmittedInput) -> Result<Self, AdmissionError> {
        crate::admission::model(ir)?;
        let expected = crate::SuiteProvenance::of(ir);
        let actual = &input.selected().suite().provenance;
        if expected.system != actual.system
            || expected.specification_version != actual.specification_version
            || expected.spec_digest != actual.spec_digest
            || expected.contract_digest != actual.contract_digest
        {
            return Err(coverage::error(
                "replay model differs from admitted suite identity",
            ));
        }
        if let coverage::Scope::Component { component } = &input
            .selected()
            .coverage()
            .expect("input/1")
            .selection
            .scope
        {
            if !ir.components().contains_key(component) {
                return Err(coverage::error("replay component is not declared"));
            }
        }
        // Reuse the legacy projection's semantic owners, then retain only the explicit typed model.
        let mut value: serde_json::Value = serde_json::from_str(&crate::web::model(ir))
            .map_err(|e| coverage::error(e.to_string()))?;
        value["spec_digest"] = serde_json::json!(expected.spec_digest);
        value["contract_digest"] = serde_json::json!(expected.contract_digest);
        let model: Model =
            serde_json::from_value(value).map_err(|e| coverage::error(e.to_string()))?;
        Ok(Self {
            wire: Wire {
                format: REPLAY_FORMAT,
                model,
                suite: SuiteReference::of(input.selected()),
                input: input.document(),
            },
            input: input.clone(),
        })
    }
    /// Admit the closed original paired JSON and its exact full original input.
    pub fn from_json(original: &str) -> Result<Self, AdmissionError> {
        let raw = Json::parse(original, "$replay")?;
        let root = raw.closed(&["format", "model", "suite", "input"], &[])?;
        if root["format"].text()? != REPLAY_FORMAT {
            return Err(coverage::error("unsupported replay format"));
        }
        validate_model_json(&root["model"])?;
        let model: Model =
            serde_json::from_str(&root["model"].raw).map_err(|e| coverage::error(e.to_string()))?;
        let suite: SuiteReference =
            serde_json::from_str(&root["suite"].raw).map_err(|e| coverage::error(e.to_string()))?;
        let input = AdmittedInput::from_json(&root["input"].raw)?;
        let actual = &input.selected().suite().provenance;
        if suite != SuiteReference::of(input.selected())
            || model.system != actual.system
            || model.version != actual.specification_version
            || model.spec_digest != actual.spec_digest.as_str()
            || model.contract_digest != actual.contract_digest.as_str()
        {
            return Err(coverage::error(
                "replay model/reference differs from selected original suite",
            ));
        }
        Ok(Self {
            wire: Wire {
                format: REPLAY_FORMAT,
                model,
                suite,
                input: input.document(),
            },
            input,
        })
    }
    /// Canonical paired document with unchanged inner and parent strings.
    pub fn to_canonical_json(&self) -> Result<String, AdmissionError> {
        coverage::canonical(&self.wire)
    }
    /// Immutable complete input, checked before replay state can be created.
    pub fn input(&self) -> &AdmittedInput {
        &self.input
    }
}
fn validate_model_json(raw: &Json) -> Result<(), AdmissionError> {
    let model = raw.closed(
        &[
            "system",
            "version",
            "spec_digest",
            "contract_digest",
            "entities",
            "commands",
            "views",
            "actors",
            "bindings",
        ],
        &[],
    )?;
    for entity in model["entities"].array()? {
        entity.closed(
            &[
                "name", "display", "identity", "initial", "states", "terminal", "fields",
            ],
            &[],
        )?;
    }
    for command in model["commands"].array()? {
        let command = command.closed(&["name", "display", "outcomes"], &[])?;
        for outcome in command["outcomes"].array()? {
            let outcome = outcome.closed(&["name", "refuses", "subject", "emits", "sets"], &[])?;
            if !outcome["subject"].null() {
                outcome["subject"].closed(&["entity", "kind", "transition", "from", "to"], &[])?;
            }
            for set in outcome["sets"].array()? {
                set.closed(&["target", "from"], &[])?;
            }
        }
    }
    for view in model["views"].array()? {
        view.closed(
            &[
                "name",
                "display",
                "entity",
                "consistency",
                "filter",
                "fields",
                "params",
            ],
            &[],
        )?;
    }
    for actor in model["actors"].array()? {
        actor.closed(&["name", "display", "may"], &[])?;
    }
    for binding in model["bindings"].array()? {
        binding.closed(&["name", "event", "command", "delivery", "failure"], &[])?;
    }
    Ok(())
}
