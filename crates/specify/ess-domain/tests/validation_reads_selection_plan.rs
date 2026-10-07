//! Validation reads the precedence classification (`story:validation-reads-selection-plan`,
//! `docs/design/selection-plan.md`): inside wave 2's phase-order override, the held-state order
//! refusal (beyond10x/ess#486), the held-state partition's input-refusal-first rule
//! (beyond10x/ess#227) and the present-related order (beyond10x/ess#282, #283) follow the order
//! the override sets. No seam of this story's own: `with_phase_order` is the one test seam.
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

// ---- the held-state order (beyond10x/ess#486) --------------------------------------------------

/// `CheckPick` declaring `branches`, then `tail`: `tests/held_state_order.rs`'s model.
fn pick(branches: &[&str], tail: &str) -> String {
    format!(
        "format: ess/20
system: demo
version: v1
domain: demo.desk
types:
  - {{name: demo.desk.PickId, kind: newtype, of: Uuid}}
entities:
  - name: demo.desk.Pick
    identity: {{name: pick_id, type: demo.desk.PickId}}
    fields:
      - {{name: revision, type: Integer}}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused]
      terminal: [Accepted, Refused]
      transitions:
        - {{name: accept, from: [Picked], to: Accepted}}
        - {{name: refuse, from: [Picked], to: Refused}}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.MakePick, demo.desk.CheckPick]
commands:
  - name: demo.desk.MakePick
    input:
      - {{name: revision, type: Integer}}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {{revision: input.revision}}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {{pick_id: {{generated: true}}}}
  - name: demo.desk.CheckPick
    input:
      - {{name: pick_id, type: demo.desk.PickId}}
      - {{name: revision, type: Integer}}
      - {{name: rush, type: Boolean}}
    outcomes:
{}{tail}events:
  - name: demo.desk.Picked
    fields: [{{name: pick_id, type: demo.desk.PickId}}]
  - name: demo.desk.PickStale
    fields: [{{name: pick_id, type: demo.desk.PickId}}]
  - name: demo.desk.PickUnlisted
    fields: [{{name: pick_id, type: demo.desk.PickId}}]
  - name: demo.desk.PickAccepted
    fields: [{{name: pick_id, type: demo.desk.PickId}}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NoRevision
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {{name: pick_id, type: demo.desk.PickId}}
      - {{name: revision, type: Integer}}
      - {{name: state, type: demo.desk.Pick.State}}
",
        branches.concat()
    )
}

const FIELD_TAIL: &str = "      - name: accepted
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
      - name: wrong-state
        wrong_state: true
        error: demo.desk.NotPicked
";

const STALE: &str = "      - name: stale
        when_subject:
          predicate: revision != input.revision
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";

const RUSHED: &str = "      - name: rushed
        when: rush == true
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
";

const UNLISTED: &str = "      - name: unlisted
        external: the current list does not name the pick
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
";

const CHECKED: &str = "      - name: checked
        when_subject_state: Picked
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
";

const LIFECYCLE_TAIL: &str = "      - name: not-picked
        error: demo.desk.NotPicked
";

/// A held-state branch whose input guard the finite prover shows disjoint from `RUSHED`'s.
const SLOW_STALE: &str = "      - name: slow-stale
        when_subject:
          predicate: revision != input.revision
        when: rush == false
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";

const HELD_AFTER_ACCEPTING: &str = "[conflicting_declaration] \
    command.demo.desk.CheckPick.outcomes.stale: `stale` is selected by the held state, which \
    answers before the accepting branch `rushed` declared above it; where both guards hold, the \
    declaration order and the precedence order disagree (hint: declare `stale` before `rushed`: \
    the held state selects first in either order)";

const ACCEPTING_AFTER_HELD: &str = "[conflicting_declaration] \
    command.demo.desk.CheckPick.outcomes.rushed: `rushed` is an accepting branch, which answers \
    before the held-state branch `stale` declared above it; where both guards hold, the \
    declaration order and the precedence order disagree (hint: declare `rushed` before `stale`: \
    an accepting branch selects first in either order)";

const EXTERNAL_AFTER_HELD: &str = "[conflicting_declaration] \
    command.demo.desk.CheckPick.outcomes.unlisted: `unlisted` is an external branch, which \
    answers before the held-state branch `checked` declared above it; where both guards hold, the \
    declaration order and the precedence order disagree (hint: declare `unlisted` before \
    `checked`: an external branch selects first in either order)";

/// Acceptance 3: with the held-state and accepting/external phases exchanged, the order #486
/// refused validates and the order it admitted is refused, against the branch now read second.
#[test]
fn the_held_state_order_refusal_follows_the_phase_order() {
    let accepting_first = pick(&[RUSHED, STALE], FIELD_TAIL);
    let held_first = pick(&[STALE, RUSHED], FIELD_TAIL);
    let external_first = pick(&[UNLISTED, CHECKED], LIFECYCLE_TAIL);
    let lifecycle_first = pick(&[CHECKED, UNLISTED], LIFECYCLE_TAIL);

    let errors = refused(&accepting_first).to_string();
    assert!(errors.contains(HELD_AFTER_ACCEPTING), "{errors}");
    accepted(&held_first);
    refused(&external_first);
    accepted(&lifecycle_first);

    with_phase_order(exchanged(Phase::HeldState, Phase::Accepting), || {
        accepted(&accepting_first);
        let errors = refused(&held_first).to_string();
        assert!(errors.contains(ACCEPTING_AFTER_HELD), "{errors}");
        assert!(
            !errors.contains("is selected by the held state"),
            "the default order's refusal is not also reported: {errors}"
        );

        accepted(&external_first);
        let errors = refused(&lifecycle_first).to_string();
        assert!(errors.contains(EXTERNAL_AFTER_HELD), "{errors}");
    });
}

