---
format: aep.planning-md/1
id: story:review-openapi-semantic-accounting
kind: story
status: draft
title: Account for every unpreserved OpenAPI constraint
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-openapi
- confidence: inferred
  path: docs/design/review-openapi-accounting.md
revision: 4
---
## Finding and source

F04 (P1) from `docs/reviews/2026-09-05-architecture-review.md:245`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-openapi/src/lib.rs:721`, `crates/generate/ess-openapi/src/lib.rs:859`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Each reviewed OpenAPI constraint that is not preserved produces a durable gap or refusal instead of a zero-gap success.

## Implementation boundary

Track consumed meaning by schema variant and OpenAPI dialect, including integer/number enums, arrays without items, local reference siblings and unresolved targets. Keep import diagnostics and provenance alongside persisted interface output through a designed compatible wrapper or versioned format; do not guess lifecycle entities.

## Validation

Use counterexamples for 3.0 and 3.1 reference semantics, integer enums, arrays without items and dangling refs; assert gaps/refusals both immediately and after writing/reloading the import result. Supported unchanged examples still round-trip.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Adding broad new OpenAPI features is optional; accurate refusal suffices for unsupported cases. Do not silently add fields to an old strict persisted envelope.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/generate/ess-openapi` — cited; owning implementation or documented surface.
- `crates/edge/ess-cli` — inferred; planned edit surface, verify before dispatch.
- `docs/design/review-openapi-accounting.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

