//! `{related: …}` through an Optional reference and across two references (ess/22,
//! beyond10x/ess#285) in the IR: the hops a chained read follows, the declared type of every
//! reference, and the value's type, `Optional<…>` where a reference may be absent.

use ess_compiler::ir::{EssIr, ResolvedPayloadField, ResolvedPayloadValue, ResolvedRelatedVia};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const CHAINED: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-values-chained.yaml");
const TWO_HOPS: &str = "{related: {via: [objective_id, initiative_id], field: outcome_id}}";
const ONE_OPTIONAL_HOP: &str = "{related: {via: initiative_id, field: outcome_id}}";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("costs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// `Book`'s `sets: outcome_id` and the `CostBooked.outcome_id` it publishes.
fn copied(model: &EssIr) -> [ResolvedPayloadField; 2] {
    let book = &model.commands()[&"demo.costs.Book".parse().unwrap()];
    let outcome = &book.outcomes[0];
    let set = outcome
        .sets
        .iter()
        .find(|set| set.target == "outcome_id")
        .expect("outcome_id is set")
        .clone();
    let published = outcome.payload[0]
        .fields
        .iter()
        .find(|field| field.target == "outcome_id")
        .expect("outcome_id is published")
        .clone();
    [set, published]
}

#[test]
fn issue_285_a_chained_read_names_each_hop_and_the_last_entity() {
    let model = ir(CHAINED);
    for copied in copied(&model) {
        let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity,
            field,
            type_ref,
        } = &copied.value
        else {
            panic!("a related source: {:?}", copied.value)
        };
        let ResolvedRelatedVia::Subject {
            field: first,
            type_ref: first_type,
        } = via
        else {
            panic!("the entry's own field: {via:?}")
        };
        assert_eq!(first, "objective_id");
        assert_eq!(first_type.written().to_string(), "demo.costs.ObjectiveId");
        assert_eq!(through.len(), 1, "{through:?}");
        assert_eq!(through[0].entity.name().to_string(), "demo.costs.Objective");
        assert_eq!(through[0].field, "initiative_id");
        assert_eq!(
            through[0].type_ref.written().to_string(),
            "Optional<demo.costs.InitiativeId>"
        );
        assert_eq!(entity.name().to_string(), "demo.costs.Initiative");
        assert_eq!(field, "outcome_id");
        // The initiative may be absent, so the value may be.
        assert_eq!(
            type_ref.written().to_string(),
            "Optional<demo.costs.OutcomeId>"
        );
        assert!(
            copied.value.describe().contains("absent"),
            "{}",
            copied.value.describe()
        );
    }
    let canonical = model.to_canonical_json();
    assert!(canonical.contains("\"through\""), "{canonical}");
}

#[test]
fn issue_285_a_one_hop_optional_read_keeps_its_declared_reference_type() {
    let model = ir(&CHAINED.replace(TWO_HOPS, ONE_OPTIONAL_HOP));
    for copied in copied(&model) {
        let ResolvedPayloadValue::RelatedField {
            via,
            through,
            entity,
            type_ref,
            ..
        } = &copied.value
        else {
            panic!("a related source: {:?}", copied.value)
        };
        assert_eq!(via.field(), "initiative_id");
        assert_eq!(
            via.type_ref().written().to_string(),
            "Optional<demo.costs.InitiativeId>"
        );
        assert_eq!(through.len(), 0, "{through:?}");
        assert_eq!(entity.name().to_string(), "demo.costs.Initiative");
        assert_eq!(
            type_ref.written().to_string(),
            "Optional<demo.costs.OutcomeId>"
        );
    }
    let canonical = model.to_canonical_json();
    assert!(!canonical.contains("\"through\""), "{canonical}");
}

#[test]
fn issue_285_a_chain_of_required_references_reads_the_field_at_its_own_type() {
    let model = ir(&CHAINED
        .replace(
            "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n    lifecycle: {initial: Active",
            "      - {name: initiative_id, type: demo.costs.InitiativeId}\n    lifecycle: {initial: Active",
        )
        .replace(
            "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}",
            "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: demo.costs.InitiativeId}",
        ));
    for copied in copied(&model) {
        let ResolvedPayloadValue::RelatedField { type_ref, .. } = &copied.value else {
            panic!("a related source: {:?}", copied.value)
        };
        assert_eq!(type_ref.written().to_string(), "demo.costs.OutcomeId");
        assert!(
            !copied.value.describe().contains("absent"),
            "{}",
            copied.value.describe()
        );
    }
}
