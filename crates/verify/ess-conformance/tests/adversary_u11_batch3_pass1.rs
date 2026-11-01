//! Adversary pass over batch 3, U11: an accepting branch below a non-creating
//! `unknown_instance:` is sent for a stored row, and a `when:` guard on the input a
//! `when_related:` guard reads its row through is decided on the identity of the row sent.
//!
//! Each case varies one shape the unit's own tests leave out and asserts the unit's claims on
//! it: every branch has a scenario or a named refusal, and no synthesized scenario fails on the
//! interpreted target. Every model is inline and neutral, so no suite-bytes table walks it.

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

/// The `dropped` branch, as [`NOTES`] writes it.
const DROPPED: &str = r"      - name: dropped
        when: 'not defined(why)'
        accepts: nothing
";

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

/// [`SLOTS`] with `refused-b` guarded by `guard`, a `when:` block.
fn slots_guarded(guard: &str) -> String {
    replaced(SLOTS, SLOT_GUARD, guard)
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

// ---------------------------------------------------------------------------------------------
// #693: guards on the identity input
// ---------------------------------------------------------------------------------------------

#[test]
fn a_negated_identity_guard_is_decided_on_the_row_sent() {
    holds(
        &slots_guarded("        when: {not: kind == b}\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn an_any_identity_guard_is_decided_on_the_row_sent() {
    holds(
        &slots_guarded("        when: {any: [kind == a, kind == c]}\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn an_enum_membership_of_two_is_decided_on_the_row_sent() {
    holds(
        &slots_guarded("        when: {kind: [a, c]}\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn a_negated_string_identity_guard_is_decided_on_the_row_sent() {
    holds(
        &slots_newtype("String", "        when: 'kind != \"x\"'\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn an_integer_identity_range_is_decided_on_the_row_sent() {
    holds(
        &slots_newtype("Integer", "        when: kind > 5\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn an_integer_identity_closed_range_is_decided_on_the_row_sent() {
    holds(
        &slots_newtype("Integer", "        when: {all: [kind >= 2, kind <= 4]}\n"),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

#[test]
fn a_uuid_identity_guard_is_decided_on_the_row_sent() {
    holds(
        &slots_newtype(
            "Uuid",
            "        when: 'kind == \"6f1c2a3b-0000-4000-8000-000000000001\"'\n",
        ),
        USE_SLOT,
        &USE_BRANCHES,
    );
}

/// A guard reading the identity input beside another input.
#[test]
fn an_identity_guard_beside_another_input_is_decided_on_the_row_sent() {
    let text = slots_guarded("        when: {all: [kind == b, urgent == true]}\n");
    let text = replaced(
        &text,
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n    outcomes:\n      - name: refused-b\n",
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n      - {name: urgent, type: Boolean}\n    outcomes:\n      - name: refused-b\n",
    );
    holds(&text, USE_SLOT, &USE_BRANCHES);
}

/// An Optional identity input naming the related row.
#[test]
fn an_optional_identity_input_guard_is_decided_on_the_row_sent() {
    let text = replaced(
        SLOTS,
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n    outcomes:\n      - name: refused-b\n",
        "    input:\n      - {name: kind, type: Optional<demo.slots.Kind>}\n    outcomes:\n      - name: refused-b\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.slots.KindRefused\n    fields:\n      - {name: kind, type: demo.slots.Kind}\n",
        "  - name: demo.slots.KindRefused\n    fields:\n      - {name: kind, type: Optional<demo.slots.Kind>}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.slots.SlotUsed\n    fields:\n      - {name: kind, type: demo.slots.Kind}\n",
        "  - name: demo.slots.SlotUsed\n    fields:\n      - {name: kind, type: Optional<demo.slots.Kind>}\n",
    );
    holds(&text, USE_SLOT, &USE_BRANCHES);
}

/// Two slots paired by their kinds, each read through its own input.
fn pair_slots(guards: &str) -> String {
    let text = replaced(
        SLOTS,
        "    kind: enum\n    variants: [a, b, c]\n",
        "    kind: newtype\n    of: String\n",
    );
    let text = replaced(
        &text,
        "    may: [demo.slots.OpenSlot, demo.slots.UseSlot]\n",
        "    may: [demo.slots.OpenSlot, demo.slots.PairSlots]\n",
    );
    let start = text
        .find("  - name: demo.slots.UseSlot\n")
        .expect("UseSlot");
    let end = text.find("events:\n").expect("events");
    let mut text = text;
    text.replace_range(
        start..end,
        &format!(
            r"  - name: demo.slots.PairSlots
    input:
      - {{name: kind, type: demo.slots.Kind}}
      - {{name: other, type: demo.slots.Kind}}
    outcomes:
{guards}      - name: missing
        when_related: {{via: input.kind, exists: false}}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {{kind: input.kind}}
      - name: other-missing
        when_related: {{via: input.other, exists: false}}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {{kind: input.other}}
      - name: disabled
        when_related:
          via: input.kind
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {{kind: input.kind}}
      - name: other-disabled
        when_related:
          via: input.other
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {{kind: input.other}}
      - name: paired
        emits: [demo.slots.SlotsPaired]
        payload:
          demo.slots.SlotsPaired: {{kind: input.kind}}
"
        ),
    );
    replaced(
        &text,
        "  - name: demo.slots.SlotUsed\n",
        "  - name: demo.slots.SlotsPaired\n",
    )
}

const PAIR: &str = "demo.slots.PairSlots";
const PAIR_BRANCHES: [&str; 5] = [
    "missing",
    "other-missing",
    "disabled",
    "other-disabled",
    "paired",
];

/// A guard on each of the two inputs a `when_related:` guard reads through.
#[test]
fn guards_on_both_related_inputs_are_decided_on_the_rows_sent() {
    let guards = "      - name: refused-x\n        when: 'other == \"x\"'\n        error: demo.slots.KindRefused\n        payload:\n          demo.slots.KindRefused: {kind: input.other}\n      - name: refused-y\n        when: 'kind == \"y\"'\n        error: demo.slots.KindRefused\n        payload:\n          demo.slots.KindRefused: {kind: input.kind}\n";
    let run = holds(&pair_slots(guards), PAIR, &PAIR_BRANCHES);
    assert_each_written_or_refused(&run, PAIR, &["refused-x", "refused-y"]);
}

/// A guard comparing the two related inputs with each other.
#[test]
fn a_guard_comparing_two_related_inputs_is_decided_on_the_rows_sent() {
    let guards = "      - name: same\n        when: kind == other\n        error: demo.slots.KindRefused\n        payload:\n          demo.slots.KindRefused: {kind: input.other}\n";
    let run = holds(&pair_slots(guards), PAIR, &PAIR_BRANCHES);
    assert_each_written_or_refused(&run, PAIR, &["same"]);
}

/// [`SLOTS`] with a String identity, `refused-b` guarded by `guard`, and a slot that rests in
/// `Live` only after three moves; `disabled` reads a `Live` slot.
fn moved_slots(guard: &str) -> String {
    let text = slots_newtype("String", guard);
    let text = replaced(
        &text,
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

/// `disabled` and `used` of `text` are witnessed, not refused, and every scenario passes.
fn disabled_and_used_witnessed(text: &str) {
    let run = holds(text, USE_SLOT, &USE_BRANCHES);
    for branch in ["disabled", "used"] {
        let id = format!("{USE_SLOT}/outcome/{branch}");
        assert!(
            written(&run, &id).is_some(),
            "{id} is witnessed: {:#?}",
            refused(&run, &id)
        );
    }
}

/// Control: the moved slot under a guard on another input is witnessed.
#[test]
fn a_moved_related_row_under_a_guard_on_another_input_is_found() {
    let text = moved_slots("        when: urgent == true\n");
    let text = replaced(
        &text,
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n    outcomes:\n      - name: refused-b\n",
        "    input:\n      - {name: kind, type: demo.slots.Kind}\n      - {name: urgent, type: Boolean}\n    outcomes:\n      - name: refused-b\n",
    );
    disabled_and_used_witnessed(&text);
}

#[test]
fn identity_guard_synthesis_is_deterministic() {
    for text in [
        SLOTS.to_owned(),
        slots_guarded("        when: {any: [kind == a, kind == c]}\n"),
        slots_newtype("String", "        when: 'kind == \"x\"'\n"),
    ] {
        let first = serde_json::to_string(&run(&text).synthesis.suite).unwrap();
        let second = serde_json::to_string(&run(&text).synthesis.suite).unwrap();
        assert_eq!(first, second, "the suite is the same twice");
    }
}

// ---------------------------------------------------------------------------------------------
// #645: an accepting branch below `unknown_instance:`
// ---------------------------------------------------------------------------------------------

const CLOSE_BRANCHES: [&str; 2] = ["dropped", "closed"];

/// A note whose identity the target mints: the accepting branch is sent a captured note.
#[test]
fn an_accepting_branch_below_unknown_on_a_minted_identity_is_sent_a_stored_row() {
    let text = replaced(
        NOTES,
        "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
        "  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}\n",
    );
    let text = replaced(
        &text,
        "    input:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: text, type: String}\n",
        "    input:\n      - {name: text, type: String}\n",
    );
    let text = replaced(
        &text,
        "          demo.notes.NoteOpened: {note_id: input.note_id}\n  - name: demo.notes.CloseNote\n",
        "          demo.notes.NoteOpened: {note_id: {generated: true}}\n  - name: demo.notes.CloseNote\n",
    );
    holds(&text, CLOSE_NOTE, &CLOSE_BRANCHES);
}

/// A stored-field refusal of the subject answers before the accepting branch: the stored row
/// `dropped` is sent for must be one the refusal leaves.
#[test]
fn an_accepting_branch_below_a_subject_refusal_is_sent_a_row_it_reaches() {
    let text = replaced(
        NOTES,
        "      - {name: text, type: String}\n    lifecycle:",
        "      - {name: text, type: String}\n      - {name: locked, type: Boolean}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "      - {name: text, type: String}\n    outcomes:\n      - name: opened\n",
        "      - {name: text, type: String}\n      - {name: locked, type: Boolean}\n    outcomes:\n      - name: opened\n",
    );
    let text = replaced(
        &text,
        "        sets: {text: input.text}\n",
        "        sets: {text: input.text, locked: input.locked}\n",
    );
    let text = replaced(
        &text,
        DROPPED,
        "      - name: locked\n        when_subject: {predicate: locked == true}\n        error: demo.notes.NoteLocked\n      - name: dropped\n        when: 'not defined(why)'\n        accepts: nothing\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "errors:\n  - {name: demo.notes.NoteLocked, fields: []}\nevents:\n",
    );
    holds(&text, CLOSE_NOTE, &["dropped", "locked", "closed"]);
}

/// An Integer identity: the stored note is opened under an integer the send repeats.
#[test]
fn an_accepting_branch_below_unknown_on_an_integer_identity_is_sent_a_stored_row() {
    let text = replaced(
        NOTES,
        "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
        "  - {name: demo.notes.NoteId, kind: newtype, of: Integer}\n",
    );
    holds(&text, CLOSE_NOTE, &CLOSE_BRANCHES);
}

/// A note opened only inside a folder that exists: the stored note `dropped` is sent for needs
/// its folder first.
#[test]
fn an_accepting_branch_below_unknown_whose_creator_needs_a_parent_passes() {
    let text = replaced(
        NOTES,
        "entities:\n",
        "entities:\n  - name: demo.notes.Folder\n    identity: {name: folder_id, type: String}\n    fields: []\n    lifecycle:\n      initial: Open\n      states: [Open]\n      terminal: [Open]\n",
    );
    let text = replaced(
        &text,
        "      - {name: text, type: String}\n    lifecycle:",
        "      - {name: text, type: String}\n      - {name: folder_id, type: String}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "    may: [demo.notes.OpenNote, demo.notes.CloseNote]\n",
        "    may: [demo.notes.OpenNote, demo.notes.CloseNote, demo.notes.MakeFolder]\n",
    );
    let text = replaced(
        &text,
        "      - {name: text, type: String}\n    outcomes:\n      - name: opened\n",
        "      - {name: text, type: String}\n      - {name: folder_id, type: String}\n    outcomes:\n      - name: no-folder\n        when_related: {via: input.folder_id, exists: false}\n        error: demo.notes.NoFolder\n      - name: opened\n",
    );
    let text = replaced(
        &text,
        "        sets: {text: input.text}\n",
        "        sets: {text: input.text, folder_id: input.folder_id}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.notes.CloseNote\n",
        "  - name: demo.notes.MakeFolder\n    input:\n      - {name: folder_id, type: String}\n    outcomes:\n      - name: made\n        creates: demo.notes.Folder\n        instance: folder_id\n        emits: [demo.notes.FolderMade]\n        payload:\n          demo.notes.FolderMade: {folder_id: input.folder_id}\n  - name: demo.notes.CloseNote\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "errors:\n  - {name: demo.notes.NoFolder, fields: []}\nevents:\n  - name: demo.notes.FolderMade\n    fields:\n      - {name: folder_id, type: String}\n",
    );
    let run = holds(&text, CLOSE_NOTE, &CLOSE_BRANCHES);
    assert!(
        written(&run, &format!("{CLOSE_NOTE}/outcome/dropped")).is_some(),
        "dropped is witnessed: {:#?}",
        refused(&run, &format!("{CLOSE_NOTE}/outcome/dropped"))
    );
}

#[test]
fn accepting_below_unknown_synthesis_is_deterministic() {
    let first = serde_json::to_string(&run(NOTES).synthesis.suite).unwrap();
    let second = serde_json::to_string(&run(NOTES).synthesis.suite).unwrap();
    assert_eq!(first, second);
}
