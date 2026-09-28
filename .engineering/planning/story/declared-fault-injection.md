---
format: aep.planning-md/3
id: story:declared-fault-injection
kind: story
status: draft
title: Faults the specification declares are injected during concurrent runs
owner: ess
relations:
- depends_on: story:external-mutation-explorer-and-toolchain
- depends_on: story:session-and-eventual-view-checks
- depends_on: story:concurrent-explorer-runner
- decomposes: epic:concurrent-history-conformance
- serves: vision:O2
revision: 1
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

`src/go/explore.go`, `src/ts/explore.ts`, the target seams for redelivery and external control in
`src/target.rs`, and `faulty.rs`.
