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
const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");
const OPTIONAL_RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-optional.yaml");

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

#[test]
fn issue_304_an_optional_input_via_keeps_its_declared_type_in_the_ir() {
    let model = ir(OPTIONAL_RELEASE);
    let command = &model.commands()[&"demo.release.PublishRelease".parse().unwrap()];
    for outcome in &command.outcomes[..2] {
        let ResolvedCondition::Related { via, entity, .. } = &outcome.condition else {
            panic!("a related guard: {:?}", outcome.condition)
        };
        let ResolvedRelatedVia::Input { field, type_ref } = via else {
            panic!("the Optional carrier remains an input via: {via:?}")
        };
        assert_eq!(field, "candidate");
        assert!(type_ref.is_optional(), "the declared wrapper is retained: {via:?}");
        assert_eq!(
            type_ref.required().written().to_string(),
            "demo.release.CandidateId"
        );
        assert_eq!(entity.name().to_string(), "demo.release.Candidate");
    }
}

#[test]
fn issue_304_ess_20_related_fixtures_compile_to_identical_ir() {
    let model = ir(RELEASE);
    let command = &model.commands()[&"demo.release.PublishRelease".parse().unwrap()];
    for outcome in &command.outcomes[..2] {
        let ResolvedCondition::Related { via, .. } = &outcome.condition else {
            panic!("the legacy related condition remains present: {:?}", outcome.condition)
        };
        let ResolvedRelatedVia::Input { field, type_ref } = via else {
            panic!("the legacy input via remains an input: {via:?}")
        };
        assert_eq!(field, "candidate");
        assert_eq!(type_ref.written().to_string(), "demo.release.CandidateId");
        assert!(!type_ref.is_optional(), "legacy required input changed: {via:?}");
    }
    let canonical = model.to_canonical_json();
    assert!(canonical.contains(r#""field": "candidate""#), "{canonical}");
    assert!(
        !canonical.contains("Optional<demo.release.CandidateId>"),
        "the old required-input model's bytes do not gain the new wrapper: {canonical}"
    );
}
