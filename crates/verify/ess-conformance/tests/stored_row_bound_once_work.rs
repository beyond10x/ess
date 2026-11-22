//! A stored row is bound to facts once and read by every stored-field predicate decided over it,
//! not bound again, whole, for each predicate.
//!
//! Every guard, hint and leaf a search decides over an arranged row reads one or two of its
//! fields, yet each decision bound every field the row holds. The cost of a decision grew with the
//! width of the row: a model whose one entity carries 128 stored fields that no guard or view
//! reads took about nine times the work of the same model without them, and two downstream
//! specifications with rows of about forty fields spent most of their synthesis binding rows.
//! Synthesis of a model of one entity whose six tags six commands set, beside twenty-four spare
//! fields nothing reads, is held to a bound on the stored fields it binds, a work bound rather
//! than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{branch_inputs_made, stored_fields_bound, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/stored-row-bound-once.yaml");

/// The stored fields the whole synthesis may bind: 210 when each row is bound once, 17,100 when
/// every predicate binds the whole row again.
const BOUND: u64 = 1_000;

/// The thirty stored fields of one row, which any synthesis binds at least once: a count below it
/// says the binds went past the counter, not that they were saved.
const FLOOR: u64 = 30;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("yard.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn a_wide_row_is_bound_once_for_every_predicate_over_it() {
    let ir = ir(MODEL);
    let (synthesis, bound) = stored_fields_bound(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(synthesis.suite.scenarios.len(), 35);
    assert!(
        bound >= FLOOR,
        "{bound} stored fields were counted, fewer than one row's {FLOOR}"
    );
    assert!(
        bound <= BOUND,
        "{bound} stored fields were bound to facts, more than {BOUND}"
    );
}

/// The inputs an arranging branch is tried with toward a stored-field goal read the branch, the
/// hints and the distinction, never the row; a search asked for them at every row it advanced by
/// the branch, and each ask made them anew. Every ask is counted, so the count of asks is what
/// the synthesis made before they were kept: 54 made for 54 asks before, 49 for 54 after on this
/// model, whose searches advance few rows each (a search walking many rows asks at every one of
/// them). The counts are exact, so one more making fails.
#[test]
fn a_branch_makes_its_inputs_toward_a_goal_once_for_every_row() {
    let ir = ir(MODEL);
    let (synthesis, asked, made) = branch_inputs_made(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(
        (asked, made),
        (54, 49),
        "an arranging branch's inputs were asked for and made"
    );
}
