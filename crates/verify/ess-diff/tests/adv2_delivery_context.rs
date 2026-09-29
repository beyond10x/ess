//! Adversary pass 2 on beyond10x/ess#195, after correction 1: a context change `ess-diff` reports
//! as a cause change must say what changed.
//!
//! `compare_bindings` reports a cause change when the resolved contexts differ, and the resolved
//! context carries each field's `naming` (wire, display, summary). `written_cause` rebuilds the
//! reported cause with `Field::new(name, type)`, which drops the naming, so a naming-only change is
//! reported with two identical sides, in the text and in the JSON.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::SemanticChange;
use ess_domain::{system::Source, RawSpecFile, Specification};

const INBOX: &str = include_str!("../../ess-conformance/tests/fixtures/delivery-context.yaml");
const CONTEXT_FIELD: &str = "        - {name: account_id, type: demo.inbox.AccountId}\n";

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("inbox.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("inbox.yaml", text);
    compile_locating(&specification, &sources, &["inbox.yaml"]).unwrap()
}

fn with_context_field(field: &str) -> String {
    assert_eq!(INBOX.matches(CONTEXT_FIELD).count(), 1);
    INBOX.replacen(CONTEXT_FIELD, field, 1)
}

/// Every binding change the delta reports names a difference: its rendered before and after
/// differ, and so do the two sides it carries in canonical JSON.
fn assert_every_binding_change_says_what_changed(before: &str, after: &str) {
    let delta = ess_diff::diff(&model(before), &model(after)).unwrap();
    let json = delta.to_canonical_json();
    for change in delta.changes() {
        let SemanticChange::Binding { changed, .. } = change else {
            continue;
        };
        let text = changed.describe();
        if let Some((_, sides)) = text.split_once(": ") {
            if let Some((was, is)) = sides.split_once(" → ") {
                assert_ne!(
                    was, is,
                    "the rendered change does not say what changed: {text}\n{json}"
                );
            }
        }
        let value = serde_json::to_value(changed).unwrap();
        if value["kind"] == "cause-changed" {
            assert_ne!(
                value["before"], value["after"],
                "a cause change carries one cause twice: {json}"
            );
        }
    }
}

/// The context field's wire name moves; the host binds it from the channel under that name.
#[test]
fn adv2_a_context_field_wire_rename_says_what_changed() {
    assert_every_binding_change_says_what_changed(
        INBOX,
        &with_context_field(
            "        - {name: account_id, type: demo.inbox.AccountId, wire: accountId}\n",
        ),
    );
}

/// A one-line summary is added to the context field: documentation only.
#[test]
fn adv2_a_context_field_summary_says_what_changed() {
    assert_every_binding_change_says_what_changed(
        INBOX,
        &with_context_field(
            "        - {name: account_id, type: demo.inbox.AccountId, summary: The subscription's account.}\n",
        ),
    );
}
