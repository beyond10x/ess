---
format: aep.planning-md/3
id: story:selection-plan-design-and-type
kind: story
status: active
title: A command's branches are ordered by one derived selection plan
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir/precedence.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: inferred
  path: crates/specify/ess-compiler/tests/selection_precedence_table.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: cited
  path: docs/design/selection-plan.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T11:44:27Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-07T11:44:27Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}}
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

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/specify/ess-compiler` (the plan type over `ResolvedCommand`, plus the pinned table test) — inferred
- **Documents:** `docs/design/selection-plan.md` (new) — cited, story "What it delivers" 1
- **Files:** `crates/specify/ess-compiler/src/ir/precedence.rs` (new), with `mod` + `pub use` in `ir.rs` beside `mod refusal;` (`ir.rs:91-92`) — inferred
- **Files:** `crates/specify/ess-domain/src/command/precedence.rs` (new), with one `pub mod` line in `command.rs:205-221` — inferred
- **Files:** `crates/specify/ess-compiler/tests/selection_precedence_table.rs` + `tests/fixtures/selection-precedence-table.tsv` (new) — inferred
- **Symbols:** `OutcomeCondition` `ess-domain/src/command.rs:523`; `ResolvedCondition` `ess-compiler/src/ir.rs:590`; `ResolvedCommand` `ir.rs:1693`; `EssIr::to_canonical_json` `ir.rs:3228` — cited
- **Names taken:** `ess_domain::selection::SelectionPlan` (`selection.rs:233`), carried as IR field `pub plan:` (`ir.rs:2394`) in `ess_compiler::ir::ResolvedSelectionPlan` (`ir.rs:2392`). The new type is neither `SelectionPlan` nor in a `selection` module — cited
- **Confidence:** medium. Every symbol and consumer site was read; the file placement is a design choice the story leaves open.
- **Would collide with:** any unit editing `ess-domain/src/command.rs` (PR #487 adds 71 lines there), `ess-compiler/src/ir.rs` near `mod refusal`, or the "precedence order" section of `docs/design/cross-record-and-stored-field-guards.md` (#487 edits it) — cited
- **Safety fact:** the plan is a method and no `Serialize` type gains a field, so `to_canonical_json` (`ir.rs:3228-3233`) and `to_compact_json`/`source_digest` (`ir.rs:3240-3260`) cannot change — inferred

### Design decisions for the implementor (inferred unless marked)

1. **Where the classification lives:** in `ess-domain` (`command/precedence.rs`), as a data-free `Phase` enum plus `fn phase(branch: BranchShape, command: Composition) -> Phase`. `ess-compiler` depends on `ess-domain` (`ess-compiler/Cargo.toml:13`), not the other way round. The shape follows `TestStrategy` (`command.rs:783`, computed by `OutcomeCondition::test_strategy` `command.rs:698`, re-used at `resolve.rs:2277`; cited). Each crate maps its own enum into `BranchShape` with no wildcard arm; `ess-compiler` orders branches into the plan.
2. **Not a per-condition method** (cited): the phase depends on the whole command — the branch's `error`/`subject`/`replays` (`execute.rs:1336-1345`), a sibling `WrongState`, several input rows and the format version (`execute.rs:677-690`), a stored `via` (`execute.rs:616`), any `RelatedSet` (`execute.rs:680`). The format version is reachable on both sides: `EssIr::format()` (`ir.rs:3008`), `spec.system().format` (`related_guard.rs:826`).
3. **Derived on demand, never stored** (cited): a constructor over `(&ResolvedCommand, FormatVersion)`, no `Serialize`. Not a `#[serde(skip)]` field (`ir.rs:846`): synthesis clones a command and drops outcomes (`synthesize/related_guard.rs:456-464`), so a cached plan would go stale.
4. **Eight phases:** `InputAbsent` and `Default` sit outside the six steps.

### ResolvedCondition variants → precedence step (14 variants, `ir.rs:590-713`; cited)

- `InputAbsent` → before step 1, its own entry (`execute.rs:356-372`)
- `ExistingInstance` → before step 1 when any `Related`/`RelatedSet` branch exists (`execute.rs:501-511`), else step 3 (`execute.rs:523`). The design says "reading an input" (`cross-record…md:746`); the code includes the stored `via` and `RelatedSet`
- `Related{via: Input, test: Absent}` → step 1, in `exists: false` declaration order (`execute.rs:1139-1179`, `related.rs:38-69`)
- `When` + `error`, no subject or replays, non-trivial guard → step 2 (`refused_by_input`, `execute.rs:1330-1349`); any other `When` + `error` at the head of `select` (`execute.rs:821-830`). Entity Runtime puts every `When` + `error` in category 0 (`ess-entity-runtime/src/lib.rs:1816-1828`)
- `UnknownInstance` → step 3 (`execute.rs:589`, `:1933`); inside `take` it also answers for a selected move (`execute.rs:1760`)
- `SubjectState`, `StateChange`, `SubjectField`, `SubjectPredicate` → step 4; a held-state-guarded input refusal too (design `:791-794`). The interpreter reads steps 4 and 6 in one declaration-order pass (`execute.rs:865-906`); #486 makes that agree
- `WrongState` → step 4, answered inside `take` when the step-6 branch's move does not start from the held state (`execute.rs:1772`, `:1963`)
- `Related{test: Holds}` + `error` → step 5 only under ess/22 and (`WrongState` or several input rows) (`execute.rs:677-690`), or a stored `via` (`execute.rs:616`); otherwise step 6
- `Related{via: Subject, …}` (stored reference) → read after steps 3-4: absent selects nothing, a missing row answers `exists: false`, a present row's refusal at step 5 (`stored_reference`, `execute.rs:1238`)
- `RelatedSet` + `error` → step 5 (`execute.rs:948-957`; `addressed_row` first, `execute.rs:602-606`); an accepting `RelatedSet` → step 6
- `When` without `error`, `External`, `ExternalWhen`, accepting `Related{Holds}` → step 6, declaration order (`execute.rs:907-933`)
- `Otherwise` → after step 6 (`execute.rs:937-943`)

### Pinned table test

- **Reuse:** copy `walk` (`external_beside_held_guard.rs:873-891`) and `models` (`:993-1052`) into an `ess-compiler` test; parse → `Specification::assemble` → `resolve::compile` only, so the `UNAFFORDABLE` exemption (`:1056-1060`) is unnecessary — cited / inferred
- **Re-pin switch:** follow `EXTERNAL_WITNESS_BASE_WRITE` (`:1084`) — cited

**Could not establish:** what happens on a command with both a stored `via` and
`existing_instance:` (interpreter checks existence first, `execute.rs:502-506`; design step 1
covers only an input `via`); whether validation's `!accepting_related_move` condition
(`related_guard.rs:901`) matters for valid models; the table's command count (the precedent pins
193 models); where a trivially-true `when:` refusal, or one with `replays:`, belongs.
