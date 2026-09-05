---
format: aep.planning-md/1
id: story:review-typed-diagnostics
kind: story
status: draft
title: Carry diagnostic identity independently of rendered wording
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: inferred
  path: crates/specify/ess-domain
- confidence: inferred
  path: docs/design/review-typed-diagnostics.md
revision: 4
---
## Finding and source

F14 (P2) from `docs/reviews/2026-09-05-architecture-review.md:481`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-compiler/src/resolve.rs:565`, `crates/specify/ess-compiler/src/source.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Changing human diagnostic wording no longer changes machine rule identity or the cited construct/source location for the migrated validation paths.

## Implementation boundary

Design typed rule identity, construct reference and source path in the existing diagnostic model, then migrate the heuristic bridge incrementally with an explicit remaining-path inventory. Preserve domain-specific codes and accumulated hints. Establish syntax spans for repeated-name and multi-file cases without inventing a generic semantic error registry.

## Validation

Compare diagnostics before/after wording-only changes; repeated names, nesting and cross-file references retain correct codes/locations. Require a fixture for every migrated rule and list any legacy heuristic path still unsupported.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No blanket claim that all parser spans are exact until every inventoried path is migrated; the story closes only when F14's current family-bridge paths have a typed replacement or explicit unsupported result.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/specify/ess-domain` — inferred; planned edit surface, verify before dispatch.
- `crates/specify/ess-compiler` — cited; owning implementation or documented surface.
- `docs/design/review-typed-diagnostics.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

