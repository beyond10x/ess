---
format: aep.planning-md/1
id: story:review-conformance-coverage
kind: story
status: draft
title: Carry conformance coverage through persisted suites and evidence
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-report-reader-validation
- depends_on: story:a-skipped-scenario-is-not-a-failed-one
- depends_on: story:review-conformance-format-design
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design/review-conformance-coverage.md
revision: 6
---
## Finding and source

F03 (P0) from `docs/reviews/2026-09-05-architecture-review.md:217`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/verify/ess-conformance/src/scenario.rs:1`, `crates/verify/ess-conformance/src/evidence.rs:18`, `crates/edge/ess-cli/src/main.rs:2210`, `crates/edge/ess-cli/src/main.rs:2319`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

After suite serialization and execution, the report reproduces the coverage classification required by the versioned exact-suite contract.

## Implementation boundary

Design the suite/report compatibility contract before coding: account for generated, authored, outside-component and refused obligations and distinguish execution status from source coverage. Strict execution requires explicit acceptance of incomplete coverage. Inventory Rust, generated Go, CLI and downstream readers; publish a migration order and immutable format versions when envelopes/meaning change. The existing skipped-count story owns splitting non-pass categories and must land first under that design.

## Validation

Round-trip complete, partial and zero-generated suites through JSON, YAML, files and Go output; prove strict refusal and explicit incomplete acceptance, exact-suite digest mismatch rejection and old-version behavior. Use an unsupported but type-correct synthesis case so fixing F09 cannot erase the coverage regression.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

The first-wave v1 reader fix is independent; do not treat legacy zero-test evidence as proof of full coverage or retroactively as failed execution. Cross-repository rollout requires an Atlas ADR and separately scoped downstream work before release.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/verify/ess-conformance` — cited; owning implementation or documented surface.
- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- `docs/design/review-conformance-coverage.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.


## Required classification checks

The matrix must detect missing generated/authored/outside/refused accounting after round-trip, language emission that drops that accounting, and evidence bound to a different suite digest. It must distinguish execution status from coverage, require explicit incomplete-coverage acceptance in strict mode, and retain the designed legacy-reader behavior. All component checks are required to satisfy the one versioned-contract classification result.