//! A branch run a value object's wrapped positions read is made once while the invariants are
//! filed, not once per position.
//!
//! Every wrapped position of a value type is observed on a row some branch acting on the view's
//! source leaves, and each position ran those branches from the first again. Synthesis of a model
//! whose one creating branch leaves a value type at five wrapped view positions is held to a bound
//! on the runs it makes rather than answers from memory, a work bound rather than a clock, and its
//! synthesis must equal the one made with every run made afresh.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, wrapped_runs_made, wrapped_runs_unkept};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("../fixtures/wrapped-runs-kept.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Without the memory every run asked for was made (made == asked): five positions, five runs of
/// the one creating branch.
#[test]
fn a_wrapped_position_reuses_the_run_of_the_position_before() {
    let ir = ir(MODEL);
    let (kept, asked, made) = wrapped_runs_made(|| synthesize(&ir));
    let unkept = wrapped_runs_unkept(|| synthesize(&ir));
    assert_eq!(
        serde_json::to_string(&kept.suite).expect("the suite serializes"),
        serde_json::to_string(&unkept.suite).expect("the suite serializes"),
        "the suite bytes"
    );
    assert_eq!(kept, unkept, "the suite, refusals and notes");
    eprintln!("wrapped: {asked} branch runs asked for, {made} made");
    assert!(asked > 1, "the wrapped positions run the branch: {asked}");
    assert!(
        made < asked,
        "{made} of {asked} branch runs were made rather than answered from memory"
    );
}
