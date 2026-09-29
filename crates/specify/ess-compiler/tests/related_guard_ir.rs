//! `when_related:` (ess/18, beyond10x/ess#211) lands in the IR as its own condition, naming the input
//! field it reads, the entity whose identity that field carries, and the test.
//!
//! A model without the key keeps its IR bytes: the variant is new and nothing else moves.

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedRelatedTest, ResolvedRelatedVia};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::TestStrategy;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

#[test]
fn issue_211_both_tests_compile_to_the_related_condition() {
    let ir = ir(SIGN_IN);
    let command = &ir.commands()[&"demo.signin.InitiateSignIn".parse().unwrap()];
    for (index, expected) in [
        ResolvedRelatedTest::Absent,
        ResolvedRelatedTest::Holds {
            predicate: "redirect_client != input.client".parse().unwrap(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        let outcome = &command.outcomes[index];
        let ResolvedCondition::Related {
            via,
            entity,
            test,
            input,
        } = &outcome.condition
        else {
            panic!("a related guard: {:?}", outcome.condition)
        };
        assert!(
            matches!(via, ResolvedRelatedVia::Input { field, .. } if field == "tenant"),
            "{via:?}"
        );
        assert_eq!(entity.name().to_string(), "demo.signin.Configuration");
        assert_eq!(*test, expected);
        assert!(input.is_none());
        assert_eq!(outcome.test_strategy, TestStrategy::ArrangeRelatedRow);
        assert!(outcome.subject.is_none(), "the refusal names no subject");
    }
    assert_eq!(command.outcomes[2].condition, ResolvedCondition::Otherwise);
}

#[test]
fn issue_211_the_canonical_ir_names_the_condition_and_omits_an_absent_input_guard() {
    let json = ir(SIGN_IN).to_canonical_json();
    assert!(json.contains(r#""kind": "related""#), "{json}");
    assert!(json.contains(r#""test": "absent""#), "{json}");
    assert!(json.contains(r#""holds": {"#), "{json}");
    assert!(json.contains("arrange_related_row"), "{json}");
    assert!(
        !json.contains(r#""input": null"#),
        "an absent input guard is omitted, not written as null"
    );
}
