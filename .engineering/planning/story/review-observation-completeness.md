---
format: aep.planning-md/1
id: story:review-observation-completeness
kind: story
status: draft
title: Preserve observation scope and selector uncertainty
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-secret-sanitization
- depends_on: story:review-infra-ir-invariants
scope:
- confidence: cited
  path: crates/infra/ess-kubernetes
- confidence: inferred
  path: crates/infra/infra-analyze
- confidence: inferred
  path: crates/infra/infra-compiler
- confidence: cited
  path: crates/infra/infra-domain
- confidence: inferred
  path: crates/infra/infra-project
- confidence: inferred
  path: crates/infra/infra-spec
- confidence: inferred
  path: docs/design/review-observation-completeness.md
revision: 8
---
## Finding and source

F06 (P1) from `docs/reviews/2026-09-05-architecture-review.md:286`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/infra/infra-domain/src/raw.rs:159`, `crates/infra/ess-kubernetes/src/lib.rs:70`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Unsupported selectors or a reduced collection scope produce qualified unknown/refused infrastructure conclusions instead of complete match-all or absence claims.

## Implementation boundary

First design typed scope/per-kind completeness and unsupported-term accounting against the existing observation/IR formats, with strict old-reader fixtures. Preserve selector semantics or refuse their use; never reduce an unrecognized expression-only selector to a broad empty conjunction. Retry only when scope is preserved or visibly acknowledged; ensure scope qualifications reach analysis, drift and projection.

## Validation

Fake kubectl fixtures distinguish permission failure, namespace fallback and genuinely scope-preserving retries. Selector fixtures cover matchLabels, matchExpressions, mixed and unknown terms. Round-trip incomplete observations and assert consumers retain uncertainty.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No live cluster needed; keep the separate infra bounded context and obtain an explicit format decision before changing persisted fields.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/infra/ess-kubernetes` — cited; owning implementation or documented surface.
- `crates/infra/infra-domain` — cited; owning implementation or documented surface.
- `crates/infra/infra-compiler` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-analyze` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-spec` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-project` — inferred; planned edit surface, verify before dispatch.
- `docs/design/review-observation-completeness.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

