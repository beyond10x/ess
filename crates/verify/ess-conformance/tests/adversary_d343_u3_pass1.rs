//! Adversary pass 1 over D343 U3
//! (`story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent`).
//!
//! The acceptance: a branch's own `when:` guard on the identity input of a subject-addressed row
//! is decided on the identity of the row arranged, its creator steered onto that identity, and
//! the branch is refused `ESS-SYNTH-003` naming the guard only where no row can meet it; with an
//! identity guard and a related row three moves away, a branch is never refused as a repeated
//! identity where a row of another identity witnesses it; synthesis stays deterministic.
//!
//! Each case varies one shape the unit's own tests leave out: a compound guard reading the
//! identity beside another input on an accepting branch below `unknown_instance:` (a subject
//! branch deciding on its own identity is refused by the compiler as `unobservable_fact`), Integer
//! and enum-membership identity guards, the refusal arm
//! where no row can meet the guard, and the moved related row over an enum and an Integer
//! identity. Every model is inline and neutral, so no suite-bytes table walks it.

use std::collections::BTreeMap;
use std::fmt::Write;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// The `dropped` branch, as [`NOTES`] writes it.
const DROPPED: &str = r"      - name: dropped
        when: 'not defined(why)'
        accepts: nothing
";

/// The String identity type, as [`NOTES`] declares it.
const STRING_ID: &str = "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n";

/// Notes are opened and closed; a close without a reason is dropped.
const NOTES: &str = r"format: ess/32
system: demo
version: v1
domain: demo.notes
summary: Notes are opened and closed; a close without a reason is dropped.
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
actors:
  - name: demo.notes.User
    may: [demo.notes.OpenNote, demo.notes.CloseNote]
commands:
  - name: demo.notes.OpenNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: opened
        creates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteOpened]
        payload:
          demo.notes.NoteOpened: {note_id: input.note_id}
  - name: demo.notes.CloseNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: why, type: Optional<String>, presence: omitted_when_absent}
    outcomes:
      - name: dropped
        when: 'not defined(why)'
        accepts: nothing
      - name: unknown
        unknown_instance: true
        refuses: false
      - name: closed
        deletes: demo.notes.Note
        instance: note_id
        emits: [demo.notes.NoteClosed]
        payload:
          demo.notes.NoteClosed: {note_id: input.note_id}
events:
  - name: demo.notes.NoteOpened
    fields:
      - {name: note_id, type: demo.notes.NoteId}
  - name: demo.notes.NoteClosed
    fields:
      - {name: note_id, type: demo.notes.NoteId}
";

const CLOSE_NOTE: &str = "demo.notes.CloseNote";
const CLOSE_BRANCHES: [&str; 2] = ["dropped", "closed"];

/// The guard of `refused-b`, as [`SLOTS`] writes it.
const SLOT_GUARD: &str = "        when: kind == b\n";

/// A slot is opened by its kind; `UseSlot` refuses kind b, then a missing or disabled slot.
const SLOTS: &str = r"format: ess/32
system: demo
version: v1
domain: demo.slots
summary: A slot is opened by its kind; using a slot of kind b is refused.
types:
  - name: demo.slots.Kind
    kind: enum
    variants: [a, b, c]
