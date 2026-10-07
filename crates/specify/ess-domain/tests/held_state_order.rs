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
