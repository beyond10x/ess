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

// Adversary pass 1 (beyond10x/ess#728).

const RECORD_BLOCK_START: &str = "  - name: demo.review.Record\n";
const ANNOTATE_BLOCK_START: &str = "  - name: demo.review.Annotate\n";
const NOT_ACCEPTED_BRANCH: &str = "      - name: not-accepted\n        when_subject: {predicate: kind != Accept}\n        error: demo.review.NotAccepted\n";

/// The `(creator, cause)` pairs after "no creating branch arranges it: ", in the order written.
fn adv728_pairs(text: &str) -> Vec<(String, String)> {
    let marker = "no creating branch arranges it: ";
    let at = text
        .find(marker)
        .unwrap_or_else(|| panic!("no creators list in: {text}"));
    text[at + marker.len()..]
        .split("; `demo.review.")
        .enumerate()
        .map(|(nth, part)| {
            let part = if nth == 0 {
                part.trim_start_matches('`').to_string()
            } else {
                format!("demo.review.{part}")
            };
            let (creator, cause) = part
                .split_once("`: ")
                .unwrap_or_else(|| panic!("no `creator`: cause in {part:?} of {text}"));
            (creator.to_string(), cause.to_string())
        })
        .collect()
}

/// Each creator is paired with its own cause: the cause names that creator's branch and not the
/// other's. A Display that wrote the first cause beside every creator passes the test above.
#[test]
fn adv728_each_creator_is_paired_with_its_own_cause() {
    let text = refusal_of(&synthesis(TWO_CREATORS), ANNOTATED);
    eprintln!("{text}");
    let pairs = adv728_pairs(&text);
    let creators: Vec<&str> = pairs.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(creators, [CONFIRMED, RECORDED], "{text}");
    assert!(
        pairs[0].1.contains(CONFIRMED) && !pairs[0].1.contains(RECORDED),
        "{text}"
    );
    assert!(
        pairs[1].1.contains(RECORDED) && !pairs[1].1.contains(CONFIRMED),
        "{text}"
    );
}

/// The same model without the `when_subject` branch: the refusal comes through `arrange`, not
/// `subject_fact::search_under`, and names both creators the same way.
#[test]
fn adv728_the_plain_arrange_route_names_every_creator() {
    let plain = TWO_CREATORS
        .replace(NOT_ACCEPTED_BRANCH, "")
        .replace("  - {name: demo.review.NotAccepted, fields: []}\n", "");
    assert_ne!(plain, TWO_CREATORS);
    let text = refusal_of(&synthesis(&plain), ANNOTATED);
    eprintln!("{text}");
    assert!(text.starts_with("ESS-SYNTH-004"), "{text}");
    let pairs = adv728_pairs(&text);
    let creators: Vec<&str> = pairs.iter().map(|(c, _)| c.as_str()).collect();
    assert_eq!(creators, [CONFIRMED, RECORDED], "{text}");
    assert!(pairs[0].1.contains(CONFIRMED), "{text}");
    assert!(pairs[1].1.contains(RECORDED), "{text}");
}

/// Declaration order is not the order tried: `Record` declared before `Confirm` gives the same
/// refusal, byte for byte.
#[test]
fn adv728_declaration_order_does_not_move_the_tried_list() {
    let start = TWO_CREATORS.find(RECORD_BLOCK_START).expect("Record");
    let end = TWO_CREATORS.find(ANNOTATE_BLOCK_START).expect("Annotate");
    let confirm = TWO_CREATORS
        .find("  - name: demo.review.Confirm\n")
        .expect("Confirm");
    let swapped = format!(
        "{}{}{}{}",
        &TWO_CREATORS[..confirm],
        &TWO_CREATORS[start..end],
        &TWO_CREATORS[confirm..start],
        &TWO_CREATORS[end..]
    );
    assert_ne!(swapped, TWO_CREATORS);
    assert_eq!(
        refusal_of(&synthesis(&swapped), ANNOTATED),
        refusal_of(&synthesis(TWO_CREATORS), ANNOTATED)
    );
}

/// `Record` reaches its branch, but only with `kind: Erratum`, and `annotated` needs a row with
/// `kind == Accept`; `Confirm` still reaches no branch. Either repair makes `annotated` reachable,
/// and the refusal names only `Confirm`: the misdirection of #728, with the second creator's
/// cause of another kind (no row it leaves satisfies the subject guard) instead of another
/// unreachable branch.
#[test]
#[ignore = "backlog: story:a-subject-route-refusal-names-every-creator-whatever-its-cause"]
fn adv728_a_creator_whose_rows_miss_the_subject_guard_is_named_too() {
    let mixed = TWO_CREATORS
        .replace(
            "      - name: incomplete\n        when: kind == Erratum\n        error: demo.review.Incomplete\n",
            "",
        )
        .replace("  - {name: demo.review.Incomplete, fields: []}\n", "");
    assert_ne!(mixed, TWO_CREATORS);
    let text = refusal_of(&synthesis(&mixed), ANNOTATED);
    eprintln!("{text}");
    assert!(text.contains(CONFIRMED), "{text}");
    assert!(
        text.contains(RECORDED),
        "`{RECORDED}` leaves rows, none with kind == Accept, and is not named: {text}"
    );
}

