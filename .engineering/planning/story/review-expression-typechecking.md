---
format: aep.planning-md/1
id: story:review-expression-typechecking
kind: story
status: draft
title: Resolve complete expression paths during validation
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
revision: 4
---
## Finding and source

F09 (P1) from `docs/reviews/2026-09-05-architecture-review.md:348`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-domain/src/command.rs:1316`, `crates/verify/ess-conformance/src/input.rs:395`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A guard such as amount.nonexistent on a Decimal is rejected during specification validation with its full typed-path diagnostic.

## Implementation boundary

Move reusable full-path resolution and operand checks to domain/compiler validation and have conformance consume the result, keeping witness search and satisfiability separate. Inventory guards and other existing expression-bearing constructs so one resolver does not diverge across consumers.

## Validation

Reject invalid nested fields and operand types; accept legal nested, optional, collection and parameter paths per the current model. Retain a type-correct expression that a particular witness engine cannot synthesize to prove validation does not promise satisfiability.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not broaden the predicate language or require a universal solver.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/specify/ess-domain` — cited; owning implementation or documented surface.
- `crates/specify/ess-compiler` — inferred; planned edit surface, verify before dispatch.
- `crates/verify/ess-conformance` — cited; owning implementation or documented surface.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

