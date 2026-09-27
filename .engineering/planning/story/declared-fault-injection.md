---
format: aep.planning-md/2
id: story:declared-fault-injection
kind: story
status: draft
title: Faults the specification declares are injected during concurrent runs
owner: ess
relations:
- depends_on: story:session-and-eventual-view-checks
- depends_on: story:explorer-takes-external-branches
- depends_on: story:concurrent-explorer-runner
- decomposes: epic:concurrent-history-conformance
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-conformance/src/faulty.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/faults.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures
revision: 4
---
# Story: faults the specification declares are injected during concurrent runs

## Outcome

The concurrent explorer injects only faults the specification declares:

- a second delivery for every `delivery: at_least_once` binding;
- a client retry for every command declaring `replays`;
- a delayed or unanswered `external:` branch, recorded as `Indeterminate`.

Each injection is part of the seed. The report lists how many of each were injected and which
declared branches they reached.

## Acceptance

- New `faulty.rs` row `DoubleApplyOnRedelivery`: found by the concurrent explorer with injection,
  missed by the same seeds without injection.
- New `faulty.rs` row `RetryCreatesSecondEntity`: found by the concurrent explorer with injection,
  missed by the same seeds without injection.
- An undeclared fault (redelivery of a binding declaring no delivery guarantee) is never injected.
  A test asserts its count is 0.

## Not covered

Target restart between operations: no specification, realization or `ConformanceTarget` method
declares restart support today (`src/target.rs`), so injecting it would be an undeclared fault.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (explorers and fault matrix) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go` — cited (embedded by `src/go/mod.rs:147`)
- **Files:** `crates/verify/ess-conformance/src/ts/explore.ts` — cited
- **Files:** `crates/verify/ess-conformance/src/target.rs` (`redeliver_event` `:225`, `ExternalOutcomeControl` `:874`) — cited
- **Files:** `crates/verify/ess-conformance/src/faulty.rs` (rows `DoubleApplyOnRedelivery`, `RetryCreatesSecondEntity`) — cited
- **Also likely:** `tests/faults.rs`, `tests/explore.rs`, `tests/fixtures/`, `src/reference.rs` — inferred
- **Confidence:** medium — the concurrent explorer does not exist yet
- **Would collide with:** `story:concurrent-explorer-runner` (both explorer files) and `story:session-and-eventual-view-checks` (`faulty.rs`) — both are dependencies, so these run in sequence
- **Open:** no Rust concurrent explorer is scoped anywhere, but the acceptance says `faulty.rs` rows are "found by the concurrent explorer"; no existing symbol is the retry seam for a `replays` command (`explore.go:601`, `explore.ts:459` only exclude it)
