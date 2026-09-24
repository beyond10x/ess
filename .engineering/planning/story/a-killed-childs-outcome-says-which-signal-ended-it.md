---
format: aep.planning-md/2
id: story:a-killed-childs-outcome-says-which-signal-ended-it
kind: story
status: draft
title: A killed child's outcome says which signal ended it
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/recovery/process.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/execution_recovery.rs
revision: 3
---
# A killed child's outcome says which signal ended it

`crates/edge/ess-cli/src/recovery/process.rs:349`. `Outcome.status: None` is produced by two
unrelated events:

- the deadline branch at `:342`, where the harness killed the child for running too long;
- `ExitStatus::code()` returning `None` at `:350`, because the child was killed by a signal — an OOM
  kill, a `SIGTERM` from a parent, anything.

Only `timed_out` separates them, and the assertion that fails does not print it.

So a failure reading `left: None, right: Some(101)` is **unattributable by construction**. It is
equally compatible with a 30-second stall and with the OOM killer taking a child that writes one
small file and calls `process::exit(101)` immediately.

## Why this is worth a story rather than a shrug

`execution_recovery.rs:2528`
`an_effect_before_failure_and_a_lost_acknowledgement_are_both_indeterminate` failed once on an
untouched base during wave 25, passed when re-run alone, and did not recur. The wave-25 unit-2
adversary tried to settle it: **240 runs at 6-way concurrency, 0 failures**, plus green in all three
of its suite runs.

It could not be settled, and the reason is not that the failure is rare. The reason is that the
outcome does not carry enough to tell the two causes apart. A test that cannot say why it failed
teaches the next reader to re-run it, and this session ran the disk to 99% full and hit
`StorageFull` in a different `ess-cli` lane, so a resource-pressure explanation is live rather than
hypothetical.

## Acceptance

An outcome that carries no exit code says which of the two produced it, and the assertion that
compares outcomes prints that distinction when it fails. A future occurrence of this flake is
attributable from its output alone.

## Scope

- `crates/edge/ess-cli/src/recovery/process.rs` — `cited`
- `crates/edge/ess-cli/tests/execution_recovery.rs` — `cited`
