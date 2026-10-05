//! Format authority for conditional aggregate measures: suite `/38` ordinary and `/39` coverage
//! (`docs/design/conditional-aggregate-measures.md`, beyond10x/ess#363), the aggregate
//! observation pair `docs/design/binding-causal-observation.md` reserved.
//!
//! # What selects it
//!
//! An aggregate scenario over a view one of whose measures declares `where:`. The rows such a
//! scenario expects are ordinary rows; what moves is their meaning. A measure that reads only the
//! rows its condition admits reports a number an implementation of the unconditioned measure
//! never would, and a reader that does not know the construct exists has no way to say so: it
//! would report the implementation broken for computing the measure the specification declares.
//! A reader that checks the number first refuses the document instead.
//!
//! Whether a view's measure is conditioned is a fact about the model, not the suite — the suite
//! carries the expected rows, never the predicate — so [`used_by`] takes the model, as
//! [`defined_aggregates`](crate::defined_aggregates) does, and [`admit_for`] refuses a suite
//! labelled below the pair wherever the model is in hand: before a browser execution, a replay or
//! any other target callback that pairs the two. A suite that does not reach such a view keeps its
//! earlier format and bytes.
//!
//! # Cumulative
//!
//! `/38` and `/39` imply every major below them that this build reads, `/36` and `/37` included.
//! Where several apply, the selection order is: seeds (`/42`, `/43`) over the expression
//! vocabulary (`/40`, `/41`) over this pair over the binding pair (`/36`, `/37`).

use ess_compiler::ir::EssIr;

use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioId, ScenarioStep};

/// The ordinary suite major that carries conditional aggregate measures.
pub const ORDINARY: u32 = 38;

/// Its coverage counterpart.
pub const COVERAGE: u32 = 39;

/// Both majors.
pub const ADMITTED: [u32; 2] = [ORDINARY, COVERAGE];

/// What a refusal of a relabelled older suite says.
pub const REQUIRES: &str = "an aggregate observation of a measure that declares `where:` (a \
                            conditional aggregate measure) requires suite/38 or /39";

/// Whether `view`, in `ir`, has a measure that declares a condition.
pub fn conditioned(ir: &EssIr, view: &ess_domain::name::QualifiedName) -> bool {
    ir.views()
        .get(view)
        .and_then(|declared| declared.aggregation.as_ref())
        .is_some_and(|aggregation| {
            aggregation
                .functions
                .values()
                .any(|aggregate| aggregate.r#where.is_some())
        })
}

/// Whether any scenario of `suite` observes a view `ir` declares with a conditioned measure: a
/// synthesized aggregate scenario, or an authored step that reads or asserts such a view. A view
/// the model does not declare is left to admission, and selects nothing here.
pub fn used_by(ir: &EssIr, suite: &ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        matches!(id, ScenarioId::Aggregate { view } if conditioned(ir, view.name()))
            || scenario.steps.iter().any(|step| match step {
                ScenarioStep::QueryView { view, .. }
                | ScenarioStep::ExpectView { view, .. }
                | ScenarioStep::EventuallyView { view, .. } => conditioned(ir, view.name()),
                _ => false,
            })
    })
}

/// The ordinary major a fresh suite needs at least.
pub fn ordinary_floor(ir: &EssIr, suite: &ConformanceSuite) -> Option<u32> {
    used_by(ir, suite).then_some(ORDINARY)
}

/// The coverage major a fresh coverage suite needs at least.
pub fn coverage_floor(ir: &EssIr, suite: &ConformanceSuite) -> Option<u32> {
    used_by(ir, suite).then_some(COVERAGE)
}

/// Refuses `suite`, read against `ir`, where it carries an aggregate observation of a conditioned
/// measure under a major below [`ORDINARY`]: before any target callback, wherever both are in hand.
pub fn admit_for(ir: &EssIr, suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    if suite.provenance.suite_version.major() < ORDINARY && used_by(ir, suite) {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            REQUIRES,
        ));
    }
    Ok(())
}
