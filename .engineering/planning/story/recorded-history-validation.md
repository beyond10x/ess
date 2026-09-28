---
format: aep.planning-md/2
id: story:recorded-history-validation
kind: story
status: implemented
title: A recorded production history is validated against the specification
owner: ess
tags:
- later-milestone
relations:
- depends_on: story:session-and-eventual-view-checks
- decomposes: epic:concurrent-history-conformance
- depends_on: story:linearizability-checker-over-the-interpreter
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/import_history.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/recorded.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/recorded
- confidence: cited
  path: crates/verify/ess-conformance/tests/recorded_history.rs
- confidence: inferred
  path: models/concurrent-history/domains/history.yaml
- confidence: inferred
  path: schemas
- confidence: cited
  path: website/docs/guides/verify-conformance.md
revision: 8
---
# Story: a recorded production history is validated against the specification (later milestone)

## Outcome

An operator converts recorded command/response logs, such as an Eventlog stream, into
`ess-history/1` through a declared adapter. `check-history` then judges the recorded history. A
field the log does not carry is reported as a coverage gap and never guessed. Precedents: AWS P +
PObserve (post-hoc validation of structured service logs) and TLA+ trace validation (SEFM 2024,
arXiv 2404.16075).

## Acceptance

- A recorded history from the billing target with `LostUpdate` active is a violation. The same run
  without the fault passes.
- A log missing return instants is refused, with the missing field named per operation.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (history adapter beside the `ess-history/1` module) — cited
- **Files:** `crates/verify/ess-conformance/src/lib.rs` — inferred
- **Files:** new adapter module in `ess-conformance/src/` — inferred, name not fixed
- **Files:** `crates/edge/ess-cli/src/main.rs` (`check-history` input path, `ConformCommand` `:447`) — cited
- **Symbols:** `LostUpdate` reused as the control, not added — cited (epic Outcome)
- **Also likely:** `crates/verify/ess-conformance/tests/` — inferred
- **Also likely:** `models/concurrent-history/domains/history.yaml`, `schemas/` — inferred, only if the adapter declaration needs a field
- **Confidence:** medium
- **Would collide with:** the history-format and checker modules, and `ConformCommand` in `ess-cli/src/main.rs`
- **Open:** "an Eventlog stream" matches no reader in `ess` today; whether the adapter reads a file format or needs the Eventlog repository is not established
