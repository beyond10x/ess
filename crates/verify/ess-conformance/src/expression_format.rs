//! Format authority for the persisted Family F expression vocabulary: suite `/40` ordinary and
//! `/41` coverage (`docs/design/expression-family-source22.md`, "Suite formats and old readers",
//! and the final review's decision 10).
//!
//! # What selects it
//!
//! A predicate a suite carries — a `satisfies` expectation or an observed selection's `first` —
//! that reads a one-segment fact on the right of a comparison that no binder names. The canonical
//! form writes that operand `{fact: …}`, and every reader before `/40` would either refuse the
//! mapping or, worse, read the bare word it replaces as text. A binder and a dotted path are not
//! such operands, so a suite comparing only those keeps its prior format and bytes (decision 5).
//!
//! One constant offset of a fact, `{offset: {fact, add|subtract}}` (A2), selects it too. Later Family F
//! units add their constructs here — `Distinct` and the derived `Utf8Bytes` selector — each one more
//! arm of [`reads`].
//!
//! # Cumulative over 36–39
//!
//! `/40` and `/41` are cumulative over `/36`–`/39`, whose vocabularies are reserved by other
//! work and not readable by this build. Until those readers land, a `/40` reader refuses that
//! vocabulary as unsupported: this build has no reader for it, so a suite carrying it does not
//! parse, and [`ADMITTED`] lists no major in that range.

use crate::{ConformanceSuite, ScenarioStep, ScenarioValue, ViewExpectation};
use ess_domain::selection::SelectionOperation;
use ess_primitives::predicate::Predicate;
use std::collections::BTreeMap;

/// The ordinary suite major that introduces the persisted expression vocabulary.
pub const ORDINARY: u32 = 40;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 41;

/// The suite majors at or above `/36` this build admits; `/36`–`/39` are reserved elsewhere.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal of a relabelled older suite says.
pub const REQUIRES: &str =
    "the expression vocabulary of a one-segment fact operand `{fact: …}`, a \
     comparison tagged `as: timestamp` or one constant offset `{offset: …}` requires suite/40 or /41";

/// Whether this predicate carries vocabulary only a `/40` reader reads.
pub fn reads(predicate: &Predicate) -> bool {
    predicate.reads_root_fact_operand() || predicate.compares_instants() || predicate.reads_offset()
}

/// Whether a typed suite needs `/40` (ordinary) or `/41` (coverage).
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(step_uses))
}

/// Reject an explicitly pinned older format before serialization or any target effect.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), crate::admission::AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(suite) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}

/// Check one raw predicate of a suite document of `major`, before it is evaluated.
pub fn admit_predicate(raw: &str, major: u32) -> Result<(), crate::admission::AdmissionError> {
    let node: ess_primitives::node::Node = serde_json::from_str(raw).map_err(|error| {
        crate::admission::AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    let predicate = Predicate::from_node(&node).map_err(|error| {
        crate::admission::AdmissionError::new("InvalidPredicate", "$predicate", error.to_string())
    })?;
    if major < ORDINARY && reads(&predicate) {
        return Err(crate::admission::AdmissionError::new(
            "UnsupportedVocabulary",
            "$predicate",
            REQUIRES,
        ));
    }
    Ok(())
}

/// [`admit_predicate`] for every `first` predicate of an observed selection.
pub fn admit_selection(raw: &str, major: u32) -> Result<(), crate::admission::AdmissionError> {
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

/// The ordinary major a fresh suite needs at least: [`ORDINARY`] when [`used_by`], else none.
pub fn ordinary_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least: [`COVERAGE`] when [`used_by`].
pub fn coverage_floor(suite: &ConformanceSuite) -> Option<u32> {
    used_by(suite).then_some(COVERAGE)
}

fn values_use(values: &BTreeMap<String, ScenarioValue>) -> bool {
    values.values().any(|value| match value {
        ScenarioValue::ObservedSelection { selection, .. } => {
            selection
                .plan
                .selectors
                .iter()
                .any(|selector| match &selector.operation {
                    SelectionOperation::First { predicate, .. } => reads(predicate),
                    SelectionOperation::FirstPresent { .. } => false,
                })
        }
        _ => false,
    })
}

fn expectation_uses(expectation: &ViewExpectation) -> bool {
    match expectation {
        ViewExpectation::Satisfies { predicate } => reads(predicate),
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
