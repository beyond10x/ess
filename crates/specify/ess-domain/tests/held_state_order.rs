//! A branch the held state selects is declared before every accepting and external branch of its
//! command (beyond10x/ess#486).
//!
//! The held state selects at step 4 of the precedence order and the accepting and external
//! branches at step 6, whatever their declaration order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"). Where both
//! guards hold, the model interpreter and Entity Runtime took the first declared and the Rust and
//! Go targets the held-state branch. Refusing the order in which the two disagree leaves one answer
//! for every specification that validates.

const NEEDLE: &str = "is selected by the held state, which answers before";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refusal(body: &str) -> String {
    spec(body).map_or_else(|errors| errors, |_| panic!("the model validates:\n{body}"))
}

/// The held-state order refusals of `body`, one line each.
fn order_refusals(body: &str) -> Vec<String> {
    spec(body).map_or_else(
        |errors| {
            errors
                .lines()
                .filter(|line| line.contains(NEEDLE))
                .map(str::to_owned)
                .collect()
        },
        |_| Vec::new(),
    )
}

/// `CheckPick` declaring `branches`, then `tail`.
fn model(branches: &[&str], tail: &str) -> String {
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

/// The default and the wrong-state answer of a command selecting on a stored field.
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

/// A `when_subject` predicate over the stored `revision`.
const STALE: &str = "      - name: stale
        when_subject:
          predicate: revision != input.revision
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";

/// A second branch the stored `revision` selects.
const FRESH_RUSH: &str = "      - name: fresh-rush
        when_subject:
          predicate: revision == input.revision
        when: rush == true
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
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

/// An input-guarded refusal: step 2, before the held state.
const NO_REVISION: &str = "      - name: no-revision
        when: revision < 0
        error: demo.desk.NoRevision
";

#[test]
fn an_accepting_branch_before_a_stored_field_branch_is_refused() {
    let errors = refusal(&model(&[RUSHED, STALE], FIELD_TAIL));
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.CheckPick.outcomes.stale: `stale` is \
             selected by the held state, which answers before the accepting branch `rushed` \
             declared above it; where both guards hold, the declaration order and the precedence \
             order disagree (hint: declare `stale` before `rushed`: the held state selects first \
             in either order)"
        ),
        "{errors}"
    );
}

#[test]
fn an_external_branch_before_a_stored_field_branch_is_refused() {
    let errors = refusal(&model(&[UNLISTED, STALE], FIELD_TAIL));
    assert!(
        errors.contains(
            "`stale` is selected by the held state, which answers before the external branch \
             `unlisted` declared above it"
        ),
        "{errors}"
    );
}

/// Every held-state branch after the first accepting or external one is named, against that first
/// one, which is the branch to declare it before.
#[test]
fn each_held_state_branch_after_the_first_later_step_is_named() {
    assert_eq!(
        order_refusals(&model(&[UNLISTED, STALE, RUSHED, FRESH_RUSH], FIELD_TAIL))
            .iter()
            .map(|line| line.split(": ").nth(1).unwrap_or_default().to_owned())
            .collect::<Vec<_>>(),
        [
            "`stale` is selected by the held state, which answers before the external branch \
             `unlisted` declared above it; where both guards hold, the declaration order and the \
             precedence order disagree (hint",
            "`fresh-rush` is selected by the held state, which answers before the external branch \
             `unlisted` declared above it; where both guards hold, the declaration order and the \
             precedence order disagree (hint",
        ]
    );
}

#[test]
fn held_state_branches_declared_first_validate() {
    for branches in [
        &[STALE, RUSHED, UNLISTED][..],
        &[STALE, FRESH_RUSH, UNLISTED, RUSHED],
        &[NO_REVISION, STALE, RUSHED],
    ] {
        let body = model(branches, FIELD_TAIL);
        if let Err(errors) = spec(&body) {
            panic!("{errors}\n{body}");
        }
    }
}

