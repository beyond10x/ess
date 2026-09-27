//! An `input_absent:` branch (ess/16, beyond10x/ess#170) in generated seams: every code target
//! refuses it by name, as it refuses `Json`, rather than decoding a request with no body as `{}`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/absent-input.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("absent-input.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_the_branch_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses an input_absent branch"));
        let text = format!("{failure:?}");
        assert!(
            text.contains("demo.notes.SubmitNote.outcomes.body-missing.input_absent"),
            "{target:?} names the branch: {text}"
        );
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

#[test]
fn the_model_without_the_marker_synthesizes() {
    let model = MODEL.replace(
        "      - name: body-missing\n        input_absent: true\n        error: demo.notes.BodyMissing\n",
        "",
    );
    assert_ne!(model, MODEL);
    let ir = ir(&model);
    for target in [Target::Rust, Target::Go] {
        assert!(synthesize_for(&ir, target).is_ok(), "{target:?}");
    }
}

/// Each target's own `workspace` refuses the branch too, as it refuses `Json`, so a caller that
/// skips `synthesize_for` gets the same failure.
#[test]
fn every_direct_workspace_entry_refuses_the_branch_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    let named = "demo.notes.SubmitNote.outcomes.body-missing.input_absent";
    let failures = [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
        ("clap", ess_synth::clap::workspace(&ir, &plan).err()),
    ];
    for (target, failure) in failures {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses an input_absent branch"));
        assert!(
            failure.to_canonical_json().contains(named),
            "{target} names the branch: {}",
            failure.to_canonical_json()
        );
    }
}
