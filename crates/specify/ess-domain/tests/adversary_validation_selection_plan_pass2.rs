//! Adversary pass 2 on `story:validation-reads-selection-plan` (wave 3, unit U3), against the
//! correction `command::refusal_settles`.
//!
//! `refusal_settles(refusal, other)` holds where the phase order reads `refusal` first, or where
//! `other` refuses too: "whichever of the two is read first answers and one refusal results in
//! either order". That holds for a pair. A partition asks it of every branch beside the refusal one
//! at a time, so where two refusals the order reads *before* the refusal are selected together, the
//! refusal is let answer alone and their overlap, which the partition refuses wherever nothing
//! read before them answers, is never counted.
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// The precedence order with `one` and `other` exchanged.
fn exchanged(one: Phase, other: Phase) -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| {
        order
            .iter()
            .position(|held| *held == phase)
            .expect("the precedence order holds every phase")
    };
    let (one, other) = (at(one), at(other));
    order.swap(one, other);
    order
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)])
}

fn accepted(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("{errors}\n{text}");
    }
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The conflict between `locked-a` and `locked-b` on `Configured` and `mode = Freeze`.
fn names_the_locked_overlap(errors: &ValidationErrors) -> bool {
    errors.as_slice().iter().any(|error| {
        let line = error.to_string();
        error.code == ValidationCode::ConflictingDeclaration
            && line.contains("held state Configured and input [mode = Freeze]")
            && line.contains("locked-a")
            && line.contains("locked-b")
    })
}

const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");

/// `RotateSecret` at `ess/18` with `mode: {Rotate, Freeze}`: `rotated` on `Configured` and
/// `mode == Rotate`, and two held-state refusals, `locked-a` and `locked-b`, both on `Configured`
/// and `mode == Freeze`. With `refusal`, the input refusal `frozen` (`mode == Freeze`) is declared
/// first; it claims every request the two locked refusals select.
fn locked(refusal: bool) -> String {
    let text = replaced(ROTATE, "format: ess/16\n", "format: ess/18\n");
    let text = replaced(
        &text,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n",
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n",
    );
    let text = replaced(
        &text,
        "      - name: too-short\n        when: secret.count < 12\n        error: demo.secrets.SecretTooShort\n",
        if refusal {
            "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n"
        } else {
            ""
        },
    );
    let text = replaced(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Rotate\n",
    );
    replaced(
        &text,
        "      - name: not-configured\n",
        "      - name: locked-a\n        when_subject_state: Configured\n        when: mode == Freeze\n        error: demo.secrets.NotActive\n      - name: locked-b\n        when_subject_state: Configured\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n      - name: not-configured\n",
    )
}

/// With the input refusals read after the held state, `locked-a` and `locked-b` are read before
/// `frozen`, so `frozen` does not answer `Configured` and `mode = Freeze`: the two locked refusals
/// do, and the partition refuses two of them selected together (the control, without `frozen`).
/// The unit admits the command, because `frozen` settles beside each locked refusal on its own.
#[test]
fn an_input_refusal_read_after_two_conflicting_held_state_refusals_does_not_hide_their_conflict() {
    // Control: two held-state refusals selected together, with nothing read before them, conflict.
    let errors = assemble(&locked(false)).expect_err("two locked refusals overlap");
    assert!(names_the_locked_overlap(&errors), "{errors}");
    // In the default order `frozen` is read first and answers alone, as at the base.
    let text = locked(true);
    accepted(&text);
    with_phase_order(
        exchanged(Phase::InputRefusal, Phase::HeldState),
        || match assemble(&text) {
            Ok(_) => panic!(
                "admitted: `frozen` is read after `locked-a` and `locked-b`, which conflict on \
                 `Configured` and `mode = Freeze`, and `refusal_settles` let `frozen` answer alone"
            ),
            Err(errors) => assert!(names_the_locked_overlap(&errors), "{errors}"),
        },
    );
}