entities:
  - name: demo.slots.Slot
    identity: {name: kind, type: demo.slots.Kind}
    fields:
      - {name: enabled, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.slots.Slots
    source: demo.slots.Slot
    consistency: read_your_writes
    fields:
      - {name: kind, type: demo.slots.Kind}
      - {name: state, type: demo.slots.Slot.State}
      - {name: enabled, type: Boolean}
actors:
  - name: demo.slots.Caller
    may: [demo.slots.OpenSlot, demo.slots.UseSlot]
errors:
  - name: demo.slots.SlotMissing
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.SlotDisabled
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.KindRefused
    fields:
      - {name: kind, type: demo.slots.Kind}
commands:
  - name: demo.slots.OpenSlot
    input:
      - {name: kind, type: demo.slots.Kind}
      - {name: enabled, type: Boolean}
    outcomes:
      - name: opened
        creates: demo.slots.Slot
        instance: kind
        sets: {enabled: input.enabled}
        emits: [demo.slots.SlotOpened]
        payload:
          demo.slots.SlotOpened: {kind: input.kind}
  - name: demo.slots.UseSlot
    input:
      - {name: kind, type: demo.slots.Kind}
    outcomes:
      - name: refused-b
        when: kind == b
        error: demo.slots.KindRefused
        payload:
          demo.slots.KindRefused: {kind: input.kind}
      - name: missing
        when_related: {via: input.kind, exists: false}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {kind: input.kind}
      - name: disabled
        when_related:
          via: input.kind
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {kind: input.kind}
      - name: used
        emits: [demo.slots.SlotUsed]
        payload:
          demo.slots.SlotUsed: {kind: input.kind}
events:
  - name: demo.slots.SlotOpened
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.SlotUsed
    fields:
      - {name: kind, type: demo.slots.Kind}
";

const USE_SLOT: &str = "demo.slots.UseSlot";
const USE_BRANCHES: [&str; 4] = ["refused-b", "missing", "disabled", "used"];

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

/// Every refusal of `wanted`, as `[code] text`.
fn refused(run: &Run, wanted: &str) -> Vec<String> {
    run.synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// The scenario `wanted` as the suite serializes it, where one is written.
fn written(run: &Run, wanted: &str) -> Option<Value> {
    run.synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap())
}

/// Every scenario passes on the interpreted target.
fn assert_all_pass(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}\n{}",
                result.status,
                result.diagnostics().collect::<Vec<_>>(),
                written(run, id).map_or_else(String::new, |scenario| {
                    serde_json::to_string_pretty(&scenario).unwrap()
                })
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// Every branch of `command` named in `branches` has a scenario or a refusal.
fn assert_each_written_or_refused(run: &Run, command: &str, branches: &[&str]) {
    for branch in branches {
        let id = format!("{command}/outcome/{branch}");
        assert!(
            written(run, &id).is_some() || !refused(run, &id).is_empty(),
            "{id} has a scenario or a refusal"
        );
    }
}

/// Both claims at once: each branch is accounted for and every scenario passes.
fn holds(text: &str, command: &str, branches: &[&str]) -> Run {
    let run = run(text);
    assert_each_written_or_refused(&run, command, branches);
    assert_all_pass(&run);
    run
}

/// [`NOTES`] with `dropped` guarded by `guard`, a `when:` line, in place of `not defined(why)`.
fn notes_dropped_when(guard: &str) -> String {
    replaced(
        NOTES,
        DROPPED,
        &format!("      - name: dropped\n{guard}        accepts: nothing\n"),
    )
}

/// `text`, a [`NOTES`] variant, with the note identity an enum of `a`, `b` and `c`.
fn enum_identity(text: &str) -> String {
    replaced(
        text,
        STRING_ID,
        "  - {name: demo.notes.NoteId, kind: enum, variants: [a, b, c]}\n",
    )
}

/// `text`, a [`NOTES`] variant, with the note identity a newtype of Integer.
fn integer_identity(text: &str) -> String {
    replaced(
        text,
        STRING_ID,
        "  - {name: demo.notes.NoteId, kind: newtype, of: Integer}\n",
    )
}

/// The identity literals every `OpenNote` of `scenario` is sent: the identities of the notes it
/// stores.
fn opened_as(scenario: &Value) -> Vec<Value> {
    scenario["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|step| {
            step["step"] == "execute_command" && step["command"] == "demo.notes.OpenNote"
        })
        .map(|step| step["input"]["note_id"]["value"].clone())
        .collect()
}

/// `branch` of `CloseNote` in `run` is witnessed on a note opened as `identity`.
fn sent_a_note_opened_as(run: &Run, branch: &str, identity: &Value) {
    let id = format!("{CLOSE_NOTE}/outcome/{branch}");
    let scenario = written(run, &id).unwrap_or_else(|| {
        panic!(
            "{id} is witnessed: a note opened as {identity} meets its guard, so it is not refused \
             (story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent): {:#?}",
            refused(run, &id)
        )
    });
    assert!(
        opened_as(&scenario).contains(identity),
        "{id} is sent a note opened as {identity}: {scenario:#}"
    );
}

