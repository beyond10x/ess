//! A literal a branch writes into a row is held to the row's entity invariants (downstream report,
//! 2026-10-19, finding 16).
//!
//! `invariants: [capacity > 0]` beside `sets: {capacity: 0}` used to validate, while the same
//! literal against a newtype's invariant was refused; the lowered rule then refused that branch on
//! every request. An invariant every field of which the branch writes with a literal, or whose
//! `state` is the state the branch arrives in, is decided at validation: one the literals make false
//! is refused, naming the branch, the invariant and the literal. A value read from the input stays
//! the run's to decide.

use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<ess_domain::Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("pool.yaml"), raw)])
}

fn admitted(body: &str) {
    if let Err(errors) = assemble(body) {
        panic!("must validate: {errors}\n{body}");
    }
}

fn refused(body: &str) -> ValidationErrors {
    assemble(body)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{body}"))
}

/// The one refusal naming the entity's invariant, which must name `fragments` too.
fn refusal_naming(errors: &ValidationErrors, fragments: &[&str]) {
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.message.contains("capacity > 0"))
        .unwrap_or_else(|| panic!("a refusal names the invariant: {errors}"));
    assert_eq!(
        refusal.code,
        ValidationCode::ConflictingDeclaration,
        "{refusal}"
    );
    let rendered = refusal.to_string();
    for fragment in fragments {
        assert!(rendered.contains(fragment), "`{fragment}`: {rendered}");
    }
}

/// A pool whose capacity is always above zero: `OpenPool` creates one, setting `{opened}`;
/// `DrainPool` moves it to `Drained`, setting `{drained}`; `ShrinkPools` changes every pool of a
/// keeper it addresses through `affects:`, setting `{shrunk}`.
fn model(opened: &str, drained: &str, shrunk: &str) -> String {
    format!(
        "format: ess/25
system: pool
version: v1
domain: pool.slots
summary: A pool always has room for at least one slot.
types:
  - {{name: pool.slots.PoolId, kind: newtype, of: Uuid}}
  - {{name: pool.slots.KeeperId, kind: newtype, of: Uuid}}
entities:
  - name: pool.slots.Keeper
    identity: {{name: keeper_id, type: pool.slots.KeeperId}}
    fields: []
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: pool.slots.Pool
    identity: {{name: pool_id, type: pool.slots.PoolId}}
    fields:
      - {{name: keeper, type: pool.slots.KeeperId}}
      - {{name: capacity, type: Integer}}
      - {{name: spare, type: Integer}}
    invariants:
      - capacity > 0
      - {{any: [state != Drained, spare == 0]}}
    lifecycle:
      initial: Open
      states: [Open, Drained]
      terminal: [Drained]
      transitions:
        - {{name: drain, from: [Open], to: Drained}}
errors:
  - {{name: pool.slots.WrongState, summary: The pool is not open., fields: []}}
events:
  - name: pool.slots.KeeperStarted
    fields: [{{name: keeper_id, type: pool.slots.KeeperId}}]
  - name: pool.slots.PoolOpened
    fields: [{{name: pool_id, type: pool.slots.PoolId}}]
  - name: pool.slots.PoolDrained
    fields: [{{name: pool_id, type: pool.slots.PoolId}}]
  - name: pool.slots.PoolsShrunk
    fields: [{{name: keeper_id, type: pool.slots.KeeperId}}]
actors:
  - name: pool.slots.Operator
    may: [pool.slots.StartKeeper, pool.slots.OpenPool, pool.slots.DrainPool, pool.slots.ShrinkPools]
commands:
  - name: pool.slots.StartKeeper
    input: []
    outcomes:
      - name: started
        creates: pool.slots.Keeper
        instance: keeper_id
        emits: [pool.slots.KeeperStarted]
        payload: {{pool.slots.KeeperStarted: {{keeper_id: {{generated: true}}}}}}
  - name: pool.slots.OpenPool
    input:
      - {{name: keeper, type: pool.slots.KeeperId}}
      - {{name: capacity, type: Integer}}
    outcomes:
      - name: opened
        creates: pool.slots.Pool
        instance: pool_id
        emits: [pool.slots.PoolOpened]
        payload: {{pool.slots.PoolOpened: {{pool_id: {{generated: true}}}}}}
        sets: {{keeper: input.keeper, {opened}}}
  - name: pool.slots.DrainPool
    input:
      - {{name: pool_id, type: pool.slots.PoolId}}
    outcomes:
      - name: drained
        moves: pool.slots.Pool.drain
        instance: pool_id
        emits: [pool.slots.PoolDrained]
        payload: {{pool.slots.PoolDrained: {{pool_id: input.pool_id}}}}
        sets: {{{drained}}}
      - {{name: wrong-state, wrong_state: true, error: pool.slots.WrongState}}
  - name: pool.slots.ShrinkPools
    input:
      - {{name: keeper_id, type: pool.slots.KeeperId}}
      - {{name: capacity, type: Integer}}
    outcomes:
      - name: shrunk
        updates: pool.slots.Keeper
        instance: keeper_id
        emits: [pool.slots.PoolsShrunk]
        payload: {{pool.slots.PoolsShrunk: {{keeper_id: input.keeper_id}}}}
        affects:
          - entity: pool.slots.Pool
            where: keeper == subject.keeper_id
            sets: {{{shrunk}}}
"
    )
}

