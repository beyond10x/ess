//! Adversary pass 1 on `story:an-authored-claim-may-compare-a-nested-field` (wave D335, U1).
//!
//! The Decisions give a dotted path the same rule on every claim surface: an act's `response:`,
//! an event `payload:`, an `error:` field and a row. On a payload, an error and a row a claim that
//! names a field whole and by a path as well is refused ("claim `x` whole or by its paths, not
//! both", `authored.rs` `split_paths`). A `response:` takes another route (`response_shape`), and
//! these cases hold it to the same rule: a response that claims `checkpoint` whole and
//! `checkpoint.format` by its path, with two different values, is a claim no return satisfies.

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const CHECKPOINT: &str = include_str!("fixtures/nested-response-checkpoint.yaml");
const SCENARIO: &str = include_str!("fixtures/nested-response-checkpoint-scenario.yaml");

fn authoring(response: &str) -> authored::Authoring {
    let raw = RawSpecFile::parse(CHECKPOINT).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("checkpoint.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let text = SCENARIO.replace(
        "response: {checkpoint.checkpoint_id: cp-1}",
        &format!("response: {response}"),
    );
    assert!(
        text.contains(&format!("response: {response}\n")),
        "the response is in place"
    );
    authored::compile(&ir, &[authored::Source::new("defer.yaml", text)])
}

/// The control: the scenario's own path claim compiles.
#[test]
fn the_fixture_path_claim_compiles() {
    let result = authoring("{checkpoint.checkpoint_id: cp-1}");
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
}

/// A response that claims `checkpoint` whole and one of its members by a path is refused, as the
/// same claim on an event payload, an error and a row is.
#[test]
fn a_response_claiming_a_field_whole_and_by_its_path_is_refused() {
    let result = authoring(
        "{checkpoint: {format: f-1, checkpoint_id: cp-1, call: lookup}, checkpoint.format: f-2}",
    );
    let printed: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    assert!(
        printed
            .iter()
            .any(|text| text.contains("checkpoint.format") && text.contains("whole")),
        "story:an-authored-claim-may-compare-a-nested-field: a response claiming `checkpoint` \
         whole and `checkpoint.format` by its path, with two values no return holds at once, is \
         refused as a payload, error or row claim is; compiled {} scenario(s), refusals \
         {printed:#?}",
        result.scenarios.len()
    );
}
