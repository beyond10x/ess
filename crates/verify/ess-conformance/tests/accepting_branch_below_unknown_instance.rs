//! An accepting branch naming no subject of its own, declared beside a non-creating
//! `unknown_instance:` branch, is sent for a stored row
//! (<https://github.com/beyond10x/ess/issues/645>).
//!
//! `CloseNote(note_id, why)` drops a close without a reason (`when: 'not defined(why)'`,
//! `accepts: nothing`), answers `unknown` for a note no row carries, and otherwise deletes the
//! note. Existence answers before every accepting branch
//! (`docs/design/cross-record-and-stored-field-guards.md`), so `dropped` is reached only for a
//! note that is stored. Synthesis sent it a literal identity and arranged no note, and the
//! interpreted target answered `unknown`. The scenario now opens a note and sends it.
//!
//! Every model is inline, so it is no repository model the suite-bytes tables walk.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// The note creator, which [`no_creator`] drops.
const OPEN_NOTE: &str = r"  - name: demo.notes.OpenNote
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
";

/// The `dropped` branch, as [`MODEL`] writes it.
const DROPPED: &str = r"      - name: dropped
        when: 'not defined(why)'
        accepts: nothing
";

/// The issue's model.
const MODEL: &str = r"format: ess/32
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

/// The `dropped` scenario `ess 0.77.0` writes for [`input_refusal`]: the refusal is sent a
/// literal note first, then read back on an opened one.
const INPUT_REFUSAL_DROPPED_0_77_0: &str = r#"{"purpose":"`demo.notes.CloseNote` answers `dropped` for an input that satisfies that branch's guard","steps":[{"step":"execute_command","command":"demo.notes.CloseNote","actor":"demo.notes.User","input":{"note_id":{"kind":"literal","value":"note_id"}}},{"step":"expect_outcome","outcome":{"command":"demo.notes.CloseNote","outcome":"dropped"}},{"step":"expect_error","error":"demo.notes.Dropped"},{"step":"expect_no_event","event":"demo.notes.NoteClosed"},{"step":"expect_no_event","event":"demo.notes.NoteOpened"},{"step":"execute_command","command":"demo.notes.OpenNote","actor":"demo.notes.User","input":{"note_id":{"kind":"literal","value":"note_id"},"text":{"kind":"literal","value":"text"}}},{"step":"expect_outcome","outcome":{"command":"demo.notes.OpenNote","outcome":"opened"}},{"step":"capture_instance","instance":"note","entity":"demo.notes.Note","event":"demo.notes.NoteOpened","field":"note_id"},{"step":"query_view","view":"demo.notes.Notes"},{"step":"snapshot_complete_subject","view":"demo.notes.Notes","subject":{"note_id":{"kind":"instance","instance":"note"}},"shape":{"identity_field":"note_id","fields":[{"name":"note_id","type":"demo.notes.NoteId"},{"name":"text","type":"String"}],"declarations":{"demo.notes.NoteId":{"kind":"newtype","of":"String"}}}},{"step":"execute_command","command":"demo.notes.CloseNote","actor":"demo.notes.User","input":{"note_id":{"kind":"instance","instance":"note"}}},{"step":"expect_outcome","outcome":{"command":"demo.notes.CloseNote","outcome":"dropped"}},{"step":"expect_error","error":"demo.notes.Dropped"},{"step":"expect_no_event","event":"demo.notes.NoteClosed"},{"step":"expect_no_event","event":"demo.notes.NoteOpened"},{"step":"query_view","view":"demo.notes.Notes"},{"step":"expect_complete_subject_unchanged","view":"demo.notes.Notes"}],"source":[{"kind":"type","name":"demo.notes.NoteId"},{"kind":"entity","name":"demo.notes.Note"},{"kind":"command","name":"demo.notes.CloseNote"},{"kind":"command","name":"demo.notes.OpenNote"},{"kind":"outcome","name":{"command":"demo.notes.CloseNote","outcome":"dropped"}},{"kind":"outcome","name":{"command":"demo.notes.OpenNote","outcome":"opened"}},{"kind":"event","name":"demo.notes.NoteClosed"},{"kind":"event","name":"demo.notes.NoteOpened"},{"kind":"error","name":"demo.notes.Dropped"},{"kind":"view","name":"demo.notes.Notes"},{"kind":"actor","name":"demo.notes.User"}]}"#;

const DROPPED_ID: &str = "demo.notes.CloseNote/outcome/dropped";
const KEPT_ID: &str = "demo.notes.CloseNote/outcome/kept";

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// [`MODEL`] whose subject-less accepting branch is the default, below `unknown_instance:`, and
/// whose deleting branch is guarded on the reason.
fn accepting_default() -> String {
    let text = replaced(MODEL, DROPPED, "");
    let text = replaced(
        &text,
        "      - name: closed\n",
        "      - name: closed\n        when: 'defined(why)'\n",
    );
    replaced(
        &text,
        "          demo.notes.NoteClosed: {note_id: input.note_id}\n",
        "          demo.notes.NoteClosed: {note_id: input.note_id}\n      - name: kept\n        accepts: nothing\n",
    )
}