/// A move command `Unwitnessable` from either side, `{moves}` its transition.
fn adv728_blocked_move(name: &str, outcome: &str, moves: &str) -> String {
    format!(
        "- name: example.desk.{name}
  input:
  - {{name: id, type: String}}
  - {{name: kind, type: example.desk.Kind}}
  outcomes:
  - name: accept
    when: kind == Accept
    error: example.desk.Refused
  - name: erratum
    when: kind == Erratum
    error: example.desk.Refused
  - name: {outcome}
    moves: example.desk.Case.{moves}
    instance: id
    emits: [example.desk.Moved]
    payload:
      example.desk.Moved: {{id: input.id}}
"
    )
}

/// `Finish` runs from `Held` and from `Closed`, two target states of `arrange_first`. `Begin`
/// creates into `Open`, `Prepare` into `Draft`, and every move out of either is a branch no input
/// reaches, a different one per creator and per target. Every target fails with a list of
/// creators, so `Unreachable::across_targets` merges them: the refusal names the blocked move of
/// each creator in both targets. A mutant keeping the first target's list (the pre-#728
/// `first.get_or_insert`) leaves the `Held` causes out, and nothing else in the suite reaches
/// the merge (no repository model's refusal digest moved).
#[test]
fn adv728_every_target_state_contributes_its_creators_causes() {
    let model = format!(
        "format: ess/35
system: example
version: v1
domain: example.desk
types:
  - {{name: example.desk.Kind, kind: enum, variants: [Accept, Erratum]}}
entities:
  - name: example.desk.Case
    identity: {{name: id, type: String}}
    fields:
      - {{name: note, type: String}}
    lifecycle:
      initial: Open
      states: [Open, Draft, Held, Closed, Done]
      terminal: [Done]
      transitions:
        - {{name: redraft, from: [Open], to: Draft}}
        - {{name: hold-open, from: [Open], to: Held}}
        - {{name: hold-draft, from: [Draft], to: Held}}
        - {{name: close-open, from: [Open], to: Closed}}
        - {{name: close-draft, from: [Draft], to: Closed}}
        - {{name: finish, from: [Held, Closed], to: Done}}
events:
  - name: example.desk.Moved
    fields: [{{name: id, type: String}}]
errors:
  - name: example.desk.Refused
commands:
- name: example.desk.Begin
  input:
  - {{name: id, type: String}}
  outcomes:
  - name: begun
    creates: example.desk.Case
    instance: id
    sets: {{note: '\"\"'}}
    emits: [example.desk.Moved]
    payload:
      example.desk.Moved: {{id: input.id}}
- name: example.desk.Prepare
  input:
  - {{name: id, type: String}}
  outcomes:
  - name: prepared
    creates: example.desk.Case
    into: Draft
    instance: id
    sets: {{note: '\"\"'}}
    emits: [example.desk.Moved]
    payload:
      example.desk.Moved: {{id: input.id}}
{}{}{}{}{}- name: example.desk.Finish
  input:
  - {{name: id, type: String}}
  outcomes:
  - name: finished
    moves: example.desk.Case.finish
    instance: id
    emits: [example.desk.Moved]
    payload:
      example.desk.Moved: {{id: input.id}}
",
        adv728_blocked_move("Redraft", "redrafted", "redraft"),
        adv728_blocked_move("HoldOpen", "held", "hold-open"),
        adv728_blocked_move("HoldDraft", "held", "hold-draft"),
        adv728_blocked_move("CloseOpen", "closed", "close-open"),
        adv728_blocked_move("CloseDraft", "closed", "close-draft"),
    );
    let result = synthesis(&model);
    let text = refusal_of(&result, "example.desk.Finish/outcome/finished");
    eprintln!("{text}");
    assert!(text.starts_with("ESS-SYNTH-004"), "{text}");
    for blocked in [
        "example.desk.CloseOpen/closed",
        "example.desk.CloseDraft/closed",
        "example.desk.HoldOpen/held",
        "example.desk.HoldDraft/held",
    ] {
        assert!(text.contains(blocked), "`{blocked}` is not named: {text}");
    }
}
