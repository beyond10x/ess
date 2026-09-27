//! Adversary cases for story:subject-guard-input-and-case-folding (beyond10x/ess#157, #140): the
//! `ess/15` gate on the case-insensitive operators at every predicate position the `ess/8` gate
//! covers, and the refusal an `input` root below `ess/15` is given.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("adv.yaml"), raw)])
}

fn guarded(format: &str, guard: &str) -> String {
    format!(
        r#"format: {format}
system: calls
version: v1
domain: calls.core
types:
  - name: calls.core.PhoneNumber
    kind: newtype
    of: String
    invariants:
      - value != ""
  - name: calls.core.Party
    kind: struct
    fields:
      - {{name: number, type: String}}
entities:
  - name: calls.core.Call
    identity: {{name: call_id, type: Uuid}}
    fields:
      - {{name: note, type: String}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: calls.core.Opened
    fields: [{{name: call_id, type: Uuid}}]
errors:
  - name: calls.core.Refused
    fields: []
commands:
  - name: calls.core.Open
    input:
      - {{name: note, type: String}}
      - {{name: phone, type: calls.core.PhoneNumber}}
      - {{name: party, type: calls.core.Party}}
    outcomes:
      - name: opened
        when: {guard}
        creates: calls.core.Call
        instance: call_id
        sets: {{note: input.note}}
        emits: [calls.core.Opened]
        payload:
          calls.core.Opened:
            call_id: {{generated: true}}
      - name: refused
        error: calls.core.Refused
"#
    )
}

fn positions() -> Vec<(&'static str, String)> {
    vec![
        (
            "a command guard",
            guarded("ess/15", "{note: {equals_ignore_case: web}}"),
        ),
        (
            "a newtype invariant",
            guarded("ess/15", "note == x").replace(
                "      - value != \"\"",
                "      - value: {in_ignore_case: [a, b]}",
            ),
        ),
        (
            "a struct invariant",
            guarded("ess/15", "note == x").replace(
                "      - {name: number, type: String}\n",
                "      - {name: number, type: String}\n    invariants:\n      - number: {equals_ignore_case: x}\n",
            ),
        ),
        (
            "an entity invariant",
            guarded("ess/15", "note == x").replace(
                "      - {name: note, type: String}\n    lifecycle:",
                "      - {name: note, type: String}\n    invariants:\n      - not: {note: {equals_ignore_case: spam}}\n    lifecycle:",
            ),
        ),
        (
            "a view filter",
            guarded("ess/15", "note == x")
                + "views:\n  - name: calls.core.Noted\n    source: calls.core.Call\n    consistency: eventual\n    filter: {note: {in_ignore_case: [urgent]}}\n    fields:\n      - {name: call_id, type: Uuid}\n",
        ),
    ]
}

const GATE: &str = "case-insensitive text operators require specification format ess/15";

#[test]
fn adv_every_predicate_position_is_admitted_at_ess_15_and_refused_below_it() {
    for (position, text) in positions() {
        if let Err(errors) = admits(&text) {
            panic!("{position} is admitted at ess/15: {errors}\n{text}");
        }
        for below in ["ess/14", "ess/8"] {
            let older = text.replace("format: ess/15", &format!("format: {below}"));
            let errors = admits(&older).expect_err(position);
            assert!(
                errors.as_slice().iter().any(|error| error.code
                    == ValidationCode::UnsupportedFormatVersion
                    && error.message.contains(GATE)),
                "{position} at {below}: {errors}"
            );
        }
    }
}

/// A bare `input` root names no input field, so `ess/15` does not make it valid either. Below
/// `ess/15` the author must be told what is wrong with the document (`unobservable_fact`, as before
/// this unit), not to raise a header that would still refuse it.
#[test]
fn adv_a_bare_input_root_below_ess_15_is_not_told_to_upgrade() {
    let text = |format: &str| {
        guarded(format, "note == x").replace(
            "      - name: refused\n        error: calls.core.Refused\n",
            "      - name: refused\n        error: calls.core.Refused\n  - name: calls.core.Poke\n    input:\n      - {name: call_id, type: Uuid}\n    outcomes:\n      - name: poked\n        when_subject:\n          predicate: {input: {eq: x}}\n        error: calls.core.Refused\n      - name: fine\n        preserves: calls.core.Call\n        instance: call_id\n",
        )
    };
    let at15 = admits(&text("ess/15")).expect_err("a bare input root is refused at ess/15");
    assert!(
        at15.as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::UnobservableFact),
        "{at15}"
    );
    let below = admits(&text("ess/14")).expect_err("and below it");
    let upgraded: Vec<_> = below
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::UnsupportedFormatVersion)
        .map(|error| error.message.clone())
        .collect();
    assert!(
        upgraded.is_empty(),
        "ess/14 tells the author to declare ess/15 ({upgraded:?}), where the same document is \
         refused with: {at15}"
    );
}
