//! `starts_with`, `ends_with` and `contains` (beyond10x/ess#95), as the predicate language reads,
//! renders and evaluates them. `docs/design/string-predicate-operators.md` is the binding design.
//!
//! The evaluation cases are the shared `text_matches` table of
//! `tests/vectors/primitive-semantics.json`, which `ess-conformance`'s corpus test hands to the Go
//! runtime as well, so one table decides every lane that evaluates these operators.

use ess_primitives::error::ParseError;
use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Predicate, TextOp, Truth};

const CORPUS: &str = include_str!("vectors/primitive-semantics.json");

fn yaml(text: &str) -> Predicate {
    let node: Node = serde_yaml::from_str(text).expect("the fixture is YAML");
    Predicate::from_node(&node).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn refused(text: &str) -> ParseError {
    let node: Node = serde_yaml::from_str(text).expect("the fixture is YAML");
    Predicate::from_node(&node).expect_err(text)
}

fn path(text: &str) -> FactPath {
    FactPath::new(text).expect("a path")
}

fn text_match(at: &str, op: TextOp, value: FactValue) -> Predicate {
    Predicate::TextMatch {
        path: path(at),
        op,
        value,
    }
}

#[test]
fn each_operator_parses_in_map_form_to_one_text_match_leaf() {
    for (keyword, op) in [
        ("starts_with", TextOp::StartsWith),
        ("ends_with", TextOp::EndsWith),
        ("contains", TextOp::Contains),
    ] {
        assert_eq!(
            yaml(&format!("caller: {{{keyword}: \"+44\"}}")),
            text_match("caller", op, FactValue::text("+44")),
            "{keyword}"
        );
        assert_eq!(op.keyword(), keyword);
        assert_eq!(TextOp::from_keyword(keyword), Some(op));
    }
}

#[test]
fn the_operand_is_the_text_it_spells_never_a_number_a_boolean_or_a_fact() {
    for (written, literal) in [
        (r#""+44""#, "+44"),
        (r#""0""#, "0"),
        (r#""a.b""#, "a.b"),
        (r#""true""#, "true"),
        (r#"'"q"'"#, "\"q\""),
        (r#""input.prefix""#, "input.prefix"),
    ] {
        assert_eq!(
            yaml(&format!("caller: {{starts_with: {written}}}")),
            text_match("caller", TextOp::StartsWith, FactValue::text(literal)),
            "{written}"
        );
    }
}

#[test]
fn an_unquoted_number_or_boolean_is_kept_as_that_value_for_validation_to_refuse() {
    assert_eq!(
        yaml("caller: {starts_with: +44}"),
        text_match(
            "caller",
            TextOp::StartsWith,
            FactValue::Number(Number::new(44.0).unwrap())
        )
    );
    assert_eq!(
        yaml("caller: {contains: true}"),
        text_match("caller", TextOp::Contains, FactValue::Bool(true))
    );
}

#[test]
fn a_list_a_mapping_or_null_operand_is_a_reader_error_naming_the_scalar_rule() {
    for written in ["[a, b]", "{a: b}", "null", "~"] {
        let error = refused(&format!("caller: {{ends_with: {written}}}"));
        assert!(
            matches!(error, ParseError::Predicate { .. }),
            "{written}: {error:?}"
        );
        assert!(
            error
                .to_string()
                .contains("a comparison operand must be a scalar"),
            "{written}: {error}"
        );
    }
}

#[test]
fn several_operators_in_one_mapping_are_conjoined() {
    assert_eq!(
        yaml(r#"sku: {starts_with: "A", ends_with: "0"}"#),
        Predicate::All(vec![
            text_match("sku", TextOp::EndsWith, FactValue::text("0")),
            text_match("sku", TextOp::StartsWith, FactValue::text("A")),
        ])
    );
}

#[test]
fn there_is_no_compact_form_and_no_second_spelling() {
    for expression in [r#"caller starts_with "+44""#, r#"caller contains "x""#] {
        let error = Predicate::parse_expression(expression).expect_err(expression);
        assert!(matches!(error, ParseError::Predicate { .. }), "{error:?}");
    }
    for alias in ["not_starts_with", "startswith", "prefix", "includes"] {
        let error = refused(&format!("caller: {{{alias}: x}}"));
        assert!(
            error.to_string().contains("unknown operator"),
            "{alias}: {error}"
        );
    }
}

#[test]
fn the_unknown_operator_message_names_the_three_operators() {
    let error = refused("caller: {begins: x}").to_string();
    for keyword in ["starts_with", "ends_with", "contains"] {
        assert!(error.contains(keyword), "{error}");
    }
}

#[test]
fn a_text_match_round_trips_through_its_document_form_byte_for_byte() {
    let values = [
        FactValue::text("+44"),
        FactValue::text("0"),
        FactValue::text("a.b"),
        FactValue::text("true"),
        FactValue::text("\"q\""),
        FactValue::text("null"),
        FactValue::text("x == y"),
        FactValue::Number(Number::new(44.0).unwrap()),
        FactValue::Bool(false),
    ];
    for op in [TextOp::StartsWith, TextOp::EndsWith, TextOp::Contains] {
        for value in &values {
            let predicate = text_match("caller", op, value.clone());
            let node = predicate.to_node();
            assert_eq!(
                Predicate::from_node(&node).as_ref(),
                Ok(&predicate),
                "{node:?}"
            );
            let json = serde_json::to_string(&predicate).unwrap();
            let reread: Predicate = serde_json::from_str(&json).unwrap();
            assert_eq!(reread, predicate, "{json}");
            // Nested under a negation and a quantifier, as a document carries it.
            let nested = Predicate::not(predicate.clone());
            assert_eq!(Predicate::from_node(&nested.to_node()), Ok(nested));
        }
    }
    assert_eq!(
        serde_json::to_value(text_match(
            "caller",
            TextOp::StartsWith,
            FactValue::text("+44")
        ))
        .unwrap(),
        serde_json::json!({"caller": {"starts_with": "+44"}}),
        "a text operand is rendered as the text, with no quotes added"
    );
}

#[test]
fn display_quotes_a_text_operand_and_leaves_a_number_bare() {
    assert_eq!(
        text_match("caller", TextOp::StartsWith, FactValue::text("+44")).to_string(),
        r#"caller starts_with "+44""#
    );
    assert_eq!(
        text_match("dialled", TextOp::EndsWith, FactValue::text("0")).to_string(),
        r#"dialled ends_with "0""#
    );
    assert_eq!(
        text_match(
            "caller",
            TextOp::Contains,
            FactValue::Number(Number::new(44.0).unwrap())
        )
        .to_string(),
        "caller contains 44"
    );
}

#[test]
fn the_path_is_read_by_every_walk_and_a_binder_is_not() {
    let predicate = yaml(
        "all:\n  - caller: {starts_with: x}\n  - exists: {in: tags, as: t, that: {t: {contains: vip}}}\n",
    );
    let paths: Vec<String> = predicate
        .fact_paths()
        .into_iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(paths, ["caller", "tags"]);
    assert_eq!(predicate.quantified_collections().len(), 1);
    assert!(predicate.uses_text_match());
    assert!(!yaml("caller == x").uses_text_match());
    assert!(
        yaml("not: {any: [a == b, {forall: {in: xs, as: e, that: {e: {ends_with: z}}}}]}")
            .uses_text_match()
    );
}

#[test]
fn the_schema_description_names_the_three_keys() {
    let schema = schemars::schema_for!(Predicate);
    let description = serde_json::to_string(&schema).unwrap();
    for keyword in ["starts_with", "ends_with", "contains"] {
        assert!(description.contains(keyword), "{description}");
    }
}

#[derive(serde::Deserialize)]
struct Corpus {
    text_matches: Vec<TextMatchVector>,
}

#[derive(serde::Deserialize)]
struct TextMatchVector {
    name: String,
    /// Absent: nothing observed at the path. `null`: observed as null, which Rust binds as nothing.
    #[serde(default, deserialize_with = "present")]
    value: Option<serde_json::Value>,
    op: String,
    literal: String,
    truth: String,
}

fn present<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<serde_json::Value>, D::Error> {
    serde::Deserialize::deserialize(deserializer).map(Some)
}

#[test]
fn every_text_match_vector_is_the_truth_the_evaluator_answers() {
    let corpus: Corpus = serde_json::from_str(CORPUS).expect("the corpus is readable");
    assert!(
        corpus.text_matches.len() >= 20,
        "the corpus states {} text matches",
        corpus.text_matches.len()
    );
    for vector in corpus.text_matches {
        let mut facts = FactStore::new();
        match &vector.value {
            None | Some(serde_json::Value::Null) => {}
            Some(serde_json::Value::String(text)) => {
                facts.set(path("caller"), FactValue::text(text));
            }
            Some(serde_json::Value::Bool(value)) => {
                facts.set(path("caller"), FactValue::Bool(*value));
            }
            Some(serde_json::Value::Number(number)) => facts.set(
                path("caller"),
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap()),
            ),
            Some(other) => panic!("{}: {other} is not a scalar", vector.name),
        }
        let op = TextOp::from_keyword(&vector.op)
            .unwrap_or_else(|| panic!("{}: {} is not an operator", vector.name, vector.op));
        let expected = match vector.truth.as_str() {
            "true" => Truth::True,
            "false" => Truth::False,
            "unknown" => Truth::Unknown,
            other => panic!("{other} is not a truth"),
        };
        let predicate = text_match("caller", op, FactValue::text(&vector.literal));
        assert_eq!(predicate.evaluate(&facts), expected, "{}", vector.name);
        // Kleene negation, as for every leaf: `not Unknown` is `Unknown`.
        assert_eq!(
            Predicate::not(predicate).evaluate(&facts),
            expected.not(),
            "not {}",
            vector.name
        );
    }
}

#[test]
fn a_non_text_literal_is_false_against_an_observed_value_and_unknown_otherwise() {
    let literal = FactValue::Number(Number::new(44.0).unwrap());
    let mut facts = FactStore::new();
    let predicate = text_match("caller", TextOp::StartsWith, literal);
    assert_eq!(predicate.evaluate(&facts), Truth::Unknown);
    facts.set(path("caller"), FactValue::text("44"));
    assert_eq!(predicate.evaluate(&facts), Truth::False);
}

#[test]
fn an_unknown_leaf_names_its_path_as_missing() {
    let outcome =
        text_match("caller", TextOp::Contains, FactValue::text("x")).outcome(&FactStore::new());
    assert_eq!(outcome.truth, Truth::Unknown);
    assert_eq!(outcome.missing_facts(), [&path("caller")]);
}
