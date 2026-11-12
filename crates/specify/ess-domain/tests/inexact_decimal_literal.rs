//! An authored `Decimal` that binary64 cannot hold exactly is refused at validation, by name.
//!
//! `amount: 0.1000000000000000000001` used to validate and compile to `0.1`: a YAML number reaches
//! the domain as an `f64`, and a guard literal is read through `Number::parse_decimal`, which keeps
//! the authored digits only where they are the canonical decimal of their binary64. A suite
//! synthesized from the collapsed value then disagreed with the specification as written. The
//! literal is refused rather than changed silently; nothing about how a value is written moves.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const INEXACT: &str = "0.1000000000000000000001";

fn model(sets: &str, when: &str, when_subject: &str) -> String {
    format!(
        "format: ess/35
system: demo
version: v1
domain: demo.pay
types:
  - {{name: demo.pay.AccountId, kind: newtype, of: Uuid}}
entities:
  - name: demo.pay.Account
    identity: {{name: account_id, type: demo.pay.AccountId}}
    fields:
      - {{name: amount, type: Decimal}}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {{name: finish, from: [Open], to: Done}}
errors:
  - {{name: demo.pay.TooSmall, summary: The amount is too small., fields: []}}
events:
  - name: demo.pay.Opened
    fields: [{{name: account_id, type: demo.pay.AccountId}}]
  - name: demo.pay.Finished
    fields: [{{name: account_id, type: demo.pay.AccountId}}]
commands:
  - name: demo.pay.Open
    input:
      - {{name: amount, type: Decimal}}
    outcomes:
      - name: too-small
        when: {when}
        error: demo.pay.TooSmall
      - name: opened
        creates: demo.pay.Account
        instance: account_id
        sets: {{amount: {sets}}}
        emits: [demo.pay.Opened]
        payload: {{demo.pay.Opened: {{account_id: {{generated: true}}}}}}
  - name: demo.pay.Finish
    input:
      - {{name: account_id, type: demo.pay.AccountId}}
    outcomes:
      - name: too-small
        when_subject:
          predicate: {when_subject}
        error: demo.pay.TooSmall
      - name: finished
        moves: demo.pay.Account.finish
        instance: account_id
        emits: [demo.pay.Finished]
        payload: {{demo.pay.Finished: {{account_id: input.account_id}}}}
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("pay.yaml"), raw)])
}

/// The refusals with the code, each rendered `[code] location: message`.
fn inexact(text: &str) -> Vec<String> {
    let errors = assemble(text).err().unwrap_or_else(|| {
        panic!("must be refused:\n{text}");
    });
    errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::InexactDecimalLiteral)
        .map(ToString::to_string)
        .collect()
}

fn all(errors: &[String]) -> String {
    errors.join("\n")
}

#[test]
fn a_sets_literal_binary64_cannot_hold_is_refused_at_its_outcome() {
    let found = inexact(&model(INEXACT, "amount < 1", "amount < 1"));
    assert_eq!(found.len(), 1, "{}", all(&found));
    let error = &found[0];
    assert!(
        error.starts_with("[inexact_decimal_literal] command.demo.pay.Open.outcomes.opened.sets"),
        "{error}"
    );
    assert!(error.contains(INEXACT), "{error}");
    assert!(error.contains("`0.1`"), "{error}");
}

#[test]
fn a_guard_literal_binary64_cannot_hold_is_refused_at_its_when() {
    let found = inexact(&model("0.5", &format!("amount < {INEXACT}"), "amount < 1"));
    assert_eq!(found.len(), 1, "{}", all(&found));
    let error = &found[0];
    assert!(
        error
            .starts_with("[inexact_decimal_literal] command.demo.pay.Open.outcomes.too-small.when"),
        "{error}"
    );
    assert!(error.contains(INEXACT), "{error}");
    assert!(error.contains("`0.1`"), "{error}");
}

#[test]
fn the_same_literal_under_when_subject_is_refused_at_its_own_location() {
    let found = inexact(&model("0.5", "amount < 1", &format!("amount < {INEXACT}")));
    assert_eq!(found.len(), 1, "{}", all(&found));
    let error = &found[0];
    assert!(
        error.starts_with(
            "[inexact_decimal_literal] command.demo.pay.Finish.outcomes.too-small.when_subject"
        ),
        "{error}"
    );
    assert!(error.contains(INEXACT), "{error}");
    assert!(error.contains("`0.1`"), "{error}");
}

