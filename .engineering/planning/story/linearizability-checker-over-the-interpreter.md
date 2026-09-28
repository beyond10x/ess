---
format: aep.planning-md/2
id: story:linearizability-checker-over-the-interpreter
kind: story
status: implemented
title: A history is checked for linearizability against the interpreter, and shrunk
owner: ess
tags:
- priority-high
relations:
- serves: vision:O2
- depends_on: story:concurrent-history-format
- decomposes: epic:concurrent-history-conformance
- depends_on: story:interpreted-command-execution
scope:
- confidence: inferred
  path: changes
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests
- confidence: cited
  path: crates/edge/ess-cli/tests/check_history.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: crates/verify/ess-conformance/src/faulty.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/linearize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/record.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/faults.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/register
- confidence: cited
  path: crates/verify/ess-conformance/tests/linearizability.rs
- confidence: inferred
  path: website/docs/guides/verify-conformance.md
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 8
---
# Story: a history is checked for linearizability against the interpreter, and shrunk

## Outcome

`ess verify conform check-history --path <spec> --history <ess-history/1>` searches for an order of
the recorded operations that the Rust interpreter accepts. The search is:

- Wing–Gong–Lowe, partitioned by subject identity (P-compositionality);
- nondeterministic in its step, where an `external:` branch, a generated identity and an eventual
  view each give more than one next state.

The algorithm follows Porcupine (https://github.com/anishathalye/porcupine, MIT), read as a
reference only; no crate or code is taken. This story checks views declared `Current`.

The verdict is `Linearizable`, `Violation`, or `Unknown` when the budget runs out. `Unknown` is
never reported as a pass. For a violation it also reports:

- the longest partial linearization found;
- a shrunk history, the smallest subset of recorded operations that is still a violation (clients,
  then operations, removed one at a time).

This is the only checker in ESS. The Go and TypeScript explorers call it
(`story:concurrent-explorer-runner`).

## Acceptance

- New `faulty.rs` row `LostUpdate` (read-modify-write without a lock on `examples/billing`): a
  recorded two-client history against it is `Violation`.
- A recorded two-client history against the unfaulted `Billing` target is `Linearizable`.
- The shrunk history for the `LostUpdate` violation holds at most 2 clients and 4 operations, and is
  itself a `Violation` when checked again.
- Committed unit histories with known verdicts are judged correctly: a register history that is not
  linearizable (Herlihy & Wing 1990, Fig. 1 style) and one that is.
- Exit codes: 0 linearizable, 1 violation, 3 unknown.
- `ess-conformance` and `ess-cli` gain no new crate dependency; `Cargo.lock` is unchanged by this
  story.

## Scope

Derived 2026-09-27 by `aep:story-scoper` against `472d35fbe`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (new checker module) — cited
- **Files:** `crates/verify/ess-conformance/src/faulty.rs` (new `Fault::LostUpdate`, enum at `:166`, `faulty::billing` at `:368`) — cited
- **Files:** `crates/verify/ess-conformance/tests/faults.rs` (matrix at `:49`) — cited
- **Files:** `crates/edge/ess-cli/src/main.rs` (`enum ConformCommand` `:447`, dispatch near `:3042`, `check-history` exit codes 0/1/3) — cited
- **Files:** `crates/verify/ess-conformance/src/lib.rs` (one `pub mod`, list at `:126-164`) — inferred
- **Files:** new checker file in `ess-conformance/src/` — inferred, name not chosen
- **Also likely:** `crates/verify/ess-conformance/tests/fixtures/` (Herlihy–Wing register histories) — inferred
- **Also likely:** `crates/edge/ess-cli/tests/` (CLI exit-code test) — inferred
- **Also likely:** `src/interpret.rs`, `src/reference.rs` — inferred, read as model and target; changed only if the interpreter lacks a step hook
- **Documents:** `changes/<name>.yaml`, `website/docs/reference/cli.md`, `website/docs/guides/verify-conformance.md` — inferred
- **Confidence:** medium-high
- **Would collide with:** any unit adding a `Fault` row, any `ConformCommand` change, any module added to `ess-conformance/src/lib.rs`
