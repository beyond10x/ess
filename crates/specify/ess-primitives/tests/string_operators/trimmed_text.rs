//! `trimmed:` (beyond10x/ess#532, ess/25) as the Rust evaluator answers it.
//! `docs/design/value-expressions.md` § E10 is the binding design.
//!
//! The evaluation cases are `tests/vectors/trimmed-text.json`, which `ess-conformance` hands to the
//! Go runtime and the TypeScript runtime reads directly, so one table decides every lane that
//! evaluates the operator.

use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, Truth};

const CORPUS: &str = include_str!("../vectors/trimmed-text.json");

#[derive(serde::Deserialize)]
struct Corpus {
    trimmed: Vec<Vector>,
}

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    #[serde(default)]
    value: Option<serde_json::Value>,
    operator: serde_json::Value,
    truth: String,
}

#[test]
fn every_trimmed_vector_is_the_truth_the_evaluator_answers() {
    let corpus: Corpus = serde_json::from_str(CORPUS).expect("the corpus is readable");
    assert!(
        corpus.trimmed.len() >= 25,
        "the corpus states {} trimmed vectors",
        corpus.trimmed.len()
    );
    let source = FactPath::new("source").unwrap();
    for vector in corpus.trimmed {
        let mut facts = FactStore::new();
        match &vector.value {
            None => {}
            Some(serde_json::Value::String(text)) => {
                facts.set(source.clone(), FactValue::text(text));
            }
            Some(serde_json::Value::Bool(value)) => {
                facts.set(source.clone(), FactValue::Bool(*value));
            }
            Some(serde_json::Value::Number(number)) => facts.set(
                source.clone(),
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap()),
            ),
            Some(other) => panic!("{}: a scalar value, not {other}", vector.name),
        }
        let node: Node = serde_json::from_value(
            serde_json::json!({"source": {"trimmed": vector.operator.clone()}}),
        )
        .unwrap();
        let predicate =
            Predicate::from_node(&node).unwrap_or_else(|error| panic!("{}: {error}", vector.name));
        assert!(predicate.uses_trim(), "{}", vector.name);
        let want = match vector.truth.as_str() {
            "true" => Truth::True,
            "false" => Truth::False,
            "unknown" => Truth::Unknown,
            other => panic!("{}: {other} is not a truth", vector.name),
        };
        assert_eq!(predicate.evaluate(&facts), want, "{}", vector.name);
        assert_eq!(
            Predicate::not(predicate.clone()).evaluate(&facts),
            want.not(),
            "not {}",
            vector.name
        );
        // The canonical form reads back as the same predicate, so a suite or an IR carrying it
        // decides the same.
        assert_eq!(
            Predicate::from_node(&predicate.to_node()).unwrap(),
            predicate,
            "{}",
            vector.name
        );
    }
}
