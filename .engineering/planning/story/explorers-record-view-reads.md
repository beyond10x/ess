---
format: aep.planning-md/3
id: story:explorers-record-view-reads
kind: story
status: active
title: The Go and TypeScript explorers record view reads
summary: Concurrent explorers write rows on view reads so Go/TS targets are held to read_your_writes and eventual
relations:
- serves: vision:O2
- depends_on: story:declared-fault-injection
- depends_on: story:session-and-eventual-view-checks
- depends_on: story:concurrent-explorer-runner
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/explore_concurrent.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-concurrent-billing-target.mjs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-concurrent-driver.mjs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-target.mjs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore_concurrent_billing_target.go
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore_concurrent_driver_test.go
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore_target.go
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T13:49:31Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-09-28T13:49:31Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":5}}}
---
# Story: the Go and TypeScript explorers record view reads

## Outcome

The concurrent explorers (`ExploreConcurrent` in `src/go/explore.go`, `exploreConcurrent` in
`src/ts/explore.ts`) record commands only; their histories never carry `rows`. Reported by the
implementors of `story:concurrent-explorer-runner` ("the writers record commands only and never
write `rows`") and `story:declared-fault-injection` ("the Go and TypeScript explorers make no view
reads, so they cannot show `DoubleApplyOnRedelivery`; only the Rust recorder does"), 2026-09-28.

The checker already judges view reads at their declared consistency
(`story:session-and-eventual-view-checks`). The explorers gain view reads: each client may read the
specification's views, and the answer's row identities are written as `rows` on a `Returned` read,
so an adopter's Go or TypeScript target is held to `read_your_writes` and `eventual` the way the Rust
recorder's targets are.

## Acceptance

- Against a `stale-read` mutant of the Go and of the TypeScript explore fixture target, at least one
  of 200 seeds produces a history `check-history` reports as a violation naming the read; the
  unmutated targets give 0 violations over the same seeds.
- A Go history and a TypeScript history from one seed are equal bytes, reads included.
- With injection on, `DoubleApplyOnRedelivery`'s explore-fixture counterpart is caught through a
  view read in both languages, and missed without injection.

## Scope

Derived 2026-09-28 by `aep:story-scoper` against the integration tree plus the unmerged
`story:declared-fault-injection` changes. Every line is **cited** or **inferred**.

- **Primary surface:** the Go and TypeScript explorer sources in `crates/verify/ess-conformance` — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go` (`exploreRecordHistory`, `exploreRecording.complete`, `exploreHistoryBytes`, `exploreOperation`) — cited
- **Files:** `crates/verify/ess-conformance/src/ts/explore.ts` (`exploreConcurrent` and its recording/history-bytes port) — cited
- **Also likely:** `tests/fixtures/explore_target.go`, `tests/fixtures/explore-target.mjs` (a `stale-read` mutant; `explore.yaml` declares `explore.desk.OpenTickets` read_your_writes and `TicketsByItems` eventual) — inferred
- **Also likely:** `tests/fixtures/explore_concurrent_billing_target.go`, `tests/fixtures/explore-concurrent-billing-target.mjs`, `tests/fixtures/explore_concurrent_driver_test.go`, `tests/fixtures/explore-concurrent-driver.mjs` (double-apply counterpart on billing's `at_least_once` binding) — inferred
- **Also likely:** `crates/edge/ess-cli/tests/explore_concurrent.rs` — inferred
- **Reference only:** `src/sessions.rs`, `src/linearize.rs`, `src/history.rs` — cited; the checker and format already carry reads
- **Confidence:** medium
- **Would collide with:** any unit touching the explorers' concurrent section, the concurrent fixtures, or `explore_concurrent.rs` — including `story:declared-fault-injection` (unmerged) and `story:concurrent-history-records-inputs`
- **Byte order:** `rows` is written after `outcome` and before `retry_of`, `outcome` `read`, empty `subject_key`, matching `src/history.rs` field order — inferred
