//! Adversary, pass 1, for unit E-U7 (String `.utf8_bytes`, beyond10x/ess#233).
//!
//! The design (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`") asks that
//! synthesis witness every byte-length guard so that a target measuring text by anything but its
//! UTF-8 bytes fails a scenario, and that `.count` and `.utf8_bytes` on one text be made to disagree.
//! The unit's own suite holds only plain input guards comparing one byte length with a literal.
//! Each case here takes another shape — two byte lengths, a byte length against an input Integer,
//! a stored-row guard, `.count` and `.utf8_bytes` on one text in one guard — synthesizes the suite
//! from the model, and runs it against the native interpreter (which must pass) and against the
//! native interpreter of the same model with every `.utf8_bytes` written `.count`: a target that
//! counts Unicode scalar values, which must fail a scenario deciding the guard.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const INPUT: &str = r"format: ess/22
system: notes
version: v1
domain: notes.core
errors:
  - {name: notes.core.Refused, summary: Refused.}
events:
  - {name: notes.core.Accepted, fields: []}
commands:
  - name: notes.core.File
    input:
      - {name: first, type: String}
      - {name: second, type: String}
      - {name: limit, type: Integer}
    outcomes:
      - name: refused
        when: {GUARD}
        error: notes.core.Refused
      - name: accepted
        emits: [notes.core.Accepted]
";

const STORED: &str = r"format: ess/22
system: notes
version: v1
domain: notes.core
entities:
  - name: notes.core.Note
    identity: {name: note_id, type: Uuid}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: notes.core.Opened
    fields:
      - {name: note_id, type: Uuid}
  - name: notes.core.Edited
    fields: []
commands:
  - name: notes.core.Open
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: notes.core.Note
        instance: note_id
        sets: {label: input.label}
        emits: [notes.core.Opened]
        payload:
          notes.core.Opened:
            note_id: {generated: true}
  - name: notes.core.Edit
    input:
      - {name: note_id, type: Uuid}
      - {name: to, type: String}
    outcomes:
      - name: held
        when_subject:
          predicate: {GUARD}
        preserves: notes.core.Note
        instance: note_id
      - name: edited
        updates: notes.core.Note
        instance: note_id
        sets: {label: input.to}
        emits: [notes.core.Edited]
views:
  - name: notes.core.Notes
    source: notes.core.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: Uuid}
      - {name: state, type: notes.core.Note.State}
      - {name: label, type: String}
";

fn ir(template: &str, guard: &str) -> EssIr {
    let text = template.replace("{GUARD}", guard);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{guard}: {error:?}"))
}

fn run(suite: &ConformanceSuite, model: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Every way `guard` (in `template`) is not witnessed on both sides of `outcomes`, not passed by
/// the healthy interpreter, or passed whole by the interpreter that counts scalar values instead.
fn problems(template: &str, guard: &str, outcomes: &[&str]) -> Vec<String> {
    let model = ir(template, guard);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let mut found: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "refused {}: {} {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for wanted in outcomes {
        if !ids.iter().any(|id| id == wanted) {
            found.push(format!("no scenario {wanted}"));
        }
    }
    for (id, status) in run(&synthesis.suite, model) {
        if status != Status::Passed {
            found.push(format!("healthy {id}: {status:?}"));
        }
    }
    let scalars = ir(template, &guard.replace(".utf8_bytes", ".count"));
    let faulty = run(&synthesis.suite, scalars);
    let caught: Vec<&String> = faulty
        .iter()
        .filter(|(id, status)| {
            **status == Status::Failed && outcomes.iter().any(|outcome| id == outcome)
        })
        .map(|(id, _)| id)
        .collect();
    if caught.is_empty() {
        found.push(format!(
            "a target counting scalar values passes every scenario deciding `{guard}`: {faulty:?}; \
             sent {:?}",
            sent(&synthesis.suite, outcomes)
        ));
    }
    found
}

/// The texts each scenario deciding one of `outcomes` sends in its last command.
fn sent(suite: &ConformanceSuite, outcomes: &[&str]) -> Vec<(String, Vec<(String, String)>)> {
    suite
        .scenarios
        .iter()
        .filter(|(key, _)| outcomes.iter().any(|outcome| key.to_string() == *outcome))
        .map(|(key, scenario)| {
            let input = scenario
                .steps
                .iter()
                .rev()
                .find_map(|step| match step {
                    ess_conformance::ScenarioStep::ExecuteCommand { input, .. } => Some(
                        input
                            .iter()
                            .map(|(name, value)| (name.clone(), format!("{value:?}")))
                            .collect(),
                    ),
                    _ => None,
                })
                .unwrap_or_default();
            (key.to_string(), input)
        })
        .collect()
}

const REFUSED: &str = "notes.core.File/outcome/refused";
const ACCEPTED: &str = "notes.core.File/outcome/accepted";
const HELD: &str = "notes.core.Edit/outcome/held";
const EDITED: &str = "notes.core.Edit/outcome/edited";

// ---- controls: the unit's own shape -----------------------------------------------------------

#[test]
fn adv_u7_control_literal_bound() {
    assert_eq!(
        problems(INPUT, "first.utf8_bytes > 8", &[REFUSED, ACCEPTED]),
        Vec::<String>::new()
    );
}

// ---- two byte lengths, and a byte length against an input fact ------------------------------

