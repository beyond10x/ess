//! The ess/15 outcome shapes in generated seams (`docs/design/outcome-shapes.md`).
//!
//! A declared `unknown_instance:` branch is an outcome like any other, so the Rust and Go seams
//! carry its variant; and because it is the answer for an identity no record carries, the
//! wrong-state branch beside it needs no payload-free second spelling.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::unknown_instance::unknown_instance_answer;
use ess_synth::{synthesize_for, Target};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emitted(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("the model synthesizes: {failure:?}"))
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_declared_unknown_instance_branch_is_the_answer_and_needs_no_second_spelling() {
    let ir = ir();
    for command in ir.commands().values() {
        assert!(
            unknown_instance_answer(&ir, command).is_none(),
            "{} declares its unknown-instance answer, so no wrong-state spelling is added",
            command.name
        );
    }
}

#[test]
fn both_seams_carry_the_unknown_instance_variant() {
    let ir = ir();
    let rust = emitted(&ir, Target::Rust);
    assert!(
        rust.contains("NoSuchCall"),
        "the Rust seam names the branch"
    );
    let go = emitted(&ir, Target::Go);
    assert!(go.contains("NoSuchCall"), "the Go seam names the branch");
}

#[test]
fn the_rust_entity_has_a_typed_constructor_for_each_creation_state() {
    let rust = emitted(&ir(), Target::Rust);
    assert!(
        rust.contains("pub fn new(data: CallData) -> Self"),
        "initial keeps `new`"
    );
    assert!(
        rust.contains("pub fn new_ringing(data: CallData) -> Self"),
        "`into: Ringing` has its own typed constructor"
    );
    assert!(
        rust.contains(
            "Its constructors rest in the states a creation lands in: `Dialing`, `Ringing`."
        ),
        "the doc names every creation state"
    );
    assert!(
        !rust.contains("pub fn new_active("),
        "an entity created only into `initial` gains nothing"
    );
}
