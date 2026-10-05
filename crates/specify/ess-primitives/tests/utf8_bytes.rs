//! Family F part C (beyond10x/ess#233): the UTF-8 byte length of a String.
//!
//! `label.utf8_bytes <= 255` compares the number of bytes the text's UTF-8 encoding takes, which
//! `.count` — the number of Unicode scalar values — does not say. The resolved predicate carries it
//! as its own operand, `{utf8_bytes: label}`, never as the path `label.utf8_bytes`, so a declared
//! member named `utf8_bytes` keeps meaning that member and no evaluator decides by a suffix.
//! `docs/design/expression-family-source22.md`, "String `.utf8_bytes`" and final review decisions 11
//! and 19, is the design; the source-format resolver that reads the compact spelling lives in
//! `ess-domain`.

use ess_primitives::facts::{FactStore, FactValue};
use ess_primitives::predicate::{reading_source22_operands, Predicate, Truth};

fn read(text: &str) -> Result<Predicate, String> {
    serde_json::from_str::<Predicate>(text).map_err(|error| error.to_string())
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("a predicate serialises")
}

fn text(label: &str) -> FactStore {
    let mut store = FactStore::new();
    store.set_path("label", FactValue::text(label));
    store
}

#[test]
fn utf8_the_canonical_left_selector_reads_and_writes_back_byte_for_byte() {
    for written in [
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":4.0}}"#,
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"eq","right":{"utf8_bytes":"other"}}}"#,
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"ne","right":{"fact":"limit"}}}"#,
        r#"{"compare":{"left":{"utf8_bytes":"window.label"},"op":"gt","right":{"fact":"limits.max"}}}"#,
    ] {
        let parsed = read(written).unwrap_or_else(|error| panic!("{written}: {error}"));
        assert_eq!(json(&parsed), written, "written back unchanged");
        assert!(parsed.reads_utf8_bytes(), "{written} reads a byte length");
    }
}

#[test]
fn utf8_the_canonical_right_selector_is_the_tagged_operand() {
    let written = r#"{"limit":{"gte":{"utf8_bytes":"label"}}}"#;
    let parsed = read(written).expect("reads");
    assert_eq!(json(&parsed), written);
    assert!(parsed.reads_utf8_bytes());
    let mut store = text("é");
    store.set_path("limit", FactValue::count(2));
    assert_eq!(parsed.evaluate(&store), Truth::True);
    store.set_path("limit", FactValue::count(1));
    assert_eq!(parsed.evaluate(&store), Truth::False);
}

