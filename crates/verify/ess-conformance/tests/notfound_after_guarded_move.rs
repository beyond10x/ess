//! An `external:` not-found branch declared after a `when:`-guarded accepting branch (a downstream
//! adopter's report of 2026-11-04 on `ess/34`).
//!
//! Under the precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The
//! precedence order"; `docs/design/input-guard-overlap-precedence.md`) an accepting branch
//! declared before an `external:` one answers every input its guard claims. With one `Move`
//! variant `opened` claims every input, so `not-found` is unreachable as declared. Synthesis refuses
//! it once, naming the repair (`before_row: true` on a not-found declared first), and never as
//! `ESS-SYNTH-019`, whose help says the specification is fine and ess is broken.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Refusal, Synthesis},
    ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// The reproduction: `opened` (`when: move == Open`) declared before `not-found` (`external:`),
/// whose error carries a field named like the command's input. `StockBin` creates the bin, so the
/// only refusals left are about `HandleBin`.
const SHELF: &str = r"format: ess/34
system: demo
version: v1
domain: demo.shelf
types:
  - name: demo.shelf.BinId
    kind: newtype
    of: Uuid
  - name: demo.shelf.Move
    kind: enum
    variants: [Open]
entities:
  - name: demo.shelf.Bin
    identity:
      name: bin_id
      type: demo.shelf.BinId
    lifecycle:
      initial: Closed
      states: [Closed, Opened]
      terminal: [Opened]
      transitions:
        - name: open
          from: [Closed]
          to: Opened
errors:
  - name: demo.shelf.BinNotFound
    fields:
      - name: bin_id
        type: demo.shelf.BinId
commands:
  - name: demo.shelf.StockBin
    input: []
    outcomes:
      - name: stocked
        creates: demo.shelf.Bin
        instance: bin_id
        emits:
          - demo.shelf.BinStocked
        payload:
          demo.shelf.BinStocked:
            bin_id:
              generated: true
  - name: demo.shelf.HandleBin
    input:
      - name: bin_id
        type: demo.shelf.BinId
      - name: move
        type: demo.shelf.Move
    outcomes:
      - name: opened
        when: move == Open
        moves: demo.shelf.Bin.open
        instance: bin_id
        emits:
          - demo.shelf.BinOpened
      - name: not-found
        external: no bin carries input.bin_id
        error: demo.shelf.BinNotFound
events:
  - name: demo.shelf.BinStocked
    fields:
      - name: bin_id
        type: demo.shelf.BinId
  - name: demo.shelf.BinOpened
";

const OPENED: &str = "      - name: opened
        when: move == Open
        moves: demo.shelf.Bin.open
        instance: bin_id
        emits:
          - demo.shelf.BinOpened
";

const NOT_FOUND: &str = "      - name: not-found
        external: no bin carries input.bin_id
        error: demo.shelf.BinNotFound
";

const NOT_FOUND_ID: &str = "demo.shelf.HandleBin/outcome/not-found";

fn edit(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the model");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("shelf.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("shelf.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

/// The refusals about `not-found`'s scenario.
fn not_found_refusals(synthesis: &Synthesis) -> Vec<&Refusal> {
    synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == NOT_FOUND_ID)
        })
        .collect()
}

/// Every refusal, as `code: text (help)`, sorted.
fn listed(synthesis: &Synthesis) -> Vec<String> {
    let mut out: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal} ({})", refusal.code(), refusal.hint()))
        .collect();
    out.sort();
    out
}

fn codes(synthesis: &Synthesis) -> Vec<String> {
    let mut out: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}",
                refusal.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default()
            )
        })
        .collect();
    out.sort();
    out
}

