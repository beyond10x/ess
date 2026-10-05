//! A command guarded by several related rows (ess/22, beyond10x/ess#283) stays an obligation: the
//! generated `when_related:` behaviour (beyond10x/ess#319) reads one row per command. Its contract
//! states the order the interpreter applies across rows — missing rows in the declaration order of
//! their `exists: false` branches before anything else, and the first declared present-related
//! predicate refusal before every accepting branch, with or without `wrong_state:`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, SynthesisPlan, Target};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");
const START: &str = "demo.run.StartRun";

fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

/// The reason and contract `StartRun` is owed with.
fn owed(ir: &EssIr) -> (String, String) {
    let plan = SynthesisPlan::of(ir);
    match plan.disposition_of(CapabilityKind::CommandBehavior, START) {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            (obligation.reason.describes(), obligation.contract.clone())
        }
        other => panic!("a command reading several related rows stays owed: {other:?}"),
    }
}

#[test]
fn issue_283_a_command_reading_several_rows_stays_owed_naming_why() {
    let ir = compile_text(MODEL);
    let (why, _) = owed(&ir);
    assert!(why.contains("several related rows"), "{why}");
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("the model synthesizes");
        assert!(
            matches!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::CommandBehavior, START),
                Some(SynthesisDisposition::Obligation(_))
            ),
            "{target:?}"
        );
        // A command reading one row beside it is still generated.
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.run.InstallSwitch"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}

#[test]
fn issue_283_the_contract_states_the_order_across_rows_without_wrong_state() {
    let (_, contract) = owed(&compile_text(MODEL));
    assert!(
        contract.contains(
            "read the related rows in the declaration order of their `exists: false` branches, \
             and the first missing one answers"
        ),
        "{contract}"
    );
    assert!(
        contract.contains(
            "then choose the first declared present `when_related:` predicate refusal whose \
             predicate and optional input guard hold, across rows; then accepting"
        ),
        "the refusal precedes acceptance though no `wrong_state:` is declared: {contract}"
    );
}
