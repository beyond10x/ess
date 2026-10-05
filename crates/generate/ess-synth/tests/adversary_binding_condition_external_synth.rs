//! Adversary pass 1 against #268 slice 2: the design now says generated targets "do not deliver
//! external events at all (the delivery is an obligation)". Synthesis of a model with an external
//! (delivery-context) binding must then answer — with a workspace that owes the delivery, or with
//! a named refusal — and never abort the process.
//!
//! The model is `ess-conformance`'s `delivery-context.yaml` at ess/22, with the condition the
//! unit's own `binding_condition_external.rs` gives its binding, and as written (no condition).

use std::panic::{catch_unwind, AssertUnwindSafe};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

const INBOX: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/delivery-context.yaml");

fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` once in the model");
    text.replacen(from, to, 1)
}

/// `binding_condition_external.rs`'s `conditioned`, verbatim in effect.
fn conditioned() -> String {
    let text = rewrite(INBOX, "format: ess/18\n", "format: ess/22\n");
    let text = rewrite(
        &text,
        "types:\n",
        "types:\n  - name: demo.inbox.Kind\n    kind: enum\n    variants: [note, ship]\n  - name: demo.inbox.Ref\n    kind: struct\n    fields:\n      - {name: id, type: String}\n",
    );
    let text = rewrite(
        &text,
        "      - {name: from, type: String}\n",
        "      - {name: from, type: String}\n      - {name: kind, type: demo.inbox.Kind}\n      - {name: order, type: Optional<demo.inbox.Ref>}\n",
    );
    rewrite(
        &text,
        "      context_authority: account-messages\n",
        "      context_authority: account-messages\n      where: [defined(event.order), event.kind == ship]\n",
    )
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("inbox.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn aborted(ir: &EssIr) -> Vec<String> {
    [
        ("rust", Target::Rust),
        ("go", Target::Go),
        ("web", Target::Web),
    ]
    .into_iter()
    .filter_map(|(name, target)| {
        match catch_unwind(AssertUnwindSafe(|| synthesize_for(ir, target).map(|_| ()))) {
            Ok(Ok(())) => None,
            Ok(Err(failure)) => {
                println!("{name}: refused: {failure:?}");
                None
            }
            Err(panic) => Some(format!(
                "{name}: {}",
                panic
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
                    .unwrap_or_default()
                    .lines()
                    .next()
                    .unwrap_or_default()
            )),
        }
    })
    .collect()
}

#[test]
fn a_conditioned_external_binding_synthesizes_or_is_refused_never_aborts() {
    assert_eq!(aborted(&ir_of(&conditioned())), Vec::<String>::new());
}

#[test]
fn an_external_binding_synthesizes_or_is_refused_never_aborts() {
    assert_eq!(aborted(&ir_of(INBOX)), Vec::<String>::new());
}
