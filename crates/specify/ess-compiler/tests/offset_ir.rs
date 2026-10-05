//! The compiled IR carries one constant offset as its closed canonical mapping
//! (`docs/design/expression-family-source22.md`, A2): `upper > lower + 5` is
//! `{offset: {fact: lower, add: 5}}` in the guard, never the text `lower + 5`; the same guard written
//! canonically compiles to the same IR; and a model without an offset carries no such mapping.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn model(format: u32, when: &str) -> String {
    format!(
        "format: ess/{format}
system: pool
version: v1
domain: pool.lease
errors:
  - {{name: pool.lease.Refused, summary: The lease is refused.}}
events:
  - {{name: pool.lease.Accepted, fields: []}}
commands:
  - name: pool.lease.Open
    input:
      - {{name: lower, type: Integer}}
      - {{name: upper, type: Integer}}
      - {{name: issued_at, type: Timestamp}}
      - {{name: expires_at, type: Timestamp}}
    outcomes:
      - name: refused
        when: {when}
        error: pool.lease.Refused
      - name: accepted
        emits: [pool.lease.Accepted]
"
    )
}

/// The canonical IR with every whitespace character removed.
fn ir(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("pool.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn a2_the_ir_carries_the_canonical_offset() {
    let compact = ir(&model(22, "upper > lower + 5"));
    assert!(
        compact.contains(r#"{"upper":{"gt":{"offset":{"add":5.0,"fact":"lower"}}}}"#),
        "{compact}"
    );
    let canonical = ir(&model(22, "{upper: {gt: {offset: {fact: lower, add: 5}}}}"));
    assert_eq!(compact, canonical, "one guard, one IR");
    let instants = ir(&model(22, "expires_at <= issued_at - 24h"));
    assert!(
        instants
            .contains(r#"{"expires_at":{"lte":{"offset":{"fact":"issued_at","subtract":"24h"}}}}"#),
        "{instants}"
    );
}

#[test]
fn a2_a_model_without_an_offset_carries_none() {
    let older = ir(&model(21, "upper > 5"));
    assert!(!older.contains("offset"), "{older}");
    let current = ir(&model(22, "upper > 5"));
    assert!(!current.contains("offset"), "{current}");
}
