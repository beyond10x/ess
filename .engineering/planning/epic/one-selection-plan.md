---
format: aep.planning-md/3
id: epic:one-selection-plan
kind: epic
status: draft
title: Command branch selection is one derived plan that every consumer reads
refs:
- provider: github
  reference: beyond10x/ess#470
relations:
- serves: vision:O2
revision: 4
---
# Epic: Command branch selection is one derived plan that every consumer reads

## Why

External review of beyond10x/ess#470, finding 2 (medium-high). The precedence order of a command's
branches is stated once in prose (`docs/design/cross-record-and-stored-field-guards.md`, "The
precedence order") and re-derived as control flow in six places:

| consumer | where the order is derived |
|---|---|
| validation | `crates/specify/ess-domain/src/command.rs` (`validate_held_state_order`, #486), `command/subject_state.rs` (`validate_partition`), `command/related_guard.rs` |
| model interpreter | `crates/verify/ess-conformance/src/interpret/execute.rs` (`responding_core`, `select`, `orders_present_related_refusal`) |
| synthesis | `crates/verify/ess-conformance/src/synthesize.rs` (`sibling_refusals`), `synthesize/subject_fact.rs`, `synthesize/related_guard.rs`, `synthesize/row_set.rs` |
| Entity Runtime lowering | `crates/generate/ess-entity-runtime/src/lib.rs` (the `sort_by_key` over branch categories) |
| Rust and Go emitters | `crates/generate/ess-synth/src/rust/behaviour.rs` (`body`), `go/behaviour.rs`, `determined.rs` |
| mutation | `crates/verify/ess-conformance/src/mutate.rs` (`precedence_sites`, precedence-swap mutants) |
| explorer reference model (Go) | `crates/verify/ess-conformance/src/go/explore.go` (copies `interpret::execute::select` by name, reads `ir.json`) |

beyond10x/ess#486 was two of them disagreeing: the interpreter and the lowering read held-state and
accepting branches in declaration order, the emitters held-state first. #487 closed it with a
validation rule that makes declaration order and precedence agree on every valid command; the
order still has no single executable definition.

## Outcome

Each command has one typed, derived selection plan: its branches grouped into precedence phases,
each phase ordered. The interpreter, the lowering, the emitters, synthesis, mutation and validation
read that plan. A new guard kind is placed in the plan, and no consumer re-derives where it answers.

## Constraints

- Derived, never persisted: `EssIr::to_canonical_json` (`crates/specify/ess-compiler/src/ir.rs`)
  bytes do not change, and no format version is minted (AGENTS.md, "Determinism and formats").
- No behaviour change for a model that validates: every repository model's synthesized suite, the
  byte-pinned code under `generated/` and the lowered Entity Runtime definitions stay identical,
  with two named exceptions, each listed command by command in the PR that makes it:
  - an addition a story names (`story:overlap-witnesses-per-phase`'s overlap sends);
  - a held-state branch declared after an accepting or external branch whose input guards the
    finite prover shows disjoint. #486 still admits that order; the plan moves the held-state branch
    first, so generated or lowered bytes change and no answer does.
- Names already taken: `ess_domain::selection::SelectionPlan` (binding first-occurrence selection)
  and `ess_compiler::ir::ResolvedSelectionPlan`; the plan takes another.
- Validation runs in `ess-domain`, below `ess-compiler`, so a plan built only from
  `ResolvedCommand` is out of its reach. Where the phase classification lives is the first
  story's design decision.

## Waves

| wave | stories |
|---|---|
| 2 | `story:selection-plan-design-and-type` |
| 3 | `story:interpreter-reads-selection-plan`, `story:entity-runtime-lowering-reads-selection-plan`, `story:generated-behaviour-reads-selection-plan` |
| 4 | `story:synthesis-reads-selection-plan`, in units: a full-bytes suite pin first, then the query, then `subject_fact`, `related_guard` and `row_set` in parallel (its Scope) |
| 5 | `story:overlap-witnesses-per-phase` (also owns `mutate.rs` `precedence_sites`), `story:validation-reads-selection-plan` |

Every wave starts from a `main` that contains PR #487 (beyond10x/ess#486): it adds the validation rule wave 5 reads, the `selection_precedence.rs` tests waves 3 cite, and edits `external_beside_held_guard.rs`, which wave 4 pins.

## Out of scope

- Review findings 3 (CI prerequisite archive) and 4 (`generate --check --format`).
- Issues #471, #473, #474, #479 and #480, which the `ess` controller holds.
- The Go explorer model (`src/go/explore.go`): it reads `ir.json`, which carries no plan (the
  first constraint), so it keeps its own copy. Exporting the plan into the explorer's input is a
  later story, once the plan exists.
