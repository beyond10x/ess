---
format: aep.planning-md/3
id: story:rust-recorder-does-not-lose-a-creation
kind: story
status: draft
title: The Rust recorder does not lose a creation it observed
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/sessions.rs
revision: 4
---
# Story: the Rust recorder does not lose a creation it observed

## Outcome

The Rust session recorder writes a read that shows the row of a creation which never answered the
way the Go and TypeScript explorers do, so a correct Rust target is not reported as a
`FutureRead` violation for it.

## Problem

Handoff from the closed concurrent-history session (item 4): the reads unit
(`story:explorers-record-view-reads`) fixed a lost-creation exposure in the Go and TypeScript
explorers, and its implementor reported the same exposure in the Rust recorder. Confirmed by code
reading at current `main`:

- Go and TypeScript leave out of a read's `rows` one row no written operation names for each
  creation of the view's entity that was invoked before the read returned and never answered
  (`crates/verify/ess-conformance/src/go/explore.go:3583-3587`, `exploreWrittenRows` at `:4097-4103`;
  `crates/verify/ess-conformance/src/ts/explore.ts:3728-3750`, `writtenRows`).
- The Rust recorder writes every row the view answered
  (`crates/verify/ess-conformance/src/sessions.rs:498-505`, `rows` at `:521-533`). A call whose
  completion is `Err` stays `Indeterminate` with an empty `subject_key` (`sessions.rs:461-466`, the
  operation starts `Indeterminate` at `:243`), so if the target executed the creation before the
  error, no operation names its row.
- The checker reports a read showing a row no recorded operation addresses as `FutureRead`
  (`crates/verify/ess-conformance/src/linearize.rs:1804-1807`, module docs `:155-156`).

So a Rust `ConformanceTarget` that creates a row, loses the answer (a transport error, a timeout),
and shows the row to another client's read before or after, gets a `FutureRead` violation it did not
commit. The injected `Unanswered` fault does not trigger it (the target never executes that call,
`sessions.rs:446-452`), and an injected `Delayed` call names its identity (`:471-480`); the exposure
is a target's own error after execution.

## Acceptance

- A test in `crates/verify/ess-conformance/tests/` records, with `sessions::record`, a target whose
  creation executes and then completes with `Err`, interleaved so another client's read of a view
  of that entity shows the row; on current `main` `check-history` reports `FutureRead` for it (red),
  after the change it reports no violation (green).
- The same history shape, written by the Go and the TypeScript explorer from one seed, and by the
  Rust recorder, has equal `rows` on that read.
- A row no operation names and no unanswered creation can account for is still written and still
  reported as `FutureRead`.

## Scope

- cited: `crates/verify/ess-conformance/src/sessions.rs`
- cited: `crates/verify/ess-conformance/src/go/explore.go` (reference behaviour, unchanged)
- cited: `crates/verify/ess-conformance/src/ts/explore.ts` (reference behaviour, unchanged)
- inferred: `crates/verify/ess-conformance/tests/view_consistency.rs` (beside the existing `sessions::record` tests)

confidence: high
