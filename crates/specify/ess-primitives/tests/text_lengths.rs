//! `.count` on text: the number of Unicode scalar values (beyond10x/ess#104).
//!
//! `docs/design/string-alphabet-and-length.md`, section 3, *Evaluation*, is the binding design. The
//! rule, the same in every lane: a bound fact wins; otherwise, when the last segment is `count` and
//! the parent path is bound to text, the value is the text's scalar count. Leaf reads only — a
//! quantifier's cardinality is untouched.
//!
//! The rows are the shared `text_lengths` table of `tests/vectors/primitive-semantics.json`, which
//! `ess-conformance` hands to the Go runtime and to the TypeScript runtime as well.

use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

const CORPUS: &str = include_str!("vectors/primitive-semantics.json");

fn yaml(text: &str) -> Predicate {
    let node: Node = serde_yaml::from_str(text).expect("the fixture is YAML");
    Predicate::from_node(&node).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn path(text: &str) -> FactPath {
    FactPath::new(text).expect("a path")
}

#[derive(serde::Deserialize)]
struct Corpus {
    text_lengths: Vec<TextLength>,
}

#[derive(serde::Deserialize)]
struct TextLength {
    name: String,
    facts: serde_json::Map<String, serde_json::Value>,
    path: String,
    expected: serde_json::Value,
}

fn store(facts: &serde_json::Map<String, serde_json::Value>) -> FactStore {
    let mut store = FactStore::new();
    for (at, value) in facts {
        match value {
            // Rust binds nothing for a null, as the flattener does.
            serde_json::Value::Null => {}
            serde_json::Value::String(text) => store.set(path(at), FactValue::text(text)),
            serde_json::Value::Bool(flag) => store.set(path(at), FactValue::Bool(*flag)),
            serde_json::Value::Number(number) => store.set(
                path(at),
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap()),
            ),
            other => panic!("{at}: {other} is not a scalar"),
        }
    }
    store
}

fn compare(at: &str, value: f64) -> Predicate {
    Predicate::Compare {
        left: ess_primitives::predicate::Operand::Fact(path(at)),
        op: ess_primitives::predicate::CompareOp::Eq,
        right: ess_primitives::predicate::Operand::Literal(
            FactValue::number(value).expect("finite"),
        ),
    }
}

#[test]
fn every_text_length_vector_is_the_count_the_evaluator_reads() {
    let corpus: Corpus = serde_json::from_str(CORPUS).expect("the corpus is readable");
    assert!(
        corpus.text_lengths.len() >= 8,
        "the corpus states {} text lengths",
        corpus.text_lengths.len()
    );
    for vector in corpus.text_lengths {
        let facts = store(&vector.facts);
        match &vector.expected {
            serde_json::Value::Number(count) => {
                let count = count.as_f64().unwrap();
                assert_eq!(
                    compare(&vector.path, count).evaluate(&facts),
                    Truth::True,
                    "{}: `{} == {count}`",
                    vector.name,
                    vector.path
                );
                assert_eq!(
                    compare(&vector.path, count + 1.0).evaluate(&facts),
                    Truth::False,
                    "{}: `{} == {}`",
                    vector.name,
                    vector.path,
                    count + 1.0
                );
            }
            serde_json::Value::String(word) if word == "unknown" => {
                assert_eq!(
                    compare(&vector.path, 0.0).evaluate(&facts),
                    Truth::Unknown,
                    "{}: `{}` has no length",
                    vector.name,
                    vector.path
                );
            }
            other => panic!("{}: {other} is not an expectation", vector.name),
        }
    }
}

/// One sample of every predicate kind that reads a leaf, each reading `keys.count`.
///
/// The `match` is exhaustive with no wildcard, so a new leaf kind cannot be added to the grammar
/// without being added here — the mutant "one evaluator read left on `fact`" is killed per kind.
fn samples() -> Vec<Predicate> {
    let samples = vec![
        yaml("keys.count == 3"),
        // The literal on the left: the grammar writes a fact first, so this side is built.
        Predicate::Compare {
            left: ess_primitives::predicate::Operand::Literal(FactValue::count(3)),
            op: ess_primitives::predicate::CompareOp::Eq,
            right: ess_primitives::predicate::Operand::Fact(path("keys.count")),
        },
        yaml("defined(keys.count)"),
        yaml("keys.count"),
        yaml("keys.count: {in: [3, 4]}"),
        yaml("keys.count: {not_in: [1, 2]}"),
        yaml("exists: {in: parts, as: p, that: p.count == 3}"),
    ];
    for sample in &samples {
        match sample {
            Predicate::Compare { .. }
            | Predicate::Defined(_)
            | Predicate::Truthy(_)
            | Predicate::AnyOf { .. }
            | Predicate::NoneOf { .. }
            | Predicate::Exists(_) => {}
            // A string operator reads text, not a number, so it has no `.count` sample; a
            // connective reads through its children. Each is named, so a new kind is a compile
            // error here rather than a silent gap.
            Predicate::TextMatch { .. }
            | Predicate::Forall(_)
            | Predicate::All(_)
            | Predicate::Any(_)
            | Predicate::Not(_)
            | Predicate::Always
            | Predicate::Never => panic!("{sample} is not a leaf sample"),
        }
    }
    samples
}

#[test]
fn a_text_reads_as_if_its_count_were_bound_in_every_leaf_kind() {
    let mut bare = FactStore::new();
    bare.set(path("keys"), FactValue::text("abc"));
    bare.set(path("parts.count"), FactValue::count(1));
    bare.set(path("parts.0"), FactValue::text("xyz"));
    let mut bound = bare.clone();
    bound.set(path("keys.count"), FactValue::count(3));
    bound.set(path("parts.0.count"), FactValue::count(3));
    for sample in samples() {
        assert_eq!(
            sample.evaluate(&bare),
            sample.evaluate(&bound),
            "`{sample}` over bound text reads differently from the bound count"
        );
        assert_ne!(
            sample.evaluate(&bare),
            Truth::Unknown,
            "`{sample}` over bound text is decided"
        );
    }
}

#[test]
fn the_cause_walk_observes_a_text_length_rather_than_calling_it_missing() {
    let mut facts = FactStore::new();
    facts.set(path("keys"), FactValue::text("abc"));
    let outcome = yaml("keys.count > 64").outcome(&facts);
    assert_eq!(outcome.truth, Truth::False);
    let cause = &outcome.causes[0];
    assert_eq!(
        cause.observed,
        vec![(path("keys.count"), FactValue::count(3))]
    );
    assert!(cause.missing.is_empty(), "{:?}", cause.missing);
}

#[test]
fn a_quantifier_over_text_stays_unknown_because_cardinality_is_not_a_length() {
    let mut facts = FactStore::new();
    facts.set(path("keys"), FactValue::text(""));
    assert_eq!(
        yaml("forall: {in: keys, as: k, that: k == x}").evaluate(&facts),
        Truth::Unknown
    );
    facts.set(path("keys"), FactValue::text("abc"));
    assert_eq!(
        yaml("exists: {in: keys, as: k, that: k == a}").evaluate(&facts),
        Truth::Unknown
    );
    assert_eq!(facts.cardinality(&path("keys")), None);
}
