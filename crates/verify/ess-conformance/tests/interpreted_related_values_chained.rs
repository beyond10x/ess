//! The native interpreter reads `{related: …}` through an Optional reference and across two
//! references (ess/22, beyond10x/ess#285): an absent reference yields an absent value, a present
//! one that names no row is the missing-row failure it always was, and a chain reads the row the
//! second reference names, not any other.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const CHAINED: &str = include_str!("fixtures/related-values-chained.yaml");
const TWO_HOPS: &str = "{related: {via: [objective_id, initiative_id], field: outcome_id}}";
const ONE_OPTIONAL_HOP: &str = "{related: {via: initiative_id, field: outcome_id}}";

fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("costs.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn text(value: &str) -> Node {
    Node::Text(value.into())
}
fn fields(pairs: &[(&str, Node)]) -> BTreeMap<String, Node> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone()))
        .collect()
}
fn correlation() -> CorrelationId {
    CorrelationId::new("related-chained").unwrap()
}
fn target(source: &str) -> Interpreted {
    let target = Interpreted::for_model(model(source));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.costs/authored/read".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
}
fn setup(target: &Interpreted, entity: &str, identity: &str, values: BTreeMap<String, Node>) {
    let state = if entity == "CostEntry" {
        "Booked"
    } else {
        "Active"
    };
    target
        .establish_entity(EntitySetupRequest {
            entity: format!("demo.costs.{entity}").parse().unwrap(),
            identity: text(identity),
            fields: values,
            state: state.parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();
}
fn book(
    target: &Interpreted,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: "demo.costs.Book".parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: correlation(),
    })
}
fn entries(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.costs.CostEntries".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
fn absent(value: Option<&Node>) -> bool {
    matches!(value, None | Some(Node::Null))
}

/// Two initiatives with distinct outcomes, and an objective naming the second; a decoy objective
/// naming the first.
fn world(target: &Interpreted) {
    setup(
        target,
        "Initiative",
        "initiative-a",
        fields(&[("outcome_id", text("outcome-a"))]),
    );
    setup(
        target,
        "Initiative",
        "initiative-b",
        fields(&[("outcome_id", text("outcome-b"))]),
    );
    setup(
        target,
        "Objective",
        "objective-decoy",
        fields(&[("initiative_id", text("initiative-a"))]),
    );
    setup(
        target,
        "Objective",
        "objective-b",
        fields(&[("initiative_id", text("initiative-b"))]),
    );
    setup(target, "Objective", "objective-none", BTreeMap::new());
    setup(
        target,
        "Objective",
        "objective-ghost",
        fields(&[("initiative_id", text("initiative-ghost"))]),
    );
}

#[test]
fn issue_285_a_chained_read_follows_both_references() {
    let target = target(CHAINED);
    world(&target);
    // The entry's own Optional `initiative_id` names the other initiative: a reader taking one hop
    // through it would publish `outcome-a`.
    let result = book(
        &target,
        fields(&[
            ("objective_id", text("objective-b")),
            ("initiative_id", text("initiative-a")),
            ("cents", Node::Number(5_i64.into())),
        ]),
    )
    .unwrap();
    assert_eq!(
        result.direct_events[0].payload.get("outcome_id"),
        Some(&text("outcome-b"))
    );
    assert_eq!(
        entries(&target)[0].get("outcome_id"),
        Some(&text("outcome-b"))
    );
}

#[test]
fn issue_285_an_absent_second_reference_yields_an_absent_value() {
    let target = target(CHAINED);
    world(&target);
    let result = book(
        &target,
        fields(&[
            ("objective_id", text("objective-none")),
            ("initiative_id", text("initiative-a")),
            ("cents", Node::Number(5_i64.into())),
        ]),
    )
    .unwrap();
    assert!(
        absent(result.direct_events[0].payload.get("outcome_id")),
        "{:?}",
        result.direct_events[0].payload
    );
    let rows = entries(&target);
    assert!(absent(rows[0].get("outcome_id")), "{rows:?}");
}

#[test]
fn issue_285_a_present_reference_naming_no_row_is_a_missing_row_not_an_absent_value() {
    let target = target(CHAINED);
    world(&target);
    for objective in ["objective-ghost", "objective-unknown"] {
        let error = book(
            &target,
            fields(&[
                ("objective_id", text(objective)),
                ("cents", Node::Number(5_i64.into())),
            ]),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("related"),
            "{objective}: {error}"
        );
    }
    assert_eq!(entries(&target).len(), 0, "a failed read writes nothing");
}

#[test]
fn issue_285_an_absent_optional_reference_yields_an_absent_value_in_one_hop() {
    let target = target(&CHAINED.replace(TWO_HOPS, ONE_OPTIONAL_HOP));
    world(&target);
    let absent_result = book(
        &target,
        fields(&[
            ("objective_id", text("objective-b")),
            ("cents", Node::Number(5_i64.into())),
        ]),
    )
    .unwrap();
    assert!(
        absent(absent_result.direct_events[0].payload.get("outcome_id")),
        "{:?}",
        absent_result.direct_events[0].payload
    );
    let present = book(
        &target,
        fields(&[
            ("objective_id", text("objective-b")),
            ("initiative_id", text("initiative-a")),
            ("cents", Node::Number(5_i64.into())),
        ]),
    )
    .unwrap();
    assert_eq!(
        present.direct_events[0].payload.get("outcome_id"),
        Some(&text("outcome-a"))
    );
    let missing = book(
        &target,
        fields(&[
            ("objective_id", text("objective-b")),
            ("initiative_id", text("initiative-ghost")),
            ("cents", Node::Number(5_i64.into())),
        ]),
    )
    .unwrap_err();
    assert!(missing.to_string().contains("related"), "{missing}");
}
