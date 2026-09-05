---
format: aep.planning-md/1
id: story:review-composition-contract
kind: story
status: draft
title: State the composition client plan's actual guarantees
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-composition
- confidence: inferred
  path: website/docs
revision: 3
---
## Finding and source

F17 (P2) from `docs/reviews/2026-09-05-architecture-review.md:549`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-composition/src/lib.rs:470`, `crates/specify/ess-composition/src/lib.rs:898`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Composition documentation and examples describe exact surface selection with byte-buffer transport without implying end-to-end typed payload compatibility.

## Implementation boundary

Document operation membership and pinned model identity separately from payload shape/codec compatibility. Demonstrate what the current plan verifies and does not verify. If a concrete typed-client requirement is selected, draft a versioned plan design referencing existing resolved shapes before decomposing its implementation.

## Validation

Trace the worked example to ResolvedService and ClientTransport, with a negative boundary case showing that payload compatibility is not established by membership alone; run composition tests and site-build.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

F17 is an explicit tradeoff, not a broken typed-client implementation; new client generation is not required merely by the review and authority stays out of payloads.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/specify/ess-composition` — cited; owning implementation or documented surface.
- `website/docs` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

