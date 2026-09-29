//! A delivery-context change as `ess verify diff` reports it (beyond10x/ess#195): the cause
//! change that holds an `external` cause is `ess-diff/10` vocabulary, and its rendered text names
//! the channel and the context fields, so a context-only change reads as one.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::SemanticChange;
use ess_domain::{system::Source, RawSpecFile, Specification};

const INBOX: &str = include_str!("../../ess-conformance/tests/fixtures/delivery-context.yaml");

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("inbox.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("inbox.yaml", text);
    compile_locating(&specification, &sources, &["inbox.yaml"]).unwrap()
}

fn described(after: &str) -> (ess_diff::EssDelta, Vec<String>) {
    let delta = ess_diff::diff(&model(INBOX), &model(after)).unwrap();
    let texts = delta
        .changes()
        .iter()
        .filter_map(|change| match change {
            SemanticChange::Binding { changed, .. } => Some(changed.describe()),
            _ => None,
        })
        .collect();
    (delta, texts)
}

#[test]
fn a_cause_change_holding_an_external_cause_is_ess_diff_10() {
    let (delta, _) = described(&INBOX.replacen(
        "context_authority: account-messages",
        "context_authority: tenant-messages",
        1,
    ));
    assert_eq!(delta.format.to_string(), "ess-diff/10");
    let json = delta.to_canonical_json();
    let older = json.replace("\"ess-diff/10\"", "\"ess-diff/9\"");
    assert_ne!(older, json);
    let raw: ess_diff::RawEssDelta = serde_json::from_str(&older).unwrap();
    let refused = ess_diff::EssDelta::try_from(raw).expect_err("an ess-diff/9 reader refuses");
    assert!(
        format!("{refused:?}").contains("UnsupportedFormatVersion"),
        "{refused:?}"
    );
}

#[test]
fn a_context_field_change_names_the_fields_on_both_sides() {
    let (_, texts) = described(&INBOX.replacen(
        "        - {name: account_id, type: demo.inbox.AccountId}\n",
        "        - {name: account_id, type: demo.inbox.AccountId}\n        - {name: shard, type: Integer}\n",
        1,
    ));
    assert!(
        texts
            .iter()
            .any(|text| text.contains("account-messages") && text.contains("shard: Integer")),
        "{texts:#?}"
    );
}

const CONTEXT_FIELD: &str = "        - {name: account_id, type: demo.inbox.AccountId}\n";

/// Every kind the delta reports, and its format.
fn kinds(after_field: &str) -> (String, Vec<String>) {
    assert_eq!(INBOX.matches(CONTEXT_FIELD).count(), 1);
    let delta = ess_diff::diff(
        &model(INBOX),
        &model(&INBOX.replacen(CONTEXT_FIELD, after_field, 1)),
    )
    .unwrap();
    let kinds = delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect();
    (delta.format.to_string(), kinds)
}

/// The wire name a channel binds a context field under is part of the cause, and the cause says
/// it on the side that has it — and nothing else is reported for it.
#[test]
fn a_context_field_wire_rename_is_a_cause_change_that_names_the_wire() {
    let (_, texts) = described(&INBOX.replacen(
        CONTEXT_FIELD,
        "        - {name: account_id, type: demo.inbox.AccountId, wire: accountId}\n",
        1,
    ));
    assert_eq!(texts.len(), 1, "{texts:#?}");
    assert!(texts[0].contains("(wire `accountId`)"), "{texts:#?}");
    let (format, kinds) =
        kinds("        - {name: account_id, type: demo.inbox.AccountId, wire: accountId}\n");
    assert_eq!(format, "ess-diff/10");
    assert!(
        !kinds.iter().any(|kind| kind.contains("unclassified")),
        "{kinds:#?}"
    );
}

/// `wire:` written out as the field's own name is the same wire name: no change.
#[test]
fn a_context_field_wire_written_as_its_own_name_is_no_change() {
    let (_, kinds) =
        kinds("        - {name: account_id, type: demo.inbox.AccountId, wire: account_id}\n");
    assert!(kinds.is_empty(), "{kinds:#?}");
}

/// A summary or display name on a context field is documentation: its own kind, never a cause
/// change and never an unclassified change.
#[test]
fn a_context_field_summary_or_display_is_documentation_not_a_cause_change() {
    for (field, kind) in [
        (
            "        - {name: account_id, type: demo.inbox.AccountId, summary: The account.}\n",
            "context-field-summary-changed",
        ),
        (
            "        - {name: account_id, type: demo.inbox.AccountId, display: Account}\n",
            "context-field-display-changed",
        ),
    ] {
        let (_, texts) = described(&INBOX.replacen(CONTEXT_FIELD, field, 1));
        assert_eq!(texts.len(), 1, "{texts:#?}");
        assert!(
            texts[0].contains("context field `account_id`"),
            "{texts:#?}"
        );
        let (format, kinds) = kinds(field);
        assert_eq!(kinds.len(), 1, "{kinds:#?}");
        assert!(kinds[0].contains(kind), "{kinds:#?}");
        assert_eq!(format, "ess-diff/10");
    }
}
