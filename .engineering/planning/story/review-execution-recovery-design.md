---
format: aep.planning-md/1
id: story:review-execution-recovery-design
kind: story
status: draft
title: Specify the recovery contract for finite deployment execution
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-execution-recovery.md
revision: 2
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1613`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A binding design enumerates recoverable outcomes for partial deployment failure and retries instead of treating a caller-supplied desired plan as proof of applied state.

## Implementation boundary

Design only: define the supported finite execution workflow, observed versus desired versus applied claims, unknown state, manual drift, retry/removal behavior and per-release versus multi-release atomicity. First model any newly introduced execution evidence in a validated typed home; do not invent receipt fields in this story. Name fake-executor acceptance vectors and exact source modules the follow-on implementation will need.

## Validation

Review a failure after each external action, interrupted evidence persistence, equal desired plans after manual change and removal retries. Each case must have one stated observable recovery outcome and a test strategy. Record unresolved ownership/cardinality/authority semantics as UNMAPPED rather than guesses.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No continuous controller or live deployment. This design alone does not close F11's execution weakness: decompose the implementation through AEP after its typed contract is validated; the parent keeps that obligation open.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `docs/design/review-execution-recovery.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

