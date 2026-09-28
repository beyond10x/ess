//! Set effects over filtered instances (ess/16, beyond10x/ess#167, #175) in generated seams: every
//! code target refuses `instances:` and `affects:` by name, as it refuses `Json`, rather than
//! emitting a seam that changes one row or none.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");

const ENDED: &str = "demo.desk.EndTeam.outcomes.ended.instances";
const NOTED: &str = "demo.desk.NoteTeam.outcomes.noted.instances";
const INVITED: &str = "demo.desk.Invite.outcomes.invited.affects";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("set-effects.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_each_set_effect_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses set effects"));
        let text = format!("{failure:?}");
        for named in [ENDED, NOTED, INVITED] {
            assert!(text.contains(named), "{target:?} names {named}: {text}");
        }
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

#[test]
fn every_direct_workspace_entry_refuses_each_set_effect_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    let failures = [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
        ("clap", ess_synth::clap::workspace(&ir, &plan).err()),
    ];
    for (target, failure) in failures {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses set effects"));
        for named in [ENDED, NOTED, INVITED] {
            assert!(
                failure.to_canonical_json().contains(named),
                "{target} names {named}: {}",
                failure.to_canonical_json()
            );
        }
    }
}