#[test]
fn utf8_a_declared_member_named_utf8_bytes_stays_an_ordinary_path() {
    let compact = read(r#""record.utf8_bytes <= 3""#).expect("reads");
    assert!(
        !compact.reads_utf8_bytes(),
        "a dotted path is a field until a resolver says otherwise"
    );
    assert_eq!(json(&compact), r#""record.utf8_bytes <= 3""#);
    let mut store = FactStore::new();
    store.set_path("record.utf8_bytes", FactValue::count(3));
    assert_eq!(compact.evaluate(&store), Truth::True);
    // A primitive evaluator never derives the selector from a suffix: the path names a field, and a
    // text bound at its parent does not answer for it.
    assert_eq!(compact.evaluate(&text("abcd")), Truth::Unknown);
}

#[test]
fn utf8_the_rendering_tells_the_selector_from_the_field() {
    let derived =
        read(r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":4}}"#).expect("reads");
    let field = read(r#""label.utf8_bytes <= 4""#).expect("reads");
    assert_eq!(derived.to_string(), "{utf8_bytes: label} <= 4");
    assert_eq!(field.to_string(), "label.utf8_bytes <= 4");
    assert_ne!(derived, field);
    let right = read(r#"{"limit":{"gte":{"utf8_bytes":"label"}}}"#).expect("reads");
    assert_eq!(right.to_string(), "limit >= {utf8_bytes: label}");
}

#[test]
fn utf8_below_source22_the_selector_mapping_is_no_operand() {
    for written in [
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":4}}"#,
        r#"{"limit":{"gte":{"utf8_bytes":"label"}}}"#,
    ] {
        let refused = reading_source22_operands(false, || read(written));
        assert!(refused.is_err(), "{written} is refused below ess/22");
    }
}

#[test]
fn utf8_counts_bytes_where_count_counts_scalars() {
    let bytes = |label: &str, bound: u32| {
        read(&format!(
            r#"{{"compare":{{"left":{{"utf8_bytes":"label"}},"op":"eq","right":{bound}}}}}"#
        ))
        .expect("reads")
        .evaluate(&text(label))
    };
    let count = |label: &str, bound: u32| {
        read(&format!(r#"{{"label.count":{{"eq":{bound}}}}}"#))
            .expect("reads")
            .evaluate(&text(label))
    };
    for (label, utf8, scalars) in [
        ("", 0, 0),
        ("abc", 3, 3),
        ("é", 2, 1),
        ("€", 3, 1),
        ("😀", 4, 1),
        ("cafe\u{301}", 6, 5),
        ("a😀é€", 10, 4),
    ] {
        assert_eq!(bytes(label, utf8), Truth::True, "{label:?} is {utf8} bytes");
        assert_eq!(
            count(label, scalars),
            Truth::True,
            "{label:?} is {scalars} scalars"
        );
        if utf8 != scalars {
            assert_eq!(
                bytes(label, scalars),
                Truth::False,
                "{label:?}: bytes, not scalars"
            );
            assert_eq!(
                count(label, utf8),
                Truth::False,
                "{label:?}: scalars, not bytes"
            );
        }
    }
}

#[test]
fn utf8_no_unicode_normalization_occurs() {
    let five = r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"eq","right":5}}"#;
    let five = read(five).expect("reads");
    assert_eq!(five.evaluate(&text("caf\u{e9}")), Truth::True, "composed");
    assert_eq!(
        five.evaluate(&text("cafe\u{301}")),
        Truth::False,
        "decomposed is 6"
    );
}

#[test]
fn utf8_a_lone_surrogate_never_reaches_a_rust_text() {
    // Rust text is UTF-8 by type: the one way a lone surrogate could arrive is a JSON escape, and
    // the JSON reader refuses it rather than measuring a replacement.
    assert!(serde_json::from_str::<String>(r#""\ud800""#).is_err());
    assert!(serde_json::from_str::<serde_json::Value>(r#"{"label":"a\udc00"}"#).is_err());
    assert!(String::from_utf16(&[0xD800]).is_err());
}

// ---- the shared vectors -----------------------------------------------------------------------

/// The shared vectors the Go and TypeScript readers answer too.
const VECTORS: &str = include_str!("vectors/utf8-bytes.json");

fn vectors() -> serde_json::Value {
    serde_json::from_str(VECTORS).expect("json")
}

fn bind(store: &mut FactStore, path: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, nested) in fields {
                let at = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                bind(store, &at, nested);
            }
        }
        serde_json::Value::Array(items) => {
            store.set_path(&format!("{path}.count"), FactValue::count(items.len()));
            for (index, item) in items.iter().enumerate() {
                bind(store, &format!("{path}.{index}"), item);
            }
        }
        serde_json::Value::String(text) => store.set_path(path, FactValue::text(text.clone())),
        serde_json::Value::Number(_) => store.set_path(
            path,
            FactValue::Number(serde_json::from_value(value.clone()).expect("a number")),
        ),
        serde_json::Value::Bool(flag) => store.set_path(path, FactValue::Bool(*flag)),
        serde_json::Value::Null => {}
    }
}

/// The text a `texts` row's code units spell, or `None` where they spell no Unicode text: a Rust
/// lane can hold no lone surrogate, so the fact is never bound and every read of it is Unknown.
fn units(vector: &serde_json::Value) -> Option<String> {
    let units: Vec<u16> = vector["units"]
        .as_array()
        .expect("units")
        .iter()
        .map(|unit| u16::try_from(unit.as_u64().expect("a unit")).expect("a code unit"))
        .collect();
    String::from_utf16(&units).ok()
}

fn bytes_predicate(bound: &serde_json::Value) -> Predicate {
    serde_json::from_value(serde_json::json!({
        "compare": {"left": {"utf8_bytes": "label"}, "op": "eq", "right": bound}
    }))
    .expect("reads")
}

/// How a lane measures a text: correctly, or wrong in the one way a fault is.
type Measure = fn(&str) -> usize;

fn scalars(text: &str) -> usize {
    text.chars().count()
}

fn utf16_units(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Graphemes, as far as the vectors need: a combining mark joins the scalar before it.
fn graphemes(text: &str) -> usize {
    text.chars()
        .filter(|character| !('\u{300}'..='\u{36f}').contains(character))
        .count()
}

#[test]
fn utf8_the_rust_evaluator_answers_the_shared_vectors() {
    let vectors = vectors();
    let mut answered = 0;
    for vector in vectors["texts"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let mut store = FactStore::new();
        if let Some(text) = units(vector) {
            store.set_path("label", FactValue::text(text));
        }
        let expected = &vector["bytes"];
        if expected.is_null() {
            assert!(units(vector).is_none(), "{name}: no Rust text holds it");
            let any = bytes_predicate(&serde_json::json!(3));
            assert_eq!(any.evaluate(&store), Truth::Unknown, "{name}");
        } else {
            assert_eq!(
                bytes_predicate(expected).evaluate(&store),
                Truth::True,
                "{name}"
            );
            let count: Predicate =
                serde_json::from_value(serde_json::json!({"label.count": {"eq": vector["count"]}}))
                    .expect("reads");
            assert_eq!(count.evaluate(&store), Truth::True, "{name}: its count");
        }
        answered += 1;
    }
    for vector in vectors["evaluate"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate: Predicate = serde_json::from_value(vector["predicate"].clone())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let mut store = FactStore::new();
        bind(&mut store, "", &vector["row"]);
        assert_eq!(
            predicate.evaluate(&store).as_str(),
            vector["truth"].as_str().expect("a truth"),
            "{name}"
        );
        answered += 1;
    }
    for vector in vectors["refused"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        assert!(
            serde_json::from_value::<Predicate>(vector["predicate"].clone()).is_err(),
            "{name} is refused"
        );
        answered += 1;
    }
    assert!(answered >= 40, "{answered} vectors answered");
}

#[test]
fn utf8_every_faulty_measure_disagrees_with_a_vector() {
    let vectors = vectors();
    let texts: Vec<(String, usize)> = vectors["texts"]
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|vector| {
            let bytes = usize::try_from(vector["bytes"].as_u64()?).ok()?;
            Some((units(vector)?, bytes))
        })
        .collect();
    let correct: Measure = str::len;
    assert!(texts.iter().all(|(text, bytes)| correct(text) == *bytes));
    for (fault, measure) in [
        ("scalar count", scalars as Measure),
        ("UTF-16 units", utf16_units as Measure),
        ("code points", scalars as Measure),
        ("graphemes", graphemes as Measure),
    ] {
        assert!(
            texts.iter().any(|(text, bytes)| measure(text) != *bytes),
            "{fault} agrees with every vector"
        );
    }
    // UTF-16 units and scalars differ too, so one vector tells those two faults apart.
    assert!(texts
        .iter()
        .any(|(text, _)| utf16_units(text) != scalars(text)));
    assert!(texts
        .iter()
        .any(|(text, _)| graphemes(text) != scalars(text)));
}
