//! Adversary pass 1 on `story:an-inexact-authored-decimal-is-refused`.
//!
//! Each case is a position the change claims to refuse, written in a way the unit's own tests do
//! not write it.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::facts::InexactDecimal;

const INEXACT: &str = "0.1000000000000000000001";

/// One command over `amount: Decimal`, with `when` written as given and `sets` as given.
fn model(when: &str, sets: &str) -> String {
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
          predicate: amount < 1
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

fn refused_as_inexact(text: &str) -> Result<(), String> {
    match assemble(text) {
        Ok(_) => Err(format!(
            "validated with no diagnostic; no inexact_decimal_literal:\n{text}"
        )),
        Err(errors) => {
            if errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::InexactDecimalLiteral)
            {
                Ok(())
            } else {
                Err(format!(
                    "refused, but not as inexact_decimal_literal:\n{errors}"
                ))
            }
        }
    }
}

/// The mapping form of a guard is a guard: `when: {amount: {lt: …}}` reads the same comparison as
/// `when: amount < …`, and its literal is a YAML number, so the digits are gone by the time the
/// predicate reader sees it.
#[test]
fn a_mapping_form_guard_literal_binary64_cannot_hold_is_refused() {
    let exact = model("{amount: {lt: 0.5}}", "0.5");
    if let Err(errors) = assemble(&exact) {
        panic!("the mapping-form guard over 0.5 must validate:\n{errors}");
    }
    let compact = model(&format!("amount < {INEXACT}"), "0.5");
    refused_as_inexact(&compact).expect("control: the compact form is refused");
    let mapping = model(&format!("{{amount: {{lt: {INEXACT}}}}}"), "0.5");
    if let Err(why) = refused_as_inexact(&mapping) {
        panic!("{why}");
    }
}

/// `1e-400` is a decimal binary64 cannot hold: it reads as `0`. The unit's own rule says so
/// (`InexactDecimal::of_text`), and a guard refuses it, but a `sets:` literal is only scanned when
/// the file holds a run of 16 digits, which an underflowing literal does not have.
#[test]
fn a_sets_literal_that_underflows_to_zero_is_refused() {
    assert!(
        InexactDecimal::of_text("1e-400").is_some(),
        "control: the unit's own rule calls 1e-400 inexact"
    );
    let text = model("amount < 1", "1e-400");
    if let Err(why) = refused_as_inexact(&text) {
        panic!("{why}");
    }
}
