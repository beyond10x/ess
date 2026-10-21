//! Adversary, wave 2026-10-08e unit e1 (beyond10x/ess#499): a constrained String newtype reached
//! through `Optional`, `List`, `Map` value, record field, nested record, union variant and a newtype
//! chain, with Unicode scalars beyond the basic plane. Every position must be checked by the native,
//! Go and TypeScript runners alike, and the model interpreter must pass its own suite.

mod adversary_e1_support;
mod support_go;

use adversary_e1_support::*;
use ess_conformance::report::Status;
use serde_json::{json, Value};

const TYPES: &str = "  - name: catalog.items.Code
    kind: newtype
    of: String
    alphabet: 'ABC-'
    prefix: 'A-'
  - name: catalog.items.Long
    kind: newtype
    of: catalog.items.Code
    invariants:
      - 'value.count >= 4'
  - name: catalog.items.Glyph
    kind: newtype
    of: String
    alphabet: \"x\u{e9}\u{1d11e}\"
    invariants:
      - 'value.count <= 2'
  - name: catalog.items.Tagged
    kind: newtype
    of: String
    invariants:
      - 'value.count >= 3'
      - value: {starts_with: T}
      - value: {ends_with: Z}
  - name: catalog.items.Inner
    kind: struct
    fields:
      - {name: code, type: catalog.items.Code}
  - name: catalog.items.Box
    kind: struct
    fields:
      - {name: inner, type: catalog.items.Inner}
      - {name: codes, type: List<catalog.items.Code>}
  - name: catalog.items.Choice
    kind: union
    tag: kind
    variants:
      coded: catalog.items.Code
      plain: String
";

const RESPONSE: &str = "      - {name: opt, type: Optional<catalog.items.Code>}
      - {name: list, type: List<catalog.items.Code>}
      - {name: map, type: 'Map<String, catalog.items.Code>'}
      - {name: boxed, type: catalog.items.Box}
      - {name: choice, type: catalog.items.Choice}
      - {name: long, type: catalog.items.Long}
      - {name: glyph, type: catalog.items.Glyph}
      - {name: tagged, type: catalog.items.Tagged}
";

fn good() -> Value {
    json!({
        "opt": "A-B",
        "list": ["A-C", "A-"],
        "map": {"k": "A-A"},
        "boxed": {"inner": {"code": "A-B"}, "codes": ["A-C"]},
        "choice": {"kind": "coded", "value": "A-C"},
        "long": "A-BC",
        "glyph": "\u{1d11e}\u{e9}",
        "tagged": "TXZ"
    })
}

/// One position broken at a time, with the verdict every runner must give.
fn vectors() -> Vec<(&'static str, Value, Status)> {
    let with = |pointer: &str, value: Value| {
        let mut response = good();
        *response.pointer_mut(pointer).unwrap() = value;
        response
    };
    vec![
        ("good", good(), Status::Passed),
        ("optional-null", with("/opt", Value::Null), Status::Passed),
        ("optional", with("/opt", json!("B-A")), Status::Failed),
        (
            "list-last",
            with("/list", json!(["A-C", "A-Z"])),
            Status::Failed,
        ),
        (
            "map-value",
            with("/map", json!({"k": "A-A", "j": "C-A"})),
            Status::Failed,
        ),
        (
            "record-field",
            with("/boxed/codes", json!(["A-", "AA"])),
            Status::Failed,
        ),
        (
            "nested-record",
            with("/boxed/inner/code", json!("A-b")),
            Status::Failed,
        ),
        (
            "union-variant",
            with("/choice", json!({"kind": "coded", "value": "a-C"})),
            Status::Failed,
        ),
        (
            "union-other",
            with("/choice", json!({"kind": "plain", "value": "zz"})),
            Status::Passed,
        ),
        (
            "chain-invariant",
            with("/long", json!("A-B")),
            Status::Failed,
        ),
        (
            "chain-prefix",
            with("/long", json!("B-ABC")),
            Status::Failed,
        ),
        (
            "astral-count",
            with("/glyph", json!("\u{1d11e}\u{1d11e}x")),
            Status::Failed,
        ),
        (
            "astral-two",
            with("/glyph", json!("\u{1d11e}\u{1d11e}")),
            Status::Passed,
        ),
        (
            "decomposed",
            with("/glyph", json!("e\u{301}")),
            Status::Failed,
        ),
        (
            "empty-alphabet-only",
            with("/glyph", json!("")),
            Status::Passed,
        ),
        (
            "text-match-start",
            with("/tagged", json!("XTZ")),
            Status::Failed,
        ),
        (
            "text-match-end",
            with("/tagged", json!("TZX")),
            Status::Failed,
        ),
        (
            "text-match-count",
            with("/tagged", json!("TZ")),
            Status::Failed,
        ),
        (
            "text-match-long",
            with("/tagged", json!("T\u{1d11e}Z")),
            Status::Passed,
        ),
        ("empty-with-prefix", with("/opt", json!("")), Status::Failed),
    ]
}

