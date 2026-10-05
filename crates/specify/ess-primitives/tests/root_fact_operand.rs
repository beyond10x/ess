//! Family F part A1 (beyond10x/ess#225, #233): a one-segment fact on the right of a comparison.
//!
//! `a == b` cannot be read back as a comparison of two facts without knowing that `b` is a root of
//! the place it is written in, so the canonical writer spells such an operand as an explicit
//! mapping, `a: {eq: {fact: b}}`, and the reader takes that mapping as a fact. A binder or a dotted
//! path is unambiguous already and keeps its compact bytes. `docs/design/expression-family-source22.md`
//! is the design; the source-format resolver that decides when `b` is a root lives in `ess-domain`.

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Quantified, Truth};

fn path(text: &str) -> FactPath {
    text.parse().expect("a fact path")
}

fn facts(left: &str, op: CompareOp, right: &str) -> Predicate {
    Predicate::Compare {
        kind: ess_primitives::predicate::CompareKind::Value,
        left: Operand::Fact(path(left)),
        op,
        right: Operand::Fact(path(right)),
    }
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("a predicate serialises")
}

fn read(text: &str) -> Result<Predicate, String> {
    serde_json::from_str::<Predicate>(text).map_err(|error| error.to_string())
}

#[test]
fn the_canonical_fact_mapping_reads_as_a_fact_under_every_operator_spelling() {
    for (keyword, op) in [
        ("eq", CompareOp::Eq),
        ("==", CompareOp::Eq),
        ("ne", CompareOp::Ne),
        ("lt", CompareOp::Lt),
        ("lte", CompareOp::Le),
        ("gt", CompareOp::Gt),
        ("gte", CompareOp::Ge),
    ] {
        let text = format!(r#"{{"upper": {{"{keyword}": {{"fact": "lower"}}}}}}"#);
        assert_eq!(read(&text), Ok(facts("upper", op, "lower")), "{text}");
    }
    assert_eq!(
        read(r#"{"upper": {"eq": {"fact": "window.lower"}}}"#),
        Ok(facts("upper", CompareOp::Eq, "window.lower")),
        "a dotted path in the mapping is the same fact"
    );
}

#[test]
fn a_one_segment_root_fact_is_written_as_the_explicit_mapping_and_reads_back() {
    for op in [
        CompareOp::Eq,
        CompareOp::Ne,
        CompareOp::Lt,
        CompareOp::Le,
        CompareOp::Gt,
        CompareOp::Ge,
    ] {
        let predicate = facts("task_id", op, "depends_on");
        let written = json(&predicate);
        assert_eq!(
            written,
            format!(
                r#"{{"task_id":{{"{}":{{"fact":"depends_on"}}}}}}"#,
                op.keyword()
            ),
        );
        assert_eq!(read(&written), Ok(predicate.clone()), "{written}");
        assert!(predicate.reads_root_fact_operand(), "{written}");
    }
}

#[test]
fn a_dotted_fact_keeps_its_compact_bytes() {
    let predicate = facts("window.start", CompareOp::Lt, "window.end");
    assert_eq!(json(&predicate), r#""window.start < window.end""#);
    assert!(!predicate.reads_root_fact_operand());
}

#[test]
fn a1_binder_shadowing_compact_bytes_at_the_writer() {
    // `b` is the inner binder: compact, as #289 shipped it, and no root fact is read.
    let predicate = Predicate::Forall(Box::new(Quantified {
        over: path("tags"),
        bind: "a".to_owned(),
        body: Predicate::Forall(Box::new(Quantified {
            over: path("banned"),
            bind: "b".to_owned(),
            body: facts("a", CompareOp::Ne, "b"),
        })),
    }));
    let written = json(&predicate);
    assert_eq!(
        written,
        r#"{"forall":{"as":"a","in":"tags","that":{"forall":{"as":"b","in":"banned","that":"a != b"}}}}"#
    );
    assert_eq!(read(&written), Ok(predicate.clone()));
    assert!(!predicate.reads_root_fact_operand());

    // Outside the binder's scope the same word is a root fact again.
    let outside = Predicate::all(vec![
        Predicate::Forall(Box::new(Quantified {
            over: path("banned"),
            bind: "b".to_owned(),
            body: facts("b", CompareOp::Ne, "x"),
        })),
        facts("tag", CompareOp::Ne, "b"),
    ]);
    let written = json(&outside);
    assert!(
        written.contains(r#"{"b":{"ne":{"fact":"x"}}}"#),
        "{written}"
    );
    assert!(
        written.contains(r#"{"tag":{"ne":{"fact":"b"}}}"#),
        "{written}"
    );
    assert_eq!(read(&written), Ok(outside.clone()));
    assert!(outside.reads_root_fact_operand());
}

#[test]
fn display_tells_a_root_fact_from_the_text_it_is_spelled_like() {
    let fact = facts("task_id", CompareOp::Eq, "depends_on");
    let text = Predicate::Compare {
        kind: ess_primitives::predicate::CompareKind::Value,
        left: Operand::Fact(path("task_id")),
        op: CompareOp::Eq,
        right: Operand::Literal(FactValue::text("depends_on")),
    };
    assert_eq!(fact.to_string(), "task_id == {fact: depends_on}");
    assert_eq!(text.to_string(), "task_id == depends_on");
    assert_ne!(fact.to_string(), text.to_string());
    let bound = Predicate::Exists(Box::new(Quantified {
        over: path("banned"),
        bind: "b".to_owned(),
        body: facts("tag", CompareOp::Eq, "b"),
    }));
    assert_eq!(bound.to_string(), "exists b in banned: (tag == b)");
}

#[test]
fn a_malformed_fact_mapping_is_refused_rather_than_read_as_a_literal() {
    for text in [
        r#"{"a": {"eq": {"fact": 1}}}"#,
        r#"{"a": {"eq": {"fact": "b", "add": 1}}}"#,
        r#"{"a": {"eq": {"facts": "b"}}}"#,
        r#"{"a": {"eq": {"fact": "not a path"}}}"#,
        r#"{"a": {"eq": {}}}"#,
    ] {
        assert!(read(text).is_err(), "{text} is refused: {:?}", read(text));
    }
}

#[test]
fn the_spelled_parse_marks_exactly_the_unquoted_undotted_words() {
    let cases: &[(&str, &[bool])] = &[
        (r#""a == b""#, &[true]),
        (r#""a == \"b\"""#, &[false]),
        (r#""a == 'b'""#, &[false]),
        (r#"{"a": {"eq": "b"}}"#, &[true]),
        (r#"{"a": {"eq": "\"b\""}}"#, &[false]),
        (r#"{"a": "b"}"#, &[false]),
        (r#""a == b.c""#, &[false]),
        (r#""a == 1""#, &[false]),
        (r#""a == true""#, &[false]),
        (r#""a == 1.5""#, &[false]),
        (r#"{"a": {"eq": {"fact": "b"}}}"#, &[false]),
        (r#"{"a": {"eq": 3}}"#, &[false]),
        (
            r#"{"forall": {"in": "tags", "as": "b", "that": "tag != b"}}"#,
            &[false],
        ),
        (
            r#"{"all": ["a == b", "c == \"d\"", {"not": "e < f"}]}"#,
            &[true, false, true],
        ),
        (r#"{"x": {"gte": "y", "lt": "z.w"}}"#, &[true, false]),
        (r#""defined(a)""#, &[]),
        (r#""a == hello world""#, &[false]),
    ];
    for (text, want) in cases {
        let node: Node = serde_json::from_str(text).expect("json");
        let spelled = Predicate::from_node_spelled(&node).expect("parses");
        assert_eq!(spelled.words, *want, "{text}");
        assert_eq!(
            Ok(spelled.predicate),
            Predicate::from_node(&node).map_err(|error| error.to_string()),
            "the spelled parse reads the predicate the ordinary parse reads: {text}"
        );
    }
    let compact = Predicate::parse_expression_spelled("not a == b").expect("parses");
    assert_eq!(compact.words, vec![true]);
}

#[test]
fn an_absent_operand_is_unknown_and_never_the_text_it_is_spelled_like() {
    let predicate = facts("task_id", CompareOp::Ne, "depends_on");
    let mut store = FactStore::new();
    store.set_path("task_id", FactValue::text("depends_on"));
    assert_eq!(predicate.evaluate(&store), Truth::Unknown);
    store.set_path("depends_on", FactValue::text("depends_on"));
    assert_eq!(predicate.evaluate(&store), Truth::False);
    store.set_path("depends_on", FactValue::text("other"));
    assert_eq!(predicate.evaluate(&store), Truth::True);
}

/// The shared vectors the Go and TypeScript readers answer too.
const VECTORS: &str = include_str!("vectors/root-fact-operand.json");

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
        serde_json::Value::Array(items) => {
            store.set_path(&format!("{path}.count"), FactValue::count(items.len()));
            for (index, item) in items.iter().enumerate() {
                flatten(store, &format!("{path}.{index}"), item);
            }
        }
        serde_json::Value::String(text) => store.set_path(path, FactValue::text(text.clone())),
        serde_json::Value::Number(number) => store.set_path(
            path,
            FactValue::number(number.as_f64().expect("finite")).expect("a number"),
        ),
        serde_json::Value::Bool(value) => store.set_path(path, FactValue::Bool(*value)),
        serde_json::Value::Null => {}
    }
}

#[test]
fn the_rust_reader_answers_the_shared_vectors() {
    let vectors: serde_json::Value = serde_json::from_str(VECTORS).expect("json");
    for vector in vectors["evaluate"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate: Predicate = serde_json::from_value(vector["predicate"].clone())
            .unwrap_or_else(|error| {
                panic!("{name}: {error}");
            });
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
