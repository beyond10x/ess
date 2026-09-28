//! A bounded retry (ess/16, beyond10x/ess#165) in generated runtimes: every code target that
//! delivers bindings refuses it by name. Their retry holds the event for the next pump and counts
//! no attempts, so emitting it would retry forever where the specification says three times.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";
const NAMED: &str = "bindings.notify-ledger.on_failure.retry";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_the_bound_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses a bounded retry"));
        let text = format!("{failure:?}");
        assert!(text.contains(NAMED), "{target:?} names the binding: {text}");
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

#[test]
fn the_model_with_an_unbounded_retry_synthesizes() {
    let ir = ir(&MODEL.replace(BLOCK, "retry"));
    for target in [Target::Rust, Target::Go] {
        assert!(synthesize_for(&ir, target).is_ok(), "{target:?}");
    }
}

/// Each target's own `workspace` refuses the bound too, so a caller that skips `synthesize_for`
/// gets the same failure.
#[test]
fn every_direct_workspace_entry_that_delivers_bindings_refuses_the_bound_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    let failures = [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
    ];
    for (target, failure) in failures {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses a bounded retry"));
        assert!(
            failure.to_canonical_json().contains(NAMED),
            "{target} names the binding: {}",
            failure.to_canonical_json()
        );
    }
}
