//! A bounded retry (ess/16, beyond10x/ess#165) is compared as part of the failure policy: a
//! changed attempt count or final set is `FailureChanged`, and the words name the bound.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_diff::change::{BindingChange, SemanticChange};
use ess_diff::diff;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn failure_change(after: &str) -> BindingChange {
    let revised = MODEL.replace(BLOCK, after);
    assert_ne!(revised, MODEL);
    let changes = diff(&compiled(MODEL), &compiled(&revised))
        .expect("one system")
        .changes()
        .to_vec();
    assert_eq!(changes.len(), 1, "{changes:?}");
    let SemanticChange::Binding { changed, .. } = &changes[0] else {
        panic!("a binding change: {changes:?}");
    };
    changed.clone()
}

#[test]
fn a_changed_attempt_count_is_a_failure_change_naming_the_bound() {
    assert_eq!(
        failure_change("retry: {attempts: 4, final: [demo.ledger.Unknown]}"),
        BindingChange::FailureChanged {
            before: "retry, 3 attempts, final `rejected`".to_owned(),
            after: "retry, 4 attempts, final `rejected`".to_owned(),
        }
    );
}

#[test]
fn dropping_the_bound_is_a_failure_change_even_though_the_word_stays_retry() {
    assert_eq!(
        failure_change("retry"),
        BindingChange::FailureChanged {
            before: "retry, 3 attempts, final `rejected`".to_owned(),
            after: "retry".to_owned(),
        }
    );
}