/// Inside the override, guards the finite prover shows disjoint still leave the order free.
#[test]
fn disjoint_guards_stay_free_in_the_exchanged_order() {
    with_phase_order(exchanged(Phase::HeldState, Phase::Accepting), || {
        accepted(&pick(&[SLOW_STALE, RUSHED], FIELD_TAIL));
        accepted(&pick(&[RUSHED, SLOW_STALE], FIELD_TAIL));
    });
}

// ---- input refusals before the held state (beyond10x/ess#227) ---------------------------------

const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");

/// `RotateSecret` with a `mode` input, its refusal `frozen` and its state-guarded `rotated` both
/// selected by `mode == Freeze` in `Configured`: `tests/refusal_beside_state.rs`'s decided overlap.
fn decided_overlap() -> String {
    let typed = replaced(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze]}\n",
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
    replaced(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Freeze\n",
    )
}

/// The input refusal answers the overlap where its phase is read before the held state's; with the
/// two exchanged the held-state partition no longer takes it first, and the overlap is a conflict.
#[test]
fn the_input_refusal_answers_first_only_where_its_phase_is_read_first() {
    let text = decided_overlap();
    accepted(&text);
    with_phase_order(exchanged(Phase::InputRefusal, Phase::HeldState), || {
        let errors = refused(&text);
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "held state Configured and input [mode = Freeze] select 2 branches: frozen, rotated"
            ),
            "{errors}"
        );
    });
}

/// A refusal whose guard the prover declines (`secret.count < 12`) leaves the joint proof only
/// because no request it claims reaches the branches after it; with the input refusals read after
/// the held state, it stays, and the proof is unavailable beside a state refusal.
#[test]
fn an_undecidable_input_refusal_leaves_the_proof_only_where_its_phase_is_read_first() {
    accepted(ROTATE);
    with_phase_order(exchanged(Phase::InputRefusal, Phase::HeldState), || {
        let errors = refused(ROTATE);
        assert!(
            has(
                &errors,
                ValidationCode::NonExhaustiveBranches,
                "subject-state/input coverage is open, unsupported, or exceeds 64 joint assignments"
            ),
            "{errors}"
        );
    });
}

// ---- present-related refusals before the accepting branches (beyond10x/ess#282, #283) --------

const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");

/// `PublishRelease` at `ess/22` beside `wrong_state:` (#282), its present-related refusal
/// `not-accepted` overlapping the accepting `held` and `published`, which split on `publish`.
fn release_beside_wrong_state() -> String {
    let text = replaced(RELEASE, "format: ess/20\n", "format: ess/22\n");
    let text = replaced(
        &text,
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
        "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
    );
    let text = replaced(
        &text,
        "      - {name: candidate, type: demo.release.CandidateId}\n",
        "      - {name: candidate, type: demo.release.CandidateId}\n      - {name: publish, type: Boolean}\n",
    );
    replaced(
        &text,
        "      - name: published\n",
        "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: held\n        when: publish == false\n        preserves: demo.release.Release\n        instance: release_id\n      - name: published\n        when: publish == true\n",
    )
}

/// One related row: the selected refusal answers before the accepting branches where its phase is
/// read first; with the two exchanged the overlap is a conflict.
#[test]
fn a_present_related_refusal_answers_first_only_where_its_phase_is_read_first() {
    let text = release_beside_wrong_state();
    accepted(&text);
    with_phase_order(exchanged(Phase::PresentRelated, Phase::Accepting), || {
        let errors = refused(&text);
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "select 2 branches: not-accepted, published"
            ),
            "{errors}"
        );
    });
}

const MULTIPLE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");

/// `StartRun` over two rows (#283), starting only under a granted capability: `switch-paused`
/// overlaps `started` on a paused switch and a granted capability.
fn two_rows() -> String {
    replaced(
        MULTIPLE,
        "      - name: started\n        creates: demo.run.Run\n",
        "      - name: started\n        when_related: {via: input.capability, predicate: state == Granted}\n        creates: demo.run.Run\n",
    )
}

/// Several related rows: the first declared refusal answers before the accepting branches where
/// its phase is read first; with the two exchanged the overlap is a conflict.
#[test]
fn a_refusal_over_several_rows_answers_first_only_where_its_phase_is_read_first() {
    let text = two_rows();
    accepted(&text);
    with_phase_order(exchanged(Phase::PresentRelated, Phase::Accepting), || {
        let errors = refused(&text);
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "select 2 branches: switch-paused, started"
            ),
            "{errors}"
        );
    });
}