/// Every refusal of `id` is `ESS-SYNTH-003` naming `guard` as a predicate renders it, and none is
/// the repeated-identity refusal.
fn refused_naming(run: &Run, id: &str, guard: &str) {
    let refusals = refused(run, id);
    assert!(!refusals.is_empty(), "{id} has a refusal: {refusals:#?}");
    for refusal in &refusals {
        assert!(
            !refusal.contains("creates a second row"),
            "{id} is not refused as a repeated identity: {refusal}"
        );
        assert!(
            refusal.starts_with("[ESS-SYNTH-003]") && refusal.contains(&format!("`{guard}`")),
            "{id} is refused naming `{guard}`: {refusal}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// A compound guard reading the identity beside another input
// ---------------------------------------------------------------------------------------------

/// `dropped` below `unknown_instance:` is guarded by `note_id == "keep"` together with the reason
/// input. A note opened as `"keep"` meets the guard, so the creator is steered onto it and the
/// branch is witnessed; it is not refused as though no row could meet it.
#[test]
fn a_compound_identity_guard_below_unknown_is_sent_the_row_of_that_identity() {
    let text =
        notes_dropped_when("        when: {all: ['note_id == \"keep\"', 'not defined(why)']}\n");
    let run = run(&text);
    sent_a_note_opened_as(&run, "dropped", &Value::from("keep"));
    assert_each_written_or_refused(&run, CLOSE_NOTE, &CLOSE_BRANCHES);
    assert_all_pass(&run);
}

/// The same over an enum identity: `note_id == b` beside the reason input is sent a note opened
/// as `b`.
#[test]
fn a_compound_enum_identity_guard_below_unknown_is_sent_the_row_of_that_identity() {
    let text = enum_identity(&notes_dropped_when(
        "        when: {all: [note_id == b, 'not defined(why)']}\n",
    ));
    let run = run(&text);
    sent_a_note_opened_as(&run, "dropped", &Value::from("b"));
    assert_each_written_or_refused(&run, CLOSE_NOTE, &CLOSE_BRANCHES);
    assert_all_pass(&run);
}

// ---------------------------------------------------------------------------------------------
// Integer and membership identity guards, and the refusal arm
// ---------------------------------------------------------------------------------------------

/// An Integer identity range below `unknown_instance:`: `dropped` is sent a note whose identity
/// is above 5, and every scenario passes.
#[test]
fn an_integer_identity_range_below_unknown_is_sent_a_row_inside_it() {
    let text = integer_identity(&notes_dropped_when("        when: note_id > 5\n"));
    let run = run(&text);
    let id = format!("{CLOSE_NOTE}/outcome/dropped");
    let scenario = written(&run, &id).unwrap_or_else(|| {
        panic!(
            "{id} is witnessed on a note above 5: {:#?}",
            refused(&run, &id)
        )
    });
    assert!(
        opened_as(&scenario)
            .iter()
            .any(|value| value.as_f64().is_some_and(|number| number > 5.0)),
        "{id} is sent a note opened above 5: {scenario:#}"
    );
    assert_each_written_or_refused(&run, CLOSE_NOTE, &CLOSE_BRANCHES);
    assert_all_pass(&run);
}

/// An enum membership below `unknown_instance:`: `dropped` is sent a note opened as `b` or `c`,
/// and `closed` one opened as `a`.
#[test]
fn an_enum_identity_membership_below_unknown_leaves_each_branch_its_row() {
    let text = enum_identity(&notes_dropped_when("        when: {note_id: [b, c]}\n"));
    let run = holds(&text, CLOSE_NOTE, &CLOSE_BRANCHES);
    let dropped = format!("{CLOSE_NOTE}/outcome/dropped");
    let scenario = written(&run, &dropped)
        .unwrap_or_else(|| panic!("{dropped} is witnessed: {:#?}", refused(&run, &dropped)));
    let opened = opened_as(&scenario);
    assert!(
        opened.contains(&Value::from("b")) || opened.contains(&Value::from("c")),
        "{dropped} is sent a note opened as b or c: {scenario:#}"
    );
    sent_a_note_opened_as(&run, "closed", &Value::from("a"));
}

/// No note `b` can be opened: `dropped`, guarded by `note_id == b`, is reached by no row, so it is
/// refused `ESS-SYNTH-003` naming the guard and not written.
#[test]
fn an_identity_guard_no_row_meets_is_refused_naming_it() {
    let text = enum_identity(&notes_dropped_when("        when: note_id == b\n"));
    let text = replaced(
        &text,
        "      - name: opened\n",
        "      - name: refused-b\n        when: note_id == b\n        error: demo.notes.Refused\n      - name: opened\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "errors:\n  - {name: demo.notes.Refused, fields: []}\nevents:\n",
    );
    let run = holds(&text, CLOSE_NOTE, &CLOSE_BRANCHES);
    let id = format!("{CLOSE_NOTE}/outcome/dropped");
    assert!(
        written(&run, &id).is_none(),
        "{id} is reached by no row and is not written: {:#}",
        written(&run, &id).unwrap_or_default()
    );
    refused_naming(&run, &id, "note_id == b");
}

// ---------------------------------------------------------------------------------------------
// A related row three moves away, under identity guards of other types
// ---------------------------------------------------------------------------------------------

/// `text`, a [`SLOTS`] variant, with a slot that rests in `Live` only after three moves;
/// `disabled` reads a `Live` slot.
fn moved(text: &str) -> String {
    let text = replaced(
        text,
        "      initial: Open\n      states: [Open]\n      terminal: [Open]\n",
        "      initial: Open\n      states: [Open, Armed, Primed, Live]\n      terminal: [Live]\n      transitions:\n        - {name: arm, from: [Open], to: Armed}\n        - {name: prime, from: [Armed], to: Primed}\n        - {name: go, from: [Primed], to: Live}\n",
    );
    let text = replaced(
        &text,
        "          predicate: enabled == false\n",
        "          predicate: {all: [enabled == false, state == Live]}\n",
    );
    let text = replaced(
        &text,
        "    may: [demo.slots.OpenSlot, demo.slots.UseSlot]\n",
        "    may: [demo.slots.OpenSlot, demo.slots.UseSlot, demo.slots.Arm, demo.slots.Prime, demo.slots.Go]\n",
    );
    let mut steps = String::new();
    for (command, transition) in [("Arm", "arm"), ("Prime", "prime"), ("Go", "go")] {
        writeln!(
            steps,
            "  - name: demo.slots.{command}\n    input:\n      - {{name: kind, type: demo.slots.Kind}}\n    outcomes:\n      - name: moved\n        moves: demo.slots.Slot.{transition}\n        instance: kind\n        emits: [demo.slots.SlotOpened]\n        payload:\n          demo.slots.SlotOpened: {{kind: input.kind}}"
        )
        .unwrap();
    }
    replaced(
        &text,
        "  - name: demo.slots.UseSlot\n",
        &format!("{steps}  - name: demo.slots.UseSlot\n"),
    )
}

/// [`SLOTS`] with `Kind` a newtype of `of` and `refused-b` guarded by `guard`.
fn slots_newtype(of: &str, guard: &str) -> String {
    let text = replaced(
        SLOTS,
        "    kind: enum\n    variants: [a, b, c]\n",
        &format!("    kind: newtype\n    of: {of}\n"),
    );
    replaced(&text, SLOT_GUARD, guard)
}

/// [`SLOTS`] with an extra Boolean input `urgent` on `UseSlot`.
fn with_urgent(text: &str) -> String {
    replaced(
        text,
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n    outcomes:\n      - name: refused-b\n",
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n      - {name: urgent, type: Boolean}\n    outcomes:\n      - name: refused-b\n",
    )
}

/// `disabled` and `used` of `text` are witnessed, every scenario passes, and no branch of
/// `UseSlot` is refused as a repeated identity.
fn moved_branches_witnessed(text: &str) -> Run {
    let run = holds(text, USE_SLOT, &USE_BRANCHES);
    for branch in USE_BRANCHES {
        let id = format!("{USE_SLOT}/outcome/{branch}");
        for refusal in refused(&run, &id) {
            assert!(
                !refusal.contains("creates a second row"),
                "{id} is not refused as a repeated identity where a row of another identity \
                 witnesses it (story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent): \
                 {refusal}"
            );
        }
    }
    for branch in ["disabled", "used"] {
        let id = format!("{USE_SLOT}/outcome/{branch}");
        assert!(
            written(&run, &id).is_some(),
            "{id} is witnessed: three kinds keep the rows apart: {:#?}",
            refused(&run, &id)
        );
    }
    run
}

/// The enum identity of [`SLOTS`], `kind == b`, beside a slot three moves away: `disabled` needs
/// at most three rows (a `Live` slot not `b`, its boundary runs on `b` and on the third kind), so
/// it is witnessed, never refused as creating a kind twice.
#[test]
fn an_enum_identity_guard_beside_a_moved_related_row_is_witnessed() {
    moved_branches_witnessed(&moved(SLOTS));
}

/// The Integer identity range `kind > 5` beside a slot three moves away is witnessed.
#[test]
fn an_integer_identity_range_beside_a_moved_related_row_is_witnessed() {
    moved_branches_witnessed(&moved(&slots_newtype(
        "Integer",
        "        when: kind > 5\n",
    )));
}

/// The String identity guard of the acceptance, held to the control: under `kind == "x"`,
/// `disabled` and `used` are witnessed as they are with the guard on another input, and carry no
/// refusal the control does not.
#[test]
fn a_string_identity_guard_beside_a_moved_related_row_matches_the_control() {
    let guarded = moved_branches_witnessed(&moved(&slots_newtype(
        "String",
        "        when: 'kind == \"x\"'\n",
    )));
    let control = run(&with_urgent(&moved(&slots_newtype(
        "String",
        "        when: urgent == true\n",
    ))));
    for branch in ["disabled", "used"] {
        let id = format!("{USE_SLOT}/outcome/{branch}");
        let extra = refused(&guarded, &id);
        let base = refused(&control, &id);
        assert!(
            extra.len() <= base.len(),
            "{id} carries no refusal the control does not: guarded {extra:#?}, control {base:#?}"
        );
    }
}

#[test]
fn compound_identity_guard_synthesis_is_deterministic() {
    for text in [
        notes_dropped_when("        when: {all: ['note_id == \"keep\"', 'not defined(why)']}\n"),
        enum_identity(&notes_dropped_when("        when: {note_id: [b, c]}\n")),
        moved(SLOTS),
    ] {
        let first = serde_json::to_string(&run(&text).synthesis.suite).unwrap();
        let second = serde_json::to_string(&run(&text).synthesis.suite).unwrap();
        assert_eq!(first, second, "the suite is the same twice");
    }
}
