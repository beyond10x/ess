//! Adversarial case for unit A1 (ess#93): a predicate the base build persisted still reads back.
//!
//! The base build's `Operand` display quoted a text literal only when it contained a dot or was
//! empty, so the text `null` — written quoted by its author, `note == "null"` — was persisted in
//! canonical JSON as the compact string `note == null`, and the text `A1 && gift` as
//! `sku == A1 && gift`. Both round-tripped under the base parser, which is why the compact form was
//! chosen. The strings below are those persisted spellings, byte for byte.

use ess_primitives::facts::FactValue;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

fn text_comparison(path: &str, text: &str) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(path.parse().expect("path")),
        op: CompareOp::Eq,
        right: Operand::Literal(FactValue::text(text)),
    }
}

/// Rewritten by coordinator decision, correction round 1 finding 4: these persisted spellings are
/// **refused**, not read back — nothing committed carries them, and the release notes say such an
/// artifact is regenerated. What the refusal owes its reader is the spelling it refused and the
/// repair: quote it. The current writer renders the same predicates quoted, and those read back.
#[test]
fn a_text_comparison_the_base_build_persisted_is_refused_naming_its_spelling_and_the_quote_repair()
{
    let mut unhelpful = Vec::new();
    for (persisted, path, text, named) in [
        (r#""note == null""#, "note", "null", "`note == null`"),
        (r#""note == ~""#, "note", "~", "`note == ~`"),
        (
            r#""sku == A1 && gift""#,
            "sku",
            "A1 && gift",
            "`A1 && gift`",
        ),
    ] {
        match serde_json::from_str::<Predicate>(persisted) {
            Err(error)
                if error.to_string().contains(&format!("quote \"{text}\""))
                    && error.to_string().contains(named) => {}
            other => unhelpful.push(format!("{persisted}: {other:?}")),
        }
        let current = serde_json::to_string(&text_comparison(path, text).to_string())
            .expect("a string serialises");
        assert_eq!(
            serde_json::from_str::<Predicate>(&current).ok(),
            Some(text_comparison(path, text)),
            "the current writer's spelling {current} reads back"
        );
    }
    assert!(
        unhelpful.is_empty(),
        "a base-build spelling is refused (decision: regenerate such artifacts), and the refusal \
         names the spelling and says to quote it:\n{}",
        unhelpful.join("\n")
    );
}
