---
format: aep.planning-md/3
id: story:session-and-eventual-view-checks
kind: story
status: draft
title: Session and eventual views are checked at their declared strength
owner: ess
relations:
- serves: vision:O2
- depends_on: story:linearizability-checker-over-the-interpreter
- decomposes: epic:concurrent-history-conformance
revision: 1
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
- The billing target's eventual `InvoiceById` (`Billing::DEFAULT_LAG`) passes, and a variant that
  never converges fails.

## Scope

The checker module from `story:linearizability-checker-over-the-interpreter`, and `faulty.rs`.