/// Input guards no request satisfies together never both hold, so their order is free; a held-state
/// branch reading an input the prover declines (`revision`, an open `Integer`) is refused beside
/// any accepting guard.
#[test]
fn disjoint_input_guards_validate_in_either_order() {
    let slow_stale = "      - name: slow-stale
        when_subject:
          predicate: revision != input.revision
        when: rush == false
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";
    let body = model(&[RUSHED, slow_stale], FIELD_TAIL);
    if let Err(errors) = spec(&body) {
        panic!("{errors}\n{body}");
    }
    let open = "      - name: slow-stale
        when_subject:
          predicate: revision != input.revision
        when: revision > 0
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";
    assert_eq!(
        order_refusals(&model(&[RUSHED, open], FIELD_TAIL)).len(),
        1,
        "an open input guard is not shown disjoint"
    );
}

/// An external branch guarded by the input is refused above a held-state branch only where its guard
/// can hold together with the held-state branch's own.
#[test]
fn a_guarded_external_branch_is_refused_only_where_the_guards_can_both_hold() {
    let guarded = "      - name: unlisted
        external: the current list does not name the pick
        when: rush == true
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickUnlisted]
        payload:
          demo.desk.PickUnlisted: {pick_id: input.pick_id}
";
    let slow_stale = "      - name: slow-stale
        when_subject:
          predicate: revision != input.revision
        when: rush == false
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
";
    let body = model(&[guarded, slow_stale], FIELD_TAIL);
    if let Err(errors) = spec(&body) {
        panic!("{errors}\n{body}");
    }
    let errors = refusal(&model(&[guarded, STALE], FIELD_TAIL));
    assert!(
        errors.contains(
            "`stale` is selected by the held state, which answers before the external branch \
             `unlisted` declared above it"
        ),
        "{errors}"
    );
}

/// An input-guarded refusal answers at step 2, before the held state, so it may be declared first.
#[test]
fn an_input_refusal_declared_first_is_not_an_accepting_branch() {
    assert_eq!(
        order_refusals(&model(&[NO_REVISION, STALE], FIELD_TAIL)),
        Vec::<String>::new()
    );
}

/// A command selecting on a held lifecycle state, with an external branch declared above it.
#[test]
fn an_external_branch_before_a_lifecycle_branch_is_refused() {
    let checked = "      - name: checked
        when_subject_state: Picked
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
";
    let tail = "      - name: not-picked
        error: demo.desk.NotPicked
";
    let errors = refusal(&model(&[UNLISTED, checked], tail));
    assert!(
        errors.contains(
            "`checked` is selected by the held state, which answers before the external branch \
             `unlisted` declared above it"
        ),
        "{errors}"
    );
    assert_eq!(
        order_refusals(&model(&[checked, UNLISTED], tail)),
        Vec::<String>::new()
    );
}

/// beyond10x/ess#534: `SignIn` checks a credential before it reads the seat.
const SIGN_IN: &str = include_str!("../../ess-compiler/tests/fixtures/external-before-row.yaml");

/// The second adopter: a revocation authenticating the client before it reads the token.
const REVOKE: &str =
    include_str!("../../ess-compiler/tests/fixtures/external-before-row-revoke.yaml");

/// The `before_row:` line of both fixtures.
const KEY: &str = "        before_row: true\n";

/// The `ConflictingDeclaration` refusals naming `before_row`, one line each.
fn before_row_refusals(body: &str) -> Vec<String> {
    spec(body).map_or_else(
        |errors| {
            errors
                .lines()
                .filter(|line| line.contains("before_row"))
                .map(str::to_owned)
                .collect()
        },
        |_| Vec::new(),
    )
}

/// An external refusal marked `before_row: true` answers at step 2, before existence and the held
/// state, so the held-state branches declared after it validate (ess/25, beyond10x/ess#534): the
/// issue's `SignIn` and the revocation that authenticates its client first.
#[test]
fn external_refusal_before_row_validates_above_held_state_branches_at_ess_25() {
    for body in [SIGN_IN, REVOKE] {
        assert!(body.contains(KEY));
        if let Err(errors) = spec(body) {
            panic!("{errors}\n{body}");
        }
    }
}

