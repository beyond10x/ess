//! `shorthand: <field>` on a struct (`ess/27`, beyond10x/ess#594) moving is a typed change,
//! `type/<name>/shorthand-changed`, `ess-diff/20` vocabulary, never
//! `system/<name>/unclassified-changed`.
//!
//! - Declaring one widens what the struct admits, every long form and now the field's scalar
//!   alone: `expanded`.
//! - Dropping one, or moving it to another field, narrows: a short form written against the
//!   earlier revision is refused, or read as another field. `narrowed`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `{shorthand}` is the line written on `Operation`, `{op}` and `{alt}` the types of its two
/// fields.
const MODEL: &str = "\
format: ess/27
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Operation
    kind: struct
{shorthand}    fields:
      - name: op
        type: {op}
      - name: alt
        type: {alt}
actors:
  - name: tracker.ops.Operator
    may:
      - tracker.ops.Run
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
    response:
      - name: operation
        type: tracker.ops.Operation
    outcomes:
      - name: ran
        returns: true
";

const SHORTHAND: &str = "type/tracker.ops.Operation/shorthand-changed";
const UNCLASSIFIED: &str = "system/tracker/unclassified-changed";

/// The model with `shorthand:` naming `field` (none when `None`); the named field is the required
/// one and the other is `Optional`.
fn model(shorthand: Option<&str>) -> EssIr {
    let (op, alt) = match shorthand {
        Some("alt") => ("Optional<String>", "String"),
        _ => ("String", "Optional<String>"),
    };
    let text = MODEL
        .replace(
            "{shorthand}",
            &shorthand.map_or_else(String::new, |field| format!("    shorthand: {field}\n")),
        )
        .replace("{op}", op)
        .replace("{alt}", alt);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tracker.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn relation(delta: &EssDelta) -> SemanticRelation {
    let json = delta.to_canonical_json();
    assert!(!json.contains(UNCLASSIFIED), "{json}");
    delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == SHORTHAND)
        .unwrap_or_else(|| panic!("`{SHORTHAND}` in {json}"))
        .relation()
}

#[test]
fn diff_classifies_a_shorthand_change() {
    let (plain, short, moved) = (model(None), model(Some("op")), model(Some("alt")));

    let added = classified(&plain, &short).unwrap();
    assert_eq!(relation(&added), SemanticRelation::Expanded);
    assert_eq!(
        added
            .changes()
            .iter()
            .map(|change| change.id().to_string())
            .collect::<Vec<_>>(),
        [SHORTHAND],
        "{}",
        added.to_canonical_json()
    );
    let text = ess_diff::render::text(&added);
    assert!(text.contains("shorthand"), "{text}");

    let removed = classified(&short, &plain).unwrap();
    assert_eq!(relation(&removed), SemanticRelation::Narrowed);

    let to_another = classified(&short, &moved).unwrap();
    assert_eq!(relation(&to_another), SemanticRelation::Narrowed);

    // `ess-diff/20` vocabulary: the default delta carries it, reads back, and an earlier format
    // neither writes nor reads it.
    let delta = diff(&plain, &short).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/20", "{json}");
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");
    let refused = delta
        .to_canonical_json_for("ess-diff/19".parse().unwrap())
        .expect_err("ess-diff/19 cannot write it");
    assert!(
        refused.to_string().contains("not representable"),
        "{refused}"
    );
    let mut older: serde_json::Value = serde_json::from_str(&json).unwrap();
    older["format"] = serde_json::Value::from("ess-diff/19");
    let raw: RawEssDelta = serde_json::from_value(older).unwrap();
    let refused = EssDelta::try_from(raw).expect_err("an `ess-diff/19` reader refuses it");
    assert!(
        refused.to_string().contains("outside the vocabulary"),
        "{refused}"
    );

    // An unmoved shorthand is no change, and a model without one keeps its delta format.
    assert_eq!(diff(&short, &short).unwrap().changes().len(), 0);
    assert_eq!(diff(&plain, &plain).unwrap().changes().len(), 0);
}
