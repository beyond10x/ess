//! Adversarial cases against the `ess-history-adapter/1` reader (`story:recorded-log-adapter-domain`).
//!
//! The story's claim is that `schemas/ess-history-adapter.schema.json` admits exactly the documents
//! `recorded::adapter` admits, and the brief's rule is that import behaviour does not change. The
//! reader's own doc says a source is "the word `absent` or `{ pointer: /… }`, and no other
//! spelling", every other shape "refused where it stands". Each case below holds it to one of those.

use ess_conformance::history::{Completion, SpecDigest};
use ess_conformance::recorded::{self, CompletionSource};
use serde_json::{json, Value};

const ADAPTER: &str = include_str!("fixtures/recorded/adapter.yaml");

/// The committed adapter with one exact edit, which must be present to apply.
fn edited(from: &str, to: &str) -> String {
    assert_eq!(
        ADAPTER.matches(from).count(),
        1,
        "`{from}` occurs once in the committed adapter"
    );
    ADAPTER.replacen(from, to, 1)
}

fn assert_refused_at(text: &str, at: &str) {
    match recorded::adapter(text) {
        Ok(adapter) => panic!(
            "the reader admits an undeclared spelling (the base reader refused it): {:?}",
            adapter.fields
        ),
        Err(refusal) => {
            assert_eq!(refusal.code(), "import.adapter-malformed");
            assert!(
                refusal.to_string().contains(at),
                "the refusal names `{at}`: {refusal}"
            );
        }
    }
}

/// A YAML tag on a source's pointer. The base reader (`#[serde(untagged)]`) refused it:
/// "untagged and internally tagged enums do not support enum input". The hand-written reader reads
/// the mapping in place and the tag is dropped, so `!x /request/client` reads as `/request/client`.
/// A tag on the source itself (`client: !x { pointer: … }`) is still refused, so the reader is not
/// even consistent with itself.
#[test]
fn a_tagged_pointer_is_refused_as_the_base_reader_refused_it() {
    assert_refused_at(
        &edited(
            "    pointer: /request/client\n",
            "    pointer: !x /request/client\n",
        ),
        "fields.client",
    );
}

/// A YAML tag on the completion's `values` map: refused by the base reader, admitted now.
#[test]
fn a_tagged_values_map_is_refused_as_the_base_reader_refused_it() {
    assert_refused_at(
        &edited("    values:\n", "    values: !x\n"),
        "fields.completion",
    );
}

/// `ok: !Returned` is `serde_yaml`'s tagged spelling of the enum value `Returned`: the YAML form of
/// the one-key map `{ Returned: null }` the adapter-document table refuses as "undeclared, admitted
/// only through serde's untagged buffer". The base reader refused it; the new one admits it, and a
/// YAML-to-JSON conversion of it is not a document the schema admits.
#[test]
fn a_completion_value_written_as_a_yaml_tag_is_refused() {
    assert_refused_at(
        &edited("      ok: Returned\n", "      ok: !Returned\n"),
        "fields.completion",
    );
}

/// A JSON document whose pointer holds a character outside the Basic Multilingual Plane, written
/// the way RFC 8259 section 7 spells it as an escape: a surrogate pair. Every JSON reader — and so
/// the schema — reads it as the same document as the one with the raw character. The reader reads
/// JSON through YAML, and refuses the escaped spelling: the schema admits a document the reader
/// refuses. `json.dumps` writes every such character this way by default.
#[test]
fn a_json_surrogate_pair_escape_is_the_same_document_as_the_raw_character() {
    let document = |pointer: &str| {
        format!(
            r#"{{"format": "ess-history-adapter/1", "fields": {{"operation_id": "absent", "client": {{"pointer": "{pointer}"}}, "command": "absent", "subject_key": "absent", "invoked_at": "absent", "returned_at": "absent", "outcome": "absent", "completion": "absent"}}}}"#
        )
    };
    let raw = document("/\u{1F600}");
    // `/` then the escape `😀`, spelled in pieces so no tool folds it into the character.
    let escaped = document(&["/", "\\", "ud83d", "\\", "ude00"].concat());
    assert!(
        escaped.contains("\\ud83d"),
        "the document carries the escape"
    );
    assert_eq!(
        serde_json::from_str::<Value>(&raw).expect("raw is JSON"),
        serde_json::from_str::<Value>(&escaped).expect("escaped is JSON"),
        "one JSON document, two spellings"
    );
    let admitted = recorded::adapter(&raw).expect("the raw spelling is admitted");
    assert_eq!(
        recorded::adapter(&escaped)
            .unwrap_or_else(|refusal| panic!("the escaped spelling is refused: {refusal}")),
        admitted
    );
}

/// An adapter may map a numeric completion word: `values: { 200: Returned }` is admitted, and the
/// map holds the word `200`. A log line whose status is the number 200 is then refused as "200 is
/// not a word the adapter's `values` maps" — the adapter it names maps exactly that word. An
/// admitted adapter declares a mapping import can never apply.
#[test]
fn a_numeric_completion_word_the_adapter_maps_is_mapped() {
    let adapter = recorded::adapter(&edited(
        "      ok: Returned\n      timeout: Indeterminate\n",
        "      200: Returned\n      504: Indeterminate\n",
    ))
    .expect("an adapter mapping status codes is admitted");
    let CompletionSource::Mapped(mapped) = &adapter.fields.completion else {
        panic!("the completion is mapped");
    };
    assert_eq!(mapped.values.get("200"), Some(&Completion::Returned));
    let line = json!({
        "correlation": "3f0c6a8e-1d2b-4c5a-9e7f-0a1b2c3d4e5f",
        "request": {
            "client": "worker-0",
            "command": "billing.invoice.Issue",
            "subject": "inv-1",
            "at_ms": 10,
        },
        "response": { "status": 200, "outcome": "Ok", "at_ms": 20 },
    });
    let digest = SpecDigest::new("0123456789abcdef0123456789abcdef").expect("a digest");
    let imported = recorded::import(format!("{line}\n").as_bytes(), &adapter, &digest)
        .unwrap_or_else(|refusal| panic!("the adapter maps 200 to Returned: {refusal}"));
    assert_eq!(
        imported.history.operations[0].completion,
        Completion::Returned
    );
}
