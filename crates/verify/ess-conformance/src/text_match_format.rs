//! Format authority for string predicate operators (beyond10x/ess#95): ordinary suite/14 and
//! coverage suite/15.
//!
//! A suite carries a predicate in two places, and only there: a view's `satisfies` expectation and
//! an observed selection's plan. A string operator in a command guard is decided at synthesis and
//! never reaches the suite, so such a suite keeps its format and its bytes. A reader older than the
//! construct would refuse it as an unknown operator, which blames the document for the age of the
//! tool; the number turns that into "upgrade the tool". Each major implies the ones below it, so a
//! suite that also needs a lower major's vocabulary still takes this one
//! ([`ConformanceSuite::select_fresh_format`](crate::ConformanceSuite::select_fresh_format)).
//!
//! Suites are not type-checked, so admission also refuses a string-operator operand that is not a
//! JSON string, which validation would have refused in a model.
use crate::admission::AdmissionError;
use crate::{ConformanceSuite, ScenarioStep, ScenarioValue, ViewExpectation};
use ess_domain::selection::SelectionOperation;
use ess_primitives::facts::FactValue;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use std::collections::BTreeMap;

/// The first ordinary suite major that carries a string operator.
pub const ORDINARY: u32 = 14;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = 15;

const REQUIRES: &str = "string predicate operators require suite/14 or /15";

/// Whether a typed suite carries a string operator where a suite carries a predicate.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(step_uses))
}

/// Reject explicitly pinned older formats before serialization or target effects.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}

fn values_use(values: &BTreeMap<String, ScenarioValue>) -> bool {
    values.values().any(|value| match value {
        ScenarioValue::ObservedSelection { selection, .. } => {
            selection
                .plan
                .selectors
                .iter()
                .any(|selector| match &selector.operation {
                    SelectionOperation::First { predicate, .. } => predicate.uses_text_match(),
                    SelectionOperation::FirstPresent { .. } => false,
                })
        }
        _ => false,
    })
}

fn expectation_uses(expectation: &ViewExpectation) -> bool {
    match expectation {
        ViewExpectation::Satisfies { predicate } => predicate.uses_text_match(),
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => values_use(fields),
        _ => false,
    }
}

fn step_uses(step: &ScenarioStep) -> bool {
    match step {
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => values_use(input),
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => values_use(params),
        ScenarioStep::ExpectView { expectation, .. } => expectation_uses(expectation),
        ScenarioStep::EventuallyView {
            params,
            expectation,
            ..
        } => values_use(params) || expectation_uses(expectation),
        _ => false,
    }
}

/// Whether any string operator in `predicate` compares with something other than text.
fn non_text_operand(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::TextMatch { value, .. } => !matches!(value, FactValue::Text(_)),
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().any(non_text_operand)
        }
        Predicate::Not(inner) => non_text_operand(inner),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            non_text_operand(&quantified.body)
        }
        _ => false,
    }
}

/// Check one original predicate carrier: the construct needs suite/14 or /15, and its operand is
/// a JSON string.
pub(crate) fn admit_predicate(raw: &str, major: u32) -> Result<(), AdmissionError> {
    let node: Node = serde_json::from_str(raw).map_err(|error| {
        AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    let predicate = Predicate::from_node(&node).map_err(|error| {
        AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    if !predicate.uses_text_match() {
        return Ok(());
    }
    if major < ORDINARY {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$predicate",
            REQUIRES,
        ));
    }
    if non_text_operand(&predicate) {
        return Err(AdmissionError::new(
            "InvalidPredicate",
            "$predicate",
            "a string operator's operand must be a JSON string",
        ));
    }
    Ok(())
}

/// [`admit_predicate`] over every `first` selector predicate of an observed selection.
pub(crate) fn admit_selection(raw: &str, major: u32) -> Result<(), AdmissionError> {
    let value = crate::count_json::Json::parse(raw, "$selection")?;
    let plan = value.object()?["plan"].object()?;
    for selector in plan["selectors"].array()? {
        let fields = selector.object()?["operation"].object()?;
        if let Some(predicate) = fields.get("predicate") {
            admit_predicate(&predicate.raw, major)?;
        }
    }
    Ok(())
}
