//! Adversary pass 2 for beyond10x/ess#219: the `ess-diff/11` bump is complete over every writer
//! and reader path, and a delta mixing an `ess-diff/10` binding-context change with a prefix
//! change is written as `/11`, refused by a `/10` writer and by a `/10` reader.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::match_wildcard_for_single_variants
)]

use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_diff::{DeltaFormat, DeltaWriteRefusal, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{system::Source, RawSpecFile, Specification};

const INBOX: &str = include_str!("../../ess-conformance/tests/fixtures/delivery-context.yaml");
const ACCOUNT: &str = "{name: demo.inbox.AccountId, kind: newtype, of: String}";
const AUTHORITY: &str = "context_authority: account-messages";

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("inbox.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("inbox.yaml", text);
    compile_locating(&specification, &sources, &["inbox.yaml"]).unwrap()
}

/// The fixture with `AccountId` carrying `prefix` (none when `None`) and the channel `authority`.
fn variant(prefix: Option<&str>, authority: &str) -> String {
    assert_eq!(INBOX.matches(ACCOUNT).count(), 1);
    assert_eq!(INBOX.matches(AUTHORITY).count(), 1);
    let account = prefix.map_or_else(
        || ACCOUNT.to_owned(),
        |prefix| {
            format!("{{name: demo.inbox.AccountId, kind: newtype, of: String, prefix: {prefix}}}")
        },
    );
    INBOX.replacen(ACCOUNT, &account, 1).replacen(
        AUTHORITY,
        &format!("context_authority: {authority}"),
        1,
    )
}

fn delta(before: &str, after: &str) -> EssDelta {
    ess_diff::diff(&model(before), &model(after)).unwrap()
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta.changes().iter().map(|c| c.id().to_string()).collect()
}

fn read(json: &str) -> Result<EssDelta, String> {
    let raw: RawEssDelta = serde_json::from_str(json).map_err(|e| format!("parse: {e}"))?;
    EssDelta::try_from(raw).map_err(|e| format!("{e:?}"))
}

/// Both an `ess-diff/10` kind (external cause) and an `ess-diff/11` kind (prefix) in one delta.
#[test]
fn a_binding_context_change_beside_a_prefix_change_is_ess_diff_11() {
    let mixed = delta(
        &variant(None, "account-messages"),
        &variant(Some("acc_"), "tenant-messages"),
    );
    let ids = ids(&mixed);
    assert!(ids.iter().any(|id| id.contains("prefix-added")), "{ids:#?}");
    assert!(
        ids.iter().any(|id| id.contains("cause-changed")),
        "{ids:#?}"
    );
    assert!(
        !ids.iter().any(|id| id.contains("unclassified")),
        "{ids:#?}"
    );
    assert_eq!(mixed.format.to_string(), "ess-diff/11");
    let prefix = mixed
        .changes()
        .iter()
        .find(|c| c.id().to_string().contains("prefix-added"))
        .unwrap();
    assert_eq!(prefix.relation(), SemanticRelation::Narrowed);

    // The /10 writer refuses, naming the prefix change and not the cause change.
    let refused = mixed
        .to_canonical_json_for(DeltaFormat::parse("ess-diff/10").unwrap())
        .expect_err("a /10 writer refuses a prefix change");
    match refused {
        DeltaWriteRefusal::UnrepresentableChange { change, .. } => {
            assert!(change.to_string().contains("prefix-added"), "{change}");
        }
        other => panic!("{other:?}"),
    }

    // /11 round-trips through the reader unchanged.
    let json = mixed.to_canonical_json();
    assert!(json.contains("\"format\": \"ess-diff/11\""), "{json}");
    let back = read(&json).unwrap();
    assert_eq!(back.to_canonical_json(), json);
    assert_eq!(
        mixed
            .to_canonical_json_for(DeltaFormat::parse("ess-diff/11").unwrap())
            .unwrap(),
        json
    );

    // A /10 reader refuses exactly one change: the prefix one, not the external cause.
    let older = json.replace("\"ess-diff/11\"", "\"ess-diff/10\"");
    let refusal = read(&older).expect_err("a /10 reader refuses a prefix change");
    assert!(refusal.contains("UnsupportedFormatVersion"), "{refusal}");
    assert!(refusal.contains("prefix-added"), "{refusal}");
    assert!(!refusal.contains("cause-changed"), "{refusal}");
}

/// A delta with a binding-context kind and no prefix kind keeps `/10`, and its bytes do not name
/// `/11`; it may still be written as `/11` and read back.
#[test]
fn a_binding_context_change_alone_keeps_ess_diff_10_and_may_be_written_as_11() {
    let context = delta(
        &variant(Some("acc_"), "account-messages"),
        &variant(Some("acc_"), "tenant-messages"),
    );
    assert_eq!(context.format.to_string(), "ess-diff/10");
    let json = context.to_canonical_json();
    assert!(!json.contains("ess-diff/11"), "{json}");
    assert!(!json.contains("prefix-"), "{json}");
    let as_11 = context
        .to_canonical_json_for(DeltaFormat::parse("ess-diff/11").unwrap())
        .unwrap();
    assert_eq!(as_11, json.replace("\"ess-diff/10\"", "\"ess-diff/11\""));
    let back = read(&as_11).unwrap();
    assert_eq!(back.format.to_string(), "ess-diff/11");
    assert_eq!(back.changes(), context.changes());
}

/// A prefix change alone on an `ess/18` model is `/11` and every writer from `/2` to `/10`
/// refuses it, each naming the prefix change.
#[test]
fn every_writer_below_11_refuses_a_prefix_change_and_12_is_unsupported() {
    let prefix = delta(
        &variant(Some("acc_"), "account-messages"),
        &variant(Some("acc_x"), "account-messages"),
    );
    assert_eq!(
        ids(&prefix),
        vec!["type/demo.inbox.AccountId/prefix-changed".to_owned()]
    );
    assert_eq!(prefix.format.to_string(), "ess-diff/11");
    assert_eq!(prefix.changes()[0].relation(), SemanticRelation::Narrowed);
    for major in 1..=10 {
        let format = DeltaFormat::parse(&format!("ess-diff/{major}")).unwrap();
        match prefix.to_canonical_json_for(format) {
            Err(DeltaWriteRefusal::UnrepresentableChange { change, .. }) => {
                assert!(change.to_string().contains("prefix-changed"), "{change}");
            }
            other => panic!("ess-diff/{major}: {other:?}"),
        }
    }
    let twelve = DeltaFormat::parse("ess-diff/12").unwrap();
    assert!(!twelve.is_supported());
    assert!(matches!(
        prefix.to_canonical_json_for(twelve),
        Err(DeltaWriteRefusal::UnsupportedFormat { .. })
    ));
    let json = prefix.to_canonical_json();
    let refusal = read(&json.replace("\"ess-diff/11\"", "\"ess-diff/12\""))
        .expect_err("a /12 document is refused");
    assert!(refusal.contains("ess-diff/11"), "{refusal}");
}

/// The text rendering carries the prefix change beside the cause change, in the canonical order.
#[test]
fn the_text_rendering_of_a_mixed_delta_names_both() {
    let mixed = delta(
        &variant(Some("acc_"), "account-messages"),
        &variant(None, "tenant-messages"),
    );
    let text = ess_diff::render::text(&mixed);
    assert!(text.contains("prefix `acc_` → (none)"), "{text}");
    assert!(text.contains("tenant-messages"), "{text}");
    assert_eq!(
        mixed.count(SemanticRelation::Expanded),
        1,
        "{:#?}",
        ids(&mixed)
    );
}
