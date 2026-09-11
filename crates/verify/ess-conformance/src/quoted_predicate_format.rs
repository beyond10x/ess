//! Format authority for text comparisons requiring a corrected structured operand reader.
use crate::{ConformanceSuite, ScenarioStep, ScenarioValue, ViewExpectation};
use ess_domain::selection::SelectionOperation;
use std::collections::BTreeMap;

/// Whether a typed suite needs the corrected text-operand reader (ordinary8/coverage9).
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(step_uses))
}

/// Reject explicitly pinned old formats before serialization or target effects.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), crate::admission::AdmissionError> {
    if suite.provenance.suite_version.major() < 8 && used_by(suite) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "lossless quoted text comparisons require suite/8 or /9",
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
                    SelectionOperation::First { predicate, .. } => {
                        predicate.requires_lossless_text_reader()
                    }
                    SelectionOperation::FirstPresent { .. } => false,
                })
        }
        _ => false,
    })
}

fn expectation_uses(expectation: &ViewExpectation) -> bool {
    match expectation {
        ViewExpectation::Satisfies { predicate } => predicate.requires_lossless_text_reader(),
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

/// Check only the original structured comparison operands, before typed normalization can
/// replace their spelling with a legacy-compatible compact form.
pub(crate) fn admit_predicate(
    raw: &str,
    major: u32,
) -> Result<(), crate::admission::AdmissionError> {
    let node: ess_primitives::node::Node = serde_json::from_str(raw).map_err(|error| {
        crate::admission::AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    // Bound and validate the grammar before following its known predicate positions.
    ess_primitives::predicate::Predicate::from_node(&node).map_err(|error| {
        crate::admission::AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    if major < 8 && original_uses(&node) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$predicate",
            "normalized structured comparison operands require suite/8 or /9",
        ));
    }
    Ok(())
}

fn original_uses(node: &ess_primitives::node::Node) -> bool {
    use ess_primitives::node::Node;
    match node {
        Node::Seq(children) => children.iter().any(original_uses),
        Node::Map(fields) => fields.iter().any(|(key, child)| match key.as_str() {
            "all" | "and" | "all_of" | "any" | "or" | "none" | "none_of_these" | "not" => {
                original_uses(child)
            }
            "forall" | "exists" => match child {
                Node::Map(fields) => fields.get("that").is_some_and(original_uses),
                _ => false,
            },
            _ => match child {
                Node::Map(operators) => operators.iter().any(|(operator, value)| {
                    matches!(
                        operator.as_str(),
                        "eq" | "equals"
                            | "=="
                            | "ne"
                            | "not_equals"
                            | "!="
                            | "lt"
                            | "<"
                            | "le"
                            | "lte"
                            | "<="
                            | "gt"
                            | ">"
                            | "ge"
                            | "gte"
                            | ">="
                    ) && scalar_changes(value)
                }),
                _ => false,
            },
        }),
        _ => false,
    }
}

fn scalar_changes(node: &ess_primitives::node::Node) -> bool {
    use ess_primitives::{
        facts::FactValue,
        node::Node,
        predicate::{Operand, Predicate},
    };
    let Node::Text(text) = node else {
        return false;
    };
    let wrapped = Node::Map(
        [(
            "value".into(),
            Node::Map([("eq".into(), node.clone())].into()),
        )]
        .into(),
    );
    matches!(Predicate::from_node(&wrapped), Ok(Predicate::Compare { right, .. })
        if right != Operand::Literal(FactValue::Text(text.clone())))
}

pub(crate) fn admit_selection(
    raw: &str,
    major: u32,
) -> Result<(), crate::admission::AdmissionError> {
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
