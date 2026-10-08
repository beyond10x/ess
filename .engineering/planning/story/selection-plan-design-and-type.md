---
format: aep.planning-md/3
id: story:selection-plan-design-and-type
kind: story
status: implemented
title: A command's branches are ordered by one derived selection plan
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir/precedence.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/precedence_plan.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/selection_precedence_table.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/precedence_classification.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/precedence_plan_answers.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/precedence_plan_override.rs
- confidence: cited
  path: docs/design/selection-plan.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T11:44:27Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-07T11:44:27Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-10-07T13:10:56Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":6,"verification":1}}}
---
# Story: A command's branches are ordered by one derived selection plan

## Why

`epic:one-selection-plan`. Before any consumer can read the precedence order, the order has to exist
as data. This story writes the design page and the type, and pins the plan of every repository
command so the consumer stories that follow change no answer.

## What it delivers

1. `docs/design/selection-plan.md`: the plan's phases, mapped one to one onto the six steps of
   "The precedence order" (`docs/design/cross-record-and-stored-field-guards.md`) and the
   compositions that move a branch between them (ess/22 present-related refusals beside
   `wrong_state:`, several related rows, a stored reference, row sets, an input refusal guarded by
   the held state). It settles three decisions and records each with its reason:
   - where the phase classification lives so that `ess-domain` validation can read it
     (`OutcomeCondition` and the outcome's `error`, `subject` and `replays`) as well as the IR
     consumers (`ResolvedCondition`); the Scope's recommendation is a data-free classification in
     `ess-domain` that both crates map into;
   - the type's name (`ess_domain::selection::SelectionPlan` and
     `ess_compiler::ir::ResolvedSelectionPlan` are taken);
   - that the plan is derived on demand and never serialized.
2. The classification in `ess-domain` and the plan type and constructor in `ess-compiler`, with unit
   tests per phase.
3. One test seam every consumer story uses: a scoped, `#[doc(hidden)]` phase-order override in
   `ess-domain`'s classification (for example `with_phase_order(order, || …)` on a thread-local),
   which the classification and every plan constructor honour. A consumer builds its plan inside its
   own entry point (`responding_core`, `lower_command`, the emitters' `Writer`, the synthesis
   searches, `CommandSpec::validate`), so an override the constructor honours is the only seam that
   reaches all of them. The consumer stories' "phases exchanged" tests use it and add no seam of
   their own.
4. A table test over every command of every repository model (the walker in
   `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`, `models`, is the precedent):
   one line per command, its phases and their branches, pinned in a fixture file.

## Acceptance

- `docs/design/selection-plan.md` exists, names the plan type, and names for each of the six
  precedence steps the phase that holds it.
- The classification is reachable from `ess-domain`: a test in `crates/specify/ess-domain/tests/`
  classifies one branch of each `OutcomeCondition` variant, and its match has no wildcard arm.
- The plan constructor is total over every `ResolvedCondition` variant: a match with no wildcard arm,
  so a new variant fails to compile until it is placed.
- The override reaches both sides: a test in `ess-domain` exchanges two phases inside the override
  and the classification follows; a test in `ess-conformance` does the same and a plan built by
  `ess-compiler`'s constructor inside the closure has the exchanged order.
- The pinned table lists every command of every repository model that compiles, and the test fails
  when one command's plan changes.
- The same table pins each repository model's `EssIr::to_canonical_json` digest, written at the
  story's base before any code changes, and every digest is unchanged after.

## Out of scope

Any consumer reading the plan: those are the next four stories.

## Scope

Rewritten 2026-10-07 at wave close from the implementor's confirmation table and the merged diff
(`6cb4b1dd0`). The scoper's original lines are kept below with each correction stated.

### What landed (cited, from the merge)

- `docs/design/selection-plan.md` (new): eight phases over the six precedence steps, the
  classification's home, the name, derived-never-serialized, and where the plan's reading and the
  written precedence pages part.
- `crates/specify/ess-domain/src/command/precedence.rs` (new): `Phase`, `Rank`, `Place`,
  `Composition` (with `upsert` and a private `ReadFirst`), `place`, `order`, and the scoped
  `#[doc(hidden)]` phase-order override; one `pub mod` line in `command.rs`.
- `crates/specify/ess-compiler/src/ir/precedence.rs` (new): `ess_compiler::ir::PrecedencePlan`,
  `PlannedPhase`; `mod` + `pub use` in `ir.rs`.
- Tests: `ess-domain/tests/precedence_classification.rs`,
  `ess-compiler/tests/precedence_plan.rs`, `ess-compiler/tests/selection_precedence_table.rs` with
  `tests/fixtures/selection-precedence-table.tsv` (header `models=196 commands=338`), two fixture
  models (`precedence-upsert-unknown-instance.yaml`, `precedence-row-set-updating-branch.yaml`),
  `ess-conformance/tests/precedence_plan_override.rs`, `precedence_plan_answers.rs`, and the
  adversary files `adversary_selection_plan_u1_pass1.rs` (both crates) and `…_pass2.rs`.

### The scoper's inferred lines, as the implementor found them

| scoper's line | found |
|---|---|
| primary surface `ess-compiler` | confirmed; the classification itself is in `ess-domain` |
| `ir/precedence.rs` beside `mod refusal;` | confirmed |
| `command/precedence.rs`, one `pub mod` in `command.rs:205-221` | confirmed (line 213) |
| table test + `.tsv` fixture | confirmed |
| no `Serialize` field, IR bytes cannot move | confirmed by measurement: every digest equals the base write |
| decision 1, `fn phase(BranchShape, Composition) -> Phase` | **corrected**: `place(..) -> Place {phase, rank}`; a rank places `existing_instance`, `unknown_instance`, `wrong_state` and an unguarded refusal inside their phase |
| decision 4, eight phases | confirmed |
| compile-only walk needs no exemption list | confirmed: 194 models at the time, 138 compile, 56 fail assembly as single files |
| "the precedent pins 193 models" | **corrected**: 194 at base, 196 with the unit's two fixtures |
| stored `via` with `existing_instance` | existence first, as the interpreter; no repository command has it |
| trivially-true or `replays:` refusal | validation refuses `replays:` beside `error:`; an unguarded refusal is placed by `Rank::Unguarded` (in `PresentRelated` on stored-reference and row-set commands, first in `HeldState` elsewhere) |

### Found by the adversary passes and corrected (introduced by the unit)

- an unguarded refusal on stored-reference and row-set commands (C1, C2);
- `unknown_instance:` on a row-set upsert closes `PresentRelated` (R1);
- the row-set `wrong_state:` rule reads "every accepting branch acting on the row moves, and none
  from the state it holds" (R3);
- the table fails on an unpinned model and re-pins from its own header (T).

### Left for later stories

- `related_guard::orders_present_refusals` (validation) does not order a stored reference's
  refusals first when an accepting related branch moves the subject; the interpreter does. No
  repository command reaches it: `story:validation-reads-selection-plan`.
- Multi-file systems outside `examples/` are not in the table (the walker's limit).
