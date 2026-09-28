//! The reader's half of the adapter-document table.
//!
//! `tests/fixtures/recorded/adapter-documents.json` lists `ess-history-adapter/1` documents with the
//! verdict [`recorded::adapter`] gives each. This file holds the reader to those verdicts;
//! `crates/specify/ess-domain/tests/ess_history_adapter_schema.rs` holds
//! `schemas/ess-history-adapter.schema.json` to the same ones. A document the reader and the schema
//! disagree on fails one of the two.

use ess_conformance::history::Completion;
use ess_conformance::recorded::{self, CompletionSource, WordKind};
use serde_json::{json, Value};

const TABLE: &str = include_str!("fixtures/recorded/adapter-documents.json");

/// `base` with each edit applied: `{pointer, value}` sets, `{pointer, remove: true}` removes.
fn apply(base: &Value, edits: &[Value]) -> Value {
    let mut document = base.clone();
    for edit in edits {
        let pointer = edit["pointer"].as_str().expect("an edit names a pointer");
        if pointer.is_empty() {
            document = edit["value"].clone();
            continue;
        }
        let (parent, key) = pointer.rsplit_once('/').expect("a pointer below the root");
        let object = document
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .unwrap_or_else(|| panic!("the edit's parent `{parent}` is an object"));
        if edit.get("remove") == Some(&Value::Bool(true)) {
            assert!(
                object.remove(key).is_some(),
                "`{pointer}` is there to remove"
            );
        } else {
            object.insert(key.to_owned(), edit["value"].clone());
        }
    }
    document
}

#[test]
fn the_reader_gives_every_tabled_document_its_verdict() {
    let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
    let cases = table["cases"].as_array().expect("the table lists cases");
    assert!(
        cases.len() >= 25,
        "the table holds too few cases to say anything"
    );
    let mut disagreements = Vec::new();
    for case in cases {
        let name = case["name"].as_str().expect("a case is named");
        let admitted = case["admitted"].as_bool().expect("a case has a verdict");
        let document = apply(
            &table["base"],
            case["edits"].as_array().expect("a case lists its edits"),
        );
        let text = serde_json::to_string(&document).expect("a document serializes");
        match (recorded::adapter(&text), admitted) {
            (Ok(_), true) => {}
            (Err(refusal), false) => {
                assert_eq!(
                    refusal.code(),
                    "import.adapter-malformed",
                    "{name}: refused as a malformed adapter"
                );
                // `at`: the place inside the document the refusal must name.
                if let Some(at) = case.get("at").and_then(Value::as_str) {
                    if !refusal.to_string().contains(at) {
                        disagreements.push(format!(
                            "{name}: the refusal does not name `{at}`: {refusal}"
                        ));
                    }
                }
            }
            (Ok(_), false) => disagreements.push(format!("{name}: the reader admits it")),
            (Err(refusal), true) => {
                disagreements.push(format!("{name}: the reader refuses it: {refusal}"));
            }
        }
    }
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n"));
}

/// The table's base is the committed adapter fixture the import tests read, written as JSON.
#[test]
fn the_tables_base_is_the_committed_adapter() {
    let table: Value = serde_json::from_str(TABLE).expect("the table is JSON");
    let base = serde_json::to_string(&table["base"]).expect("the base serializes");
    assert_eq!(
        recorded::adapter(&base).expect("the base is admitted"),
        recorded::adapter(include_str!("fixtures/recorded/adapter.yaml"))
            .expect("the committed adapter is admitted"),
    );
}

// What the JSON table cannot say: YAML integer words, and YAML tags.

const ADAPTER: &str = include_str!("fixtures/recorded/adapter.yaml");

fn with_values(values: &str) -> String {
    let from = "      ok: Returned\n      timeout: Indeterminate\n";
    assert_eq!(
        ADAPTER.matches(from).count(),
        1,
        "the fixture spells its values once"
    );
    ADAPTER.replacen(from, values, 1)
}

fn words(text: &str) -> recorded::Words<Completion> {
    let adapter = recorded::adapter(text).unwrap_or_else(|refusal| panic!("admitted: {refusal}"));
    match adapter.fields.completion {
        CompletionSource::Mapped(mapped) => mapped.values,
        CompletionSource::Absent(_) => panic!("the completion is mapped"),
    }
}

/// An integer word maps log integers and a string word log strings, never the other.
#[test]
fn integer_and_string_words_each_map_their_own_kind() {
    let words = words(&with_values(
        "      200: Returned\n      \"201\": Indeterminate\n",
    ));
    assert_eq!(words.kind("200"), Some(WordKind::Integer));
    assert_eq!(words.kind("201"), Some(WordKind::Text));
    assert_eq!(words.meaning(&json!(200)), Some(&Completion::Returned));
    assert_eq!(words.meaning(&json!("200")), None);
    assert_eq!(
        words.meaning(&json!("201")),
        Some(&Completion::Indeterminate)
    );
    assert_eq!(words.meaning(&json!(201)), None);
    assert_eq!(
        words.meaning(&json!(200.0)),
        None,
        "a float is not the integer 200"
    );
}

