//! Adversarial cases, second pass, against the `ess-history-adapter/1` reader
//! (`story:recorded-log-adapter-domain`).
//!
//! The reader's doc (`recorded::adapter`) says: a `{`-leading document is JSON and "means what it
//! means to every JSON reader"; any other document is YAML and "a YAML tag anywhere in it is
//! refused"; a `!` that is text "leaves no tagged value, and the document is read as written"; and
//! a word is written once. Each case holds the reader to one of those sentences.

use ess_conformance::history::Completion;
use ess_conformance::recorded::{self, CompletionSource, FieldSource};

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

fn assert_refused(text: &str, why: &str) {
    match recorded::adapter(text) {
        Ok(adapter) => panic!("{why}; the reader admits it: {:?}", adapter.fields),
        Err(refusal) => assert_eq!(refusal.code(), "import.adapter-malformed"),
    }
}

fn mapped(adapter: &recorded::Adapter) -> &recorded::MappedCompletion {
    let CompletionSource::Mapped(mapped) = &adapter.fields.completion else {
        panic!("the completion is mapped");
    };
    mapped
}

/// A JSON adapter whose `values` object is spelled by `values` (raw JSON text).
fn json_adapter(client: &str, values: &str) -> String {
    format!(
        r#"{{"format": "ess-history-adapter/1", "fields": {{"operation_id": "absent", "client": {client}, "command": "absent", "subject_key": "absent", "invoked_at": "absent", "returned_at": "absent", "outcome": "absent", "completion": {{"pointer": "/response/status", "values": {values}}}}}}}"#
    )
}

/// A core-schema tag written with no space after a JSON-like key's `:` inside a flow mapping —
/// `{"pointer":!!str /request/client}` — is a tag (YAML 1.2 allows the value adjacent to the `:`
/// after a quoted key in flow context).
///
/// Re-pinned in correction round 3 by coordinator decision: YAML's core-schema tags are standard
/// YAML and resolve to the plain value, so the core-tagged document, spaced or adjacent, reads as
/// the untagged one. A local tag (`!x`) in the same positions is refused.
#[test]
fn a_core_tag_adjacent_to_a_flow_key_colon_is_refused() {
    let untagged = recorded::adapter(ADAPTER).expect("the committed adapter is admitted");
    for spelling in [
        "  client: {\"pointer\": !!str /request/client}\n",
        "  client: {\"pointer\":!!str /request/client}\n",
    ] {
        let core = recorded::adapter(&edited(
            "  client:\n    pointer: /request/client\n",
            spelling,
        ))
        .unwrap_or_else(|refusal| panic!("{spelling:?}: a core tag is refused: {refusal}"));
        assert_eq!(core, untagged, "{spelling:?} reads as the untagged adapter");
    }
    let spaced = edited(
        "  client:\n    pointer: /request/client\n",
        "  client: {\"pointer\": !x /request/client}\n",
    );
    assert_refused(&spaced, "guard: the spaced local tag is a tag");
    let adjacent = edited(
        "  client:\n    pointer: /request/client\n",
        "  client: {\"pointer\":!x /request/client}\n",
    );
    assert_refused(&adjacent, "`!x` adjacent to `:` is the same YAML tag");
}

/// A values word that is text holding `!` followed by an escaped quote: `"done !\""`. The `!`
/// follows a space, so `tag_tokens` takes `!\` as a token, stopping at the `"` the backslash
/// escapes. Rewriting it to `!ess-tag-0` deletes the backslash, the string now closes early, the
/// rewritten text does not parse, and a valid, tag-free adapter is refused as "begins like a YAML
/// tag".
#[test]
fn a_bang_before_an_escaped_quote_in_a_quoted_word_is_text() {
    let text = edited(
        "      timeout: Indeterminate\n",
        "      timeout: Indeterminate\n      \"done !\\\"\": Returned\n",
    );
    let adapter = recorded::adapter(&text).unwrap_or_else(|refusal| {
        panic!("a tag-free adapter is refused: {refusal}");
    });
    assert_eq!(
        mapped(&adapter).values.get("done !\""),
        Some(&Completion::Returned)
    );
}

/// libyaml refuses a simple (implicit) key longer than 1024 characters. A quoted word just under
/// that limit holding ` !` is a valid adapter; the rewrite lengthens the `!` to `!ess-tag-0`,
/// pushes the key over the limit, the rewritten text does not parse, and the valid adapter is
/// refused as "begins like a YAML tag".
#[test]
fn a_long_word_holding_a_bang_is_not_refused_as_a_tag() {
    let word = format!("{} !", "x".repeat(1016));
    let text = edited(
        "      timeout: Indeterminate\n",
        &format!("      timeout: Indeterminate\n      \"{word}\": Returned\n"),
    );
    let adapter = recorded::adapter(&text)
        .unwrap_or_else(|refusal| panic!("a tag-free adapter is refused: {refusal}"));
    assert_eq!(
        mapped(&adapter).values.get(&word),
        Some(&Completion::Returned)
    );
}

/// A JSON adapter that maps the word `ok` twice. The YAML spelling of the same document is refused
/// ("the word `ok` is mapped twice"); the JSON spelling goes through `serde_json::Value`, whose
/// object keeps the last member, so it is admitted and `ok` silently means `Indeterminate`.
#[test]
fn a_json_word_mapped_twice_is_refused_as_its_yaml_spelling_is() {
    let yaml = edited(
        "      timeout: Indeterminate\n",
        "      timeout: Indeterminate\n      ok: Indeterminate\n",
    );
    assert_refused(&yaml, "guard: the YAML spelling maps `ok` twice");
    let json = json_adapter(
        r#"{"pointer": "/request/client"}"#,
        r#"{"ok": "Returned", "ok": "Indeterminate"}"#,
    );
    assert_refused(&json, "the JSON spelling maps `ok` twice");
}

/// A JSON adapter that names the source `client` twice. The base reader read JSON through
/// `serde_yaml` into the derived struct, which refuses a duplicate field; the JSON-first reader
/// keeps the last member and admits it.
#[test]
fn a_json_field_written_twice_is_refused_as_the_base_reader_refused_it() {
    let json = r#"{"format": "ess-history-adapter/1", "fields": {"operation_id": "absent", "client": "absent", "client": {"pointer": "/request/client"}, "command": "absent", "subject_key": "absent", "invoked_at": "absent", "returned_at": "absent", "outcome": "absent", "completion": "absent"}}"#;
    assert_refused(json, "the JSON spelling names `client` twice");
}

/// A YAML flow-style adapter — valid YAML, not JSON (its keys are unquoted) — was admitted by the
/// base reader. It opens with `{`, so the reader now hands it to `serde_json` and refuses it with a
/// JSON syntax error, though the reader's doc says it reads documents "written as YAML or JSON".
#[test]
fn a_yaml_flow_style_adapter_is_read_as_yaml() {
    let text = "{format: ess-history-adapter/1, fields: {operation_id: absent, client: {pointer: /request/client}, command: absent, subject_key: absent, invoked_at: absent, returned_at: absent, outcome: absent, completion: absent}}\n";
    let adapter = recorded::adapter(text)
        .unwrap_or_else(|refusal| panic!("a YAML flow-style adapter is refused: {refusal}"));
    assert_eq!(
        adapter.fields.client,
        FieldSource::Pointer(recorded::Pointer {
            pointer: "/request/client".to_owned()
        })
    );
}
