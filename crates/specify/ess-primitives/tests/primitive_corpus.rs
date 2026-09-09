//! The normative primitive corpus, read from Rust.
//!
//! `tests/vectors/primitive-semantics.json` is the one document
//! `docs/design/review-primitive-semantics.md` names, and the same document
//! `crates/verify/ess-conformance/tests/primitive_corpus.rs` hands to the Go conformance runtime
//! and to the browser adapter. A number is decided here; an admission grammar is decided here and
//! answered identically in three languages there.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use ess_primitives::facts::{FactValue, Number};

const CORPUS: &str = include_str!("vectors/primitive-semantics.json");

#[derive(serde::Deserialize)]
struct Corpus {
    numbers: Vec<NumberVector>,
    orderings: Vec<OrderingVector>,
    admission: Vec<AdmissionVector>,
}

#[derive(serde::Deserialize)]
struct NumberVector {
    name: String,
    from: From_,
    integral: bool,
    as_i64: Option<i64>,
    display: String,
    json: String,
}

#[derive(serde::Deserialize)]
enum From_ {
    #[serde(rename = "integer")]
    Integer(i64),
    #[serde(rename = "decimal")]
    Decimal(String),
    #[serde(rename = "binary64")]
    Binary64(f64),
}

#[derive(serde::Deserialize)]
struct OrderingVector {
    name: String,
    left: String,
    right: String,
    ordering: String,
}

#[derive(serde::Deserialize)]
struct AdmissionVector {
    name: String,
    kind: String,
    value: serde_json::Value,
    admitted: bool,
}

fn corpus() -> Corpus {
    serde_json::from_str(CORPUS).expect("the corpus is a document this test's shape can read")
}

fn build(from: &From_) -> Number {
    match from {
        From_::Integer(value) => Number::from(*value),
        // The authored spelling of a decimal, through the same literal parser a predicate uses.
        From_::Decimal(text) => FactValue::parse_literal(text)
            .as_number()
            .unwrap_or_else(|| panic!("{text} is a numeric literal")),
        From_::Binary64(value) => Number::new(*value).expect("a finite binary64"),
    }
}

#[test]
fn every_number_vector_keeps_its_exact_value_its_spelling_and_its_bytes() {
    for vector in corpus().numbers {
        let number = build(&vector.from);
        assert_eq!(
            number.is_integral(),
            vector.integral,
            "{}: is_integral",
            vector.name
        );
        assert_eq!(number.as_i64(), vector.as_i64, "{}: as_i64", vector.name);
        assert_eq!(
            number.to_string(),
            vector.display,
            "{}: Display",
            vector.name
        );
        assert_eq!(
            serde_json::to_string(&number).expect("a finite number serialises"),
            vector.json,
            "{}: the bytes this wave does not move",
            vector.name
        );
    }
}

#[test]
fn every_ordering_vector_compares_exactly_and_facts_agree_with_numbers() {
    let corpus = corpus();
    let by_name: BTreeMap<&str, Number> = corpus
        .numbers
        .iter()
        .map(|vector| (vector.name.as_str(), build(&vector.from)))
        .collect();
    for vector in &corpus.orderings {
        let left = by_name[vector.left.as_str()];
        let right = by_name[vector.right.as_str()];
        let expected = match vector.ordering.as_str() {
            "less" => Ordering::Less,
            "greater" => Ordering::Greater,
            "equal" => Ordering::Equal,
            other => panic!("{other} is not an ordering"),
        };
        assert_eq!(left.cmp(&right), expected, "{}", vector.name);
        assert_eq!(
            (left == right),
            expected == Ordering::Equal,
            "{}: Eq agrees with Ord",
            vector.name
        );
        // A fact is the number, so two facts are distinct exactly when the numbers are.
        assert_eq!(
            (FactValue::Number(left) == FactValue::Number(right)),
            expected == Ordering::Equal,
            "{}: as facts",
            vector.name
        );
    }
}

#[test]
fn the_text_grammars_answer_every_admission_vector_the_corpus_states() {
    for vector in corpus().admission {
        let Some(text) = vector.value.as_str() else {
            continue;
        };
        let grammar = match vector.kind.as_str() {
            "uuid" => ess_primitives::facts::is_canonical_uuid,
            "bytes" => ess_primitives::facts::is_padded_base64,
            _ => continue,
        };
        assert_eq!(grammar(text), vector.admitted, "{}", vector.name);
    }
}

#[test]
fn the_two_grammars_refuse_what_no_named_form_of_the_value_is() {
    // Neither grammar trims, and neither accepts an inner newline: a value that differs from an
    // admitted one only by surrounding whitespace is a second spelling of one value, which is the
    // thing `ess-gen`'s `UUID_PATTERN` comment says one form exists to prevent.
    assert!(!ess_primitives::facts::is_canonical_uuid(
        " 0f8fad5b-d9cb-469f-a165-70867728950e"
    ));
    assert!(!ess_primitives::facts::is_padded_base64("AA== "));
    assert!(!ess_primitives::facts::is_padded_base64("AA=\n="));
}
