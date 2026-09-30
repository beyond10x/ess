//! An outcome's error payload sources (ess/19) are compared, not left to the residual
//! (beyond10x/ess#253).
//!
//! `payload:` keyed by the error a refusal reports says where each of the error's fields comes
//! from, and the compiled model carries it as `ResolvedOutcome::error_payload`. Declaring sources
//! on an outcome is `outcome-error-payload-added`, dropping them is `outcome-error-payload-removed`
//! and replacing them is `outcome-error-payload-changed`: one change per outcome, related as the
//! event payload's `outcome-payload-changed` is (`changed`: no command change decides a direction).
//! Each is `ess-diff/12` vocabulary: `ess-diff/11` shipped in 0.42.0 without them, so a `/3`–`/11`
//! writer and reader refuse them.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, render, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `commands` commands, each with an input-guarded refusal reporting an error whose one field is
/// named like the input, and an accepting branch. `sources` is the refusal's error payload block
/// (none when `None`), written under the given `format`.
fn model(format: &str, commands: usize, sources: Option<&str>) -> String {
    let payload = sources.map_or(String::new(), |sources| {
        format!("        payload:\n          demo.order.TooMany: {sources}\n")
    });
    let mut text = format!(
        "format: {format}\nsystem: demo\nversion: v1\ndomain: demo.order\n\
         errors:\n  - name: demo.order.TooMany\n    fields:\n      - {{name: quantity, type: Integer}}\n\
         events:\n  - name: demo.order.Checked\n    fields:\n      - {{name: quantity, type: Integer}}\n\
         commands:\n"
    );
    for index in 0..commands {
        let _ = write!(
            text,
            "  - name: demo.order.Place{index}\n    input:\n      - {{name: quantity, type: Integer}}\n    \
             outcomes:\n      - name: too-many\n        when: quantity > 10\n        \
             error: demo.order.TooMany\n{payload}      - name: checked\n        \
             emits: [demo.order.Checked]\n        payload:\n          \
             demo.order.Checked: {{quantity: input.quantity}}\n"
        );
    }
    text
}

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("orders.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn delta(before: &str, after: &str) -> EssDelta {
    diff(&ir(before), &ir(after)).unwrap()
}

const INPUT: &str = "{quantity: input.quantity}";
const LITERAL: &str = "{quantity: 10}";

#[test]
fn the_reported_bump_from_ess_16_classifies_the_added_sources() {
    // The downstream report: an ess/16 refusal, bumped to ess/19 with its error's field sourced
    // from the input of the same name.
    let delta = delta(&model("ess/16", 1, None), &model("ess/19", 1, Some(INPUT)));
    let json = delta.to_canonical_json();
    assert!(!json.contains("unclassified"), "{json}");
    let ids: Vec<String> = delta.changes().iter().map(|c| c.id().to_string()).collect();
    assert_eq!(
        ids,
        ["command/demo.order.Place0/outcome-error-payload-added/too-many"],
        "{json}"
    );
}

#[test]
fn each_error_payload_change_is_classified_in_json_and_text() {
    for (before, after, kind, text_before, text_after) in [
        (
            None,
            Some(INPUT),
            "outcome-error-payload-added",
            None,
            Some("demo.order.TooMany.quantity <- input.quantity"),
        ),
        (
            Some(INPUT),
            Some(LITERAL),
            "outcome-error-payload-changed",
            Some("demo.order.TooMany.quantity <- input.quantity"),
            Some("demo.order.TooMany.quantity <- literal `10`"),
        ),
        (
            Some(INPUT),
            None,
            "outcome-error-payload-removed",
            Some("demo.order.TooMany.quantity <- input.quantity"),
            None,
        ),
    ] {
        let delta = delta(&model("ess/19", 1, before), &model("ess/19", 1, after));
        let json = delta.to_canonical_json();
        assert_eq!(delta.format.to_string(), "ess-diff/12", "{json}");
        assert!(!json.contains("unclassified"), "{json}");
        assert_eq!(delta.changes().len(), 1, "{json}");
        let change = &delta.changes()[0];
        let id = format!("command/demo.order.Place0/{kind}/too-many");
        assert_eq!(change.id().to_string(), id, "{json}");
        // Mirrors `outcome-payload-changed`: no command change decides a direction.
        assert_eq!(change.relation(), SemanticRelation::Changed, "{json}");
        assert_eq!(change.minimum_format(), 12);
        assert!(json.contains(&format!(r#""kind": "{kind}""#)), "{json}");

        // Each side carries its own lines; an absent side is not written.
        let document: serde_json::Value = serde_json::from_str(&json).unwrap();
        let changed = &document["changes"][0]["change"]["changed"];
        assert_eq!(changed["outcome"], "too-many", "{json}");
        for (key, line) in [("before", text_before), ("after", text_after)] {
            match line {
                Some(line) => assert_eq!(changed[key], serde_json::json!([line]), "{json}"),
                None => assert!(changed.get(key).is_none(), "{key}: {json}"),
            }
        }
        let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

        let text = render::text(&delta);
        assert!(text.contains(&id), "{text}");
        assert!(
            text.contains("command demo.order.Place0: outcome `too-many`"),
            "{text}"
        );
        assert!(text.contains("error payload"), "{text}");
    }
}

#[test]
fn nineteen_outcomes_yield_nineteen_classified_changes() {
    let delta = delta(
        &model("ess/16", 19, None),
        &model("ess/19", 19, Some(INPUT)),
    );
    let json = delta.to_canonical_json();
    assert!(!json.contains("unclassified"), "{json}");
    assert_eq!(delta.changes().len(), 19, "{json}");
    for change in delta.changes() {
        let id = change.id().to_string();
        assert!(
            id.starts_with("command/demo.order.Place")
                && id.ends_with("/outcome-error-payload-added/too-many"),
            "{id}"
        );
    }
}

#[test]
fn an_error_payload_change_is_refused_below_ess_diff_12() {
    let delta = delta(&model("ess/19", 1, None), &model("ess/19", 1, Some(INPUT)));
    let json = delta.to_canonical_json();
    for old in 3..=11 {
        let format = format!("ess-diff/{old}");
        let refused = delta
            .to_canonical_json_for(format.parse().unwrap())
            .expect_err("an older writer refuses");
        assert!(
            refused.to_string().contains("outcome-error-payload-added"),
            "{format}: {refused}"
        );
        let older = json.replace("\"ess-diff/12\"", &format!("\"{format}\""));
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
fn unchanged_error_payload_sources_are_silent() {
    let same = delta(
        &model("ess/19", 2, Some(INPUT)),
        &model("ess/19", 2, Some(INPUT)),
    );
    assert!(same.is_empty(), "{}", same.to_canonical_json());
    assert_eq!(same.format.to_string(), "ess-diff/2");
}
