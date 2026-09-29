//! A newtype's `prefix:` (ess/15) is compared, not left to the residual (beyond10x/ess#219).
//!
//! Declaring a prefix narrows the values the type admits (`prefix-added`), dropping one widens them
//! (`prefix-removed`), and replacing one is `prefix-changed`: narrowed when the new prefix extends
//! the old one, widened when the old one extends the new one, and otherwise `changed` — the same
//! decidable-by-comparison rule `alphabet-changed` follows. Each is `ess-diff/11` vocabulary:
//! `ess-diff/10` shipped in 0.41.0 without them, so a `/3`–`/10` writer and reader refuse them.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, render, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const HEADER: &str = "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\n";

/// A model with two text newtypes, each with its own prefix line (absent when `None`).
fn model(channel: Option<&str>, topic: Option<&str>, extra: &str) -> String {
    let line = |prefix: Option<&str>| {
        prefix.map_or(String::new(), |prefix| {
            format!("    prefix: \"{prefix}\"\n")
        })
    };
    format!(
        "{HEADER}types:\n  - name: demo.msgs.Channel\n    kind: newtype\n    of: String\n{}{extra}  \
         - name: demo.msgs.Topic\n    kind: newtype\n    of: String\n{}",
        line(channel),
        line(topic)
    )
}

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("msgs.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn delta(before: &str, after: &str) -> EssDelta {
    diff(&ir(before), &ir(after)).unwrap()
}

#[test]
fn each_prefix_change_is_classified_with_its_relation_in_json_and_text() {
    for (before, after, kind, relation, verb) in [
        (
            None,
            Some("ord_"),
            "prefix-added",
            SemanticRelation::Narrowed,
            "narrows",
        ),
        (
            Some("ord_"),
            None,
            "prefix-removed",
            SemanticRelation::Expanded,
            "widens",
        ),
        (
            Some("ord_"),
            Some("inv_"),
            "prefix-changed",
            SemanticRelation::Changed,
            "changes",
        ),
        (
            Some("ord_"),
            Some("ord_x"),
            "prefix-changed",
            SemanticRelation::Narrowed,
            "narrows",
        ),
        (
            Some("ord_x"),
            Some("ord_"),
            "prefix-changed",
            SemanticRelation::Expanded,
            "widens",
        ),
    ] {
        let delta = delta(&model(before, None, ""), &model(after, None, ""));
        let json = delta.to_canonical_json();
        assert_eq!(delta.format.to_string(), "ess-diff/11", "{json}");
        assert!(!json.contains("unclassified"), "{json}");
        assert_eq!(delta.changes().len(), 1, "{json}");
        let change = &delta.changes()[0];
        let id = format!("type/demo.msgs.Channel/{kind}");
        assert_eq!(change.id().to_string(), id, "{before:?} → {after:?}");
        assert_eq!(change.relation(), relation, "{before:?} → {after:?}");
        assert_eq!(change.minimum_format(), 11);
        assert!(json.contains(&format!(r#""kind": "{kind}""#)), "{json}");
        assert!(
            json.contains(&format!(r#""relation": "{}""#, relation.written())),
            "{json}"
        );
        // Each side carries its own value, never the other's; an absent side is not written.
        let document: serde_json::Value = serde_json::from_str(&json).unwrap();
        let changed = &document["changes"][0]["change"]["changed"];
        for (key, value) in [("before", before), ("after", after)] {
            assert_eq!(
                changed.get(key).and_then(serde_json::Value::as_str),
                value,
                "{key}: {json}"
            );
        }
        let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

        let text = render::text(&delta);
        assert!(text.contains(&id), "{text}");
        assert!(
            text.contains(&format!("{verb:<8} type demo.msgs.Channel: prefix ")),
            "{text}"
        );
        let shown = |value: Option<&str>| value.map_or("(none)".to_owned(), |v| format!("`{v}`"));
        assert!(
            text.contains(&format!("prefix {} → {}", shown(before), shown(after))),
            "{text}"
        );
    }
}

#[test]
fn every_newtype_whose_prefix_moves_gets_its_own_change() {
    let delta = delta(
        &model(Some("ord_"), Some("t/"), ""),
        &model(None, Some("u/"), ""),
    );
    let json = delta.to_canonical_json();
    let ids: Vec<String> = delta.changes().iter().map(|c| c.id().to_string()).collect();
    assert_eq!(
        ids,
        [
            "type/demo.msgs.Channel/prefix-removed",
            "type/demo.msgs.Topic/prefix-changed",
        ],
        "{json}"
    );
    assert!(json.contains(r#""before": "t/""#), "{json}");
    assert!(json.contains(r#""after": "u/""#), "{json}");
    assert!(!json.contains("unclassified"), "{json}");
}

#[test]
fn a_prefix_change_is_refused_below_ess_diff_11() {
    let delta = delta(&model(None, None, ""), &model(Some("ord_"), None, ""));
    let json = delta.to_canonical_json();
    for old in 3..=10 {
        let format = format!("ess-diff/{old}");
        let refused = delta
            .to_canonical_json_for(format.parse().unwrap())
            .expect_err("an older writer refuses");
        assert!(
            refused.to_string().contains("prefix-added"),
            "{format}: {refused}"
        );
        let older = json.replace("\"ess-diff/11\"", &format!("\"{format}\""));
        assert_ne!(older, json);
        let raw: RawEssDelta = serde_json::from_str(&older).unwrap();
        let refused = EssDelta::try_from(raw).expect_err("an older reader refuses");
        let refused = format!("{refused:?}");
        assert!(
            refused.contains("UnsupportedFormatVersion"),
            "{format}: {refused}"
        );
    }
}

#[test]
fn a_pair_without_a_prefix_change_keeps_its_format_and_reports_no_prefix_kind() {
    // The same prefix on both sides, beside an alphabet change: an `ess-diff/8` delta, as before.
    let alphabet = "    alphabet: \"abcdefghijklmnopqrstuvwxyz_/\"\n";
    let delta_8 = delta(
        &model(Some("ord_"), Some("t/"), ""),
        &model(Some("ord_"), Some("t/"), alphabet),
    );
    let json = delta_8.to_canonical_json();
    assert_eq!(delta_8.format.to_string(), "ess-diff/8", "{json}");
    assert!(!json.contains("prefix"), "{json}");
    assert!(!json.contains("unclassified"), "{json}");

    // No change at all under an unchanged prefix: an empty delta at the floor format.
    let same = delta(
        &model(Some("ord_"), Some("t/"), ""),
        &model(Some("ord_"), Some("t/"), ""),
    );
    assert!(same.is_empty(), "{}", same.to_canonical_json());
    assert_eq!(same.format.to_string(), "ess-diff/2");
}
