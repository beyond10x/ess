//! A row several commands move through a few states is searched for once per question, not once
//! per branch that asks.
//!
//! Every branch of a command that reads stored fields arranges its row with the same search: the
//! same entity, the command's guards as hints, the same creating rows to start from. Only what the
//! branch accepts differs. Each search advances every row it reaches by every move the model
//! declares, checking each input chosen toward the guards for the branch it selects, and each row
//! it reaches is checked again for the branch the command's inputs select there. So a search costs
//! more with each command that moves the row, and a command with more branches asks it more often.
//! A downstream specification of this shape, one entity moved by seven commands, took more than
//! three minutes to synthesize. Synthesis is held to a bound on the rows and inputs it checks for
//! the branch they select, a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{branch_selections_tried, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/rows-moved-by-many-commands.yaml");

/// The rows and inputs the whole synthesis may check for the branch they select: 10,167 when each
/// search and each row's selections are made once per synthesis, 27,429 when every branch makes
/// them anew.
const BOUND: u64 = 13_000;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
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
fn rows_moved_by_many_commands_are_checked_within_a_bound() {
    let ir = ir(MODEL);
    let (synthesis, tried) = branch_selections_tried(|| synthesize(&ir));
    assert!(
        tried <= BOUND,
        "rows and inputs were checked {tried} times for the branch they select, more than \
         {BOUND}\nscenarios: {:#?}\nrefusals: {:#?}",
        ids(&synthesis),
        refusals(&synthesis)
    );
}
