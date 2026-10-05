//! The compiled IR carries a row-set guard as its own condition and a filtered read as its own
//! payload value (`docs/design/filtered-related-reads.md`, "Compatibility and targets"):
//! `ResolvedCondition::RelatedSet` (`kind: related_set`) with the resolved entity, the typed
//! `where`, the tagged test and the input guard only where one is written; and
//! `ResolvedPayloadValue::RelatedSelection` (`kind: related_selection`) with the same selector,
//! the field read and the type it is read at. A model without either carries neither.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");
const UNIQUE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/unique-within-scope.yaml");

/// The canonical IR with every whitespace character removed.
fn ir(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("rows.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

const SELECTION: &str = r#""selection":{"entity":"demo.jobs.Attempt","where":{"all":["worker_id==input.worker_id","batch_id==input.batch_id"]}}"#;

#[test]
fn row_set_guard_is_a_related_set_condition_with_a_tagged_test() {
    let compact = ir(READS);
    for test in [
        r#""test":{"count":{"gt":1}}"#,
        r#""test":{"count":{"eq":0}}"#,
        r#""test":{"forall":"delay<=input.limit"}"#,
    ] {
        let expected = format!(r#"{{"kind":"related_set",{SELECTION},{test}}}"#);
        assert!(compact.contains(&expected), "{expected}\n{compact}");
    }
    let unique = ir(UNIQUE);
    assert!(
        unique.contains(r#""kind":"related_set""#) && unique.contains(r#""test":{"exists":true}"#),
        "{unique}"
    );
    // The identity-addressed guard keeps its own condition and bytes.
    assert!(!compact.contains(r#""kind":"related""#), "{compact}");
}

#[test]
fn filtered_read_is_a_related_selection_value() {
    let compact = ir(READS);
    let delay = format!(
        r#"{{"kind":"related_selection",{SELECTION},"field":"delay","type_ref":{{"kind":"primitive","name":"integer"}}}}"#
    );
    assert!(compact.contains(&delay), "{delay}\n{compact}");
    assert!(
        compact.contains(r#""field":"note","type_ref":{"kind":"optional""#),
        "{compact}"
    );
    // Never the `related_field` value, whose `via` is an identity.
    assert!(!compact.contains("related_field"), "{compact}");
}

#[test]
fn a_model_without_row_sets_carries_neither_kind() {
    let leases = ir(include_str!(
        "../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml"
    ));
    assert!(!leases.contains("related_set"), "{leases}");
    assert!(!leases.contains("related_selection"), "{leases}");
}
