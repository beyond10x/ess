---
format: aep.planning-md/1
id: story:review-report-reader-validation
kind: story
status: implemented
title: Validate standalone conformance report claims on read
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance
revision: 7
---
## Finding and source

F03 (P0) from `docs/reviews/2026-09-05-architecture-review.md:217`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Current source: `crates/verify/ess-conformance/src/evidence.rs:18`. Observations are attributed to that review.

## Acceptance

The standalone v1 report reader refuses unknown versions and internally contradictory report claims that currently deserialize successfully.

## Implementation boundary

Validate the documented v1 format, supported suite versions, non-pass counts, listed non-passes and overall status using the actual v1 producer semantics. In v1, scenarios_failed counts non-passes; do not reinterpret it as failures only. Preserve existing valid canonical bytes, including zero-scenario reports as execution summaries with no additional coverage claim. Check Rust and generated Go producers when defining valid cases; only reject contradictions demonstrable from the closed report.

## Validation

Add reader mutation cases for other/99, failed > total, list/count disagreement, passed with non-passes and impossible status combinations; retain passing, failed, error/unsupported/skipped and empty v1 producer fixtures. Exercise the public read boundary and Serde paths actually used by callers; run `cargo test --locked -p ess-conformance` with executed case counts.

## Compatibility and exclusions

No new fields, suite/report format bump, source coverage guarantee or downstream AEP change. The existing skipped-count story and review-conformance-coverage own the versioned migration.

## Scope

Confirmed at implementation head `6b887663736d088307b4f8957de8488740110e6a` from the implementor's confirmation table, final diff and package runner outputs.

- **Primary surface:** `crates/verify/ess-conformance` — cited; reader, Rust/Go producer semantics and suite-format authority all reside in this crate.
- **Implementation:** `src/evidence.rs`, `StandaloneConformanceReport::deserialize` and private claim validation — cited; every public Serde read route passes through the closed wire DTO and the same semantic checks.
- **Tests:** existing `src/evidence.rs` test module plus `tests/report_reader_adversary.rs` — cited; the proposal's inferred test placement is now confirmed. Inline Go bytes preserve the current writer's field order and v1 count semantics.
- **Documents:** no separate package document edit — cited; the implementor confirmed that this is an existing-format reader repair. Engineering reports and planning remain coordinator-owned.
- **Scope correction:** no additional package was needed and no prior inferred package/document boundary proved wrong — cited; the generator, CLI and downstream workflow adapter were untouched.
- **Limits:** serialization field order, valid v1 canonical bytes, opaque identity strings, zero-scenario execution summaries and the union of Rust/Go non-pass semantics are preserved — cited.
- **Confidence:** high — cited; final committed source and tests establish the scope.
- **Would collide with:** any unit touching `crates/verify/ess-conformance` — inferred; retain the existing crate-directory typed scope for future computation.

## Scoping decisions

Validate every public deserialization route used by callers. Because v1 persists no producer identity, accept the union of valid Rust and Go producer cases and refuse only contradictions derivable from the document; do not guess the producer from one status token.