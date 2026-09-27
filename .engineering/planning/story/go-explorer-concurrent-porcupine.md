---
format: aep.planning-md/2
id: story:go-explorer-concurrent-porcupine
kind: story
status: archived
title: The Go explorer checks concurrent runs with an in-house linearizability checker
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:concurrent-history-conformance
- serves: vision:O2
revision: 2
---
# Story: the Go explorer checks concurrent runs with an in-house linearizability checker

## Outcome

The emitted Go conformance package's explorer (`src/go/explore.go`) gains a concurrent mode: a
sequential prefix, then 2–4 goroutine clients issuing commands at once. The recorded operations are
checked by a linearizability checker written in this package:

- Wing–Gong–Lowe search with P-compositionality (one sub-history per subject identity);
- a nondeterministic step, where one operation may lead to several next states;
- a `timeout` budget, whose exhaustion is `Unknown` and never a pass.

The algorithm follows Porcupine (https://github.com/anishathalye/porcupine, MIT). It is read as a
reference only; no code or module is taken. The step function is the explorer's existing IR model,
bound to `spec_digest` through `ir.json`, so it is the specification's own model in Go. An
operation with no answer gets a return time after every other operation
(https://s2.dev/blog/linearizability).

This is the first deliverable of the epic. Go adopters get a concurrency check before the Rust
interpreter exists. **The emitted package gains no module requirement.** The history this mode
writes is `ess-history/1` once `story:concurrent-history-format` lands. Until then it is the
explorer's own result document.

## Acceptance

- A `lost-update` mutant in `tests/fixtures/explore_target.go` (read-modify-write without a lock) is
  reported `Illegal` by at least one of 200 seeds.
- The unmutated `explore_target.go` gives 0 `Illegal` results over the same 200 seeds.
- The unmutated `explore_target.go` gives 0 `Unknown` results over the same 200 seeds.
- The checker's unit tests include a register history known to be non-linearizable (Herlihy & Wing
  1990, Fig. 1 style) and one known to be linearizable, each with the expected verdict.
- The emitted `go.mod` and generated sources name no module outside the Go standard library.
  `tests/explore.rs` asserts this.

## Scope

`src/go/explore.go`, a new checker file beside it under `src/go/`, `src/go/mod.rs` (the emitted file
list), `tests/explore.rs`, the Go explore fixture target.
