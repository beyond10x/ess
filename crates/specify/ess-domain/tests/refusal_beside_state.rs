//! An input-guarded refusal beside branches that act on an existing record through its held state
//! (beyond10x/ess#227, first filed as #213).
//!
//! A `when:` + `error:` branch naming no subject is admitted on a command whose other branches
//! select by `when_subject_state:`. It is answered before existence and before the held state, the
//! precedence `docs/design/outcome-shapes.md` documents for #209, so the joint state × input
//! partition counts an input it claims as the refusal's in every held state. No authored key is
//! added, so no format gates it: the shape validates at the format the issue was filed against.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("secrets.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

const TOO_SHORT: &str = "      - name: too-short\n        when: secret.count < 12\n        error: demo.secrets.SecretTooShort\n";

#[test]
fn the_issue_reproduction_validates_at_every_format_from_ess_16() {
    accepted(ROTATE);
    for format in ["ess/17", "ess/18"] {
        let text = replaced(ROTATE, "format: ess/16\n", &format!("format: {format}\n"));
        accepted(&text);
    }
}

#[test]
fn the_refusal_is_admitted_wherever_it_is_declared() {
    // After the state-guarded branch, and after the default, as well as first.
    let without = replaced(ROTATE, TOO_SHORT, "");
    let after_rotated = replaced(
        &without,
        "      - name: not-configured\n",
        &format!("{TOO_SHORT}      - name: not-configured\n"),
    );
    accepted(&after_rotated);
    let last = replaced(
        &without,
        "        error: demo.secrets.NotConfigured\n",
        &format!("        error: demo.secrets.NotConfigured\n{TOO_SHORT}"),
    );
    accepted(&last);
}

#[test]
fn a_refusal_that_names_the_subject_it_updates_is_still_refused() {
    let text = replaced(
        ROTATE,
        TOO_SHORT,
        "      - name: too-short\n        when: secret.count < 12\n        updates: demo.secrets.Configuration\n        instance: tenant_id\n        error: demo.secrets.SecretTooShort\n",
    );
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::RefusalMutatedState, "too-short"),
        "{errors}"
    );
}

/// A closed input guard the refusal and a state-guarded success share: the partition proves it, and
/// every held state with that input selects the refusal alone.
const MODE: &str = "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n";

fn with_mode(text: &str) -> String {
    let typed = replaced(
        text,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze]}\n",
    );
    replaced(
        &typed,
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n",
        MODE,
    )
}

#[test]
fn a_decided_overlap_with_a_state_guarded_success_is_the_refusal_s() {
    let text = replaced(
        &with_mode(ROTATE),
        TOO_SHORT,
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n",
    );
    // `mode == Freeze` in `Configured` selects `frozen` and `rotated` together; the refusal is
    // answered first, so the case is the refusal's alone rather than a conflict.
    let text = replaced(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Freeze\n",
    );
    accepted(&text);
}

/// Two refusals the same input selects: the first declared answers, as Entity Runtime orders
/// them (decision for beyond10x/ess#227 adversary pass 1), so the case is `frozen`'s alone.
#[test]
fn two_refusals_the_same_input_selects_are_answered_by_the_first_declared() {
    let text = replaced(
        &with_mode(ROTATE),
        TOO_SHORT,
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n      - name: frozen-again\n        when: mode == Freeze\n        error: demo.secrets.NotConfigured\n",
    );
    accepted(&text);
}

/// `too-short` (a guard the prover cannot decide) beside `frozen: mode == Freeze` (one it can),
/// with no default: `rotated` answers `mode == Rotate` in `Configured`, a state refusal answers it
/// in `Pending` and `Revoked`, and `frozen` answers `mode == Freeze` in every state. Only the
/// undecidable refusal leaves the partition; the decidable one still covers its inputs, so the
/// command is exhaustive.
#[test]
fn a_decidable_refusal_beside_an_undecidable_one_still_covers_its_inputs() {
    let text = replaced(
        &with_mode(ROTATE),
        TOO_SHORT,
        &format!(
            "{TOO_SHORT}      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n"
        ),
    );
    let text = replaced(&text, "format: ess/16\n", "format: ess/18\n");
    let text = replaced(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Rotate\n",
    );
    let text = replaced(
        &text,
        "      - name: not-configured\n        error: demo.secrets.NotConfigured\n",
        "      - name: not-configured\n        when_subject_state: [Pending, Revoked]\n        when: mode == Rotate\n        error: demo.secrets.NotConfigured\n",
    );
    accepted(&text);
}

#[test]
fn a_refusal_that_also_reads_the_held_state_keeps_the_ess_18_gate() {
    // `when_subject_state:` on a refusal naming no subject is #201's construct, still ess/18.
    let text = replaced(
        ROTATE,
        TOO_SHORT,
        "      - name: too-short\n        when: secret.count < 12\n        when_subject_state: Pending\n        error: demo.secrets.SecretTooShort\n",
    );
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/18"),
        "{errors}"
    );
}

// ---- the other held-state and stored-row shapes ---------------------------------------------

/// `rotated` selected by `when_state_changes: true` on the `configure` move (from `Pending`) instead
/// of a literal state.
pub fn state_changes(text: &str) -> String {
    replaced(
        text,
        "        when_subject_state: Configured\n        updates: demo.secrets.Configuration\n",
        "        when_state_changes: true\n        moves: demo.secrets.Configuration.configure\n",
    )
}

/// `rotated` selected by a stored enum field, `when_subject: {field: tier, equals: Basic}`.
pub fn stored_field(text: &str) -> String {
    let typed = replaced(
        text,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Tier, kind: enum, variants: [Basic, Gold]}\n",
    );
    let field = replaced(
        &typed,
        "      - {name: secret, type: String}\n    lifecycle:",
        "      - {name: secret, type: String}\n      - {name: tier, type: demo.secrets.Tier}\n    lifecycle:",
    );
    let set = replaced(
        &field,
        "        sets: {secret: \"initial-secret-value\"}\n",
        "        sets: {secret: \"initial-secret-value\", tier: Basic}\n",
    );
    replaced(
        &set,
        "        when_subject_state: Configured\n",
        "        when_subject: {field: tier, equals: Basic}\n",
    )
}

#[test]
fn the_refusal_is_admitted_beside_a_state_change_guard() {
    accepted(&state_changes(ROTATE));
}

#[test]
fn the_refusal_is_admitted_beside_a_stored_field_guard() {
    accepted(&stored_field(ROTATE));
}
