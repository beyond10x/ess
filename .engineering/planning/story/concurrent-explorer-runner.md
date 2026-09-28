---
format: aep.planning-md/3
id: story:concurrent-explorer-runner
kind: story
status: draft
title: The Go and TypeScript explorers record concurrent histories and hand them to ess
owner: ess
tags:
- priority-high
relations:
- depends_on: story:linearizability-checker-over-the-interpreter
- serves: vision:O2
- supersedes: story:go-explorer-concurrent-porcupine
- depends_on: story:outcome-shapes-beyond-ess-14
- depends_on: story:concurrent-history-format
- decomposes: epic:concurrent-history-conformance
revision: 1
---
# Story: the Go and TypeScript explorers record concurrent histories and hand them to `ess`

## Outcome

The emitted Go and TypeScript explorers (`src/go/explore.go`, `src/ts/explore.ts`) gain a
concurrent mode. It runs a sequential prefix, then 2–4 clients issuing commands at once against the
adopter's target. Each call's invoke and return instants are recorded into `ess-history/1`, and an
operation with no answer is `Indeterminate`.

The explorers do not decide linearizability themselves. They run
`ess verify conform check-history` on the history they wrote and report its verdict. For a violation
they also report its shrunk history. The checker, the model and the shrinking exist once, in Rust,
in the `ess` binary the adopter already installed to generate the package. The emitted packages add
no module or package dependency.

This story supersedes `story:go-explorer-concurrent-porcupine`, which put a separate checker in the
Go package. That design was replaced on 2026-09-27 by one checker in Rust.

## Acceptance

- Against the `lost-update` mutant of the Go explore fixture target, at least one of 200 seeds
  produces a history that `check-history` reports as a violation.
- The same holds for the `lost-update` mutant of the TypeScript explore fixture target.
- Against the unmutated targets, the same 200 seeds give 0 violations and 0 `Unknown`.
- The same seed produces the same history bytes on two runs, in each language.
- The emitted Go module names no module outside the Go standard library.
- The emitted TypeScript package declares no dependency outside the Node standard library.
- When `ess` is not on `PATH`, the concurrent mode fails with a message naming `ess`. It is never
  skipped.

## Scope

`src/go/explore.go`, `src/ts/explore.ts`, the Go and TS explore fixture targets, `tests/explore.rs`.