/// Without the key the external refusal is a step-6 branch, and the #486 refusal of every
/// held-state branch declared after it is unchanged, word for word.
#[test]
fn external_refusal_without_the_key_keeps_the_486_refusal() {
    let errors = refusal(&SIGN_IN.replace(KEY, ""));
    for branch in ["reopened-busy", "reopened", "opened"] {
        assert!(
            errors.contains(&format!(
                "[conflicting_declaration] command.demo.desk.SignIn.outcomes.{branch}: `{branch}` \
                 is selected by the held state, which answers before the external branch \
                 `bad-credential` declared above it"
            )),
            "{branch}: {errors}"
        );
    }
    let errors = refusal(&REVOKE.replace(KEY, ""));
    for branch in ["issued-to-another-client", "already-revoked"] {
        assert!(
            errors.contains(&format!(
                "`{branch}` is selected by the held state, which answers before the external \
                 branch `invalid-client` declared above it"
            )),
            "{branch}: {errors}"
        );
    }
}

/// Only an external refusal reads no row: the key is refused on an accepting external branch, on a
/// branch that is not external, beside `compensates:`, where its error payload reads the row, below
/// `ess/25`, declared after a branch a later step answers and on a command reading a related row.
#[test]
fn before_row_on_an_accepting_external_or_below_ess_25_is_refused() {
    // An accepting external branch acts on the row, so reads it.
    let accepting = SIGN_IN.replace(
        "        error: demo.desk.BadCredential\n",
        "        moves: demo.desk.Seat.close\n        instance: user_id\n        emits: \
         [demo.desk.Closed]\n        payload: {demo.desk.Closed: {user_id: input.user_id}}\n",
    );
    let refused = before_row_refusals(&accepting);
    assert!(
        refused.iter().any(|line| line.contains(
            "[conflicting_declaration] command.demo.desk.SignIn.outcomes.bad-credential.before_row"
        ) && line.contains("names no error")),
        "{refused:#?}"
    );

    // Below ess/25 the key is refused naming ess/25.
    let errors = refusal(&SIGN_IN.replace("format: ess/25", "format: ess/24"));
    assert!(
        errors.contains(
            "[unsupported_format_version] command.demo.desk.SignIn.outcomes.bad-credential.\
             before_row"
        ) && errors.contains("ess/25"),
        "{errors}"
    );

    // A branch that is not external.
    let plain = SIGN_IN.replace(
        "        external: secret is not the digest the service derives from user_id\n",
        "        when: secret == \"\"\n",
    );
    let refused = before_row_refusals(&plain);
    assert!(
        refused
            .iter()
            .any(|line| line.contains("conflicting_declaration") && line.contains("external")),
        "{refused:#?}"
    );

    // Declared after a branch the held state selects.
    let bad = "      - name: bad-credential\n        external: secret is not the digest the \
               service derives from user_id\n        before_row: true\n        error: \
               demo.desk.BadCredential\n";
    let after = SIGN_IN.replace(bad, "").replace(
        "      - name: first\n",
        &format!("{bad}      - name: first\n"),
    );
    let refused = before_row_refusals(&after);
    assert!(
        refused.iter().any(
            |line| line.contains("conflicting_declaration") && line.contains("`reopened-busy`")
        ),
        "{refused:#?}"
    );

    // Its error payload reading the stored row.
    let reading = SIGN_IN
        .replace(
            "        error: demo.desk.BadCredential\n",
            "        error: demo.desk.BadCredential\n        payload:\n          \
             demo.desk.BadCredential: {call_ref: {subject: call_ref}}\n",
        )
        .replace(
            "  - name: demo.desk.BadCredential\n",
            "  - name: demo.desk.BadCredential\n    fields: [{name: call_ref, type: String}]\n",
        );
    let refused = before_row_refusals(&reading);
    assert!(
        refused
            .iter()
            .any(|line| line.contains("conflicting_declaration") && line.contains("reads")),
        "{refused:#?}"
    );
}

