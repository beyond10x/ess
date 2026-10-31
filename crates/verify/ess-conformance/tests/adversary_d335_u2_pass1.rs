//! Adversary pass 1 on
//! `story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type` (wave D335, U2).
//!
//! F1 (Decisions): from `ess-scenario/6` "a literal input of an authored act that breaks its
//! type's alphabet, prefix or invariant at any depth is ESS-AUTHOR-015 naming the field and the
//! constraint". The unit's check reads a field only where its whole value is a literal
//! (`ScenarioValue::as_literal`); a struct or list that also holds an `{$instance: …}` compiles to
//! `ScenarioValue::Members` / `List`, and its literal members are never read. These cases put the
//! out-of-type literal beside such a reference. F2: the 003 names the constraint the value breaks.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

const STORY: &str = "story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(SpecSource::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// Rings, and a rollout whose `pair` and `pairs` hold a ring identity beside a constrained note.
const RINGS: &str = r#"format: ess/32
system: release
version: v1
domain: release.rings
types:
  - {name: release.rings.ReleaseRingId, kind: newtype, of: Uuid}
  - {name: release.rings.RolloutId, kind: newtype, of: Uuid}
  - name: release.rings.Note
    kind: newtype
    of: String
    alphabet: "abc"
  - name: release.rings.RingPair
    kind: struct
    fields:
      - {name: primary, type: release.rings.ReleaseRingId}
      - {name: note, type: release.rings.Note}
entities:
  - name: release.rings.ReleaseRing
    identity: {name: ring_id, type: release.rings.ReleaseRingId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: release.rings.Rollout
    identity: {name: rollout_id, type: release.rings.RolloutId}
    fields: []
    lifecycle: {initial: Planned, states: [Planned], terminal: [Planned]}
events:
  - name: release.rings.RingCreated
    fields:
      - {name: ring_id, type: release.rings.ReleaseRingId}
      - {name: label, type: String}
  - name: release.rings.RolloutPlanned
    fields:
      - {name: rollout_id, type: release.rings.RolloutId}
commands:
  - name: release.rings.CreateRing
    input:
      - {name: label, type: String}
    outcomes:
      - name: created
        creates: release.rings.ReleaseRing
        instance: ring_id
        emits: [release.rings.RingCreated]
        payload:
          release.rings.RingCreated: {ring_id: {generated: true}, label: input.label}
        sets: {label: input.label}
  - name: release.rings.PlanRollout
    input:
      - {name: pair, type: release.rings.RingPair}
      - {name: pairs, type: "List<release.rings.RingPair>"}
    outcomes:
      - name: planned
        creates: release.rings.Rollout
        instance: rollout_id
        emits: [release.rings.RolloutPlanned]
        payload:
          release.rings.RolloutPlanned: {rollout_id: {generated: true}}
"#;

/// Ring `a` created and captured, then one rollout sending `pair` and `pairs` as written.
fn rollout(pair: &str, pairs: &str) -> String {
    format!(
        "type: ess-scenario/6
domain: release.rings
scenario: plan-a-rollout
summary: A rollout names a ring beside a note.
arrange:
  - instance: a
    entity: release.rings.ReleaseRing
timeline:
  - at: 2026-01-05T09:00:00Z
    command: release.rings.CreateRing
    input: {{label: alpha}}
    outcome: created
    events:
      - event: release.rings.RingCreated
        payload: {{label: alpha}}
    capture: {{instance: a, event: release.rings.RingCreated, field: ring_id}}
  - at: 2026-01-05T09:00:01Z
    command: release.rings.PlanRollout
    input:
      pair: {pair}
      pairs: {pairs}
    outcome: planned
"
    )
}

const LITERAL_RING: &str = "\"00000000-0000-4000-8000-000000000001\"";

/// The control: a note outside the alphabet in an all-literal `pair` is refused, and the
/// in-type note beside a reference compiles.
#[test]
fn an_all_literal_out_of_type_member_is_refused_and_an_in_type_one_beside_a_reference_compiles() {
    let model = ir(RINGS);
    let literal = authoring(
        &model,
        &rollout(&format!("{{primary: {LITERAL_RING}, note: xyz}}"), "[]"),
    );
    assert!(
        literal
            .refusals
            .iter()
            .any(|refusal| refusal.code().to_string() == "ESS-AUTHOR-015"
                && refusal.to_string().contains("alphabet")),
        "{STORY}: the all-literal control: {:#?}",
        literal.refusals
    );
    let in_type = authoring(
        &model,
        &rollout("{primary: {$instance: a}, note: abc}", "[]"),
    );
    assert!(
        in_type.refusals.is_empty(),
        "the in-type control: {:#?}",
        in_type.refusals
    );
}

/// F1 at depth: the same out-of-type literal `note`, in a struct whose other member is a
/// reference, and in a list element beside one.
#[test]
fn an_out_of_type_literal_member_beside_a_reference_is_refused() {
    let model = ir(RINGS);
    let mut missed = Vec::new();
    for (pair, pairs) in [
        ("{primary: {$instance: a}, note: xyz}", "[]"),
        (
            "{primary: {$instance: a}, note: abc}",
            "[{primary: {$instance: a}, note: xyz}]",
        ),
    ] {
        let result = authoring(&model, &rollout(pair, pairs));
        let refused = result.refusals.iter().any(|refusal| {
            refusal.code().to_string() == "ESS-AUTHOR-015"
                && refusal.to_string().contains("alphabet")
        });
        if !refused {
            missed.push(format!(
                "pair {pair}, pairs {pairs}: {} scenario(s), refusals {:?}",
                result.scenarios.len(),
                result
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            ));
        }
    }
    assert!(
        missed.is_empty(),
        "{STORY}: a literal `note` outside the alphabet of release.rings.Note, sent beside an \
         `{{$instance: a}}`, is a literal input outside its type at depth and is ESS-AUTHOR-015: \
         {missed:#?}"
    );
}

/// F2 (Decisions): ESS-SYNTH-003 "names the type, its first broken constraint and the rule". The
/// unit's test reads the type and the rule only; this reads the constraint: each 003 on the
/// fixture's refusal quotes the alphabet or the invariant its value breaks.
#[test]
fn a_type_excluded_refusal_names_the_broken_constraint() {
    let model = ir(include_str!(
        "fixtures/refusal-guard-outside-input-type.yaml"
    ));
    let result = ess_conformance::synthesize::synthesize(&model);
    let about: Vec<String> = result
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("invalid-code") && refusal.contains("ESS-SYNTH-003"))
        .collect();
    assert!(!about.is_empty(), "{STORY}: the guard stays unwitnessed");
    for refusal in &about {
        assert!(
            refusal.contains("alphabet of demo.codes.Code")
                || refusal.contains("value.utf8_bytes >= 1")
                || refusal.contains("value.utf8_bytes <= 256"),
            "{STORY}: the refusal names the constraint the value breaks: {refusal}"
        );
    }
}
