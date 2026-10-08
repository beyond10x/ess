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
#[ignore = "the interpreter cannot choose a value under an upper-bound or text-match invariant; its fix is in src/interpret/response.rs, which another wave holds (story:response-string-newtype-constraints-checked-not-refused, Remaining)"]
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
#[ignore = "the interpreter cannot choose a value under an upper-bound count invariant; its fix is in src/interpret/response.rs, which another wave holds (story:response-string-newtype-constraints-checked-not-refused, Remaining)"]
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
