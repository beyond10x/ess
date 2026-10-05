//! A binding's event-payload condition as `ess verify diff` reports it (ess/22,
//! beyond10x/ess#268): added, changed or removed, it is one `predicate-changed` change carrying
//! the condition before and after as written, `ess-diff/14` vocabulary and so always classified.
//! A pair of models without a condition keeps its earlier format and bytes.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::SemanticChange;
use ess_domain::{system::Source, RawSpecFile, Specification};

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";

fn model(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("messages.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("messages.yaml", text);
    compile_locating(&specification, &sources, &["messages.yaml"]).unwrap()
}

/// The model with `received`'s condition replaced by `condition` (empty: none), its required input
/// filled from a member every occurrence carries so that every variant compiles.
fn with_condition(condition: &str) -> String {
    assert_eq!(MODEL.matches(WHERE).count(), 1);
    MODEL.replace(WHERE, condition).replace(
        "      order_id: event.order.id\n",
        "      order_id: event.message_id\n",
    )
}

fn written(before: &str, after: &str) -> (ess_diff::EssDelta, serde_json::Value) {
    let delta = ess_diff::diff(&model(before), &model(after)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    (delta, json)
}

fn predicate_changes(delta: &ess_diff::EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .filter(|change| matches!(change, SemanticChange::Binding { .. }))
        .map(|change| change.id().to_string())
        .collect()
}

#[test]
fn adding_a_condition_is_one_classified_predicate_change() {
    let (delta, json) = written(&with_condition(""), &with_condition(WHERE));
    assert_eq!(
        predicate_changes(&delta),
        ["binding/received/predicate-changed"]
    );
    assert_eq!(delta.format.to_string(), "ess-diff/14");
    let entry = &json["changes"][0];
    let change = &entry["change"]["changed"];
    assert_eq!(change["kind"], "predicate-changed", "{json:#}");
    assert!(change.get("before").is_none(), "{json:#}");
    assert!(!change["after"].is_null(), "{json:#}");
    assert!(entry["compatibility"].is_object(), "classified: {json:#}");
    assert_eq!(entry["relation"], "changed", "{json:#}");
}

#[test]
fn changing_a_condition_carries_both_sides() {
    let (delta, json) = written(
        &with_condition(WHERE),
        &with_condition("      where: event.kind == ship\n"),
    );
    assert_eq!(
        predicate_changes(&delta),
        ["binding/received/predicate-changed"]
    );
    let entry = &json["changes"][0];
    let change = &entry["change"]["changed"];
    assert!(
        !change["before"].is_null() && !change["after"].is_null(),
        "{json:#}"
    );
    assert_ne!(change["before"], change["after"], "{json:#}");
    let described = match &delta.changes()[0] {
        SemanticChange::Binding { changed, .. } => changed.describe(),
        other => panic!("{other:?}"),
    };
    assert!(described.contains("event.kind == ship"), "{described}");
}

#[test]
fn removing_a_condition_is_one_predicate_change_with_no_after() {
    let (delta, json) = written(&with_condition(WHERE), &with_condition(""));
    assert_eq!(
        predicate_changes(&delta),
        ["binding/received/predicate-changed"]
    );
    let entry = &json["changes"][0];
    let change = &entry["change"]["changed"];
    assert!(!change["before"].is_null(), "{json:#}");
    assert!(change.get("after").is_none(), "{json:#}");
}

#[test]
fn the_same_condition_is_no_change() {
    let (delta, _) = written(&with_condition(WHERE), &with_condition(WHERE));
    assert_eq!(delta.changes().len(), 0, "{:?}", delta.changes());
}

#[test]
fn a_predicate_change_reads_back_and_an_older_reader_refuses_it() {
    let (delta, json) = written(&with_condition(""), &with_condition(WHERE));
    let text = delta.to_canonical_json();
    let raw: ess_diff::RawEssDelta = serde_json::from_str(&text).unwrap();
    let read = ess_diff::EssDelta::try_from(raw).expect("an ess-diff/14 reader reads it back");
    assert_eq!(read.to_canonical_json(), text);
    let older = text.replace("\"ess-diff/14\"", "\"ess-diff/13\"");
    assert_ne!(older, text, "{json:#}");
    let raw: ess_diff::RawEssDelta = serde_json::from_str(&older).unwrap();
    let refused = ess_diff::EssDelta::try_from(raw).expect_err("an ess-diff/13 reader refuses");
    assert!(
        format!("{refused:?}").contains("UnsupportedFormatVersion"),
        "{refused:?}"
    );
    assert!(delta
        .to_canonical_json_for(ess_diff::DeltaFormat::parse("ess-diff/13").unwrap())
        .is_err());
}

/// The same change written without the condition anywhere: no `/14`, no classification.
#[test]
fn a_model_without_a_condition_keeps_its_delta_format_and_bytes() {
    let before = with_condition("");
    let after = before.replace(
        "  - id: received\n    when:\n      event: demo.messages.MessageReceived\n    invoke: {command: demo.messages.MessageEvent}\n    mapping:\n      message_id: event.message_id\n      order_id: event.message_id\n    delivery: at_least_once\n    on_failure: retry\n",
        "  - id: received\n    when:\n      event: demo.messages.MessageReceived\n    invoke: {command: demo.messages.MessageEvent}\n    mapping:\n      message_id: event.message_id\n      order_id: event.message_id\n    delivery: at_least_once\n    on_failure: drop\n",
    );
    assert_ne!(before, after);
    let (delta, json) = written(&before, &after);
    assert_eq!(
        predicate_changes(&delta),
        ["binding/received/failure-changed"]
    );
    assert!(delta.format.major() < 14, "{json:#}");
    assert!(
        json["changes"][0].get("compatibility").is_none(),
        "{json:#}"
    );
}
