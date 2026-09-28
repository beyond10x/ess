---
format: aep.planning-md/2
id: story:concurrent-explorer-runner
kind: story
status: implemented
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
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/explore_concurrent.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/explore_package.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/explore.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore-driver.mjs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore-target.mjs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/explore_driver_test.go
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/explore_target.go
revision: 9
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
- A history written by the Go runner and one written by the TypeScript runner from the same seed
  over `examples/billing` are equal bytes (moved here from `story:concurrent-history-format`,
  2026-09-27).
- The emitted Go module names no module outside the Go standard library.
- The emitted TypeScript package declares no dependency outside the Node standard library.
- When `ess` is not on `PATH`, the concurrent mode fails with a message naming `ess`. It is never
  skipped.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance`, the emitted explorer sources and their test harness — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go` — cited
- **Files:** `crates/verify/ess-conformance/src/ts/explore.ts` — cited
- **Files:** `crates/verify/ess-conformance/tests/explore.rs` — cited
- **Files:** `crates/verify/ess-conformance/tests/fixtures/explore_target.go` — cited (gets the `lost-update` mutant)
- **Files:** `crates/verify/ess-conformance/tests/fixtures/explore-target.mjs` — cited (gets the `lost-update` mutant)
- **Symbols:** `ESS_EXPLORE_MUTANT` in both fixture targets — cited
- **Also likely:** `tests/fixtures/explore_driver_test.go`, `tests/fixtures/explore-driver.mjs` — inferred
- **Also likely:** `src/ts/mod.rs` (`manifest()`), `src/go/mod.rs` (`EXPLORE_GO`) — inferred
- **Also likely:** `crates/edge/ess-cli/tests/explore_package.rs` — inferred
- **Confidence:** medium — the story's own paths omitted the crate prefix; corrected here
- **Would collide with:** any unit touching `ess-conformance/src/{go,ts}/explore.*` or `tests/fixtures/explore*`
- **Stale reference:** the epic cites `explore.ts:717` on `be44a3365`; on `472d35fbe` the sequential awaits are around lines 1179–1335
