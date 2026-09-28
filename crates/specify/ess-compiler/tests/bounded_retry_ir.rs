//! A bounded retry (ess/16, beyond10x/ess#165) lands in the IR resolved against the invoked
//! command: the attempts, and the outcomes its `final` names make final, whether the document named
//! an outcome or the error it reports. `ResolvedBinding::on_failure` answers a policy of its own,
//! so a projection that matches it cannot render a bound as an unbounded retry.

use ess_compiler::ir::{EssIr, ResolvedBinding, ResolvedFailure};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::binding::BindingName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("bounded-retry.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn binding(ir: &EssIr) -> &ResolvedBinding {
    &ir.bindings()[&BindingName::new("notify-ledger").unwrap()]
}

#[test]
fn a_final_error_resolves_to_the_outcomes_that_report_it() {
    let ir = ir(MODEL);
    let binding = binding(&ir);
    let ResolvedFailure::BoundedRetry { bound } = binding.on_failure() else {
        panic!(
            "a bounded retry is its own policy: {:?}",
            binding.on_failure()
        );
    };
    assert_eq!(bound.attempts, 3);
    let finals: Vec<&str> = bound
        .final_outcomes
        .iter()
        .map(ess_domain::command::OutcomeName::as_str)
        .collect();
    assert_eq!(finals, vec!["rejected"]);
    assert!(bound.is_final(&"rejected".parse().unwrap()));
    assert!(!bound.is_final(&"unavailable".parse().unwrap()));

    let json = serde_json::to_value(binding).unwrap();
    assert_eq!(json["failure"], "retry");
    assert_eq!(
        json["retry"],
        serde_json::json!({"attempts": 3, "final": ["rejected"]})
    );
}

#[test]
fn a_final_outcome_name_resolves_to_itself_in_declaration_order() {
    let model = MODEL.replace(
        BLOCK,
        "retry: {attempts: 2, final: [rejected, unavailable]}",
    );
    let ir = ir(&model);
    let ResolvedFailure::BoundedRetry { bound } = binding(&ir).on_failure() else {
        panic!("a bounded retry");
    };
    let finals: Vec<&str> = bound
        .final_outcomes
        .iter()
        .map(ess_domain::command::OutcomeName::as_str)
        .collect();
    assert_eq!(finals, vec!["unavailable", "rejected"]);
}

#[test]
fn a_bare_retry_is_the_unbounded_policy_and_serializes_as_it_did() {
    let ir = ir(&MODEL.replace(BLOCK, "retry"));
    let binding = binding(&ir);
    assert_eq!(binding.on_failure(), ResolvedFailure::Retry);
    assert!(binding.retry.is_none());
    let json = serde_json::to_value(binding).unwrap();
    assert!(json.get("retry").is_none(), "{json}");
}
