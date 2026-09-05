---
format: aep.planning-md/1
id: story:review-consumer-coverage
kind: story
status: draft
title: Require explicit consumer coverage for model extensions
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-semantic-diff-coverage
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: inferred
  path: docs/design/review-consumer-coverage.md
revision: 4
---
## Finding and source

F16 (P1) from `docs/reviews/2026-09-05-architecture-review.md:522`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `Taskfile.yml:138`, `crates/edge/ess-xtask/src/main.rs:54`, `docs/reviews/2026-09-05-architecture-review.md:532`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The extension gate fails when a new semantic construct or field has no tested support or explicit refusal for an inventoried consumer.

## Implementation boundary

Maintain a typed consumer matrix for validation, IR, references, diff/impact, projections, synthesis and conformance, with links to executable cases or explicit unsupported/refusal evidence. Add a Rust xtask gate that checks the matrix against the actual authoritative model surface; avoid a parallel hand-maintained list silently omitting fields. A coverage record is not a substitute for running its behavioral test.

## Validation

Mutation proof: add a representative semantic field/consumer obligation without coverage and observe failure, then supply supported/refused behavior evidence and pass. Include concrete F01 rows. The existing fuzz story owns general document fuzzing rather than this matrix.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not rewrite passing gates or claim every target supports every construct; unknown is a valid visible matrix entry with owned follow-up work.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/edge/ess-xtask` — cited; owning implementation or documented surface.
- `docs/design/review-consumer-coverage.md` — inferred; planned edit surface, verify before dispatch.
- `Taskfile.yml` — cited; owning implementation or documented surface.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