#[test]
fn adv_u7_two_byte_lengths_equal() {
    assert_eq!(
        problems(
            INPUT,
            "first.utf8_bytes == second.utf8_bytes",
            &[REFUSED, ACCEPTED]
        ),
        Vec::<String>::new()
    );
}

#[test]
fn adv_u7_two_byte_lengths_ordered() {
    assert_eq!(
        problems(
            INPUT,
            "first.utf8_bytes > second.utf8_bytes",
            &[REFUSED, ACCEPTED]
        ),
        Vec::<String>::new()
    );
}

#[test]
fn adv_u7_byte_length_against_an_input_integer() {
    assert_eq!(
        problems(INPUT, "first.utf8_bytes > limit", &[REFUSED, ACCEPTED]),
        Vec::<String>::new()
    );
}

#[test]
fn adv_u7_byte_length_on_the_right_of_an_input_integer() {
    assert_eq!(
        problems(INPUT, "limit < first.utf8_bytes", &[REFUSED, ACCEPTED]),
        Vec::<String>::new()
    );
}

// ---- `.count` and `.utf8_bytes` on one text, in one guard ------------------------------------

/// Two scalars, at least eight bytes: only text of two four-byte scalars takes the refusing
/// branch (`😀😀`). The design names `.count`/`.utf8_bytes` disagreement as a required control.
#[test]
fn adv_u7_count_and_bytes_disagreeing_in_one_guard_is_witnessed() {
    let model = ir(
        INPUT,
        r#"{all: ["first.count <= 2", "first.utf8_bytes >= 8"]}"#,
    );
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert_eq!(refusals, Vec::<String>::new());
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(ids.iter().any(|id| id == REFUSED), "{ids:#?}");
}

// ---- a stored-row guard ---------------------------------------------------------------------

#[test]
fn adv_u7_stored_row_byte_length_against_a_literal() {
    assert_eq!(
        problems(STORED, "label.utf8_bytes > 8", &[HELD, EDITED]),
        Vec::<String>::new()
    );
}

#[test]
#[ignore = "follow-up: a stored-row guard comparing an input length with the row's length is never witnessed true, for `.count` too (F6)"]
fn adv_u7_stored_row_byte_length_against_an_input_byte_length() {
    assert_eq!(
        problems(
            STORED,
            "input.to.utf8_bytes > label.utf8_bytes",
            &[HELD, EDITED]
        ),
        Vec::<String>::new()
    );
}

// ---- controls for the refusals above: the same shapes over `.count` -------------------------

/// The synthesis refusals and missing scenarios alone, for a guard holding no byte length.
fn witnessed(template: &str, guard: &str, outcomes: &[&str]) -> Vec<String> {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(template, guard));
    let mut found: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    for wanted in outcomes {
        if !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == *wanted)
        {
            found.push(format!("no scenario {wanted}"));
        }
    }
    found
}

#[test]
#[ignore = "follow-up: a stored-row guard comparing an input `.count` with the row's `.count` is never witnessed true (F6, pre-existing)"]
fn adv_u7_control_stored_row_count_against_an_input_count() {
    assert_eq!(
        witnessed(STORED, "input.to.count > label.count", &[HELD, EDITED]),
        Vec::<String>::new()
    );
}

#[test]
fn adv_u7_control_count_disagreeing_with_a_count() {
    assert_eq!(
        witnessed(
            INPUT,
            r#"{all: ["first.count <= 2", "second.count >= 8"]}"#,
            &[REFUSED, ACCEPTED]
        ),
        Vec::<String>::new()
    );
}

// ---- a view filter ----------------------------------------------------------------------------

const FILTERED: &str = r"format: ess/22
system: notes
version: v1
domain: notes.core
entities:
  - name: notes.core.Note
    identity: {name: note_id, type: Uuid}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: notes.core.Opened
    fields:
      - {name: note_id, type: Uuid}
commands:
  - name: notes.core.Open
    input:
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: notes.core.Note
        instance: note_id
        sets: {label: input.label}
        emits: [notes.core.Opened]
        payload:
          notes.core.Opened:
            note_id: {generated: true}
views:
  - name: notes.core.Short
    source: notes.core.Note
    consistency: read_your_writes
    filter: {GUARD}
    fields:
      - {name: note_id, type: Uuid}
      - {name: label, type: String}
";

/// A view filtering on a byte length: the suite is synthesized without refusal, the interpreter
/// passes it, and the interpreter that counts scalar values fails some scenario.
#[test]
#[ignore = "follow-up: a view filter is witnessed by one scenario on one side only, for `.count` and plain equality too (F4)"]
fn adv_u7_view_filter_byte_length() {
    let guard = "label.utf8_bytes <= 4";
    let model = ir(FILTERED, guard);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let mut found: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    for (id, status) in run(&synthesis.suite, model) {
        if status != Status::Passed {
            found.push(format!("healthy {id}: {status:?}"));
        }
    }
    let faulty = run(
        &synthesis.suite,
        ir(FILTERED, &guard.replace(".utf8_bytes", ".count")),
    );
    if !faulty.values().any(|status| *status == Status::Failed) {
        found.push(format!(
            "a target counting scalar values passes every scenario of `{guard}`: {faulty:?}"
        ));
    }
    assert_eq!(found, Vec::<String>::new());
}
