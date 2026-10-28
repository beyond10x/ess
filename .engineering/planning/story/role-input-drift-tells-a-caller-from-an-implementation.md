---
format: aep.planning-md/3
id: story:role-input-drift-tells-a-caller-from-an-implementation
kind: story
status: active
title: Role input drift tells a caller from an implementation
summary: 'Adopter report N12: role_input_drift on a producer sending a subset (ess#666).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#666
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/implements-drift
- confidence: cited
  path: crates/specify/ess-composition/tests/roles.rs
- confidence: cited
  path: docs/design/composition-contract-roles.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:29Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:30Z", actor: "human:timo", revision: 11}
---
## Problem

An adopter gets `role_input_drift` on 13 fields with `stands: implements` where "the producer sends a subset". Under the `implements` rule (`crates/specify/ess-composition/src/roles.rs:1-15`) a local command missing a contract input is drift. If the local command is the contract's caller, no stance fits it. Adopter report N12, ess 0.68.0. Needs a minimal repro first.

## Decisions

Decline, with the idiom. Decided by the operator under the wave's option A (approval-record:d133-role-input-drift-keeps-the-rule, decider operator, 2026-10-28).

- `role_input_drift` on a `stands: implements` entry whose local command lacks contract inputs stays. The rule is unchanged: an implementation answers the contract command, so it accepts every input the contract's caller sends (`docs/design/composition-contract-roles.md`, check 4).
- A caller of a contract command is not a role entry. The idiom: leave the command out of `roles` (nothing is compared), or, where the local command decides before the contract command succeeds, use `stands: gates` (which compares no inputs). `docs/design/composition-contract-roles.md` states this, and the idiom is recorded in the specification artifact `specification:composition-caller-is-not-a-role-entry`.
- The rule is pinned by the fixture test `an_implementation_missing_a_contract_input_drifts`, in `crates/specify/ess-composition/tests/roles.rs`. The existing `an_implements_entry_whose_inputs_drift_is_refused` already covers a retyped input and a missing required input; the new test adds a missing `Optional` contract input.
- Follow-up waits for the reporter's answer (which side sends the subset; whether the contract's fields are `Optional`). B, a `stands: calls` role, lands at the next `ess-composition/N` after `/5` (`/6`). C, `implements` tolerates a missing `Optional` contract input behind `reader: true`. Neither is in this story's scope.

## Acceptance

- Characterization pin (the rule is unchanged, so this passes before and after; it keeps a later
  change from loosening the rule silently): `an_implementation_missing_a_contract_input_drifts`, a
  `stands: implements` entry whose local command lacks a contract input is refused as
  `role_input_drift`, naming the entry, both commands and the missing field, for a required
  contract input and for an `Optional` one. New fixture
  `crates/specify/ess-composition/tests/fixtures/implements-drift/` (a copy of the two systems with a
  contract command that has an `Optional` input; `contract-role/` stays unchanged because its IR
  bytes are pinned). Command: `cargo test -p ess-composition --locked --test roles an_implementation_missing_a_contract_input_drifts`.
- The documentation change, the story's transition: `docs/design/composition-contract-roles.md`
  gains the sentence "A caller of a contract command is not a role entry: give its command no
  entry, or `stands: gates` when it only admits the call." Before the edit
  `git grep -n -F "A caller of a contract command is not a role entry" docs/design/composition-contract-roles.md`
  exits 1; after it, it exits 0 and prints the line.

## Scope

- `crates/specify/ess-composition/tests/roles.rs`: the new test function beside `an_implements_entry_whose_inputs_drift_is_refused` (cited).
- `docs/design/composition-contract-roles.md`: the caller sentence in check 4 (cited).
- `crates/specify/ess-composition/tests/fixtures/implements-drift/`: new fixture (inferred).

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
