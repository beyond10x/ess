//! Adversary pass 1 on `story:validation-reads-selection-plan` (wave 3, unit U3).
//!
//! The unit's claim: validation takes each branch's phase from `precedence::place` and the
//! answering order from `precedence::phase_order`. The property these cases hold it to: an
//! override that leaves the relative order of two phases unchanged leaves the verdict on an
//! overlap between branches of those two phases unchanged. Both partitions gate their tie-break on
//! the position of a third phase, so a phase move that does not involve the overlapping branches
//! changes the verdict.
//!
//! The last case pins the verdict the coordinator kept (R2): `wrong_state:` beside a present-related
//! refusal and a present-related acceptance, `ess/22`, input row, is refused, in the default order
//! and inside the override, because `orders_present_refusals` keeps its own admission rule.
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

/// Whether `first` is read before `second` in `order`.
fn before(order: [Phase; 8], first: Phase, second: Phase) -> bool {
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    at(first) < at(second)
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

// ---- subject_state: an input refusal overlapping only an accepting branch ----------------------

const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");

/// `RotateSecret` with `mode: {Rotate, Freeze, Hold}`: the input refusal `frozen` (`mode ==
/// Freeze`), the held-state `rotated` (`Configured`, `mode == Rotate`) and the accepting `kept`
/// (`mode != Rotate`, every held state). `frozen` overlaps `kept` alone, on `mode = Freeze` in every
/// held state; it never overlaps `rotated`.
fn refusal_beside_accepting() -> String {
    let typed = replaced(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze, Hold]}\n",
    );
    let text = replaced(
        &typed,
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n",
        "      - {name: tenant_id, type: demo.secrets.TenantId}\n      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n",
    );
    let text = replaced(
        &text,
        "      - name: too-short\n        when: secret.count < 12\n        error: demo.secrets.SecretTooShort\n",
        "      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.SecretTooShort\n",
    );
    let text = replaced(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Rotate\n",
    );
    replaced(
        &text,
        "      - name: not-configured\n",
        "      - name: kept\n        when: mode != Rotate\n        updates: demo.secrets.Configuration\n        instance: tenant_id\n        sets: {secret: input.secret}\n        emits: [demo.secrets.SecretRotated]\n        payload: {demo.secrets.SecretRotated: {tenant_id: input.tenant_id}}\n      - name: not-configured\n",
    )
}

/// Exchanging `InputRefusal` with `HeldState` leaves `InputRefusal` before `Accepting`, so `frozen`
/// still answers before `kept`, and the only overlap in the command is theirs. The unit's
/// `input_refusals_answer_first` asks whether the input refusals precede *both* `HeldState` and
/// `Accepting`, and drops the tie-break for every pair once one of the two moves: the overlap
/// becomes a conflict.
#[test]
fn an_input_refusal_still_answers_an_accepting_branch_it_is_still_read_before() {
    let text = refusal_beside_accepting();
    accepted(&text);
    // Control: an exchange that keeps `InputRefusal` before both phases changes nothing.
    with_phase_order(exchanged(Phase::InputRefusal, Phase::Existence), || {
        accepted(&text);
    });
    let order = exchanged(Phase::InputRefusal, Phase::HeldState);
    assert!(before(order, Phase::InputRefusal, Phase::Accepting));
    with_phase_order(order, || {
        if let Err(errors) = assemble(&text) {
            panic!(
                "the override moved neither `frozen`'s phase nor `kept`'s relative to the \
                 other, and the verdict on their overlap changed:\n{errors}"
            );
        }
    });
}

// ---- related_guard: an input refusal overlapping a present-related refusal --------------------

const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");

