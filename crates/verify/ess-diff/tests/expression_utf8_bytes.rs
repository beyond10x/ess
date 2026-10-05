//! A byte length becoming, or ceasing to be, a comparison operand is a behaviour change, and the
//! diff renders it as the derived selector it is (`docs/design/expression-family-source22.md`,
//! "String `.utf8_bytes`", final review decision 7): `{utf8_bytes: label}`, never the path
//! `label.utf8_bytes`, which a reader cannot tell from a declared member of that name.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(guard: &str) -> String {
    format!(
        r"format: ess/22
system: notes
version: v1
domain: notes.core
types:
  - name: notes.core.Sizes
    kind: struct
    fields:
      - {{name: utf8_bytes, type: Integer}}
errors:
  - name: notes.core.Refused
    summary: The note is refused.
events:
  - {{name: notes.core.Filed, fields: []}}
commands:
  - name: notes.core.File
    input:
      - {{name: label, type: String}}
      - {{name: sizes, type: notes.core.Sizes}}
    outcomes:
      - name: refused
        when: {guard}
        error: notes.core.Refused
      - name: filed
        emits: [notes.core.Filed]
"
    )
}

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("notes.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn diff_a_byte_length_against_a_member_or_a_count_is_behaviour() {
    let before = ir(&model("label.utf8_bytes > 8"));
    for (guard, rendered) in [
        ("sizes.utf8_bytes > 8", "when sizes.utf8_bytes > 8"),
        ("label.count > 8", "when label.count > 8"),
        ("label.utf8_bytes > 9", "when {utf8_bytes: label} > 9"),
    ] {
        let delta = diff(&before, &ir(&model(guard))).unwrap();
        let json = delta.to_canonical_json();
        assert!(
            json.contains("outcome-condition-changed"),
            "{guard}: {json}"
        );
        assert!(
            json.contains(r#""before": "when {utf8_bytes: label} > 8""#)
                && json.contains(&format!(r#""after": "{rendered}""#)),
            "{guard}: {json}"
        );
    }
}

#[test]
fn an_unchanged_byte_length_is_no_change() {
    let before = ir(&model("label.utf8_bytes > 8"));
    let delta = diff(&before, &ir(&model("label.utf8_bytes > 8"))).unwrap();
    assert!(
        !delta.to_canonical_json().contains("condition-changed"),
        "{}",
        delta.to_canonical_json()
    );
}
