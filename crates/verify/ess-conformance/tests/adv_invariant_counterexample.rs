//! Adversary cases for `ESS-SYNTH-025` (beyond10x/ess#738): a declared refusal or guard that keeps
//! the input inside the invariant gets no refusal; the bounded search reports what it reaches.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, RefusalCause, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const HEAD: &str = "format: ess/35
system: bounded
version: v1
summary: A counter whose value stays inside its invariant.
domain: bounded.counter
types:
  - {name: bounded.counter.CounterId, kind: newtype, of: String}
entities:
  - name: bounded.counter.Counter
    identity: {name: counter_id, type: bounded.counter.CounterId}
    fields:
      - {name: value, type: Integer}
      - {name: cap, type: Integer}
    invariants:
      - INVARIANT
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
actors:
  - name: bounded.counter.Operator
    attributes: []
    may: [bounded.counter.Set, bounded.counter.Adjust, bounded.counter.Counters]
errors:
  - {name: bounded.counter.ValueTooLarge, summary: The value is outside., fields: []}
events:
  - name: bounded.counter.CounterSet
    fields:
      - {name: counter_id, type: bounded.counter.CounterId}
views:
  - name: bounded.counter.Counters
    source: bounded.counter.Counter
    consistency: read_your_writes
    fields:
      - {name: counter_id, type: bounded.counter.CounterId}
      - {name: value, type: Integer}
      - {name: cap, type: Integer}
commands:
";

const INPUT: &str = "    input:
      - {name: counter_id, type: bounded.counter.CounterId}
      - {name: value, type: Integer}
      - {name: cap, type: Integer}
";

const EMITS: &str = "        emits: [bounded.counter.CounterSet]
        payload:
          bounded.counter.CounterSet: {counter_id: input.counter_id}
";

/// `Set` creates the counter, guarded inside `value <= 8` with an `otherwise` refusal, so it
/// contributes no `ESS-SYNTH-025` of its own.
fn guarded_set() -> String {
    format!(
        "  - name: bounded.counter.Set
{INPUT}    outcomes:
      - name: set
        when: value <= 8
        creates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value, cap: 8}}
{EMITS}      - {{name: otherwise, error: bounded.counter.ValueTooLarge}}
"
    )
}

/// `Set` with branches the caller writes.
fn set_with(outcomes: &str) -> String {
    format!("  - name: bounded.counter.Set\n{INPUT}    outcomes:\n{outcomes}")
}

fn adjust_with(outcomes: &str) -> String {
    format!("  - name: bounded.counter.Adjust\n{INPUT}    outcomes:\n{outcomes}")
}

/// An `Adjust` that only reads (no copy), so the command exists for the actor's grant.
fn idle_adjust() -> String {
    adjust_with(&format!(
        "      - name: touched
        updates: bounded.counter.Counter
        instance: counter_id
        sets: {{cap: 8}}
{EMITS}"
    ))
}

fn spec(invariant: &str, commands: &str) -> String {
    format!("{}{commands}", HEAD.replace("INVARIANT", invariant))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("counter.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn broken(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| matches!(refusal.cause, RefusalCause::InvariantInputUnanswered { .. }))
        .map(ToString::to_string)
        .collect()
}

fn unguarded(name: &str, effect: &str) -> String {
    let addressed = if effect == "creates" {
        "        unknown_instance: true\n"
    } else {
        ""
    };
    format!(
        "      - name: {name}
{addressed}        {effect}: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value, cap: input.cap}}
{EMITS}"
    )
}

/// `set` (create, unknown identity) and `reset` (update), both copying unguarded.
fn both() -> String {
    unguarded("set", "creates") + &unguarded("reset", "updates")
}

// ---- false positives: the model declares the outcome for the outside input ---------------------

/// A refusal answered at the held-state step (`when_subject_state: Live` beside `when: value > 8`)
/// answers `value: 9` on every row `adjusted` can take, since `Live` is the only state and the
/// held-state step precedes the accepting branches (the one precedence rule, `ess/27`): the model
/// declares the outcome. `ESS-SYNTH-025` must not name `adjusted`.
#[test]
fn adv_a_held_state_refusal_before_an_unguarded_update_gets_no_refusal() {
    let adjust = adjust_with(&format!(
        "      - name: too-large
        when_subject_state: Live
        when: value > 8
        error: bounded.counter.ValueTooLarge
      - name: adjusted
        updates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value}}
{EMITS}"
    ));
    let synthesis = synthesize(&ir(&spec("value <= 8", &(guarded_set() + &adjust))));
    let witnessed = synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "bounded.counter.Adjust/outcome/too-large");
    assert_eq!(
        broken(&synthesis),
        Vec::<String>::new(),
        "the refusal's own scenario is synthesized: {witnessed}"
    );
}

/// `when: {not: …}` and `when: {all: […]}` keep the input inside.
#[test]
fn adv_connective_guards_that_keep_the_input_inside_get_no_refusal() {
    for guard in [
        "{not: \"value > 8\"}",
        "{all: [\"value <= 8\", \"value >= 0\"]}",
    ] {
        let set = set_with(&format!(
            "      - name: set
        when: {guard}
        creates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value, cap: 8}}
{EMITS}      - {{name: otherwise, error: bounded.counter.ValueTooLarge}}
"
        ));
        let synthesis = synthesize(&ir(&spec("value <= 8", &(set + &idle_adjust()))));
        assert_eq!(broken(&synthesis), Vec::<String>::new(), "guard {guard}");
    }
}

/// A declared refusal whose `when:` is an `any` answers every outside input.
#[test]
fn adv_an_any_refusal_that_answers_the_input_gets_no_refusal() {
    let set = set_with(&format!(
        "      - name: too-large
        when: {{any: [\"value > 8\", \"value < -100\"]}}
        error: bounded.counter.ValueTooLarge
{}{}",
        unguarded("set", "creates").replace("cap: input.cap", "cap: 8"),
        unguarded("reset", "updates").replace("cap: input.cap", "cap: 8"),
    ));
    let synthesis = synthesize(&ir(&spec("value <= 8", &(set + &idle_adjust()))));
    assert_eq!(broken(&synthesis), Vec::<String>::new());
}

// ---- false negatives inside the bounded search -------------------------------------------------

/// A refusal that answers only part of the outside (`value > 9`) leaves `value: 9` unanswered.
#[test]
fn adv_a_refusal_that_answers_only_part_of_the_outside_still_names_the_hole() {
    let set = set_with(&format!(
        "      - name: too-large
        when: value > 9
        error: bounded.counter.ValueTooLarge
{}",
        both().replace("cap: input.cap", "cap: 8"),
    ));
    let synthesis = synthesize(&ir(&spec("value <= 8", &(set + &idle_adjust()))));
    let found = broken(&synthesis);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].contains("value: 9"), "{found:#?}");
}

/// A lower bound: `value >= 1` is broken by `value: 0`.
#[test]
fn adv_a_lower_bound_invariant_is_named() {
    let set = set_with(&both().replace("cap: input.cap", "cap: 8"));
    let synthesis = synthesize(&ir(&spec("value >= 1", &(set + &idle_adjust()))));
    let found = broken(&synthesis);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].contains("value: 0"), "{found:#?}");
}

/// An invariant over two copied inputs, `value <= cap`.
#[test]
fn adv_an_invariant_over_two_copied_inputs_is_named() {
    let set = set_with(&both());
    let synthesis = synthesize(&ir(&spec("value <= cap", &(set + &idle_adjust()))));
    let found = broken(&synthesis);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].contains("value <= cap"), "{found:#?}");
}
