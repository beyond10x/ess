//! An external refusal answered before the row (`before_row: true`, ess/25, beyond10x/ess#534) in
//! the IR: carried on the outcome as `before_row`, left out where `false`, so every model without
//! the key keeps its IR bytes and an authored document without it keeps its written form.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::RawOutcome;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SIGN_IN: &str = include_str!("fixtures/external-before-row.yaml");

/// A model of `ess/22` with an external refusal and no `before_row:` key.
const UNMARKED: &str = include_str!("fixtures/refusal-with-effect.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn the_key_is_carried_on_the_outcome() {
    let ir = ir(SIGN_IN);
    let command = &ir.commands()[&"demo.desk.SignIn".parse().unwrap()];
    let marked: Vec<&str> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.before_row)
        .map(|outcome| outcome.name.as_str())
        .collect();
    assert_eq!(marked, ["bad-credential"]);
    assert_eq!(
        ir.to_canonical_json().matches("\"before_row\"").count(),
        1,
        "written once, on the marked outcome"
    );
}

/// A model without the key writes no `before_row` in its IR, and a document without it reads and
/// writes back without one: neither its IR bytes nor its authored form move.
#[test]
fn a_model_without_the_key_keeps_its_ir_bytes_and_its_written_form() {
    assert!(!UNMARKED.contains("before_row"));
    assert!(!ir(UNMARKED).to_canonical_json().contains("before_row"));
    // Every outcome written back as authored: the key only where it is set.
    for (text, marked) in [(UNMARKED, 0), (SIGN_IN, 1)] {
        let raw = RawSpecFile::parse(text).expect("the model parses");
        let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
        let written: Vec<String> = spec
            .commands()
            .values()
            .flat_map(|command| command.outcomes.iter().cloned())
            .map(|outcome| serde_json::to_string(&RawOutcome::from(outcome)).unwrap())
            .filter(|written| written.contains("before_row"))
            .collect();
        assert_eq!(written.len(), marked, "{written:#?}");
        assert!(written
            .iter()
            .all(|line| line.contains("\"before_row\":true")));
    }
    // `before_row: false` is the document without the key.
    let unset: RawOutcome = ess_yaml::from_str(
        "name: bad-credential\nexternal: a cause\nbefore_row: false\nerror: demo.desk.Bad\n",
    )
    .expect("the outcome parses");
    let written = serde_json::to_string(&unset).expect("the outcome serializes");
    assert!(!written.contains("before_row"), "{written}");
}
