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
//! One constant offset of a fact, `{offset: {fact, add|subtract}}` (A2), selects it too, and so does
//! the derived UTF-8 byte length of a text, `{utf8_bytes: <path>}` (decision 11) — never a path that
//! merely ends in `utf8_bytes`, which is a declared member. Later Family F units add their constructs
//! here, each one more arm of [`reads`]: distinct list members, `{distinct: {in, as, by, kind}}`,
//! whose key kind every reader requires.
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
     comparison tagged `as: timestamp`, one constant offset `{offset: …}`, the UTF-8 byte length \
     `{utf8_bytes: …}` or distinct list members `{distinct: …}` requires suite/40 or /41";

/// What a refusal of a `distinct` without its key kind says: a suite never carries a key whose
/// equality its reader would have to infer.
pub const UNKINDED: &str =
    "`{distinct: …}` in a suite names its key kind (`kind:`), which no reader infers";

/// Whether this predicate carries vocabulary only a `/40` reader reads.
pub fn reads(predicate: &Predicate) -> bool {
    predicate.reads_root_fact_operand()
        || predicate.compares_instants()
        || predicate.reads_offset()
        || predicate.reads_utf8_bytes()
        || predicate.reads_distinct()
}

/// Whether a suite runner reads this predicate over a view row with every sequence bound element
/// by element, beside its `.count` (final review decision 1): under `/40` and `/41`, for the
/// collection reads a `distinct` makes. A predicate without one keeps the row binding it had, so
/// no older suite's verdict moves.
pub fn binds_sequences(predicate: &Predicate) -> bool {
    predicate.reads_distinct()
}

/// Whether some `distinct` of this predicate names no key kind, which no suite admits.
pub fn unkinded(predicate: &Predicate) -> bool {
    predicate
        .distincts()
        .iter()
        .any(|(distinct, _)| distinct.key_kind.is_none())
}

/// Whether a typed suite needs `/40` (ordinary) or `/41` (coverage).
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .any(|scenario| scenario.steps.iter().any(|step| step_carries(step, reads)))
}

/// Reject an explicitly pinned older format before serialization or any target effect.
pub fn admit_suite(suite: &ConformanceSuite) -> Result<(), crate::admission::AdmissionError> {
    if suite.scenarios.values().any(|scenario| {
        scenario
            .steps
            .iter()
            .any(|step| step_carries(step, unkinded))
    }) {
        return Err(crate::admission::AdmissionError::new(
            "InvalidPredicate",
            "$suite",
            UNKINDED,
        ));
    }
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
    if unkinded(&predicate) {
        return Err(crate::admission::AdmissionError::new(
            "InvalidPredicate",
            "$predicate",
            UNKINDED,
        ));
    }
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

fn values_use(values: &BTreeMap<String, ScenarioValue>, test: fn(&Predicate) -> bool) -> bool {
    values.values().any(|value| match value {
        ScenarioValue::ObservedSelection { selection, .. } => {
            selection
                .plan
                .selectors
                .iter()
                .any(|selector| match &selector.operation {
                    SelectionOperation::First { predicate, .. } => test(predicate),
                    SelectionOperation::FirstPresent { .. } => false,
                })
        }
        _ => false,
    })
}

fn expectation_uses(expectation: &ViewExpectation, test: fn(&Predicate) -> bool) -> bool {
    match expectation {
        ViewExpectation::Satisfies { predicate } => test(predicate),
        ViewExpectation::Contains { fields }
        | ViewExpectation::Excludes { fields }
        | ViewExpectation::At { fields, .. } => values_use(fields, test),
        _ => false,
    }
}

fn step_carries(step: &ScenarioStep, test: fn(&Predicate) -> bool) -> bool {
    match step {
        ScenarioStep::ExecuteCommand { input, .. }
        | ScenarioStep::ExpectInvocation { input, .. } => values_use(input, test),
        ScenarioStep::QueryView { params, .. }
        | ScenarioStep::ExpectHalt { params, .. }
        | ScenarioStep::EventuallyHalt { params, .. } => values_use(params, test),
        ScenarioStep::ExpectView { expectation, .. } => expectation_uses(expectation, test),
        ScenarioStep::EventuallyView {
            params,
            expectation,
            ..
        } => values_use(params, test) || expectation_uses(expectation, test),
        _ => false,
    }
}
