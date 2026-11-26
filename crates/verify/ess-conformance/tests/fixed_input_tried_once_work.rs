//! A row search whose input fields the arrangement fixed tries each input it can send once.
//!
//! The setup run of a creating branch arranged as its own scenario is sent the field values the
//! caller chose: every candidate input the search builds is overwritten with them, so candidates
//! that differed only in those fields become one input. Tried again for each copy, on every row
//! the search visits, the rows beside and the branch were decided many times over for the same
//! input; on one downstream model 1,239,300 inputs were tried where 52,530 differed. Synthesis is
//! held to trying each such input once, a work bound rather than a clock: guards over a field the
//! chosen values overwrite add candidates, and no input tried.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{fixed_input_work, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = include_str!("fixtures/a-creator-chosen-values-beside-several-rows.yaml");

/// The start of the outcomes of the command whose setup run is sent the chosen grant name and
/// mode.
const ISSUING: &str = "      - {name: mode, type: shop.desk.Mode}\n    outcomes:\n";

/// A refusal of the creating command guarded on the grant name alone: it adds candidates that
/// differ in the name, which the chosen values overwrite.
const NAME_GUARDS: &str =
    "      - name: no-push\n        when: {name: push}\n        error: shop.desk.GrantRefused\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model, with the creating command guarded on the grant name where `guarded`.
fn desk(guarded: bool) -> String {
    assert!(MODEL.contains(ISSUING), "the creating command is declared");
    if guarded {
        MODEL.replacen(ISSUING, &format!("{ISSUING}{NAME_GUARDS}"), 1)
    } else {
        MODEL.to_owned()
    }
}

/// The suite, and `(searched, tried)`: the input lists row searches with fixed input fields built
/// for a row, and the inputs of them they tried there.
fn work(guarded: bool) -> (Synthesis, (u64, u64)) {
    let model = ir(&desk(guarded));
    fixed_input_work(|| synthesize(&model))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// Without the guard every candidate holds one of two inputs the chosen values leave apart, and
/// both are tried on each row; the guard on the name multiplies the candidates, and the chosen
/// name makes them those two inputs again.
#[test]
fn an_input_the_chosen_values_make_equal_is_tried_once() {
    let (plain, (searched_plain, tried_plain)) = work(false);
    let (guarded, (searched_guarded, tried_guarded)) = work(true);
    assert!(
        refusals(&plain).is_empty(),
        "refusals: {:#?}",
        refusals(&plain)
    );
    assert!(
        refusals(&guarded).is_empty(),
        "refusals: {:#?}",
        refusals(&guarded)
    );
    assert!(
        searched_plain >= 1 && searched_guarded >= 1,
        "rows are searched with chosen values: {searched_plain} and {searched_guarded}"
    );
    assert_eq!(
        tried_guarded * searched_plain,
        tried_plain * searched_guarded,
        "the inputs tried per row, without the guard on the chosen name ({tried_plain} on \
         {searched_plain} rows) and with it ({tried_guarded} on {searched_guarded} rows)"
    );
}