#[test]
fn every_inexact_position_is_refused_once() {
    let found = inexact(&model(
        INEXACT,
        &format!("amount < {INEXACT}"),
        &format!("amount < {INEXACT}"),
    ));
    assert_eq!(found.len(), 3, "{}", all(&found));
}

#[test]
fn a_value_binary64_holds_validates_with_no_diagnostic() {
    for sets in ["0.1", "0.5", "1e2", "0.30000000000000004"] {
        let text = model(sets, "amount < 0.1", "amount < 0.1");
        if let Err(errors) = assemble(&text) {
            panic!("`sets: {{amount: {sets}}}` must validate:\n{errors}");
        }
    }
}

#[test]
fn a_quoted_or_text_literal_is_not_this_refusal() {
    // A quoted decimal over a `Decimal` is refused already, as a type mismatch, and a quoted
    // guard literal is text: neither is a number that collapsed.
    let text = model(
        &format!("\"{INEXACT}\""),
        &format!("amount < \"{INEXACT}\""),
        "amount < 1",
    );
    let errors = assemble(&text).expect_err("the quoted sets literal is refused");
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.code != ValidationCode::InexactDecimalLiteral),
        "{errors}"
    );
}

/// The mapping form of a guard is a YAML number, an `f64` before any predicate reader sees it: the
/// digits are read back out of the text at every predicate position, not only the compact one.
#[test]
fn a_mapping_form_guard_is_refused_at_its_when_as_the_compact_form_is() {
    let compact = inexact(&model("0.5", &format!("amount < {INEXACT}"), "amount < 1"));
    assert_eq!(compact.len(), 1, "control: the compact form is refused");
    for when in [
        format!("{{amount: {{lt: {INEXACT}}}}}"),
        format!("{{amount: {INEXACT}}}"),
        format!("{{amount: {{in: [0.5, {INEXACT}]}}}}"),
        format!("{{amount: {{any_of: [0.5, {INEXACT}]}}}}"),
        format!("{{any: [{{amount: {{gt: 2}}}}, {{amount: {INEXACT}}}]}}"),
        format!("{{all: [{{amount: {{gt: 2}}}}, {{amount: {{lte: {INEXACT}}}}}]}}"),
    ] {
        let found = inexact(&model("0.5", &when, "amount < 1"));
        assert_eq!(found.len(), 1, "{when}\n{}", all(&found));
        assert!(
            found[0].starts_with(
                "[inexact_decimal_literal] command.demo.pay.Open.outcomes.too-small.when"
            ),
            "{when}\n{}",
            found[0]
        );
        assert!(found[0].contains(INEXACT), "{when}\n{}", found[0]);
        assert!(found[0].contains("`0.1`"), "{when}\n{}", found[0]);
    }
}

#[test]
fn a_mapping_form_guard_under_when_subject_is_refused_at_its_own_location() {
    let found = inexact(&model(
        "0.5",
        "amount < 1",
        &format!("{{amount: {{lt: {INEXACT}}}}}"),
    ));
    assert_eq!(found.len(), 1, "{}", all(&found));
    assert!(
        found[0].starts_with(
            "[inexact_decimal_literal] command.demo.pay.Finish.outcomes.too-small.when_subject"
        ),
        "{}",
        found[0]
    );
}

#[test]
fn a_mapping_form_guard_over_a_value_binary64_holds_validates() {
    for when in [
        "{amount: {lt: 0.5}}",
        "{amount: 0.1}",
        "{amount: {in: [0.5, 1e2, 0.30000000000000004]}}",
    ] {
        let text = model("0.5", when, "amount < 1");
        if let Err(errors) = assemble(&text) {
            panic!("`when: {when}` must validate:\n{errors}");
        }
    }
}

#[test]
fn a_literal_whose_exponent_leaves_binary64_is_refused_with_fifteen_digits_or_fewer() {
    for sets in ["1e-400", "2.5e-330", "-1e-400"] {
        let found = inexact(&model(sets, "amount < 1", "amount < 1"));
        assert_eq!(found.len(), 1, "{sets}\n{}", all(&found));
        assert!(found[0].contains(sets), "{sets}\n{}", found[0]);
    }
    let found = inexact(&model("0.5", "{amount: {lt: 1e-400}}", "amount < 1"));
    assert_eq!(found.len(), 1, "{}", all(&found));
}
