//! A binding's event-payload condition (ess/22, beyond10x/ess#268) in generated runtimes: every
//! code target that delivers bindings refuses it by name. Their dispatch invokes for every
//! occurrence and has no condition evaluator, so emitting it would invoke where the binding skips
//! and unwrap an Optional member the condition proves present without checking it.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/binding-condition.yaml");
const NAMED: &str = "bindings.received.when.where";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("binding-condition.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_code_target_refuses_the_condition_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Rust, Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses a conditioned binding"));
        let text = format!("{failure:?}");
        assert!(text.contains(NAMED), "{target:?} names the binding: {text}");
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

#[test]
fn every_direct_workspace_entry_that_delivers_bindings_refuses_the_condition_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    for (target, failure) in [
        ("rust", ess_synth::rust::workspace(&ir, &plan).err()),
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
    ] {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses a conditioned binding"));
        assert!(
            failure.to_canonical_json().contains(NAMED),
            "{target} names the binding: {}",
            failure.to_canonical_json()
        );
    }
}

#[test]
fn the_same_bindings_without_a_condition_are_not_refused_for_one() {
    let plain = MODEL
        .replace(
            "      where: [defined(event.order), event.kind == ship]\n",
            "",
        )
        .replace(
            "      order_id: event.order.id\n",
            "      order_id: event.message_id\n",
        );
    let ir = ir(&plain);
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(!text.contains("when.where"), "{target:?}: {text}");
        }
    }
}