/// A refusal that changes its addressed row (`compensates: true`) reads it, so it never answers
/// before it.
#[test]
fn before_row_beside_compensates_is_refused() {
    let text = include_str!("../../ess-compiler/tests/fixtures/refusal-with-effect.yaml")
        .replace("format: ess/22", "format: ess/25")
        .replace(
            "        compensates: true\n",
            "        compensates: true\n        before_row: true\n",
        );
    let refused = before_row_refusals(&text);
    assert!(
        refused.iter().any(|line| line.contains(
            "[conflicting_declaration] command.shop.order.JoinOrder.outcomes.failed.before_row"
        ) && line.contains("compensates")),
        "{refused:#?}"
    );
}

/// Step 1 (a related row read through the input, or a row set) answers before step 2; whether a
/// `before_row` external should answer before it is not designed, so the pair is refused.
#[test]
fn before_row_on_a_command_reading_a_related_row_is_refused() {
    let text =
        include_str!("../../ess-compiler/tests/fixtures/precedence-upsert-unknown-instance.yaml")
            .replace("format: ess/23", "format: ess/25")
            .replacen(
                "      - name: unknown-book\n",
                "      - name: unauthenticated\n        external: the curator is not \
                 authenticated\n        before_row: true\n        error: demo.shelf.Refused\n      \
                 - name: unknown-book\n",
                1,
            );
    let refused = before_row_refusals(&text);
    assert!(
        refused.iter().any(|line| line.contains(
            "[conflicting_declaration] command.demo.shelf.Shelve.outcomes.unauthenticated.\
             before_row"
        ) && line.contains("when_related")),
        "{refused:#?}"
    );
}

/// `before_row: false` is the document without the key: the #486 refusal stands.
#[test]
fn before_row_false_is_the_key_left_out() {
    let errors = refusal(&SIGN_IN.replace(KEY, "        before_row: false\n"));
    assert!(
        errors.contains(
            "`reopened-busy` is selected by the held state, which answers before the external \
             branch `bad-credential` declared above it"
        ),
        "{errors}"
    );
}

// ---- the one precedence rule (G4): the guard-pair corpus ------------------------------------

const PAIRS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml");

/// The guard-pair corpus in `format`: every command validates in `ess/27`; below it, each pair
/// keeps the refusal it had.
fn pairs_at(
    format: &str,
) -> Result<ess_domain::Specification, ess_primitives::error::ValidationErrors> {
    let text = PAIRS.replacen("format: ess/27\n", &format!("format: {format}\n"), 1);
    assert!(text.starts_with(&format!("format: {format}\n")));
    let raw = ess_domain::spec::RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("pairs.yaml"), raw)])
}

/// The refusals of the corpus one format below the rule that name `command`, each with its hint.
fn pair_refusals(command: &str) -> Vec<String> {
    let Err(errors) = pairs_at("ess/26") else {
        panic!("below ess/27 the corpus is refused");
    };
    errors
        .as_slice()
        .iter()
        .filter(|error| error.to_string().contains(&format!("command.{command}")))
        .map(ToString::to_string)
        .collect()
}

/// Under the one rule an `external:` refusal marked `before_row: true` answers at step 1, before a
/// related row read at step 4; below it the pair keeps its refusal, the hint naming `ess/27`.
#[test]
fn format_a_admits_before_row_beside_when_related() {
    pairs_at("ess/27").unwrap_or_else(|errors| panic!("the corpus validates at ess/27: {errors}"));
    let refusals = pair_refusals("demo.pairs.BeforeRowBesideRelated");
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.contains("before_row") && refusal.contains("ess/27")),
        "{refusals:#?}"
    );
}
