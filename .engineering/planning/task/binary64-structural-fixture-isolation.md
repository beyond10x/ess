---
format: aep.planning-md/1
id: task:binary64-structural-fixture-isolation
kind: task
status: implemented
title: Allocate an isolated target-directory fixture for Binary64 publication tests
relations:
- derived_from: story:binary64-structural-codecs
- serves: vision:O2
revision: 4
---
## Problem

The main CI run https://github.com/beyond10x/ess/actions/runs/34333417466 failed at
`crates/edge/ess-cli/tests/binary64_structural_adversary.rs:70`: creating
`types-report.json` returned `AlreadyExists`. The fixture root at lines 25–28 uses
only the process ID beneath `CARGO_TARGET_TMPDIR`; cached output can survive a
runner and collide when that PID is reused.

## Scope

Repair allocation and lifetime in `crates/edge/ess-cli/tests/binary64_structural_adversary.rs`.
Use a unique temporary directory beneath `CARGO_TARGET_TMPDIR`, held for the entire
test, with a test-only allocation dependency in `crates/edge/ess-cli/Cargo.toml`
and its root lockfile if needed. Preserve every publication assertion and the
target filesystem boundary. Keep the exact Docs System input and generated passive
publication controls unchanged.

## Acceptance

Reproduce the old setup failure with a pre-existing PID-named report directory.
The repaired test passes with that stale directory still present, uses a separately
allocated root, and retains its fixture through every Rust and Go assertion.
Run the complete `task check` gate with native linker flags scoped to the native
target, then publish an admitted bot candidate and integrate when required checks
are green. Record actual evidence before marking this task implemented.
