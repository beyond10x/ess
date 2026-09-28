---
format: aep.planning-md/2
id: story:concurrent-history-records-inputs
kind: story
status: draft
title: A concurrent history records each command's input
summary: ess-history records no inputs, so a Linearizable verdict can be wrong for an input-dependent fault
relations:
- serves: vision:O2
- informed_by: review-result:linearizability-checker-over-the-interpreter-adversary-pass-2
- depends_on: story:concurrent-history-format
revision: 1
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

Not scoped yet.
