---
format: aep.planning-md/3
id: story:concurrent-history-format
kind: story
status: implemented
title: 'A concurrent history has a format: ess-history/1'
owner: ess
relations:
- serves: vision:O2
- decomposes: epic:concurrent-history-conformance
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests
- confidence: cited
  path: crates/edge/ess-xtask/tests/history_model.rs
- confidence: cited
  path: crates/specify/ess-domain/tests
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-conformance/src/history.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: models/concurrent-history/domains/history.yaml
- confidence: cited
  path: models/concurrent-history/system.yaml
- confidence: cited
  path: schemas
- confidence: cited
  path: schemas/ess-history.schema.json
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T21:48:31Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T21:55:09Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T23:23:18Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}, imported: true}
---
# Story: a concurrent history has a format

## Outcome

`ess-history/1` records a concurrent run: the `spec_digest`, the seed, the client count, and per
operation its client, command, subject identity, invoke instant, return instant and outcome. An
operation with no answer is `Indeterminate`, and its return instant is read as "after every other
operation" (the S2 rule, https://s2.dev/blog/linearizability). Go and TypeScript runners write it;
the Rust checker reads it. The document is the one `models/concurrent-history/` declares.

## Acceptance

- A history whose `spec_digest` differs from the compiled IR is refused with a named diagnostic
  before any check runs.
- An operation marked `Returned` with no return instant is refused by name.
- An operation whose return instant precedes its invoke instant is refused by name.
- The Rust type is held to `models/concurrent-history/` by a test that fails when a declared field
  or enum value is missing from the type, or the type carries one the model does not declare.

Revised 2026-09-27 (wave planning): the line "a history written by the Go runner and one written by
the TypeScript runner from the same seed over `examples/billing` are equal bytes" moved to
`story:concurrent-explorer-runner`, which owns both writers. Reason, from the `story-scoper` report
of 2026-09-27: that line needs seeded concurrent runs, which only the runner produces, and keeping it
here put `src/go/explore.go` and `src/ts/explore.ts` in two stories' scope.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`, revised when the Go/TS equality line
moved to `story:concurrent-explorer-runner`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` — cited ("new format module in `ess-conformance`")
- **Files:** `crates/verify/ess-conformance/src/history.rs` (new) — inferred, the module name is not given
- **Files:** `crates/verify/ess-conformance/src/lib.rs` — inferred, one `mod` line
- **Files:** `schemas/` — cited ("its JSON Schema under `schemas/`")
- **Also likely:** `crates/edge/ess-xtask/src/main.rs` — inferred, if the schema is regenerated like `DOCUMENT_SCHEMA` (line 52)
- **Also likely:** `crates/edge/ess-xtask/tests/` (model-drift test, the `model_enums.rs` pattern) — inferred
- **Also likely:** `crates/verify/ess-conformance/tests/` (a new history test) — inferred
- **Symbols:** `ess_primitives::evidence::SpecDigest` — cited (`ess-conformance/src/evidence.rs:24`)
- **Documents:** `models/concurrent-history/domains/history.yaml` — inferred, it has no format tag for `ess-history/1`
- **Confidence:** medium
- **Would collide with:** anything adding a module to `ess-conformance/src/lib.rs`, and `ess-xtask` schema projection
- **Not here:** the Go and TS writers — they belong to `story:concurrent-explorer-runner`
