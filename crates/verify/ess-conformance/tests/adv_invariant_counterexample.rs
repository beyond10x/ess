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

/// A held-state refusal that compares the stored field with the input answers every input below
/// the row's value: `stale` (`when_subject: value >= input.value`) precedes `adjusted`, and every
/// row holds `value >= 1` (`Set` is guarded on it), so `adjusted` only ever stores a value above
/// one. No input leaves the counter outside `value >= 1` unanswered, and `ESS-SYNTH-025` must not
/// name `adjusted` (the shape of the #251 reproduction's `Revise`, without its upper bound).
#[test]
fn adv2_a_stale_check_on_the_held_value_answers_the_input_below_the_bound() {
    let set = set_with(&format!(
        "      - name: set
        when: value >= 1
        creates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value, cap: 8}}
{EMITS}      - {{name: otherwise, error: bounded.counter.ValueTooLarge}}
"
    ));
    let adjust = adjust_with(&format!(
        "      - name: stale
        when_subject:
          predicate: value >= input.value
        error: bounded.counter.ValueTooLarge
      - name: adjusted
        updates: bounded.counter.Counter
        instance: counter_id
        sets: {{value: input.value}}
{EMITS}"
    ));
    let synthesis = synthesize(&ir(&spec("value >= 1", &(set + &adjust))));
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

// ---- pass 3: holes the held-row narrowing (`held_rows_answer`) must not hide --------------------

/// `HEAD` with one more stored field and the same field on both commands' input.
fn spec_with_field(field: &str, invariant: &str, commands: &str) -> String {
    let line = format!("      - {{name: {field}}}\n");
    let head = HEAD.replace("INVARIANT", invariant).replace(
        "      - {name: cap, type: Integer}\n    invariants",
        &format!("      - {{name: cap, type: Integer}}\n{line}    invariants"),
    );
    format!("{head}{}", commands.replace("FIELD\n", &line))
}

/// An input that also carries the extra field, written `FIELD` until [`spec_with_field`] fills it.
const INPUT_FIELD: &str = "    input:
      - {name: counter_id, type: bounded.counter.CounterId}
      - {name: value, type: Integer}
FIELD
";

/// `Set` creates guarded on `guard`, copying `sets`; `Adjust` refuses on `refusal` (a
/// `when_subject:` predicate) before `adjusted` copies `copied: input.copied` unguarded.
fn refused_before_copy(guard: &str, sets: &str, refusal: &str, copied: &str) -> String {
    format!(
        "  - name: bounded.counter.Set
{INPUT_FIELD}    outcomes:
      - name: set
        when: {guard}
        creates: bounded.counter.Counter
        instance: counter_id
        sets: {{{sets}}}
{EMITS}      - {{name: otherwise, error: bounded.counter.ValueTooLarge}}
  - name: bounded.counter.Adjust
{INPUT_FIELD}    outcomes:
      - name: held
        when_subject:
          predicate: {refusal}
        error: bounded.counter.ValueTooLarge
      - name: adjusted
        updates: bounded.counter.Counter
        instance: counter_id
        sets: {{{copied}: input.{copied}}}
{EMITS}"
    )
}

fn names_adjusted(synthesis: &Synthesis) -> bool {
    broken(synthesis)
        .iter()
        .any(|refusal| refusal.contains("Adjust/adjusted"))
}

/// `when_subject: value < 10` refuses only rows below ten; a row holding `value: 10` meets
/// `value >= 1` and is not refused, so `adjusted` stores `value: 0` there. The held rows are built
/// from the invariant's literal (0, 1, 2) and never from the guard's own `10`, so every admitted
/// row meets the guard and the hole is hidden. Before the narrowing `ESS-SYNTH-025` named it.
#[test]
fn adv3_a_stored_guard_that_holds_below_its_own_literal_does_not_hide_the_hole() {
    let commands = refused_before_copy(
        "value >= 1",
        "value: input.value, cap: 8, cap2: 1",
        "value < 10",
        "value",
    );
    let synthesis = synthesize(&ir(&spec_with_field(
        "cap2, type: Integer",
        "value >= 1",
        &commands,
    )));
    assert!(names_adjusted(&synthesis), "{:#?}", broken(&synthesis));
}

/// The ownership check `owner != input.owner` refuses a foreign row only; on the owner's own row
/// `adjusted` stores `value: 0`, breaking `value >= 1`. The guard reads a text field the ladder has
/// no rung for (only numbers the input sends are rungs), so `held_rows_answer` answers `None` and
/// the hole is hidden, although the row holding the input's own `owner` decides it.
#[test]
fn adv3_an_ownership_check_does_not_hide_the_hole() {
    let commands = refused_before_copy(
        "value >= 1",
        "value: input.value, cap: 8, owner: input.owner",
        "owner != input.owner",
        "value",
    );
    let synthesis = synthesize(&ir(&spec_with_field(
        "owner, type: String",
        "value >= 1",
        &commands,
    )));
    assert!(names_adjusted(&synthesis), "{:#?}", broken(&synthesis));
}

/// `owner != input.owner and value <= input.value` refuses a raise of a foreign row only; on the
/// owner's own row `value: 9` is stored, breaking `value <= 8`. The ladder gives the text field
/// `owner` the numbers the input sends (8, 9, 10), a number is never equal to the input's text, so
/// every row "meets" `owner != input.owner` and the guard is decided on `value` alone: answered.
#[test]
fn adv3_a_text_field_given_numeric_rungs_does_not_answer_the_input() {
    let commands = refused_before_copy(
        "value <= 8",
        "value: input.value, cap: 8, owner: input.owner",
        "{all: [\"owner != input.owner\", \"value <= input.value\"]}",
        "value",
    );
    let synthesis = synthesize(&ir(&spec_with_field(
        "owner, type: String",
        "value <= 8",
        &commands,
    )));
    assert!(names_adjusted(&synthesis), "{:#?}", broken(&synthesis));
}

/// A Decimal field steps by one: beside `amount > 0` the rows are -1, 0 and 1 (and 0, 1, 2 had the
/// guard's own literal been read), so `amount >= 1` holds of every admitted row, yet a stored
/// `amount: 0.5` meets the invariant, is not refused, and `adjusted` stores `amount: 0`.
#[test]
fn adv3_a_decimal_row_between_the_rungs_does_not_hide_the_hole() {
    let commands = refused_before_copy(
        "amount > 0",
        "value: input.value, cap: 8, amount: input.amount",
        "amount >= 1",
        "amount",
    );
    let synthesis = synthesize(&ir(&spec_with_field(
        "amount, type: Decimal",
        "amount > 0",
        &commands,
    )));
    assert!(names_adjusted(&synthesis), "{:#?}", broken(&synthesis));
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