/// `PublishRelease` at `ess/22` beside `wrong_state:` (#282), with `mode: {Hold, Publish, Block}`:
/// the present-related refusal `not-accepted` (`state != Accepted`), the input refusal `blocked`
/// (`mode == Block`) and the accepting `held` and `published`, which split the other two modes.
/// `not-accepted` and `blocked` are selected together on a proposed candidate and `mode = Block`;
/// no accepting branch is selected there.
fn related_refusal_beside_input_refusal() -> String {
    let text = replaced(RELEASE, "format: ess/20\n", "format: ess/22\n");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.release.CandidateId, kind: newtype, of: Uuid}\n  - {name: demo.release.Mode, kind: enum, variants: [Hold, Publish, Block]}\n",
    );
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n  - {name: demo.release.Blocked, summary: The request is blocked., fields: []}\n",
    );
    let text = replaced(
        &text,
        "      - {name: candidate, type: demo.release.CandidateId}\n",
        "      - {name: candidate, type: demo.release.CandidateId}\n      - {name: mode, type: demo.release.Mode}\n",
    );
    replaced(
        &text,
        "      - name: published\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: blocked\n        when: mode == Block\n        error: demo.release.Blocked\n      - name: held\n        when: mode == Hold\n        preserves: demo.release.Release\n        instance: release_id\n      - name: published\n        when: mode == Publish\n",
    )
}

/// Exchanging `PresentRelated` with `Accepting` leaves `InputRefusal` before `PresentRelated`, so
/// `blocked` still answers before `not-accepted`, and no accepting branch takes part in their
/// overlap. The unit's `selected_count` gates the whole count on `PresentRelated` preceding
/// `Accepting`, so a pair with no accepting branch in it becomes a conflict.
///
/// Only that pair is asserted: the overlaps of `not-accepted` with the accepting `held` and
/// `published` do change order under this exchange, and the unit refuses them by design.
#[test]
fn an_input_refusal_beside_a_present_related_refusal_is_not_decided_by_the_accepting_phase() {
    const PAIR: &str = "related [state = Proposed] and input [mode = Block] select 2 branches: \
                        not-accepted, blocked";
    let text = related_refusal_beside_input_refusal();
    accepted(&text);
    // Control: an exchange that keeps `PresentRelated` before `Accepting` changes nothing.
    with_phase_order(exchanged(Phase::PresentRelated, Phase::HeldState), || {
        accepted(&text);
    });
    let order = exchanged(Phase::PresentRelated, Phase::Accepting);
    assert!(before(order, Phase::InputRefusal, Phase::PresentRelated));
    with_phase_order(order, || {
        let errors = refused(&text);
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "select 2 branches: not-accepted, published"
            ),
            "the exchange reaches the partition: {errors}"
        );
        assert!(
            !has(&errors, ValidationCode::ConflictingDeclaration, PAIR),
            "the override moved neither `blocked`'s phase nor `not-accepted`'s relative to the \
             other, and the verdict on their overlap changed:\n{errors}"
        );
    });
}

// ---- R2: the verdict the coordinator kept ------------------------------------------------------

/// `PublishRelease` at `ess/22`, input row, `wrong_state:` beside the present-related refusal
/// `not-accepted` and the present-related acceptance `published` (`state == Accepted`).
fn wrong_state_beside_related_acceptance() -> String {
    let text = replaced(RELEASE, "format: ess/20\n", "format: ess/22\n");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
    );
    replaced(
        &text,
        "      - name: published\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: published\n        when_related: {via: input.candidate, predicate: state == Accepted}\n",
    )
}

const R2_REFUSAL: &str = "command.demo.release.PublishRelease.outcomes: \
    `demo.release.PublishRelease` selects on a related row (`when_related`) and on a `wrong_state` \
    branch; which of the two answers first is not stated (hint: guard the command on the related \
    row alone, or split the other guard into a command of its own)";

/// R2 stays validation's verdict: refused in the default order, and inside every override that
/// moves the phases its branches answer in.
#[test]
fn r2_wrong_state_beside_a_related_acceptance_stays_refused() {
    let text = wrong_state_beside_related_acceptance();
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, R2_REFUSAL),
        "{errors}"
    );
    for order in [
        exchanged(Phase::PresentRelated, Phase::Accepting),
        exchanged(Phase::HeldState, Phase::PresentRelated),
        exchanged(Phase::HeldState, Phase::Accepting),
    ] {
        with_phase_order(order, || {
            let errors = refused(&text);
            assert!(
                has(&errors, ValidationCode::ConflictingDeclaration, R2_REFUSAL),
                "{order:?}: {errors}"
            );
        });
    }
}
