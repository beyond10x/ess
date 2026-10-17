---
format: aep.planning-md/3
id: story:explorer-model-applies-selected-row-sets-and-draws-decimals
kind: story
status: draft
title: The explorer model applies selected-row effects and draws Decimal inputs
relations:
- depends_on: story:go-explorer-draws-read-only-commands
revision: 1
---
## Outcome

The generated explorers' reference model applies a selected-rows effect (`moves:` with
`instances: {where: …}` and its `sets:`) to every selected row, as synthesis and the interpreter
do, and the sequential explorer draws `Decimal` inputs as it draws `Integer` ones, instead of
excluding the command.

Closes https://github.com/beyond10x/ess/issues/538.

## Evidence

- https://github.com/beyond10x/ess/issues/538 (ess 0.59.0): after an outcome with `moves:`,
  `instances: {where: …}` and `sets:`, the model keeps the selected rows' old field values, so the
  first view read reports a disagreement the implementation did not make (`last_reason` stays
  `none` where the rows were set to `hungup`). A command with a `Decimal` input is excluded
  ("dial_level is a decimal").
- Still on `origin/main` (260a3d1e3, read 2026-10-13):
  - A selected-rows outcome carries `instances` with `subject` `None` (`ResolvedOutcome::instances`,
    `crates/specify/ess-compiler/src/ir.rs:975-978`). `exploreEffectRefusal`
    (`crates/verify/ess-conformance/src/go/explore.go:852-856`) returns no reason when `subject` is
    absent, so the command is drawn; the model update (`explore.go:2060-2106`) reads only
    `outcome["subject"]`, so the selected rows are never moved or set.
  - `exploreResolveAs` (`explore.go:369-389`) answers `unsupported` for `decimal` unless
    `concurrent`; the sequential drawer already has a `"integer", "decimal"` arm
    (`explore.go:1854`), and the TypeScript explorer the same split (`ts/explore.ts:315-335`,
    `:1563`).

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/explore.rs`,
  `the_model_moves_and_sets_every_selected_row`: a model with `Close` (`moves: Seat.close`,
  `instances: {where: team == input.team}`, `sets: {last_reason: hungup}`) and a view over `Seat`;
  exploration against a correct target finds no disagreement after `Close`, and a target that
  leaves one selected row's `last_reason` unset is reported at that step.
- Same file, `a_selected_row_effect_leaves_unselected_rows_alone`: rows of another team keep their
  state and fields in the model.
- Same file, `a_decimal_input_is_drawn_sequentially`: a command with a `Decimal` input is in the
  explored set, and its drawn values satisfy the input's declared scale.
- The TypeScript explorer agrees on all three (`ts/explore.test.ts` or the parity test the
  repository already runs for the explorers).

## Scope

- cited: `crates/verify/ess-conformance/src/go/explore.go` (`exploreResolveAs`, the model update
  after `explorePerform`); `crates/verify/ess-conformance/tests/explore.rs`.
- inferred: `crates/verify/ess-conformance/src/ts/explore.ts` (same two places); the Decimal
  comparison in `exploreValueAgrees` (`explore.go:1313`) for a decimal echoed in a payload;
  `docs/design/set-effects-over-filtered-instances.md` (the effect's meaning the model follows);
  CHANGELOG `[Unreleased]` "Fixed".

## Plan

- depends_on: `story:go-explorer-draws-read-only-commands` (same files, `go/explore.go` and
  `ts/explore.ts`; lands after it).
- du estimate: 8G (ess-conformance `--test explore`, Go cache under `target/go-build`).
- adversary: yes (the model judges every target; a set-valued effect touches many rows).
