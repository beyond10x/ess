---
format: aep.planning-md/3
id: story:a-killed-childs-outcome-says-which-signal-ended-it
kind: story
status: implemented
title: A killed child's outcome says which signal ended it
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/recovery/process.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/execution_recovery.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/fake_recovery.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T07:30:24Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-03T07:30:24Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-08T09:55:07Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}}
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

## Integrated reviewed handoff

The source owner's active story and independently reviewed commit0e7fb770827c10038985105c8f4997ea74545043 were transferred in the final handoff. Exact main e68684ef matches the patch parent on all three touched files, and the patch applied unchanged as d2fdeb4ca on the grouped release carrier. The diagnostic preserves actual Unix signal, existing timeout/indeterminate decisions, absent non-Unix signal, and complete assertion output. No serialized journal format or retry disposition changes.

The original consumer-child-signal-pass1 review approves the exact patch with no findings. Retained evidence reports same-test red-to-green, execution_recovery120 passed/0 failed, formatting and strict lint. These are prior source-owner observations; final combined release gates remain required. No generic restart capability is claimed. The receiving coordinator preserves original source and review evidence until publication.
