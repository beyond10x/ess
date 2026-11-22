//! Which branch a command selects on a stored row for an input is decided once per synthesis, not
//! once per search that advances the row.
//!
//! A search for a row a branch reads advances every row it meets by every move the model declares.
//! For a move of a command that itself reads stored fields, it checks every input it offers for
//! the branch the command selects on that row. Every moving branch of that command asks the same
//! question, and so does every other search whose goal differs, because each branch reading a
//! different field searches with different hints. A downstream specification of 110 commands took
//! more than half an hour to synthesize, nine in ten of these checks repeating one made before.
//! Synthesis of a model of one entity whose fields twelve commands set, each refusing a row whose
//! own field is already set, is held to a bound on the rows and inputs it checks for the branch
//! they select, a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{branch_selections_tried, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/stored-rows-decided-once.yaml");

/// The rows and inputs the whole synthesis may check for the branch they select: 204 when each
/// command's pick on a row is kept for the synthesis, 348 when every search decides it anew.
const BOUND: u64 = 260;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn a_row_many_commands_set_is_decided_within_a_bound() {
    let ir = ir(MODEL);
    let (synthesis, tried) = branch_selections_tried(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert!(
        tried <= BOUND,
        "rows and inputs were checked {tried} times for the branch they select, more than \
         {BOUND}\nscenarios: {:#?}",
        ids(&synthesis)
    );
}