const FROM_INPUT: (&str, &str, &str) = (
    "capacity: input.capacity, spare: 1",
    "spare: 0",
    "capacity: input.capacity",
);

#[test]
fn a_literal_write_breaking_an_entity_invariant_is_refused() {
    let (opened, drained, shrunk) = FROM_INPUT;
    admitted(&model(opened, drained, shrunk));

    // The downstream case: a move writing a literal the invariant refuses.
    refusal_naming(
        &refused(&model(opened, "spare: 0, capacity: 0", shrunk)),
        &[
            "`drained`",
            "pool.slots.DrainPool",
            "`0`",
            "pool.slots.Pool",
        ],
    );
    // A creating branch.
    refusal_naming(
        &refused(&model("capacity: 0, spare: 1", drained, shrunk)),
        &["`opened`", "pool.slots.OpenPool", "`0`"],
    );
    // An `affects:` entry's `sets:`.
    refusal_naming(
        &refused(&model(opened, drained, "capacity: -2")),
        &["`shrunk`", "pool.slots.ShrinkPools", "`-2`", "affects"],
    );
    // A literal the invariant holds for is admitted.
    admitted(&model("capacity: 3, spare: 1", drained, "capacity: 1"));
}

#[test]
fn the_state_a_branch_arrives_in_is_read_beside_its_literals() {
    let (opened, _, shrunk) = FROM_INPUT;
    // `state != Drained` or `spare == 0`: the move arrives in `Drained`, so a spare of 2 breaks it.
    let errors = refused(&model(opened, "spare: 2", shrunk));
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.message.contains("spare == 0"))
        .unwrap_or_else(|| panic!("the state-reading invariant is named: {errors}"));
    assert_eq!(refusal.code, ValidationCode::ConflictingDeclaration);
    assert!(refusal.message.contains("`drained`"), "{refusal}");
    assert!(
        refusal.message.contains("leaves the row in `Drained`"),
        "{refusal}"
    );
    // Created in `Open`, a spare of 2 is admitted.
    admitted(&model(
        "capacity: input.capacity, spare: 2",
        "spare: 0",
        shrunk,
    ));
    // An `affects:` entry that moves no row leaves its state unknown, so the invariant reading it
    // is the run's to decide.
    admitted(&model(opened, "spare: 0", "spare: 2"));
}

#[test]
fn a_value_read_from_the_input_is_not_decided_at_validation() {
    // Every write of `capacity` reads the input: no literal makes the invariant false.
    let (opened, drained, shrunk) = FROM_INPUT;
    admitted(&model(opened, drained, shrunk));
}