/// [`MODEL`] whose `dropped` is guarded by an orderable Integer input.
fn orderable_guard() -> String {
    let text = replaced(
        MODEL,
        "      - {name: why, type: Optional<String>, presence: omitted_when_absent}\n",
        "      - {name: count, type: Integer}\n",
    );
    replaced(
        &text,
        DROPPED,
        "      - name: dropped\n        when: 'count > 3'\n        accepts: nothing\n",
    )
}

/// [`MODEL`] whose `dropped` refuses the input instead of accepting it.
fn input_refusal() -> String {
    let text = replaced(
        MODEL,
        DROPPED,
        "      - name: dropped\n        when: 'not defined(why)'\n        error: demo.notes.Dropped\n",
    );
    replaced(
        &text,
        "events:\n",
        "errors:\n  - {name: demo.notes.Dropped, fields: []}\nevents:\n",
    )
}

/// [`MODEL`] with no command creating a note.
fn no_creator() -> String {
    let text = replaced(MODEL, OPEN_NOTE, "");
    replaced(
        &text,
        "    may: [demo.notes.OpenNote, demo.notes.CloseNote]\n",
        "    may: [demo.notes.CloseNote]\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
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

fn scenario(run: &Run, wanted: &str) -> Value {
    written(run, wanted)
        .unwrap_or_else(|| panic!("{wanted} is synthesized: {:#?}", refused(run, wanted)))
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
                serde_json::to_string_pretty(&scenario(run, id)).unwrap()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// The steps of `scenario`.
fn steps(scenario: &Value) -> &[Value] {
    scenario["steps"].as_array().expect("steps").as_slice()
}

/// Asserts that `scenario`'s first send of `CloseNote` names a note an `OpenNote` step before it
/// created and captured, and returns that send's input.
fn sent_for_a_stored_note(scenario: &Value) -> serde_json::Map<String, Value> {
    let steps = steps(scenario);
    let act = steps
        .iter()
        .position(|step| {
            step["step"] == "execute_command" && step["command"] == "demo.notes.CloseNote"
        })
        .unwrap_or_else(|| panic!("CloseNote is sent: {scenario:#}"));
    let opened = steps[..act]
        .iter()
        .any(|step| step["step"] == "execute_command" && step["command"] == "demo.notes.OpenNote");
    assert!(opened, "a note is opened before the act: {scenario:#}");
    let captured: Vec<&str> = steps[..act]
        .iter()
        .filter(|step| step["step"] == "capture_instance" && step["entity"] == "demo.notes.Note")
        .filter_map(|step| step["instance"].as_str())
        .collect();
    let input = steps[act]["input"].as_object().expect("an input").clone();
    let note = &input["note_id"];
    assert_eq!(
        note["kind"], "instance",
        "note_id names a captured note: {scenario:#}"
    );
    let name = note["instance"].as_str().expect("an instance name");
    assert!(
        captured.contains(&name),
        "note_id names the note captured before the act ({captured:?}): {scenario:#}"
    );
    input
}

#[test]
fn dropped_is_sent_for_a_stored_note() {
    let run = run(MODEL);
    let dropped = scenario(&run, DROPPED_ID);
    let input = sent_for_a_stored_note(&dropped);
    assert!(
        !input.contains_key("why"),
        "why is left out, as `dropped` needs: {dropped:#}"
    );
}

#[test]
fn interpreted_target_passes_every_close_note_scenario() {
    let run = run(MODEL);
    assert_eq!(
        run.results.len(),
        4,
        "four scenarios: {:?}",
        run.results.keys().collect::<Vec<_>>()
    );
    assert_all_pass(&run);
}

#[test]
fn a_default_naming_no_subject_is_sent_for_a_stored_row() {
    let run = run(&accepting_default());
    let kept = scenario(&run, KEPT_ID);
    sent_for_a_stored_note(&kept);
    assert_all_pass(&run);
}

#[test]
fn an_orderable_guard_is_sent_for_a_stored_note() {
    let run = run(&orderable_guard());
    let dropped = scenario(&run, DROPPED_ID);
    sent_for_a_stored_note(&dropped);
    let result = run.results.get(DROPPED_ID).expect("dropped is run");
    assert_eq!(
        result.status,
        Status::Passed,
        "{:#?}\n{dropped:#}",
        result.diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn an_input_refusal_beside_unknown_instance_keeps_its_send() {
    let run = run(&input_refusal());
    let dropped = scenario(&run, DROPPED_ID);
    let pinned: Value = serde_json::from_str(INPUT_REFUSAL_DROPPED_0_77_0).unwrap();
    assert_eq!(dropped, pinned, "{dropped:#}");
    let first = &steps(&dropped)[0];
    assert_eq!(first["command"], "demo.notes.CloseNote", "{dropped:#}");
    assert_eq!(first["input"]["note_id"]["kind"], "literal", "{dropped:#}");
    assert_all_pass(&run);
}

#[test]
fn no_creator_refuses_dropped_by_name() {
    let run = run(&no_creator());
    assert!(
        written(&run, DROPPED_ID).is_none(),
        "no dropped scenario is written without a note to send it"
    );
    let refusals = refused(&run, DROPPED_ID);
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    assert!(
        refusals[0].starts_with("[ESS-SYNTH-004]") && refusals[0].contains("demo.notes.Note"),
        "dropped is refused as closed is, naming the note: {refusals:#?}"
    );
    let closed = refused(&run, "demo.notes.CloseNote/outcome/closed");
    assert!(
        closed
            .iter()
            .any(|text| text.starts_with("[ESS-SYNTH-004]")),
        "closed keeps its refusal: {closed:#?}"
    );
    assert_all_pass(&run);
}

/// [`MODEL`] with a further input `extra` of type `ty` that no guard reads.
fn with_extra_input(ty: &str, types: &str) -> String {
    let text = replaced(
        MODEL,
        "      - {name: why, type: Optional<String>, presence: omitted_when_absent}\n",
        &format!(
            "      - {{name: why, type: Optional<String>, presence: omitted_when_absent}}\n      - {{name: extra, type: {ty}}}\n"
        ),
    );
    replaced(
        &text,
        "  - {name: demo.notes.NoteId, kind: newtype, of: String}\n",
        &format!("  - {{name: demo.notes.NoteId, kind: newtype, of: String}}\n{types}"),
    )
}

/// Every invocation the `dropped` scenario writes, further witnesses included, names a stored note.
fn every_dropped_send_names_a_note(run: &Run) {
    let dropped = scenario(run, DROPPED_ID);
    let mut notes = Vec::new();
    for step in steps(&dropped) {
        if step["step"] == "execute_command" && step["command"] == "demo.notes.CloseNote" {
            assert_eq!(
                step["input"]["note_id"]["kind"], "instance",
                "every send names a captured note: {dropped:#}"
            );
            notes.push(step["input"]["note_id"]["instance"].clone());
        }
    }
    assert!(
        notes.len() >= 2,
        "the scenario carries a further invocation: {dropped:#}"
    );
    notes.dedup();
    assert!(
        notes.len() >= 2,
        "the further invocation is sent a note of its own: {dropped:#}"
    );
    assert_all_pass(run);
}

/// `text` whose `dropped` accepts by publishing `CloseDropped`, naming no subject: the note and a
/// field `published` of type `ty`, valued `value`.
fn publishing(text: &str, ty: &str, value: &str) -> String {
    let text = replaced(
        text,
        DROPPED,
        &format!(
            "      - name: dropped
        when: 'not defined(why)'
        emits: [demo.notes.CloseDropped]
        payload:
          demo.notes.CloseDropped: {{note_id: input.note_id, published: {value}}}
"
        ),
    );
    replaced(
        &text,
        "events:\n",
        &format!(
            "events:
  - name: demo.notes.CloseDropped
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
      - {{name: published, type: {ty}}}
"
        ),
    )
}

/// A further invocation leaving out an `Optional` input the branch publishes through a literal
/// fallback is sent for a stored note too.
#[test]
fn a_further_invocation_leaving_an_optional_out_is_sent_for_a_stored_note() {
    let text = with_extra_input("Optional<String>", "");
    let text = publishing(&text, "String", "{input: extra, else: none}");
    every_dropped_send_names_a_note(&run(&text));
}

/// A further invocation at an expression point of what the branch publishes is sent for a stored
/// note too.
#[test]
fn a_further_invocation_at_an_expression_point_is_sent_for_a_stored_note() {
    let text = with_extra_input("Integer", "");
    let text = replaced(
        &text,
        "      - {name: extra, type: Integer}\n",
        "      - {name: extra, type: Integer}\n      - {name: total, type: Integer}\n",
    );
    let text = publishing(
        &text,
        "Decimal",
        r#"{expr: "coalesce(round(input.extra / input.total, 2, half_even), 0)"}"#,
    );
    every_dropped_send_names_a_note(&run(&text));
}

/// A further invocation sending another variant of a unit union is sent for a stored note too.
#[test]
fn a_further_invocation_sending_a_union_variant_is_sent_for_a_stored_note() {
    every_dropped_send_names_a_note(&run(&with_extra_input(
        "demo.notes.Mark",
        "  - name: demo.notes.Mark\n    kind: union\n    tag: kind\n    variants:\n      absent:\n      text: String\n",
    )));
}
