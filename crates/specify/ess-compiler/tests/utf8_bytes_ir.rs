//! The compiled IR carries the UTF-8 byte length of a String as its derived selector
//! (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`"): `label.utf8_bytes <= 8`
//! is `{compare: {left: {utf8_bytes: label}, …}}` in the guard, never the path `label.utf8_bytes`;
//! the same guard written canonically compiles to the same IR; a declared member of that name stays
//! a path; and a model without the selector carries no such mapping.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn model(format: u32, when: &str) -> String {
    format!(
        "format: ess/{format}
system: notes
version: v1
domain: notes.core
types:
  - name: notes.core.Sizes
    kind: struct
    fields:
      - {{name: utf8_bytes, type: Integer}}
errors:
  - {{name: notes.core.Refused, summary: The note is refused.}}
events:
  - {{name: notes.core.Filed, fields: []}}
commands:
  - name: notes.core.File
    input:
      - {{name: label, type: String}}
      - {{name: limit, type: Integer}}
      - {{name: sizes, type: notes.core.Sizes}}
    outcomes:
      - name: refused
        when: {when}
        error: notes.core.Refused
      - name: filed
        emits: [notes.core.Filed]
"
    )
}

/// The canonical IR with every whitespace character removed.
fn ir(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn utf8_the_ir_carries_the_derived_selector() {
    let compact = ir(&model(22, "label.utf8_bytes <= 8"));
    assert!(
        compact.contains(r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":8.0}}"#),
        "{compact}"
    );
    let canonical = ir(&model(
        22,
        "{compare: {left: {utf8_bytes: label}, op: lte, right: 8}}",
    ));
    assert_eq!(compact, canonical, "one guard, one IR");
    let right = ir(&model(22, "limit < label.utf8_bytes"));
    assert!(
        right.contains(r#"{"limit":{"lt":{"utf8_bytes":"label"}}}"#),
        "{right}"
    );
}

#[test]
fn utf8_a_member_of_that_name_and_a_model_without_it_carry_none() {
    for format in [21, 22] {
        let member = ir(&model(format, "sizes.utf8_bytes <= 8"));
        assert!(member.contains(r#""sizes.utf8_bytes<=8""#), "{member}");
        assert!(!member.contains(r#"{"utf8_bytes":"#), "{member}");
        let plain = ir(&model(format, "limit <= 8"));
        assert!(!plain.contains("utf8_bytes\":\""), "{plain}");
    }
}
