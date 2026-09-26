//! An unquoted YAML boolean or integer in `sets:` or `payload:` compiles to exactly the IR its
//! quoted spelling does (beyond10x/ess#113, `docs/design/typed-literals-and-unknown-instances.md`).
//!
//! The IR is what every digest, realization and generated artifact pins, so "the same meaning" is
//! checked as the same canonical bytes, not as a similar-looking value.

use ess_compiler::{ir::EssIr, resolve::compile_locating, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// One entity with a `Boolean` and an `Integer` field, set on creation, and an event carrying one
/// of each, filled by a literal. `{paused}`, `{tries}`, `{flag}` and `{count}` are the four
/// literal spellings under test.
fn source(paused: &str, tries: &str, flag: &str, count: &str) -> String {
    format!(
        "format: ess/4
system: sample
version: v1
domain: sample.dial
entities:
  - name: sample.dial.Attempt
    identity: {{name: attempt_id, type: Uuid}}
    fields:
      - {{name: paused, type: Boolean}}
      - {{name: tries, type: Integer}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: sample.dial.Opened
    fields:
      - {{name: attempt_id, type: Uuid}}
      - {{name: flag, type: Boolean}}
      - {{name: count, type: Integer}}
commands:
  - name: sample.dial.Open
    outcomes:
      - name: opened
        creates: sample.dial.Attempt
        instance: attempt_id
        sets:
          paused: {paused}
          tries: {tries}
        emits: [sample.dial.Opened]
        payload:
          sample.dial.Opened:
            attempt_id: {{generated: true}}
            flag: {flag}
            count: {count}
"
    )
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    let spec = Specification::assemble([(Source::new("sample.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let mut sources = SourceMap::new();
    sources.insert("sample.yaml", text);
    compile_locating(&spec, &sources, &["sample.yaml"]).unwrap_or_else(|errors| panic!("{errors}"))
}

#[test]
fn an_unquoted_boolean_and_integer_compile_to_the_bytes_of_their_quoted_form() {
    for (paused, tries, flag, count) in [("false", "0", "true", "3"), ("true", "-12", "false", "0")]
    {
        let quoted = compiled(&source(
            &format!("'{paused}'"),
            &format!("'{tries}'"),
            &format!("'{flag}'"),
            &format!("'{count}'"),
        ))
        .to_canonical_json();
        let unquoted = compiled(&source(paused, tries, flag, count)).to_canonical_json();
        assert_eq!(
            unquoted, quoted,
            "`paused: {paused}` and friends must compile to the quoted form's IR"
        );
        assert!(
            unquoted.contains(&format!("\"value\": \"{tries}\"")),
            "the literal is carried as its canonical text:\n{unquoted}"
        );
    }
}
