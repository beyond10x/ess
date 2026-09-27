//! `equals_ignore_case` and `in_ignore_case` (beyond10x/ess#140), as the predicate language reads,
//! renders and evaluates them. `docs/design/value-expressions.md` § E7 is the binding design.
//!
//! Folding is ASCII only: `A`–`Z` fold to `a`–`z`, every other byte compares as itself. The
//! evaluation cases are the shared `case_folds` table of `tests/vectors/primitive-semantics.json`,
//! which `ess-conformance` hands to the Go runtime and the TypeScript runtime reads directly, so one
//! table decides every lane that evaluates these operators.

use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{FoldOp, Predicate, Truth};

const CORPUS: &str = include_str!("vectors/primitive-semantics.json");

fn yaml(text: &str) -> Predicate {
    let node: Node = serde_yaml::from_str(text).expect("the fixture is YAML");
    Predicate::from_node(&node).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn refused(text: &str) -> String {
    let node: Node = serde_yaml::from_str(text).expect("the fixture is YAML");
    Predicate::from_node(&node).expect_err(text).to_string()
}

fn path(text: &str) -> FactPath {
    FactPath::new(text).expect("a path")
}

fn fold(at: &str, op: FoldOp, values: &[&str]) -> Predicate {
    Predicate::FoldMatch {
        path: path(at),
        op,
        values: values.iter().map(|value| FactValue::text(*value)).collect(),
    }
}

#[test]
fn each_operator_parses_in_map_form_to_one_fold_leaf() {
    assert_eq!(
        yaml("source: {equals_ignore_case: web}"),
        fold("source", FoldOp::EqualsIgnoreCase, &["web"])
    );
    assert_eq!(
        yaml("source: {in_ignore_case: [web, phone]}"),
        fold("source", FoldOp::InIgnoreCase, &["web", "phone"])
    );
    for op in FoldOp::ALL {
        assert_eq!(FoldOp::from_keyword(op.keyword()), Some(op));
    }
    assert_eq!(FoldOp::EqualsIgnoreCase.keyword(), "equals_ignore_case");
    assert_eq!(FoldOp::InIgnoreCase.keyword(), "in_ignore_case");
}

#[test]
fn the_operand_is_the_text_it_spells_never_a_fact_path() {
    assert_eq!(
        yaml(r#"source: {equals_ignore_case: "a.b"}"#),
        fold("source", FoldOp::EqualsIgnoreCase, &["a.b"])
    );
    assert_eq!(
        yaml(r#"source: {in_ignore_case: ["x.y", "0"]}"#),
        fold("source", FoldOp::InIgnoreCase, &["x.y", "0"])
    );
    // A number or a Boolean is kept as that value, for validation to refuse with a code and a site.
    assert_eq!(
        yaml("source: {equals_ignore_case: 44}"),
        Predicate::FoldMatch {
            path: path("source"),
            op: FoldOp::EqualsIgnoreCase,
            values: vec![FactValue::Number(Number::new(44.0).unwrap())],
        }
    );
}

#[test]
fn the_shapes_each_operator_does_not_take_are_refused_at_parse() {
    assert!(
        refused("source: {equals_ignore_case: [web]}").contains("equals_ignore_case"),
        "a list under the one-literal operator"
    );
    assert!(
        refused("source: {in_ignore_case: web}").contains("in_ignore_case"),
        "a scalar under the list operator"
    );
    assert!(refused("source: {in_ignore_case: [[web]]}").contains("scalar"));
    assert!(refused("source: {equals_ignore_case: {a: b}}").contains("scalar"));
    let unknown = refused("source: {equals_ignorecase: web}");
    for keyword in ["equals_ignore_case", "in_ignore_case"] {
        assert!(unknown.contains(keyword), "{unknown}");
    }
}

#[test]
fn rendering_reads_back_as_the_same_predicate() {
    for text in [
        "source: {equals_ignore_case: web}",
        r#"source: {equals_ignore_case: "a.b"}"#,
        "source: {in_ignore_case: [web, phone]}",
        "source: {in_ignore_case: []}",
        "not: {source: {in_ignore_case: [Web]}}",
        "all: [{source: {equals_ignore_case: x}}, {other: {starts_with: y}}]",
    ] {
        let predicate = yaml(text);
        let node = predicate.to_node();
        assert_eq!(
            Predicate::from_node(&node).unwrap(),
            predicate,
            "{text} renders as {node}"
        );
    }
    assert_eq!(
        yaml("source: {equals_ignore_case: web}").to_string(),
        r#"source equals_ignore_case "web""#
    );
    assert_eq!(
        yaml("source: {in_ignore_case: [web, phone]}").to_string(),
        r#"source in_ignore_case ["web", "phone"]"#
    );
}

#[test]
fn the_path_is_read_by_every_walk_and_the_construct_is_found_at_any_depth() {
    let predicate = yaml(
        "all:\n  - source: {equals_ignore_case: x}\n  - exists: {in: tags, as: t, that: {t: {in_ignore_case: [vip]}}}\n",
    );
    let paths: Vec<String> = predicate
        .fact_paths()
        .into_iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(paths, ["source", "tags"]);
    assert!(predicate.uses_case_fold());
    assert!(
        !predicate.uses_text_match(),
        "a fold is not a string operator of ess/8"
    );
    assert!(!yaml("source == x").uses_case_fold());
    assert!(!yaml("source: {starts_with: x}").uses_case_fold());
    assert!(yaml(
        "not: {any: [a == b, {forall: {in: xs, as: e, that: {e: {equals_ignore_case: z}}}}]}"
    )
    .uses_case_fold());
}

#[test]
fn the_schema_description_names_the_two_keys() {
    let schema = schemars::schema_for!(Predicate);
    let description = serde_json::to_string(&schema).unwrap();
    for keyword in ["equals_ignore_case", "in_ignore_case"] {
        assert!(description.contains(keyword), "{description}");
    }
}

#[test]
fn ascii_folding_is_exactly_the_twenty_six_letter_pairs() {
    for byte in 0u8..=127 {
        for other in 0u8..=127 {
            let (left, right) = (char::from(byte).to_string(), char::from(other).to_string());
            // Written out rather than through `eq_ignore_ascii_case`, which is the implementation.
            let fold = |value: u8| {
                if value.is_ascii_uppercase() {
                    value + 32
                } else {
                    value
                }
            };
            let expected = fold(byte) == fold(other);
            assert_eq!(
                FoldOp::EqualsIgnoreCase.holds(&left, &[&right]),
                expected,
                "{byte:#x} against {other:#x}"
            );
        }
    }
}

#[derive(serde::Deserialize)]
struct Corpus {
    case_folds: Vec<FoldVector>,
}

#[derive(serde::Deserialize)]
struct FoldVector {
    name: String,
    value: serde_json::Value,
    op: String,
    literal: serde_json::Value,
    truth: String,
}

#[test]
fn every_case_fold_vector_is_the_truth_the_evaluator_answers() {
    let corpus: Corpus = serde_json::from_str(CORPUS).expect("the corpus is readable");
    assert!(
        corpus.case_folds.len() >= 25,
        "the corpus states {} case folds",
        corpus.case_folds.len()
    );
    for vector in corpus.case_folds {
        let mut facts = FactStore::new();
        match &vector.value {
            serde_json::Value::Null => {}
            serde_json::Value::String(text) => facts.set(path("source"), FactValue::text(text)),
            serde_json::Value::Bool(value) => facts.set(path("source"), FactValue::Bool(*value)),
            serde_json::Value::Number(number) => facts.set(
                path("source"),
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap()),
            ),
            other => panic!("{}: {other} is not a scalar", vector.name),
        }
        let op = FoldOp::from_keyword(&vector.op)
            .unwrap_or_else(|| panic!("{}: {} is not an operator", vector.name, vector.op));
        let values: Vec<FactValue> = match &vector.literal {
            serde_json::Value::String(text) => vec![FactValue::text(text)],
            serde_json::Value::Array(items) => items
                .iter()
                .map(|item| FactValue::text(item.as_str().expect("a text member")))
                .collect(),
            other => panic!("{}: {other} is no literal", vector.name),
        };
        let expected = match vector.truth.as_str() {
            "true" => Truth::True,
            "false" => Truth::False,
            "unknown" => Truth::Unknown,
            other => panic!("{other} is not a truth"),
        };
        let predicate = Predicate::FoldMatch {
            path: path("source"),
            op,
            values,
        };
        assert_eq!(predicate.evaluate(&facts), expected, "{}", vector.name);
        assert_eq!(
            Predicate::not(predicate).evaluate(&facts),
            expected.not(),
            "not {}",
            vector.name
        );
    }
}

#[test]
fn a_non_text_literal_matches_nothing_and_an_unobserved_value_is_unknown() {
    let predicate = Predicate::FoldMatch {
        path: path("source"),
        op: FoldOp::InIgnoreCase,
        values: vec![FactValue::Bool(true), FactValue::text("web")],
    };
    let mut facts = FactStore::new();
    assert_eq!(predicate.evaluate(&facts), Truth::Unknown);
    facts.set(path("source"), FactValue::text("true"));
    assert_eq!(predicate.evaluate(&facts), Truth::False);
    facts.set(path("source"), FactValue::text("WEB"));
    assert_eq!(predicate.evaluate(&facts), Truth::True);
}

#[test]
fn an_unknown_leaf_names_its_path_as_missing() {
    let outcome = fold("source", FoldOp::EqualsIgnoreCase, &["web"]).outcome(&FactStore::new());
    assert_eq!(outcome.truth, Truth::Unknown);
    assert_eq!(outcome.missing_facts(), [&path("source")]);
}
