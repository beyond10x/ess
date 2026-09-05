---
format: aep.planning-md/1
id: story:review-report-reader-validation
kind: story
status: active
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
revision: 5
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

Derived 2026-09-05 by `aep-drive:story-scoper` from the story and current tree — cited.

- **Primary surface:** `crates/verify/ess-conformance` — cited; the story names its standalone report reader and package-local validation command.
- **Reader and serialization boundary:** `src/evidence.rs:17`, `src/evidence.rs:52` within the primary surface — cited; `StandaloneConformanceReport` derives `Deserialize`, and `from_json` directly delegates to Serde.
- **Symbols:** `StandaloneConformanceReport`, `StandaloneConformanceReport::from_json`, `STANDALONE_REPORT_FORMAT`, `ConformanceReport::standalone`, `report_status` — cited; declared in `src/evidence.rs:11`.
- **Rust producer semantics:** `src/evidence.rs:59`, `src/report.rs:556`, `src/report.rs:587` within the primary surface — cited; counts and lists contain every non-pass, unsupported scenarios make the overall status failed, and error-only reports become inconclusive.
- **Generated Go producer semantics:** `src/go/runtime.go:599` within the primary surface — cited; failed and skipped scenarios both contribute to the non-pass count and list; skipped-only reports are inconclusive.
- **Suite-version authority:** `SuiteFormat::parse` and `SuiteFormat::is_supported` in `src/scenario.rs:374` within the primary surface — cited; syntax parsing and support checking are separate operations.
- **Tests:** extend reader mutation and canonical round-trip coverage beside `src/evidence.rs:180`, with package-local fixtures covering Rust and generated Go output — inferred; exact fixture filenames and placement remain implementation choices.
- **Documents:** no separate document change is required by this story's acceptance — inferred; the work validates the existing format and preserves valid canonical bytes.
- **Additional surfaces:** none established — inferred; producer definitions, version handling and the read boundary all reside in the primary crate.
- **Confidence:** high — cited; the story identifies the defect site, and the current tree places the required reader, producer semantics and existing tests in one crate.
- **Would collide with:** any unit recording `crates/verify/ess-conformance` — inferred; use this single crate-directory token for collision computation, including its tests and fixtures.

## Scoping decisions

Validate every public deserialization route used by callers. Because v1 persists no producer identity, accept the union of valid Rust and Go producer cases and refuse only contradictions derivable from the document; do not guess the producer from one status token.

