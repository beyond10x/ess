---
format: aep.planning-md/2
id: story:session-and-eventual-view-checks
kind: story
status: draft
title: Session and eventual views are checked at their declared strength
owner: ess
relations:
- serves: vision:O2
- depends_on: story:linearizability-checker-over-the-interpreter
- decomposes: epic:concurrent-history-conformance
scope:
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-conformance/src/faulty.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/faults.rs
revision: 6
---
# Story: session and eventual views are checked at their declared strength

## Outcome

The checker holds each view to the consistency it declares:

- `read_your_writes`: no client reads a state older than its own last accepted write. Checked per
  client session, using the session-anomaly vocabulary of Jepsen Elle (https://github.com/jepsen-io/elle).
- `eventual`: after quiescence, every client reads the state the linearized commands produce. Reads
  before quiescence may be any earlier state.

A `Current` view keeps the linearizable check.

## Acceptance

- New `faulty.rs` row `StaleReadUnderReadYourWrites`: `OutstandingInvoices` answers from a lagged
  copy. Its history is a violation naming the client and the read.
- The fault matrix records `StaleReadUnderReadYourWrites` as caught by the history check and not by
  the single-client suite.
- The existing row `StaleReadYourWrites` (`F-VIEW-RACE`, `faulty.rs:243-246`) is still present as a
  separate row and is still caught by the single-client suite (`tests/faults.rs:335`).
- The billing target's eventual `InvoiceById` (`Billing::DEFAULT_LAG`) passes, and a variant that
  never converges fails.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` — cited
- **Files:** `crates/verify/ess-conformance/src/faulty.rs` — cited (new row `StaleReadUnderReadYourWrites`)
- **Files:** `crates/verify/ess-conformance/tests/faults.rs` — inferred, the fault matrix (`:335`)
- **Files:** the checker module from `story:linearizability-checker-over-the-interpreter` — inferred, not in the tree yet
- **Symbols:** `Billing::DEFAULT_LAG`, `Billing::with_lag` — cited (`src/reference.rs:124`, `:136`)
- **Also likely:** `crates/verify/ess-conformance/src/reference.rs` — inferred, for the never-converging variant
- **Also likely:** `crates/specify/ess-domain/src/view.rs` (`Consistency`, `:71`) — inferred, read-only
- **Confidence:** medium — the checker module's path is set by its dependency
- **Would collide with:** any unit touching `faulty.rs`, `tests/faults.rs`, or the checker module
