---
format: aep.planning-md/3
id: story:unknown-instance-and-input-guard-agree-on-order
kind: story
status: implemented
title: Validation, interpreter and explorer agree on an unknown-instance branch below an input guard
tags:
- adopter-report
- defect
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/held_state_order.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:22Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:23Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:43Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

The validator, the interpreter and the Go explorer agree on which branch answers when an
unknown-instance (`when_related exists: false` or `unknown_instance`) refusal and an input guard
declared above it both hold; a specification whose declaration order disagrees with that
precedence is refused at validation, as ESS-COMMAND-004 already does for held-state branches.

## Evidence

An adopter on ess 0.56.0: the explorer fails a target that checks in declared order, and
`specify validate` accepts either expectation. On `main`, the precedence answers `exists: false`
before input refusals (`docs/design/cross-record-and-stored-field-guards.md:746-748`,
`crates/verify/ess-conformance/src/interpret/execute.rs:516-520`); the declaration-order check
(`conflicting_declaration`) covers held-state and external steps only (design page `:762`).

## Acceptance

- A validation test: an input guard declared above an unknown-instance branch whose guards can
  hold together is refused, naming both branches and the order to declare them in.
- A test over a valid order: the interpreter and the Go explorer expect the same branch.

## Decisions

- Design first: whether the fix is a refusal (as for held state) or an authored precedence; the
  design page is updated before code.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/specify/ess-domain`, command validation of branch order — inferred (the story names the refusal code, not a crate)
- **Files:** `crates/specify/ess-domain/src/command.rs:3095-3104` (which order check runs per format) and `:3902` (`validate_step_order`) — inferred
- **Symbols:** `ConflictingDeclaration`, `validate_step_order`, `STEP_ORDER_DECLARED` (`command.rs:4731`) — inferred
- **Also likely:** `crates/specify/ess-domain/src/command/precedence.rs` (`ONE_RULE`, `:33`), only if a branch moves between phases — inferred
- **Also likely:** `crates/specify/ess-domain/tests/held_state_order.rs` — inferred
- **Documents:** `docs/design/cross-record-and-stored-field-guards.md`, "The precedence order" (`:827`) — cited; the story says it is updated before code
- **Confidence:** medium. The story's first bullet points the wrong way for `unknown_instance`: the design table puts input refusals at step 1 and `unknown_instance:` at step 2 under the one rule, so declaring the input first agrees at every format — inferred
- **Already in the tree:** ess/29+ refuses the reverse order (`command.rs:3902`; `held_state_order.rs:801`, `format_29_refuses_an_input_refusal_declared_after_unknown_instance`); the Go explorer answers input-guarded refusals first (`explore.go:1931-1941`) — cited
- **Remaining gap (inferred, not run):** `when_related: exists: false` against an input refusal: at ess/26 and below `exists: false` answers first, at ess/27-28 input answers first; below ess/29 no check refuses either declaration order for this pair.
- **Would collide with:** any unit editing the step-order validators in `ess-domain/src/command.rs` or the precedence section of the design page — inferred
- **Safety fact:** the ess/29 gate at `command.rs:3095-3104` leaves ess/27 and ess/28 with only the held-state and `before_row:` pairs, so a check added below it cannot change ess/29+ answers — inferred
- **Not established:** stale story cites (design `:746-748`, `:762`; `execute.rs:516-520`); the story's claim that the declaration-order check covers only held-state and external branches is superseded by ess/29 `validate_step_order`.
- **Size:** medium, about 150-200 lines; small (docs only) if the first bullet is retracted; no format version for a refusal gated below `ONE_RULE` — inferred
- **Targeted run:** ess-domain `--test held_state_order`; ess-conformance `--test precedence_order`, `explore_guards_first`, `related_guard_unknown_instance`; ess-cli (domain change) — inferred

## W2 ruling on acceptance bullet 1 (2026-11-03)

The W2 lead rules acceptance bullet 1 impossible as written. It asks validation to refuse an input
guard declared above an unknown-instance branch. Under
`docs/design/cross-record-and-stored-field-guards.md` (precedence) the input refusal answers at
step 1, so that order is the correct one: it validates from `ess/27`, and `ess/29` already refuses
the reverse order for both `unknown_instance:` and `exists: false`. Below `ess/29` either order
validates, as the design page documents; a refusal there would narrow a published format.

Bullet 2 holds for the `unknown_instance:` pair. W2 unit EXP1 (`ab6d56ac01`) pins it with
`crates/specify/ess-domain/tests/held_state_order.rs` (the `ess/29` refusal and the `ess/27`
admitted order) and `crates/verify/ess-conformance/tests/explore_input_beside_unknown.rs`. The
`exists: false` pair cannot be explored, because the explorers leave out any command with a
`when_related:` branch, which is an ESS gap outside this story.

The bullet is reported to the orchestrator for retraction or rewording. The story is not moved to
implemented until the acceptance is settled.

## Resolution

Acceptance bullet 1 is retracted by the orchestrator's D410 (2026-11-03): it contradicts the
precedence section of `docs/design/cross-record-and-stored-field-guards.md`, which `ess/29` already
enforces. The precedence code is unchanged. Bullet 2 is the story's acceptance, pinned by
`crates/specify/ess-domain/tests/held_state_order.rs` and
`crates/verify/ess-conformance/tests/explore_input_beside_unknown.rs` (W2 unit EXP1, `ab6d56ac01`).
The story moves to implemented when wave W2 closes.

## Delivered

W2 (D408), 2026-11-03: bullet 1 retracted by D410 (it contradicts the precedence on the design page, which `ess/29` enforces); bullet 2 is the pinned acceptance. The precedence section of docs/design/cross-record-and-stored-field-guards.md states the order per format and tests pin it; no behaviour changed. Unit ab6d56ac01; merged 3cc17ed596. Known limit: the explorers leave out `when_related:` commands.