/// A JSON object's keys are strings, so a JSON adapter's `"200"` is a string word.
#[test]
fn a_json_numeric_key_is_a_string_word() {
    let base: Value = serde_json::from_str::<Value>(TABLE).expect("JSON")["base"].clone();
    let mut document = base;
    document["fields"]["completion"]["values"] = json!({ "200": "Returned" });
    let words = words(&serde_json::to_string(&document).expect("serializes"));
    assert_eq!(words.kind("200"), Some(WordKind::Text));
    assert_eq!(words.meaning(&json!("200")), Some(&Completion::Returned));
    assert_eq!(words.meaning(&json!(200)), None);
}

/// A string and an integer spelled alike are one word, and a word is mapped once.
#[test]
fn a_word_mapped_as_a_string_and_as_an_integer_is_refused() {
    let refusal = recorded::adapter(&with_values(
        "      200: Returned\n      \"200\": Indeterminate\n",
    ))
    .expect_err("one word, mapped twice");
    assert_eq!(refusal.code(), "import.adapter-malformed");
    let text = refusal.to_string();
    assert!(text.contains("fields.completion.values"), "{text}");
    assert!(text.contains("mapped twice"), "{text}");
}

/// A word that is neither a string nor an integer is refused.
#[test]
fn a_word_that_is_neither_a_string_nor_an_integer_is_refused() {
    for word in ["2.5", "true", "~"] {
        let refusal = recorded::adapter(&with_values(&format!("      {word}: Returned\n")))
            .expect_err("not a word");
        assert!(
            refusal.to_string().contains("fields.completion.values"),
            "{word}: {refusal}"
        );
    }
}

/// A YAML tag that is not YAML's own is refused where it stands: a local tag (`!x`, `!Returned`)
/// or a local tag written verbatim (`!<!x>`), on a value or on a key.
#[test]
fn every_yaml_tag_is_refused_with_its_path() {
    for (from, to, at) in [
        (
            "format: ess-history-adapter/1",
            "format: !x ess-history-adapter/1",
            "format",
        ),
        ("  client:\n", "  client: !x\n", "fields.client"),
        (
            "    pointer: /request/client\n",
            "    pointer: !<!x> /request/client\n",
            "fields.client",
        ),
        (
            "      ok: Returned\n",
            "      !x ok: Returned\n",
            "fields.completion.values",
        ),
        (
            "      ok: Returned\n",
            "      ok: !Returned\n",
            "fields.completion.values.ok",
        ),
    ] {
        assert_eq!(ADAPTER.matches(from).count(), 1, "`{from}` occurs once");
        let refusal = recorded::adapter(&ADAPTER.replacen(from, to, 1))
            .map(|adapter| panic!("{to:?} is admitted: {adapter:?}"))
            .expect_err("a tag is refused");
        assert_eq!(refusal.code(), "import.adapter-malformed");
        let text = refusal.to_string();
        assert!(
            text.contains(at) && text.contains("YAML tag"),
            "{to}: {text}"
        );
    }
}

/// A tag in YAML's own namespace (`!!str`, `!!int`, the verbatim `!<tag:yaml.org,2002:str>`) is
/// standard YAML: the YAML reader resolves it to the plain value it names, and the adapter reads as
/// that value.
///
/// Two more are dropped by the YAML reader without a trace, so the tagged-value walk cannot see
/// them: `!!foo`, in YAML's namespace but not a core-schema tag, and `!<x>`, a verbatim tag that
/// is neither YAML's nor local. They read as the untagged value. This is a known gap, pinned here
/// so a change to it is seen, not a decision that they are part of the format.
#[test]
fn a_tag_in_yamls_own_namespace_reads_as_the_plain_value() {
    let untagged = recorded::adapter(ADAPTER).expect("admitted");
    for (from, to) in [
        (
            "    pointer: /request/client\n",
            "    pointer: !<x> /request/client\n",
        ),
        (
            "    pointer: /request/client\n",
            "    pointer: !!str /request/client\n",
        ),
        (
            "    pointer: /request/client\n",
            "    pointer: !<tag:yaml.org,2002:str> /request/client\n",
        ),
        (
            "    pointer: /request/client\n",
            "    pointer: !!foo /request/client\n",
        ),
        ("      ok: Returned\n", "      ok: !!str Returned\n"),
    ] {
        assert_eq!(ADAPTER.matches(from).count(), 1, "`{from}` occurs once");
        let read = recorded::adapter(&ADAPTER.replacen(from, to, 1))
            .unwrap_or_else(|refusal| panic!("{to:?}: refused: {refusal}"));
        assert_eq!(read, untagged, "{to:?} reads as the untagged adapter");
    }
    let words = words(&with_values(
        "      !!int 200: Returned\n      !!str 504: Indeterminate\n",
    ));
    assert_eq!(words.kind("200"), Some(WordKind::Integer));
    assert_eq!(words.kind("504"), Some(WordKind::Text));
}

/// A `!` that is text, not a tag, is read as written.
#[test]
fn a_bang_that_is_text_is_not_a_tag() {
    for pointer in [
        "\"/request/ !client\"",
        "'/request/ !client'",
        "/request/ !client",
        "/request/!client",
    ] {
        let text = ADAPTER.replacen(
            "    pointer: /request/client\n",
            &format!("    pointer: {pointer}  # !not-a-tag\n"),
            1,
        );
        let adapter = recorded::adapter(&text)
            .unwrap_or_else(|refusal| panic!("{pointer}: refused: {refusal}"));
        let recorded::FieldSource::Pointer(read) = adapter.fields.client else {
            panic!("{pointer}: the client is a pointer");
        };
        assert!(read.pointer.contains('!'), "{pointer}: read as {read:?}");
    }
}
