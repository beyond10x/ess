---
format: aep.planning-md/3
id: story:linearizability-checker-over-the-interpreter
kind: story
status: draft
title: A history is checked for linearizability against the interpreter, and shrunk
owner: ess
tags:
- priority-high
relations:
- serves: vision:O2
- depends_on: story:concurrent-history-format
- decomposes: epic:concurrent-history-conformance
- depends_on: story:interpreted-command-execution
revision: 1
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

New checker module in `ess-conformance`, one `ess verify conform` subcommand in `ess-cli`, one row in
`faulty.rs` and its matrix test.
