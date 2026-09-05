---
format: aep.planning-md/1
id: story:review-typescript-root-collision
kind: story
status: draft
title: Allocate TypeScript roots and definitions in one symbol space
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/schema-contract
revision: 2
---
## Finding and source

F07 (P1) from `docs/reviews/2026-09-05-architecture-review.md:306`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/schema-contract/src/typescript.rs:66`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A requested TypeScript root that collides with a referenced definition produces valid uniquely named output or a deterministic refusal.

## Implementation boundary

Include caller root names, definition names and reserved names in one target allocation/feasibility check without changing JSON Schema authority or wire-property names.

## Validation

Add colliding root/definition and normalized-name vectors plus a noncollision control; typecheck generated output using the repository's pinned/local TypeScript toolchain in the target gate.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No schema $id namespace decision or Rust synthesis change is required.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/generate/schema-contract` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

