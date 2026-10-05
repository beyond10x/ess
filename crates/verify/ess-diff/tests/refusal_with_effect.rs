//! Adding or removing a refusal's compensating change (`compensates: true`, ess/22,
//! beyond10x/ess#197, `docs/design/refusal-with-effect.md`) is `outcome-compensates-changed`,
//! `ess-diff/14` vocabulary (unreleased, beside `refusal-policy-changed`), classified breaking for callers and readers and compatible for
//! history: a caller retrying after the refusal, and a reader of the row, now meet a different
//! state, and nothing stored changes shape.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{Compatibility, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/refusal-with-effect.yaml");

/// A second command taking `reset`, so the model without the compensating move still gives the
/// transition a cause.
const CLOSE: &str = "  - name: shop.order.CloseOrder\n    input:\n      - {name: order_id, type: shop.order.OrderId}\n    outcomes:\n      - name: reset\n        moves: shop.order.Order.reset\n        instance: order_id\n        emits: [shop.order.OrderJoined]\n        payload:\n          shop.order.OrderJoined: {order_id: input.order_id}\nevents:\n";

const EFFECT: &str = "        compensates: true\n        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn compensating() -> String {
    replaced(MODEL, "events:\n", CLOSE)
}

fn plain() -> String {
    replaced(&compensating(), EFFECT, "")
}

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("order.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

const ID: &str = "command/shop.order.JoinOrder/outcome-compensates-changed/failed";

fn assert_compensation(delta: &EssDelta, before: bool, after: bool) {
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/14", "{json}");
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == ID)
        .unwrap_or_else(|| panic!("`{ID}` in {json}"));
    assert_eq!(change.relation(), SemanticRelation::Changed);
    assert_eq!(change.minimum_format(), 14);
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    assert_eq!(compatibility.callers(), Compatibility::Breaking, "{json}");
    assert_eq!(compatibility.readers(), Compatibility::Breaking, "{json}");
    assert_eq!(compatibility.history(), Compatibility::Compatible, "{json}");
    assert_eq!(compatibility.verdict(), Compatibility::Breaking);
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == ID)
        .map(|entry| &entry["change"]["changed"])
        .unwrap();
    assert_eq!(changed["before"], before, "{json}");
    assert_eq!(changed["after"], after, "{json}");
    // The effect moves with the marker, and is reported as the subject it always was.
    assert!(
        delta.changes().iter().any(|change| change.id().to_string()
            == "command/shop.order.JoinOrder/outcome-subject-changed/failed"),
        "{json}"
    );
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), *delta, "it reads back");
    assert!(
        delta
            .to_canonical_json_for("ess-diff/13".parse().unwrap())
            .is_err(),
        "ess-diff/13 refuses `{ID}`"
    );
    let text = ess_diff::render::text(delta);
    assert!(text.contains(ID), "{text}");
}

#[test]
fn adding_the_compensating_change_is_breaking_for_callers_and_readers() {
    let delta = ess_diff::classified(&ir(&plain()), &ir(&compensating())).unwrap();
    assert_compensation(&delta, false, true);
    // The default delta carries it too, classified, since no earlier format can.
    let default = ess_diff::diff(&ir(&plain()), &ir(&compensating())).unwrap();
    assert_compensation(&default, false, true);
}

#[test]
fn removing_it_is_the_same_kind_the_other_way() {
    let delta = ess_diff::classified(&ir(&compensating()), &ir(&plain())).unwrap();
    assert_compensation(&delta, true, false);
}

#[test]
fn a_model_without_the_marker_diffs_as_before() {
    // Another error on the plain refusal: nothing about compensation moves, so the delta keeps
    // the format it had.
    let other = replaced(
        &plain(),
        "        error: shop.order.Refused\n",
        "        error: shop.order.Closed\n",
    );
    let delta = ess_diff::classified(&ir(&plain()), &ir(&other)).unwrap();
    assert_eq!(delta.format.to_string(), "ess-diff/14");
    assert!(!delta.to_canonical_json().contains("compensates"));
}
