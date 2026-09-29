//! Adversary pass 1 for beyond10x/ess#227 (an input refusal beside held-state branches).
//!
//! `analyze_partition` retries the joint proof without every input refusal when the first proof
//! fails. That retry is all-or-nothing: one refusal the prover cannot decide (`secret.count < 12`)
//! drops the decidable ones with it, so two decidable refusals claiming one input — refused as
//! `conflicting_declaration` on their own (`two_refusals_the_same_input_selects_are_still_refused`)
//! — validate as soon as an unrelated undecidable refusal is declared beside them.
#![allow(clippy::needless_raw_string_hashes)]

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("secrets.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

const TOO_SHORT: &str = "      - name: too-short\n        when: secret.count < 12\n        error: demo.secrets.SecretTooShort\n";

const FROZEN_TWICE: &str = "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n      - name: frozen-again\n        when: mode == Freeze\n        error: demo.secrets.NotConfigured\n";

fn with_mode(text: &str) -> String {
    let typed = replaced(
        text,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze]}\n",
    );
    replaced(
        &typed,
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n    outcomes:\n",
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n    outcomes:\n",
    )
}

fn conflict(errors: &ValidationErrors) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::ConflictingDeclaration
            && error.to_string().contains("frozen, frozen-again")
    })
}

/// Control: without the undecidable refusal the two decidable ones validate, the first declared
/// answering `mode == Freeze` (the unit's own
/// `two_refusals_the_same_input_selects_are_answered_by_the_first_declared`). Superseded by the
/// precedence order (#227 correction 1): this asserted `conflicting_declaration`.
#[test]
fn control_two_decided_refusals_claiming_one_input_are_answered_first_declared() {
    let text = replaced(&with_mode(ROTATE), TOO_SHORT, FROZEN_TWICE);
    if let Err(errors) = assemble(&text) {
        assert!(!conflict(&errors), "{errors}");
        panic!("must validate:\n{errors}\n{text}");
    }
}

/// The same two refusals, with `too-short` (a guard the prover cannot decide) kept beside them.
/// `mode == Freeze` still selects `frozen` and `frozen-again` together in every held state, and
/// the first declared answers it, as without `too-short`. Superseded by the precedence order
/// (#227 correction 1): this asserted `conflicting_declaration`.
#[test]
fn an_undecidable_refusal_beside_them_does_not_change_that_answer() {
    let text = replaced(
        &with_mode(ROTATE),
        TOO_SHORT,
        &format!("{TOO_SHORT}{FROZEN_TWICE}"),
    );
    if let Err(errors) = assemble(&text) {
        assert!(!conflict(&errors), "{errors}");
        panic!("must validate:\n{errors}\n{text}");
    }
}

/// A refusal naming no subject whose `when:` reads the held state by name is not an input guard;
/// it must still be refused (it is not silently admitted by the relaxation).
#[test]
fn a_subjectless_refusal_reading_state_through_when_is_still_refused() {
    let text = replaced(
        ROTATE,
        TOO_SHORT,
        "      - name: too-short\n        when: state == Configured\n        error: demo.secrets.SecretTooShort\n",
    );
    assert!(assemble(&text).is_err(), "validated:\n{text}");
}

/// Likewise a `when:` reading a stored field of the record (`secret` is stored and is also the
/// input's name here, so use a stored-only field).
#[test]
fn a_subjectless_refusal_reading_a_stored_only_field_through_when_is_still_refused() {
    let stored = replaced(
        ROTATE,
        "      - {name: secret, type: String}\n    lifecycle:",
        "      - {name: secret, type: String}\n      - {name: label, type: String}\n    lifecycle:",
    );
    let stored = replaced(
        &stored,
        "        sets: {secret: \"initial-secret-value\"}\n",
        "        sets: {secret: \"initial-secret-value\", label: \"initial-label\"}\n",
    );
    let text = replaced(
        &stored,
        TOO_SHORT,
        "      - name: too-short\n        when: label.count < 12\n        error: demo.secrets.SecretTooShort\n",
    );
    assert!(assemble(&text).is_err(), "validated:\n{text}");
}

/// The fallback must not open a coverage gap: without the default, `Pending` and `Revoked` are
/// answered by nothing for a secret the refusal does not claim, decidable refusal or not.
#[test]
fn a_state_gap_outside_the_refusal_is_still_refused() {
    let without_default = replaced(
        ROTATE,
        "      - name: not-configured\n        error: demo.secrets.NotConfigured\n",
        "",
    );
    assert!(
        assemble(&without_default).is_err(),
        "undecidable refusal, no default"
    );
    let decided = replaced(
        &with_mode(&without_default),
        TOO_SHORT,
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n",
    );
    assert!(assemble(&decided).is_err(), "decidable refusal, no default");
}
