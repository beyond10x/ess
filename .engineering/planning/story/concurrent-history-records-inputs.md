---
format: aep.planning-md/3
id: story:concurrent-history-records-inputs
kind: story
status: draft
title: A concurrent history records each command's input
summary: ess-history records no inputs, so a Linearizable verdict can be wrong for an input-dependent fault
relations:
- serves: vision:O2
- informed_by: review-result:linearizability-checker-over-the-interpreter-adversary-pass-2
- depends_on: story:concurrent-history-format
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/explore_concurrent.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests/history_model.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/ess_history_schema_adversary.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/faulty.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/history.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/linearize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/record.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/recorded.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/sessions.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/history_format.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/linearizability.rs
- confidence: inferred
  path: models/concurrent-history/domains/history.yaml
- confidence: inferred
  path: schemas/ess-history.schema.json
revision: 4
---
# Story: a concurrent history records each command's input

## Outcome

`ess-history/1` records the outcome of each operation but not the input the client sent. The
linearizability checker (`crates/verify/ess-conformance/src/linearize.rs`) therefore explains a step
with any input `witness::candidates` builds for the command. Two consequences, from adversary pass 2
of `story:linearizability-checker-over-the-interpreter` (2026-09-28,
`review-result:linearizability-checker-over-the-interpreter-adversary-pass-2`, finding 4, marked
pre-existing):

- a `Linearizable` verdict can be wrong for a fault that depends on the input — for example a target
  that answers `accepted` to an invalid amount is explained by some valid candidate amount
  (inferred from the code, not run);
- a `Violation` can be wrong for a history that only an input outside the candidates explains.

A format revision that records the input per operation (optional, so existing histories still read)
lets the checker replay the exact input.

## Acceptance

- `ess-history` gains an optional per-operation input; histories without it still read.
- A faulty target that accepts an input the model refuses produces a history the checker reports as
  `Violation` when inputs are recorded, and the same history without inputs still reads.
- The Go and TypeScript writers record the input, and their bytes stay equal for one seed.

## Scope

Derived 2026-09-28 by `aep:story-scoper` against the integration tree plus the unmerged
`story:declared-fault-injection` changes. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (history format and linearizability checker) — cited
- **Files:** `crates/verify/ess-conformance/src/history.rs` (`Operation`, wire struct, `read`) — cited
- **Files:** `crates/verify/ess-conformance/src/linearize.rs` (`candidates`, the input loop) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go`, `src/ts/explore.ts` (history writers) — cited
- **Files:** `crates/edge/ess-cli/tests/explore_concurrent.rs` (Go/TS equal-bytes test) — cited
- **Files:** `models/concurrent-history/domains/history.yaml`, `schemas/ess-history.schema.json`, `crates/edge/ess-xtask/tests/history_model.rs` — inferred, the three move together
- **Also likely:** `src/record.rs`, `src/sessions.rs` (recorders build the request), `src/recorded.rs` (importer adapter field), `src/faulty.rs`, `tests/linearizability.rs`, `tests/history_format.rs`, `crates/specify/ess-domain/tests/ess_history_schema_adversary.rs` — inferred
- **Confidence:** medium
- **Would collide with:** every unit touching the `ess-history` operation shape, `linearize.rs`, the Go/TS writers or `explore_concurrent.rs` — including `story:declared-fault-injection` and `story:explorers-record-view-reads`
- **Open:** the input's wire type (the model has no map type; a canonical-JSON string like `rows` is the smallest change); whether an optional field keeps `ess-history/1` (the reader and schema are closed, so a new history is refused by an older reader)