fn has_scenario(synthesis: &Synthesis, id: &str) -> bool {
    synthesis
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

#[test]
fn a_not_found_after_a_guarded_move_is_refused_once_with_its_repair() {
    let result = synthesis(SHELF);
    let all = listed(&result);
    assert!(
        result
            .refusals
            .iter()
            .all(|refusal| refusal.code().to_string() != "ESS-SYNTH-019"),
        "no refusal reports a defect in ess: {all:#?}"
    );
    let refused = not_found_refusals(&result);
    assert_eq!(refused.len(), 1, "one refusal on `not-found`: {all:#?}");
    let refusal = refused[0];
    assert_eq!(refusal.code().to_string(), "ESS-SYNTH-003", "{all:#?}");
    let help = refusal.hint();
    assert!(help.contains("before_row: true"), "{help}");
    assert!(help.contains("declare"), "{help}");
    assert!(help.contains("first"), "{help}");
    let text = refusal.to_string();
    assert!(
        text.contains("opened"),
        "the refusal names the branch: {text}"
    );
    assert!(!has_scenario(&result, NOT_FOUND_ID), "{all:#?}");
}

#[test]
fn an_error_field_named_like_the_input_builds_no_step_the_precedence_refutes() {
    let with_fields = synthesis(SHELF);
    let without = synthesis(&edit(
        SHELF,
        "  - name: demo.shelf.BinNotFound\n    fields:\n      - name: bin_id\n        type: demo.shelf.BinId\n",
        "  - name: demo.shelf.BinNotFound\n",
    ));
    assert_eq!(
        codes(&with_fields),
        codes(&without),
        "with fields: {:#?}\nwithout: {:#?}",
        listed(&with_fields),
        listed(&without)
    );
    assert_eq!(not_found_refusals(&with_fields).len(), 1);
}

#[test]
fn a_not_found_declared_first_before_the_row_is_synthesized() {
    let repaired = edit(
        &edit(SHELF, OPENED, ""),
        NOT_FOUND,
        &format!(
            "      - name: not-found\n        external: no bin carries input.bin_id\n        before_row: true\n        error: demo.shelf.BinNotFound\n{OPENED}"
        ),
    );
    let result = synthesis(&repaired);
    let all = listed(&result);
    assert!(not_found_refusals(&result).is_empty(), "{all:#?}");
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == NOT_FOUND_ID)
        .map(|(_, scenario)| scenario)
        .unwrap_or_else(|| panic!("`not-found` has a scenario: {all:#?}"));
    assert!(
        scenario.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectOutcome { outcome } if outcome.outcome.to_string() == "not-found"
        )),
        "{:#?}",
        scenario.steps
    );
    for event in ["demo.shelf.BinOpened", "demo.shelf.BinStocked"] {
        assert!(
            scenario.steps.iter().any(|step| matches!(
                step,
                ScenarioStep::ExpectNoEvent { event: published } if published.to_string() == event
            )),
            "the store is left unchanged, so no `{event}` is published: {:#?}",
            scenario.steps
        );
    }
}

#[test]
fn a_reachable_not_found_after_a_guarded_move_is_still_synthesized() {
    // `Hold` is no input `opened` claims; `held`, declared after `not-found`, covers it.
    let reachable = edit(
        &edit(
            &edit(SHELF, "variants: [Open]", "variants: [Open, Hold]"),
            NOT_FOUND,
            &format!(
                "{NOT_FOUND}      - name: held\n        when: move == Hold\n        emits:\n          - demo.shelf.BinHeld\n"
            ),
        ),
        "  - name: demo.shelf.BinOpened\n",
        "  - name: demo.shelf.BinOpened\n  - name: demo.shelf.BinHeld\n",
    );
    let result = synthesis(&reachable);
    let all = listed(&result);
    assert!(not_found_refusals(&result).is_empty(), "{all:#?}");
    assert!(has_scenario(&result, NOT_FOUND_ID), "{all:#?}");
    assert!(
        result
            .refusals
            .iter()
            .all(|refusal| refusal.code().to_string() != "ESS-SYNTH-019"),
        "{all:#?}"
    );
}
