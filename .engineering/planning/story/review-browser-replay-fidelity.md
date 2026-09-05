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
revision: 3
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

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/verify/ess-conformance` — cited; owning implementation or documented surface.
- `docs/design/review-replay-subset.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

