//! An offset's base, sign or magnitude moving is a behaviour change, and the diff renders the
//! offset as the typed operand it is (`docs/design/expression-family-source22.md`, A2, final review
//! decision 7): never as the text `lower + 5`, which a reader cannot tell from a literal.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(guard: &str, invariant: &str) -> String {
    format!(
        r"format: ess/22
system: pool
version: v1
domain: pool.lease
entities:
  - name: pool.lease.Lease
    identity: {{name: lease_id, type: Uuid}}
    fields:
      - {{name: issued_at, type: Timestamp}}
      - {{name: expires_at, type: Timestamp}}
    invariants:
      - {invariant}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - name: pool.lease.Refused
    summary: The lease is refused.
events:
  - name: pool.lease.Opened
    fields:
      - {{name: lease_id, type: Uuid}}
commands:
  - name: pool.lease.Open
    input:
      - {{name: lower, type: Integer}}
      - {{name: upper, type: Integer}}
      - {{name: floor, type: Integer}}
      - {{name: issued_at, type: Timestamp}}
      - {{name: expires_at, type: Timestamp}}
    outcomes:
      - name: refused
        when: {guard}
        error: pool.lease.Refused
      - name: opened
        creates: pool.lease.Lease
        instance: lease_id
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened: {{lease_id: {{generated: true}}}}
        sets: {{issued_at: input.issued_at, expires_at: input.expires_at}}
"
    )
}

fn ir(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("pool.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

const GUARD: &str = "upper > lower + 5";
const INVARIANT: &str = "expires_at <= issued_at + 24h";

#[test]
fn diff_offset_changes_are_behaviour() {
    let before = ir(&model(GUARD, INVARIANT));
    for (guard, rendered) in [
        (
            "upper > lower + 6",
            "when upper > {offset: {fact: lower, add: 6}}",
        ),
        (
            "upper > lower - 5",
            "when upper > {offset: {fact: lower, subtract: 5}}",
        ),
        (
            "upper > floor + 5",
            "when upper > {offset: {fact: floor, add: 5}}",
        ),
    ] {
        let delta = diff(&before, &ir(&model(guard, INVARIANT))).unwrap();
        let json = delta.to_canonical_json();
        assert!(
            json.contains("outcome-condition-changed"),
            "{guard}: {json}"
        );
        assert!(
            json.contains(r#""before": "when upper > {offset: {fact: lower, add: 5}}""#)
                && json.contains(&format!(r#""after": "{rendered}""#)),
            "{guard}: {json}"
        );
    }
    let delta = diff(&before, &ir(&model(GUARD, "expires_at <= issued_at + 25h"))).unwrap();
    let json = delta.to_canonical_json();
    // An invariant whose statement changed is shown as written, which already tells the two apart
    // (its canonical form is shown only where the statement did not move).
    assert!(
        json.contains("invariants-changed")
            && json.contains(r#""expires_at <= issued_at + 24h""#)
            && json.contains(r#""expires_at <= issued_at + 25h""#),
        "{json}"
    );
}

#[test]
fn an_unchanged_offset_is_no_change() {
    let before = ir(&model(GUARD, INVARIANT));
    let delta = diff(&before, &ir(&model(GUARD, INVARIANT))).unwrap();
    let json = delta.to_canonical_json();
    assert!(!json.contains("condition-changed"), "{json}");
    assert!(!json.contains("invariants-changed"), "{json}");
}
