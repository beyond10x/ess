//! The suite format authority for `defined()` and `missing()` over an `Optional` aggregate
//! (beyond10x/ess#176), the second construct of the round-3 pair.
//!
//! From `ess/16` a predicate may ask whether an `Optional` struct, list, map or `Json` is present,
//! and a present one is present even when it is empty. A view `satisfies` expectation carries such
//! a predicate whole — the #176 invariant `any: [state == Paused, not defined(metrics)]` checked
//! against every row after a branch — and a runner reads it against the row it was sent.
//!
//! The number moves because of what an older reader does with it: a runner from 0.37.0 binds a
//! fact for each scalar leaf of a row and nothing at a struct's or a list's own path, so it reads
//! `defined(metrics)` as `false` for a queue that holds metrics, and the invariant above passes on
//! every row without being checked — a wrong verdict caused by the age of the tool. Such a suite is
//! written in ordinary suite/[`ORDINARY`] or coverage suite/[`COVERAGE`], the pair
//! [`leaf_payloads`](crate::leaf_payloads) registers, and a reader that checks the number first
//! refuses it instead. The Go and TypeScript runtimes refuse these majors by version.
//!
//! Whether a path is an aggregate is a fact about the model, not the suite: `defined(metrics)`
//! reads the same over an `Optional<Integer>`, which every runner binds, and that suite keeps its
//! earlier format and bytes. So [`used_by`] takes the model, and a suite is admitted without one:
//! admission cannot tell the two apart, and refuses nothing here. A view filter and a
//! `when_subject` guard reading the same predicate are decided at synthesis — the suite carries the
//! rows they select, not the predicate — so neither selects this format.

use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_primitives::predicate::Predicate;

use crate::input::{aggregate_presence, presence_reads};
use crate::scenario::{ConformanceSuite, ScenarioStep, ViewExpectation};

/// The first ordinary suite major that carries presence of an `Optional` aggregate.
pub const ORDINARY: u32 = crate::leaf_payloads::ORDINARY;

/// The coverage counterpart of [`ORDINARY`].
pub const COVERAGE: u32 = crate::leaf_payloads::COVERAGE;

/// Whether any view `satisfies` predicate of the suite reads `defined()` or `missing()` over a path
/// that `ir` declares, in that view's fields, as an `Optional` struct, list, map, union or `Json`.
///
/// A view the model does not declare is left to admission, and selects nothing here.
pub fn used_by(ir: &EssIr, suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectView {
                view,
                expectation: ViewExpectation::Satisfies { predicate },
            }
            | ScenarioStep::EventuallyView {
                view,
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            } => ir
                .views()
                .get(view.name())
                .is_some_and(|declared| reads_presence(ir, &declared.fields, predicate)),
            _ => false,
        })
    })
}

/// Whether `predicate`, checked against `fields`, asks for the presence of an `Optional` aggregate:
/// exactly the reads [`predicate_projectable`](crate::input::predicate_projectable) admits beyond
/// the scalar ones.
fn reads_presence(
    ir: &EssIr,
    fields: &[ess_compiler::ir::ResolvedField],
    predicate: &Predicate,
) -> bool {
    // `missing(x)` is `not defined(x)`, so one walk finds both.
    let mut presence = BTreeSet::new();
    presence_reads(predicate, &mut presence);
    if presence.is_empty() {
        return false;
    }
    let checked =
        ess_compiler::expression::check_predicate(ir, fields, predicate, "conformance projection");
    checked
        .reads
        .iter()
        .any(|read| aggregate_presence(ir, read, &presence))
}
