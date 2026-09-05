---
format: aep.planning-md/1
id: story:review-infra-ir-invariants
kind: story
status: draft
title: Keep resolved infrastructure handles valid for the IR lifetime
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: inferred
  path: crates/infra/infra-analyze
- confidence: cited
  path: crates/infra/infra-compiler
- confidence: inferred
  path: crates/infra/infra-project
- confidence: inferred
  path: crates/infra/infra-spec
revision: 6
---
## Finding and source

F02 (P0) from `docs/reviews/2026-09-05-architecture-review.md:188`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/infra/infra-compiler/src/ir.rs:470`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Public consumers cannot invalidate a resolved infrastructure handle by mutating its owning IR collections after compilation.

## Implementation boundary

Make invariant-bearing model collections private and provide read queries or validated transformations required by existing consumers. Inventory every public mutation route, including deserialization. Preserve total compiler-minted lookup behavior; migrate each actual consumer to the read API in the same integration unit.

## Validation

A compile-fail or equivalent public-API test rejects the review's clear-after-resolve mutation; positive lookup/transform tests cover all handle families. Run infra compiler and downstream infra/CLI tests. Confirm no persisted infra-ir bytes change.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This is a deliberate multi-package API migration and cannot join the first small wave; no EssIr/InfraIr merger or new infra-ir version is implied by privacy alone.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/infra/infra-compiler` — cited; owning implementation or documented surface.
- `crates/infra/infra-analyze` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-spec` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-project` — inferred; planned edit surface, verify before dispatch.
- `crates/edge/ess-cli` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

