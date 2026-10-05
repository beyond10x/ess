//! A comparison tagged to compare instants (`docs/design/expression-family-source22.md`, final
//! review decision 2), and the RFC 3339 instant grammar every lane shares (decision 14).
//!
//! The vectors are `vectors/rfc3339-instants.json`, which the Go fixture
//! `ess-conformance/tests/fixtures/instant-comparison.go` and `ess-conformance/src/ts/predicate.test.ts`
//! answer too.

use std::cmp::Ordering;

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};
use ess_primitives::time::Rfc3339Instant;

const VECTORS: &str = include_str!("vectors/rfc3339-instants.json");

fn vectors() -> serde_json::Value {
    serde_json::from_str(VECTORS).expect("json")
}

fn flatten(store: &mut FactStore, path: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, nested) in fields {
                let at = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                flatten(store, &at, nested);
            }
        }
        serde_json::Value::String(text) => store.set_path(path, FactValue::text(text.clone())),
        serde_json::Value::Number(number) => store.set_path(
            path,
            FactValue::number(number.as_f64().expect("finite")).expect("a number"),
        ),
        serde_json::Value::Bool(value) => store.set_path(path, FactValue::Bool(*value)),
        serde_json::Value::Array(_) | serde_json::Value::Null => {}
    }
}

fn tagged(left: &str, op: CompareOp, right: &str) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(left.parse::<FactPath>().expect("a path")),
        op,
        right: Operand::Fact(right.parse::<FactPath>().expect("a path")),
        kind: CompareKind::Instant,
    }
}

#[test]
fn the_rust_grammar_answers_the_shared_parse_and_order_vectors() {
    let vectors = vectors();
    for vector in vectors["parse"].as_array().expect("a list") {
        let text = vector["text"].as_str().expect("text");
        assert_eq!(
            Rfc3339Instant::parse_rfc3339(text).is_some(),
            vector["valid"].as_bool().expect("a flag"),
            "{text}"
        );
    }
    for vector in vectors["order"].as_array().expect("a list") {
        let left = Rfc3339Instant::parse_rfc3339(vector["left"].as_str().unwrap()).expect("left");
        let right =
            Rfc3339Instant::parse_rfc3339(vector["right"].as_str().unwrap()).expect("right");
        let want = match vector["ordering"].as_str().unwrap() {
            "less" => Ordering::Less,
            "equal" => Ordering::Equal,
            _ => Ordering::Greater,
        };
        assert_eq!(left.cmp(&right), want, "{vector}");
    }
}

#[test]
fn a1_timestamp_sibling_instant_order_in_the_rust_evaluator() {
    let vectors = vectors();
    for vector in vectors["tagged"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate: Predicate = serde_json::from_value(vector["predicate"].clone())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let mut store = FactStore::new();
        flatten(&mut store, "", &vector["row"]);
        assert_eq!(
            predicate.evaluate(&store).as_str(),
            vector["truth"].as_str().expect("a truth"),
            "{name}"
        );
    }
    for vector in vectors["refused"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        assert!(
            serde_json::from_value::<Predicate>(vector["predicate"].clone()).is_err(),
            "{name} is refused"
        );
    }
}

#[test]
fn the_tagged_form_is_closed_canonical_and_reads_back() {
    let predicate = tagged("valid_until", CompareOp::Gt, "valid_from");
    let written = serde_json::to_string(&predicate).expect("serialises");
    assert_eq!(
        written,
        r#"{"compare":{"as":"timestamp","left":"valid_until","op":"gt","right":{"fact":"valid_from"}}}"#
    );
    assert_eq!(
        serde_json::from_str::<Predicate>(&written).expect("reads back"),
        predicate
    );
    assert_eq!(
        predicate.to_string(),
        "valid_until > {fact: valid_from} as timestamp"
    );
    assert!(predicate.compares_instants());
    assert!(
        !predicate.reads_root_fact_operand(),
        "the tagged form is its own vocabulary"
    );
    // A dotted pair keeps the explicit operand too: the form never depends on what a root is.
    let dotted = tagged("window.end", CompareOp::Ge, "window.start");
    let written = serde_json::to_string(&dotted).expect("serialises");
    assert!(
        written.contains(r#""right":{"fact":"window.start"}"#),
        "{written}"
    );
    assert_eq!(serde_json::from_str::<Predicate>(&written).unwrap(), dotted);
    // The untagged comparison of the same facts keeps its old bytes.
    let untagged = Predicate::compare(
        Operand::Fact("window.end".parse().unwrap()),
        CompareOp::Ge,
        Operand::Fact("window.start".parse().unwrap()),
    );
    assert_eq!(
        serde_json::to_string(&untagged).unwrap(),
        r#""window.end >= window.start""#
    );
    assert!(!untagged.compares_instants());
}
