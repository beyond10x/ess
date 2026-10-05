//! Adversary pass 1 against the refusal-selected failure policy unit (ess/22, beyond10x/ess#269):
//! `binding/<name>/refusal-policy-changed` read as "typed, normalized before/after content"
//! (design, "Typed representation and consumers"), driven by a revision that changes no policy.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_diff::change::{BindingChange, SemanticChange};
use ess_diff::diff;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/refusal-policy.yaml");
const AT_LIMIT: &str = "      - name: at-limit
        external: the ledger holds too many open records
        error: demo.ledger.AtLimit
";
const WRONG_STATE: &str = "      - name: wrong-state
        external: the order is already recorded
        error: demo.ledger.WrongState
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

/// The invoked command's two refusals declared in the other order; every refusal keeps the policy
/// it had, so the binding's policy did not change. The command's own reordering is reported on the
/// command; the binding must not also carry an `ess-diff/14` change, which would make the whole
/// delta unreadable by every `/13` reader for a revision that touched no policy.
#[test]
fn adversary_reordering_the_invoked_commands_refusals_is_no_refusal_policy_change() {
    let swapped = MODEL.replace(
        &format!("{AT_LIMIT}{WRONG_STATE}"),
        &format!("{WRONG_STATE}{AT_LIMIT}"),
    );
    assert_ne!(
        swapped, MODEL,
        "the fixture declares the two refusals adjacently"
    );
    let delta = diff(&compiled(MODEL), &compiled(&swapped)).expect("one system");
    let binding: Vec<&BindingChange> = delta
        .changes()
        .iter()
        .filter_map(|change| match change {
            SemanticChange::Binding { changed, .. } => Some(changed),
            _ => None,
        })
        .collect();
    assert!(
        !binding
            .iter()
            .any(|changed| matches!(changed, BindingChange::RefusalPolicyChanged { .. })),
        "no policy moved, yet the binding reports a refusal-policy change: {binding:#?}"
    );
    assert!(
        delta.format.major() < 14,
        "a revision that changes no policy is lifted to {}: {:#?}",
        delta.format.major(),
        delta.changes()
    );
}
