//! `alphabet:` and `example:` are compared, not left to the residual (`ess-diff/8`).
//!
//! `docs/design/string-alphabet-and-length.md`, section 6: `TypeChange::AlphabetChanged` relates the
//! revisions by set membership alone, as `VariantAdded`/`VariantRemoved` do, and
//! `CommandChange::InputExampleChanged` is `Changed`. Both need `ess-diff/8`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const KEYPAD: &str = include_str!("../../ess-conformance/tests/fixtures/keypad.yaml");
const ALPHABET: &str = "    alphabet: \"0123456789*#ABCD\"\n";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("keypad.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn with_alphabet(alphabet: Option<&str>) -> String {
    let line = alphabet.map_or(String::new(), |alphabet| {
        format!("    alphabet: \"{alphabet}\"\n")
    });
    assert!(KEYPAD.contains(ALPHABET));
    KEYPAD.replacen(ALPHABET, &line, 1)
}

/// The one change between the two revisions, its relation, and the delta read back.
fn only_change(before: &str, after: &str) -> (String, SemanticRelation, EssDelta) {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/8", "{json}");
    assert!(!json.contains("unclassified"), "{json}");
    assert_eq!(delta.changes().len(), 1, "{json}");
    let change = &delta.changes()[0];
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");
    assert_eq!(change.minimum_format(), 8);
    (change.id().to_string(), change.relation(), delta.clone())
}

#[test]
fn an_alphabet_change_is_related_by_set_membership_alone() {
    for (before, after, relation) in [
        (None, Some("0123"), SemanticRelation::Narrowed),
        (Some("0123"), None, SemanticRelation::Expanded),
        (Some("0123"), Some("0123#"), SemanticRelation::Expanded),
        (Some("0123#"), Some("0123"), SemanticRelation::Narrowed),
        (Some("0123"), Some("3210"), SemanticRelation::Changed),
        (Some("0123"), Some("01AB"), SemanticRelation::Changed),
    ] {
        let (id, found, delta) = only_change(&with_alphabet(before), &with_alphabet(after));
        assert_eq!(
            id, "type/keypad.dial.KeySequence/alphabet-changed",
            "{before:?} → {after:?}"
        );
        assert_eq!(found, relation, "{before:?} → {after:?}");
        let json = delta.to_canonical_json();
        assert!(json.contains(r#""kind": "alphabet-changed""#), "{json}");
    }
}

#[test]
fn an_input_example_change_is_reported_and_changes_nothing_a_caller_may_send() {
    let example = |value: &str| {
        KEYPAD.replacen(
            "      - {name: keys, type: keypad.dial.KeySequence}\n    outcomes:",
            &format!(
                "      - {{name: keys, type: keypad.dial.KeySequence, example: \"{value}\"}}\n    outcomes:"
            ),
            1,
        )
    };
    for (before, after) in [
        (KEYPAD.to_owned(), example("12#")),
        (example("12#"), example("999")),
        (example("12#"), KEYPAD.to_owned()),
    ] {
        let (id, relation, delta) = only_change(&before, &after);
        assert_eq!(
            id,
            "command/keypad.dial.SendKeys/input-example-changed/keys"
        );
        assert_eq!(relation, SemanticRelation::Changed);
        let json = delta.to_canonical_json();
        assert!(
            json.contains(r#""kind": "input-example-changed""#),
            "{json}"
        );
    }
}

#[test]
fn a_revision_pair_that_changes_neither_stays_at_its_older_format() {
    let before = ir(KEYPAD);
    let after = ir(&KEYPAD.replace("when: keys.count > 64", "when: keys.count > 32"));
    let delta = diff(&before, &after).unwrap();
    assert_ne!(
        delta.format.to_string(),
        "ess-diff/8",
        "{}",
        delta.to_canonical_json()
    );
}
