---
format: aep.planning-md/1
id: story:review-output-ownership
kind: story
status: draft
title: Make generated output replacement recoverable and ownership-aware
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-output-containment
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: docs/design/review-output-ownership.md
revision: 3
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:2016`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A repeated generation updates only owned outputs as one recoverable operation while preserving authored files across stale-file retirement and injected failure.

## Implementation boundary

Design ownership and recovery against existing generated artifact paths before implementing staged writes. Define first-run adoption of legacy output, collision refusal, stale generated-file removal, interrupted staging/recovery and preservation of unowned authored additions. Add typed ownership data only after its design and compatibility policy are recorded.

## Validation

Inject mid-write/rename failures, rerun after interrupted staging, remove a formerly generated artifact and keep an unrelated authored file; prove either the previous complete output or recoverably staged new output, never an unexplained mixture.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Input discovery policy is separate; do not delete unknown files or infer ownership from an extension.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- `docs/design/review-output-ownership.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

