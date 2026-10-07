---
format: aep.planning-md/3
id: story:creating-an-output-root-locks-only-what-it-creates
kind: story
status: active
title: Creating a new output root takes no exclusive lock on a shared ancestor
refs:
- provider: github
  reference: beyond10x/ess#485
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/output_ownership/filesystem.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T06:43:12Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-07T06:43:12Z", actor: "human:timo", revision: 3}
---
## Outcome

Concurrent `ess generate` runs that each create a different new output root under one shared
directory (a shared `$TMPDIR`) do not refuse each other
(https://github.com/beyond10x/ess/issues/485).

## Finding

`crates/edge/ess-cli/src/output_ownership/filesystem.rs:143-160` (`Locks::acquire`) takes an
exclusive `flock` on the nearest existing ancestor of a root that does not exist yet. Measured on
ess 0.55.0: 40 of 40 concurrent pairs writing `<shared>/x$i/out` and `<shared>/y$i/out` had one
run refused with "output ownership busy at <shared>"; 0 of 40 when `x$i` and `y$i` existed first.

## Acceptance

- The 40-pair reproducer, as a test with N concurrent runs, has no `busy` refusal.
- Two runs that create the **same** new root still exclude each other: one refuses with
  "output ownership busy" naming that root, and nothing is half-written.
- No exclusive lock is taken on a directory the run does not create or own; ancestors keep their
  shared locks, and a reserved or enrolled ancestor is still refused as today.
- The model `models/output-ownership` is checked for anything this changes (locking is runtime
  behaviour the model says it does not cover); the implementor reports what it found.
