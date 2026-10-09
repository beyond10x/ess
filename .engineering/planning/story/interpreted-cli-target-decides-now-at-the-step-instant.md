---
format: aep.planning-md/3
id: story:interpreted-cli-target-decides-now-at-the-step-instant
kind: story
status: draft
title: The CLI's interpreted target decides now guards at the step's instant, and names any unsupported reason
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#510
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

`ess verify conform run --target interpreted` decides a command with a current-time (`now`) guard
against the scenario's own clock, the step's `at:`, and a scenario it still cannot decide is
reported with the reason.

## Evidence

https://github.com/beyond10x/ess/issues/510 (ess 0.56.0): 8 of 38 scenarios `unsupported`, authored ones included, with no
reason; with the `now` outcome removed, 37 of 37 pass. The CLI builds the interpreted target with
`Interpreted::for_model` (`crates/edge/ess-cli/src/main.rs`) and never calls
`with_command_clock`, and a target without a clock answers every decision that reaches a `now`
guard as Unsupported (`crates/verify/ess-conformance/src/interpret.rs`, `with_command_clock`).

## Acceptance

- The issue's reproducer passes every scenario against the interpreted target from the CLI.
- The decision instant is the executing step's `at:` (one reading per decision, as
  `occurrence_clock` requires); no host clock is read, so two runs give the same report bytes.
- A scenario the interpreted target still answers as unsupported carries the reason in the text
  and JSON reports, naming the guard.
- A test fails if the CLI target is built without the clock again.

## Scope

`crates/edge/ess-cli/src/main.rs`, `crates/verify/ess-conformance/src/interpret.rs`,
`crates/verify/ess-conformance/src/runner*`; tests in `ess-cli`.

## Design note (no build; read from the tree)

- The runner keeps the timeline's instant in its own `Clock` (`runner.rs`, `trait Clock`,
  `AdvancingClock`); the interpreted target reads a separate caller-owned `CommandClock`
  (`interpret.rs`, `with_command_clock`; `occurrence_clock.rs`, "This crate implements none").
- `SemanticCommandRequest` (`target.rs`) carries no instant, so the step's `at:` does not reach the
  target today.
- Smallest bridge: a `CommandClock` that reads a shared cell the runner writes with the step's
  instant before it sends each command (one write per step, one read per decision), built by the
  CLI next to `Interpreted::for_model`. The scripted clock in
  `crates/verify/ess-conformance/tests/support_occurrence_clock/mod.rs` shows the read-once
  contract the bridge must keep.
- Decide in the unit whether the cell lives in the runner (public API) or in the CLI; a change
  to `SemanticCommandRequest` is the alternative and touches every target.
