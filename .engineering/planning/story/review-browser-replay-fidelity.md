---
format: aep.planning-md/1
id: story:review-browser-replay-fidelity
kind: story
status: draft
title: Make browser replay faithful to its declared semantic subset
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design/review-replay-subset.md
revision: 4
---
## Finding and source

F15 (P1) from `docs/reviews/2026-09-05-architecture-review.md:500`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/verify/ess-conformance/src/web.rs:125`, `crates/verify/ess-conformance/src/web.rs:195`, `crates/verify/ess-conformance/assets/player.js:111`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Replay displays the correct supported assignment/view state or an explicit unsupported marker for each F15 counterexample instead of silently computing a plausible state.

## Implementation boundary

Preserve typed literal assignments, apply sets with state moves, retain ordering and parameter filter semantics where supported. Establish an explicit replay subset and visible unknown state/calculations otherwise. Prefer common differential vectors or an existing evaluator; keep replay and implementation conformance distinct.

## Validation

Exercise the generic player itself for sets+move, literal values, view order and parameter filters, plus unsupported semantics; compare projected state with reference vectors. The billing WASM lab alone does not satisfy this acceptance.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No production interpreter or universal evaluator; primitive comparison vectors must follow review-primitive-semantics if that migration has landed.

## Scope

Derived 2026-09-06 by story-scoper against ESS e339f4ce3d5046443ff8e99b0ea72099bc23e410.
Entries distinguish cited surfaces from inferred implementation reservations.

- **Primary surface:** `crates/verify/ess-conformance` — cited; the story names this owner, and replay projection, browser behavior and existing projection tests remain package-local.
- **Producer files and symbols:** `src/web.rs:54` (`emit`), `:75` (`model`), `:108` (`outcome`), `:123` (`subject`), `:143` (`view`), `:181` (`set_source`) within the primary surface — cited; these determine the model information available to replay.
- **Browser files and symbols:** `assets/player.js:56` (`literal`), `:68` (`groupSteps`), `:111` (`applyAct`), `:185` (`parseFilter`), `:204` (`evaluateViews`), `:262` (`mark`), and `assets/index.html:176` (state), `:200` (refusal heading), `:205` (views) within the primary surface — cited; these own the demonstrated F15 behavior and visible limitations.
- **Validation surface:** `src/web.rs:229`, `:256`, `:275` and `tests/synthesis.rs:2231` within the primary surface — cited; existing checks inspect emitted assets or collect canonical artifacts and do not exercise browser semantics.
- **Required new validation:** owner-package generic-player browser cases and independent expected vectors for every F15 counterexample, including reset/back replay and visible unsupported results — inferred; exact filenames depend on the browser harness delivered by coverage work.
- **Binding document:** `docs/design/review-replay-subset.md` — inferred; absent at this baseline and required to choose supported assignment/value/filter/order/parameter semantics, unknown propagation, markers and any replay format migration.
- **Scope limit:** this is the minimum unconditional reservation; a stronger persisted replay representation or changed CLI selection requires refreshed reservations for concrete CLI, format-catalog and public-documentation paths before dispatch — inferred.
- **Confidence:** medium — inferred; defect locations and owners are established, but accepted replay/1 intentionally leaves stronger representation and format consequence to this story.
- **Would collide with:** the conformance package's web emitter, paired-model admission, player assets, generic-browser harness and compatibility fixtures; the same package's primitive-comparison and synthesis-vector work — cited.
- **Sequencing:** refresh source/API assumptions after coverage-writer integration; preserve complete original-byte admission, pairing, selection, coverage/refusal display and historical compatibility fixtures — inferred.
- **Coordinator-owned records:** planning journal, wave selection, integration evidence and shipment records remain outside the implementation reservation — inferred.


## Fresh inspection note

The read-only `count_writer_scope8` report at ESS e339f4ce3d5046443ff8e99b0ea72099bc23e410 confirms F15 remains open. Accepted coverage transport intentionally preserves replay/1 limitations. Assignment literals in `ResolvedPayloadValue::Literal` are String (`ess-compiler/src/ir.rs:744`), not an already typed JSON scalar; conversion metadata, precise subject sources, ranking and filter AST are absent from replay/1. A richer projection requires a separate versioned binding before implementation. Runtime corrections using the existing shape must expose information loss instead of guessing. Refresh reservations and source assumptions after coverage integration. This is preparation, with no source implementation or F15 test result claimed.
