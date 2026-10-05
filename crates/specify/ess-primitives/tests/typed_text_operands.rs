//! Typed text operands (beyond10x/ess#200): `starts_with`, `ends_with` and `contains` compare a
//! text fact with a view parameter, `{param: <name>}`, or a command input, `{input: <name>}`.
//!
//! The mapping is the canonical form and reads back byte for byte; below `ess/22` it is refused as
//! the non-scalar it always was. Evaluation is the literal operators' — byte-wise and
//! case-sensitive — with the operand read as a fact: an unobserved operand is `Unknown`, never the
//! empty text, and an observed operand that is not text is `False`.
//! `docs/design/expression-family-source22.md`, "Typed text operands (#200)", is the design.

use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{reading_source22_operands, Predicate, Truth};

fn node(text: &str) -> Node {
    serde_yaml::from_str(text).expect("the fixture is YAML")
}

fn read(text: &str) -> Predicate {
    Predicate::from_node(&node(text)).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn refused(text: &str) -> String {
    Predicate::from_node(&node(text))
        .map(|predicate| format!("read as {predicate}"))
        .expect_err(text)
        .to_string()
}

fn path(text: &str) -> FactPath {
    FactPath::new(text).expect("a fact path")
}

fn facts(entries: &[(&str, FactValue)]) -> FactStore {
    let mut store = FactStore::new();
    for (at, value) in entries {
        store.set(path(at), value.clone());
    }
    store
}

#[test]
fn t200_each_operator_reads_a_parameter_and_an_input_and_writes_them_back() {
    for keyword in ["starts_with", "ends_with", "contains"] {
        for namespace in ["param", "input"] {
            let written = format!("{{note: {{{keyword}: {{{namespace}: query}}}}}}");
            let predicate = read(&written);
            assert_eq!(predicate.to_node(), node(&written), "{written}");
            assert_eq!(
                predicate.to_string(),
                format!("note {keyword} {{{namespace}: query}}"),
                "{written}"
            );
            let json = serde_json::to_string(&predicate).expect("serialises");
            assert_eq!(
                json,
                format!(r#"{{"note":{{"{keyword}":{{"{namespace}":"query"}}}}}}"#)
            );
            assert_eq!(
                serde_json::from_str::<Predicate>(&json).expect("reads back"),
                predicate
            );
            assert!(predicate.uses_text_match());
        }
    }
}

#[test]
fn t200_a_literal_operand_keeps_its_bytes() {
    let literal = read(r#"{note: {contains: "param.query"}}"#);
    assert_eq!(
        serde_json::to_string(&literal).expect("serialises"),
        r#"{"note":{"contains":"param.query"}}"#
    );
    assert_eq!(literal.to_string(), r#"note contains "param.query""#);
    assert_eq!(literal.fact_paths(), vec![&path("note")]);
}

#[test]
fn t200_the_operand_is_a_read_of_its_namespace() {
    let view = read("{note: {contains: {param: query}}}");
    assert_eq!(view.fact_paths(), vec![&path("note"), &path("param.query")]);
    let guard = read("{phone: {starts_with: {input: prefix}}}");
    assert_eq!(
        guard.fact_paths(),
        vec![&path("phone"), &path("input.prefix")]
    );
}

#[test]
fn t200_below_source22_the_mapping_is_refused_as_it_always_was() {
    for written in [
        "{note: {contains: {param: query}}}",
        "{phone: {starts_with: {input: prefix}}}",
    ] {
        let refusal = reading_source22_operands(false, || {
            Predicate::from_node(&node(written)).expect_err(written)
        });
        assert!(
            refusal
                .to_string()
                .contains("a comparison operand must be a scalar"),
            "{written}: {refusal}"
        );
    }
}

#[test]
fn t200_only_one_namespace_and_one_top_level_name_are_admitted() {
    for written in [
        "{note: {contains: {param: a.b}}}",
        "{note: {contains: {input: a.b}}}",
        "{note: {contains: {param: query, input: prefix}}}",
        "{note: {contains: {fact: query}}}",
        "{note: {contains: {caller: query}}}",
        "{note: {contains: {param: 4}}}",
        "{note: {contains: {param: [query]}}}",
        "{note: {contains: {}}}",
        "{note: {contains: [query]}}",
    ] {
        let refusal = refused(written);
        assert!(
            refusal.contains("{param: <name>}") || refusal.contains("must be a scalar"),
            "{written}: {refusal}"
        );
    }
}

/// Every operator over `note`, against `{param: query}`, with the note and the parameter given.
fn decide(keyword: &str, note: Option<FactValue>, query: Option<FactValue>) -> Truth {
    let predicate = read(&format!("{{note: {{{keyword}: {{param: query}}}}}}"));
    let mut entries = Vec::new();
    if let Some(note) = note {
        entries.push(("note", note));
    }
    if let Some(query) = query {
        entries.push(("param.query", query));
    }
    predicate.evaluate(&facts(&entries))
}

// `decide` takes optional operands, one case leaves one absent, so a present one is wrapped here.
#[allow(clippy::unnecessary_wraps)]
fn text(value: &str) -> Option<FactValue> {
    Some(FactValue::text(value))
}

#[test]
fn t200_evaluation_is_the_literal_operators_byte_wise_and_case_sensitive() {
    for (keyword, matching, refuting) in [
        // Differs from the match at the byte the operator decides on.
        ("starts_with", "hello", "hellp"),
        ("ends_with", "world", "vorld"),
        ("contains", "lo wo", "lo xo"),
    ] {
        assert_eq!(
            decide(keyword, text("hello world"), text(matching)),
            Truth::True,
            "{keyword} {matching}"
        );
        assert_eq!(
            decide(keyword, text("hello world"), text(refuting)),
            Truth::False,
            "{keyword} {refuting}"
        );
        // Case-sensitive, as the literal operators are.
        assert_eq!(
            decide(
                keyword,
                text("hello world"),
                text(&matching.to_ascii_uppercase())
            ),
            Truth::False,
            "{keyword} upper case"
        );
        // The operand is the needle, the fact the haystack: never the other way round.
        assert_eq!(
            decide(keyword, text(matching), text("hello world")),
            Truth::False,
            "{keyword} reversed"
        );
        // The empty text begins, ends and occurs in every text, byte for byte.
        assert_eq!(decide(keyword, text("hello world"), text("")), Truth::True);
        // An absent parameter is unknown, never the empty text.
        assert_eq!(decide(keyword, text("hello world"), None), Truth::Unknown);
        assert_eq!(decide(keyword, None, text(matching)), Truth::Unknown);
        // An observed operand that is not text, in an unchecked predicate, is false.
        assert_eq!(
            decide(
                keyword,
                text("hello world"),
                Some(FactValue::Number(Number::from(4_i64)))
            ),
            Truth::False
        );
        assert_eq!(
            decide(keyword, Some(FactValue::Bool(true)), text(matching)),
            Truth::False
        );
        // Negation keeps Unknown.
        let negated = read(&format!(
            "{{not: {{note: {{{keyword}: {{param: query}}}}}}}}"
        ));
        assert_eq!(
            negated.evaluate(&facts(&[("note", FactValue::text("x"))])),
            Truth::Unknown
        );
    }
}

#[test]
fn t200_an_input_operand_reads_the_path_it_was_resolved_to() {
    let guard = read("{phone: {ends_with: {input: suffix}}}");
    let store = facts(&[
        ("phone", FactValue::text("+44 20 7946 0000")),
        ("input.suffix", FactValue::text("0000")),
        ("suffix", FactValue::text("nothing")),
    ]);
    assert_eq!(guard.evaluate(&store), Truth::True);
}
