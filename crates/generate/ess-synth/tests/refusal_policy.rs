//! A refusal-selected failure policy (ess/22, beyond10x/ess#269) in generated runtimes. The Rust
//! dispatch selects the policy per refusal and counts the attempts of each occurrence, so the
//! Rust target synthesizes it (`refusal_policy_runtime.rs` runs what it writes). The Go and web
//! targets still refuse it by name: their dispatch applies one policy to every failure and counts
//! no attempts, so emitting it would apply the fallback to every declared refusal; a named refusal
//! leaves the representation owed instead of silently wrong.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target, TargetFailureCode};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-policy.yaml");
const NAMED: &str = "bindings.notify-ledger.on_failure";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn selected_cause(text: &str) -> bool {
    text.contains(NAMED) && text.contains("per refusal")
}

#[test]
fn rust_target_synthesizes_a_refusal_selected_policy() {
    let ir = ir(MODEL);
    if let Err(failure) = synthesize_for(&ir, Target::Rust) {
        panic!("Rust synthesizes a refusal-selected policy: {failure}");
    }
    let plan = ess_synth::SynthesisPlan::of(&ir);
    if let Err(failure) = ess_synth::rust::workspace(&ir, &plan) {
        panic!("the Rust workspace entry delivers the binding: {failure}");
    }
    if let Err(failure) = ess_synth::rust::single_crate(&ir, &plan) {
        panic!("the Rust single-crate entry delivers the binding: {failure}");
    }
}

/// The targets that still refuse: Go, until its dispatch selects per refusal too.
#[test]
#[allow(clippy::single_element_loop)]
fn every_code_target_refuses_the_selected_policy_by_name() {
    let ir = ir(MODEL);
    for target in [Target::Go] {
        let failure = synthesize_for(&ir, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses a refusal-selected policy"));
        let text = format!("{failure:?}");
        assert!(
            selected_cause(&text),
            "{target:?} names the binding: {text}"
        );
        assert!(
            text.contains(&format!("{:?}", TargetFailureCode::MissingRepresentation)),
            "{text}"
        );
    }
}

#[test]
fn every_direct_workspace_entry_that_delivers_bindings_refuses_the_selected_policy_by_name() {
    let ir = ir(MODEL);
    let plan = ess_synth::SynthesisPlan::of(&ir);
    for (target, failure) in [
        ("go", ess_synth::go::workspace(&ir, &plan).err()),
        ("web", ess_synth::web::workspace(&ir, &plan).err()),
    ] {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses the selected policy"));
        let json = failure.to_canonical_json();
        assert!(selected_cause(&json), "{target} names the binding: {json}");
    }
}

#[test]
fn a_universal_policy_is_not_refused_for_being_selected() {
    let universal = MODEL.replace(
        "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
",
        "    on_failure: retry\n",
    );
    assert_ne!(universal, MODEL);
    let ir = ir(&universal);
    for target in [Target::Rust, Target::Go] {
        if let Err(failure) = synthesize_for(&ir, target) {
            let text = format!("{failure:?}");
            assert!(!selected_cause(&text), "{target:?}: {text}");
        }
    }
}

#[path = "refusal_policy/shorthand.rs"]
mod shorthand;
