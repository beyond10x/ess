//! A compact comparison consumes exactly one quoted operand; structured data stays literal.
use ess_primitives::{
    facts::FactValue,
    node::Node,
    predicate::{Operand, Predicate},
};

#[test]
fn closed_quotes_cannot_hide_trailing_expression_tokens() {
    for expression in [
        r#"to == "" or text == """#,
        r"to == '' and text == ''",
        r#"to == "done" trailing"#,
        r#"to == "done""next""#,
        r#"not to == "" or text == """#,
        r#"to == "unterminated"#,
        r"to == 'unterminated",
        r#"to == "escaped\""#,
        r#"to == "slash\\" or text == """#,
    ] {
        let error = Predicate::parse_expression(expression).expect_err(expression);
        let diagnostic = error.to_string();
        assert!(
            diagnostic.contains("structured")
                && diagnostic.contains("any")
                && diagnostic.contains("all")
                && diagnostic.contains("not"),
            "{diagnostic}"
        );
    }
}

#[test]
fn a_single_quoted_operand_preserves_its_exact_literal_bytes() {
    for (expression, expected) in [
        (r#"to == """#, ""),
        (r"to == 'true'", "true"),
        (r#"to == "or and not""#, "or and not"),
        (" \tto == '  spaced  ' \n", "  spaced  "),
        (r#"to == "say \"or\"""#, r#"say \"or\""#),
        (r"to == 'it\'s valid'", r"it\'s valid"),
        (r#"to == "path\\""#, r"path\\"),
        (r#"to == "雪 and ☃""#, "雪 and ☃"),
        (r#"to == "a == b or c != d""#, "a == b or c != d"),
    ] {
        let Predicate::Compare { right, .. } = Predicate::parse_expression(expression).unwrap()
        else {
            panic!("comparison: {expression}")
        };
        assert_eq!(
            right,
            Operand::Literal(FactValue::Text(expected.into())),
            "{expression}"
        );
    }
    assert!(matches!(
        Predicate::parse_expression(r#"not to == """#).unwrap(),
        Predicate::Not(_)
    ));
}

#[test]
fn structured_composition_and_literal_data_remain_admitted() {
    let node: Node = serde_json::from_value(
        serde_json::json!({"any": ["to == \"\"", {"all": ["text == ''", {"not": "disabled"}]}]}),
    )
    .unwrap();
    assert!(Predicate::from_node(&node).is_ok());
    let node: Node =
        serde_json::from_value(serde_json::json!({"to": {"eq": "\"\" or text == \"\""}})).unwrap();
    assert!(
        Predicate::from_node(&node).is_ok(),
        "structured scalar is data, not an expression"
    );
}

#[test]
fn canonical_text_preserves_literals_and_unchanged_compact_bytes() {
    for text in [
        "\"busy\" status",
        "'busy' status",
        "true",
        "1",
        "  padded  ",
        "a\\.b",
        "say \\\"hi\\\"",
        "line\nnext",
        "\0",
        "\"quoted\"",
    ] {
        let node: Node =
            serde_json::from_value(serde_json::json!({"to": {"eq": format!("\"{text}\"")}}))
                .unwrap();
        let predicate = Predicate::from_node(&node).unwrap();
        let emitted = predicate.to_node();
        assert_eq!(
            Predicate::from_node(&emitted).unwrap(),
            predicate,
            "{text:?}: {emitted:?}"
        );
        assert_eq!(
            serde_json::from_str::<Predicate>(&serde_json::to_string(&predicate).unwrap()).unwrap(),
            predicate
        );
    }
    for expression in [
        "to == Ready",
        "to == 12",
        "to == true",
        "to == \"\"",
        "to == \"a.b\"",
    ] {
        let predicate = Predicate::parse_expression(expression).unwrap();
        assert_eq!(predicate.to_node(), Node::Text(expression.into()));
    }
}

#[test]
fn lossless_text_capability_detects_nested_comparisons_only_when_needed() {
    let literal = serde_json::json!({"to": {"eq": "\"busy\" status"}});
    for value in [
        literal.clone(),
        serde_json::json!({"not": literal}),
        serde_json::json!({"all": [{"any": [literal]}]}),
        serde_json::json!({"forall": {"in": "rows", "as": "row", "that": literal}}),
        serde_json::json!({"exists": {"in": "rows", "as": "row", "that": literal}}),
    ] {
        let node: Node = serde_json::from_value(value).unwrap();
        assert!(Predicate::from_node(&node)
            .unwrap()
            .requires_structured_text_comparison());
    }
    for value in [
        serde_json::json!({"to": {"eq": "Ready"}}),
        serde_json::json!({"all": ["to == true", "to == \"a.b\"", {"not": "disabled"}]}),
    ] {
        let node: Node = serde_json::from_value(value).unwrap();
        assert!(!Predicate::from_node(&node)
            .unwrap()
            .requires_structured_text_comparison());
    }
}