fn suite() -> ess_conformance::ConformanceSuite {
    let suite = synthesized(&model(TYPES, RESPONSE, "", ""));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/46"
    );
    suite
}

#[test]
fn native_checks_every_reachable_position() {
    let suite = suite();
    let mut wrong = Vec::new();
    for (label, response, expected) in vectors() {
        let actual = native_status(&suite, &Fixed::new(response));
        if actual != expected {
            wrong.push(format!("{label}: native {actual:?}, expected {expected:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn go_gives_the_native_verdict_at_every_position() {
    let suite = suite();
    for (label, response, expected) in vectors() {
        let verdict = go_verdict(&format!("reach-go-{label}"), &suite, Fixed::new(response));
        assert_eq!(verdict, expected.to_string(), "{label}");
    }
}

#[test]
fn typescript_gives_the_native_verdict_at_every_position() {
    let suite = suite();
    let mut wrong = Vec::new();
    for (label, response, _) in vectors() {
        let target = Fixed::new(response);
        let rust = native_verdicts(&suite, &target);
        match typescript_verdicts(&format!("reach-{label}"), &suite, target, |_, text| text) {
            Ok(typescript) if typescript == rust => {}
            Ok(typescript) => {
                wrong.push(format!("{label}: TypeScript {typescript:?}, Rust {rust:?}"));
            }
            Err(log) => wrong.push(format!("{label}: TypeScript wrote no report: {log}")),
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The interpreter's verdict on a one-field response, constrained (`TYPES`) and with every rule
/// taken out, so a gap the constraint introduces is told apart from one the position already had.
#[test]
fn the_model_interpreter_passes_each_constrained_position_it_passes_unconstrained() {
    let plain = TYPES
        .replace("    alphabet: 'ABC-'\n    prefix: 'A-'\n", "")
        .replace(
            "    invariants:\n      - 'value.count >= 3'\n      - value: {starts_with: T}\n      - value: {ends_with: Z}\n",
            "",
        )
        .replace("    invariants:\n      - 'value.count >= 4'\n", "")
        .replace("    alphabet: \"x\u{e9}\u{1d11e}\"\n    invariants:\n      - 'value.count <= 2'\n", "");
    assert!(
        !plain.contains("alphabet") && !plain.contains("invariants"),
        "{plain}"
    );
    let status = |types: &str, field: &str| {
        let text = model(types, &format!("{field}\n"), "", "");
        let result = ess_conformance::synthesize::synthesize(&ir(&text));
        let Some(_) = result
            .suite
            .scenarios
            .keys()
            .find(|id| id.to_string() == OUTCOME)
        else {
            return "refused".to_owned();
        };
        let admitted = ess_conformance::AdmittedSuite::from_suite(&result.suite).unwrap();
        let run = ess_conformance::Runner::for_suite(&result.suite).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
        );
        run.scenarios
            .iter()
            .find(|result| result.scenario.to_string() == OUTCOME)
            .map(|result| result.status.to_string())
            .unwrap()
    };
    let mut wrong = Vec::new();
    // `glyph` is left out here: its upper-bound invariant is the separate case below.
    for field in RESPONSE.lines().filter(|field| !field.contains("glyph")) {
        let unconstrained = status(&plain, field);
        let constrained = status(TYPES, field);
        if unconstrained == "passed" && constrained != "passed" {
            wrong.push(format!(
                "{}: unconstrained {unconstrained}, constrained {constrained}",
                field.trim()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Which Glyph rule leaves the interpreter without a returned value.
#[test]
fn the_model_interpreter_returns_a_value_under_an_upper_bound_count_invariant() {
    let mut wrong = Vec::new();
    for rules in [
        "    alphabet: \"x\u{e9}\u{1d11e}\"\n",
        "    alphabet: 'xyz'\n    invariants:\n      - 'value.count <= 2'\n",
        "    invariants:\n      - 'value.count <= 2'\n",
        "    alphabet: 'xyz'\n    invariants:\n      - 'value.count >= 3'\n",
        "    prefix: 'AB'\n    invariants:\n      - 'value.count >= 5'\n",
    ] {
        let types =
            format!("  - name: catalog.items.Glyph\n    kind: newtype\n    of: String\n{rules}");
        let text = model(
            &types,
            "      - {name: glyph, type: catalog.items.Glyph}\n",
            "",
            "",
        );
        let suite = synthesized(&text);
        let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
        let run = ess_conformance::Runner::for_suite(&suite).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
        );
        let status = run
            .scenarios
            .iter()
            .find(|result| result.scenario.to_string() == OUTCOME)
            .map(|result| result.status)
            .unwrap();
        if status != Status::Passed {
            wrong.push(format!("{}: {status}", rules.replace('\n', " ")));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The interpreter's own run of the outcome scenario of a model declaring `types` and returning
/// `response` (field lines).
fn interpreted(types: &str, response: &str) -> ess_conformance::ScenarioResult {
    let text = model(types, response, "", "");
    let suite = synthesized(&text);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let run = ess_conformance::Runner::for_suite(&suite).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
    );
    run.scenarios
        .iter()
        .find(|result| result.scenario.to_string() == OUTCOME)
        .cloned()
        .unwrap()
}

fn glyph(rules: &str) -> String {
    format!("  - name: catalog.items.Glyph\n    kind: newtype\n    of: String\n{rules}")
}

/// An upper bound beside a text match: the value has to keep the literal and stay short.
#[test]
fn the_model_interpreter_returns_a_value_under_an_upper_bound_with_a_text_match() {
    let mut wrong = Vec::new();
    for rules in [
        "    invariants:\n      - 'value.count <= 3'\n      - value: {starts_with: T}\n",
        "    invariants:\n      - 'value.count <= 3'\n      - value: {ends_with: Z}\n",
        "    invariants:\n      - 'value.count <= 2'\n      - value: {contains: Q}\n",
        "    alphabet: 'TXZ'\n    invariants:\n      - 'value.count <= 4'\n      - value: {starts_with: T}\n      - value: {ends_with: Z}\n",
        "    prefix: 'AB'\n    invariants:\n      - 'value.count <= 3'\n",
        "    alphabet: \"x\u{1d11e}\"\n    invariants:\n      - 'value.count < 3'\n      - value: {starts_with: \"\u{1d11e}\"}\n",
        "    invariants:\n      - 'value.count == 1'\n      - value: {starts_with: T}\n",
    ] {
        let result = interpreted(
            &glyph(rules),
            "      - {name: glyph, type: catalog.items.Glyph}\n",
        );
        if result.status != Status::Passed {
            let seen: Vec<String> = result.diagnostics().map(ToString::to_string).collect();
            wrong.push(format!(
                "{}: {} {seen:?}",
                rules.replace('\n', " "),
                result.status
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// The rules the input witness cannot meet, at every position a returned value can hold one.
#[test]
fn the_model_interpreter_meets_an_upper_bound_at_every_reachable_position() {
    let types = "  - name: catalog.items.Glyph
    kind: newtype
    of: String
    alphabet: \"x\u{e9}\u{1d11e}\"
    invariants:
      - 'value.count <= 2'
  - name: catalog.items.Tagged
    kind: newtype
    of: String
    invariants:
      - 'value.count <= 4'
      - value: {starts_with: T}
      - value: {ends_with: Z}
  - name: catalog.items.Holder
    kind: struct
    fields:
      - {name: glyph, type: catalog.items.Glyph}
  - name: catalog.items.Choice
    kind: union
    tag: kind
    variants:
      tagged: catalog.items.Tagged
";
    let mut wrong = Vec::new();
    for field in [
        "      - {name: opt, type: Optional<catalog.items.Tagged>}\n",
        "      - {name: list, type: List<catalog.items.Glyph>}\n",
        "      - {name: map, type: 'Map<String, catalog.items.Tagged>'}\n",
        "      - {name: held, type: catalog.items.Holder}\n",
        "      - {name: choice, type: catalog.items.Choice}\n",
    ] {
        let result = interpreted(types, field);
        if result.status != Status::Passed {
            let seen: Vec<String> = result.diagnostics().map(ToString::to_string).collect();
            wrong.push(format!("{}: {} {seen:?}", field.trim(), result.status));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// No text is both at least five and at most two characters long: the interpreter says it has no
/// value, by name, and does not search on.
#[test]
fn a_count_range_no_text_meets_is_refused_by_name() {
    let result = interpreted(
        &glyph("    invariants:\n      - 'value.count >= 5'\n      - 'value.count <= 2'\n"),
        "      - {name: glyph, type: catalog.items.Glyph}\n",
    );
    let seen: Vec<String> = result.diagnostics().map(ToString::to_string).collect();
    assert_eq!(result.status, Status::Unsupported, "{seen:#?}");
    assert!(
        seen.iter()
            .any(|line| line.contains("a bounded actual command response")),
        "{seen:#?}"
    );
}

const INPUT_HELP: &str =
    "  help: give the field a type that has a finite value, or drop it from the command's input";

const RESPONSE_HELP: &str = "  help: no runner checks what the line above names on a returned \
     value; give the response field a type without it, or state that rule where the returned \
     value comes from, on the entity or the input";

/// ESS-SYNTH-001 about a response field says what to change in the response, and the input
/// field's help keeps its words.
#[test]
fn a_response_field_without_a_witness_gets_its_own_help_line() {
    let record = "  - name: catalog.items.Range
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants:
      - 'low <= high'
";
    let response = refusals(&ess_conformance::synthesize::synthesize(&ir(&model(
        record,
        "      - {name: range, type: catalog.items.Range}\n",
        "",
        "",
    ))));
    let response: Vec<&String> = response
        .iter()
        .filter(|refusal| refusal.contains("record invariant on a response type"))
        .collect();
    assert!(!response.is_empty(), "{response:#?}");
    for refusal in &response {
        assert!(refusal.contains("ESS-SYNTH-001"), "{refusal}");
        assert!(refusal.ends_with(RESPONSE_HELP), "{refusal}");
        assert!(
            !refusal.contains("drop it from the command's input"),
            "{refusal}"
        );
    }
    // An input type no value meets is refused before any witness is built (ESS-SYNTH-003, no
    // candidate tried), so the gap an input field leaves is held here in the shape
    // `witness::fields` builds it.
    let input = ess_conformance::synthesize::RefusalCause::NoWitness(
        ess_conformance::witness::WitnessGap {
            path: "label".into(),
            type_ref: "catalog.items.Glyph".into(),
            reason: "has no base value its declared invariants admit",
        },
    );
    assert_eq!(format!("  help: {}", input.hint()), INPUT_HELP);
}
