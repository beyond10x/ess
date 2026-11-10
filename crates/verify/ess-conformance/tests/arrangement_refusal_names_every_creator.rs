//! An ESS-SYNTH-004 refusal about an entity several commands create names every creator tried,
//! each with its own cause, in the order they were tried (beyond10x/ess#728).
//!
//! `Decision` is created by `Confirm`, whose `no-draft` guard reads a `Draft` nothing creates, and
//! by `Record`, whose two guards between them cover every `kind`. The refusal of `Annotate` named
//! only `Confirm`, the first creator in command-name order, and sent the reader to the wrong
//! command.
//!
//! The model is inline: a YAML file anywhere in the repository is a row of
//! `synthesis_suite_bytes_table`.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::Synthesis;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const TWO_CREATORS: &str = r#"format: ess/32
system: demo
version: v1
domain: demo.review
summary: Decisions are confirmed or recorded, then annotated.
types:
  - {name: demo.review.DecisionKey, kind: newtype, of: String}
  - {name: demo.review.DraftKey, kind: newtype, of: String}
  - {name: demo.review.Kind, kind: enum, variants: [Accept, Erratum]}
entities:
  - name: demo.review.Draft
    identity: {name: draft_key, type: demo.review.DraftKey}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.review.Decision
    identity: {name: decision_key, type: demo.review.DecisionKey}
    fields:
      - {name: kind, type: demo.review.Kind}
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.review.DecisionMade
    fields:
      - {name: decision_key, type: demo.review.DecisionKey}
  - name: demo.review.DecisionNoted
    fields:
      - {name: decision_key, type: demo.review.DecisionKey}
errors:
  - {name: demo.review.NoDraft, fields: []}
  - {name: demo.review.AlreadyExists, fields: []}
  - {name: demo.review.Incomplete, fields: []}
  - {name: demo.review.NotVerified, fields: []}
  - {name: demo.review.NotAccepted, fields: []}
actors:
  - {name: demo.review.Clerk, may: [demo.review.Confirm, demo.review.Record, demo.review.Annotate]}
commands:
  - name: demo.review.Confirm
    input:
      - {name: decision_key, type: demo.review.DecisionKey}
      - {name: draft, type: demo.review.DraftKey}
      - {name: kind, type: demo.review.Kind}
    outcomes:
      - name: no-draft
        when_related: {via: input.draft, exists: false}
        error: demo.review.NoDraft
      - {name: already-exists, existing_instance: true, error: demo.review.AlreadyExists}
      - name: confirmed
        creates: demo.review.Decision
        instance: decision_key
        sets: {kind: input.kind, note: '""'}
        emits: [demo.review.DecisionMade]
        payload:
          demo.review.DecisionMade: {decision_key: input.decision_key}
  - name: demo.review.Record
    input:
      - {name: decision_key, type: demo.review.DecisionKey}
      - {name: kind, type: demo.review.Kind}
    outcomes:
      - name: incomplete
        when: kind == Erratum
        error: demo.review.Incomplete
      - name: not-verified
        when: kind == Accept
        error: demo.review.NotVerified
      - {name: already-exists, existing_instance: true, error: demo.review.AlreadyExists}
      - name: recorded
        creates: demo.review.Decision
        instance: decision_key
        sets: {kind: input.kind, note: '""'}
        emits: [demo.review.DecisionMade]
        payload:
          demo.review.DecisionMade: {decision_key: input.decision_key}
  - name: demo.review.Annotate
    input:
      - {name: decision_key, type: demo.review.DecisionKey}
      - {name: note, type: String}
    outcomes:
      - name: not-accepted
        when_subject: {predicate: kind != Accept}
        error: demo.review.NotAccepted
      - name: annotated
        updates: demo.review.Decision
        instance: decision_key
        sets: {note: input.note}
        emits: [demo.review.DecisionNoted]
        payload:
          demo.review.DecisionNoted: {decision_key: input.decision_key}
views:
  - name: demo.review.Decisions
    source: demo.review.Decision
    consistency: read_your_writes
    fields:
      - {name: decision_key, type: demo.review.DecisionKey}
      - {name: kind, type: demo.review.Kind}
      - {name: note, type: String}
"#;

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("review.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir)
}

/// The refusal of `scenario` as `<code> <scenario>: <cause>`.
fn refusal_of(result: &Synthesis, scenario: &str) -> String {
    let all: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect();
    all.iter()
        .find(|line| line.contains(&format!("{scenario}:")))
        .cloned()
        .unwrap_or_else(|| panic!("no refusal of {scenario} in {all:#?}"))
}

const ANNOTATED: &str = "demo.review.Annotate/outcome/annotated";
const CONFIRMED: &str = "demo.review.Confirm/confirmed";
const RECORDED: &str = "demo.review.Record/recorded";

#[test]
fn a_refusal_names_every_creator_tried_with_its_own_cause_in_order() {
    let text = refusal_of(&synthesis(TWO_CREATORS), ANNOTATED);
    assert!(text.starts_with("ESS-SYNTH-004"), "{text}");
    let confirmed = text
        .find(CONFIRMED)
        .unwrap_or_else(|| panic!("`{CONFIRMED}` is not named: {text}"));
    let recorded = text
        .find(RECORDED)
        .unwrap_or_else(|| panic!("`{RECORDED}` is not named: {text}"));
    assert!(confirmed < recorded, "creators out of order: {text}");
}

#[test]
fn a_refusal_with_one_creator_keeps_its_text() {
    let start = TWO_CREATORS
        .find("  - name: demo.review.Record\n")
        .expect("Record");
    let end = TWO_CREATORS
        .find("  - name: demo.review.Annotate\n")
        .expect("Annotate");
    let alone = format!("{}{}", &TWO_CREATORS[..start], &TWO_CREATORS[end..]).replace(
        "demo.review.Confirm, demo.review.Record,",
        "demo.review.Confirm,",
    );
    let text = refusal_of(&synthesis(&alone), ANNOTATED);
    assert!(text.contains(CONFIRMED), "{text}");
    assert!(!text.contains(RECORDED), "{text}");
}
