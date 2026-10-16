---
format: aep.planning-md/3
id: story:the-explorer-drives-timestamp-inputs-and-deleting-outcomes
kind: story
status: active
title: The explorer drives Timestamp inputs and deleting outcomes
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T07:01:17Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T07:01:18Z", actor: "human:timo", revision: 3}
---
## Outcome

The generated sequential and concurrent explorers (Go, TypeScript and the interpreter's own) drive
every command whose inputs and subject effects the reference model can decide, and agree on which
those are. Two exclusions that removed whole lifecycles from exploration go: a `Timestamp` input is
drawn by the sequential explorer as the concurrent one already draws it
(https://github.com/beyond10x/ess/issues/582), and an outcome that `deletes` its subject from a
supplied identity is performed and its row removed from the model, so a single-use resource (a
code, a token, a reservation) is explored through its redemption
(https://github.com/beyond10x/ess/issues/583). An exclusion that remains names its reason, and the
sequential and concurrent explorers name the same one for the same command.

## Acceptance

- `the_sequential_explorer_draws_a_timestamp_input`: a model whose creating command takes
  `expiry: Timestamp` and whose redeeming command guards `expiry < now` is explored in Go and
  TypeScript with no exclusion of either command, and both outcomes of the guard are reached.
  Fails before the change.
- `a_deleting_outcome_is_explored_and_its_row_is_gone`: a command that deletes its subject by a
  supplied identity is performed; a later call with the same identity is answered as the model
  answers an unknown instance, in Go and TypeScript. A planted target that keeps the row after the
  delete fails exploration with a shrunk trace. Fails before the change.
- `the_explorers_name_one_reason_for_a_timestamp_input_and_a_delete`: for a command with a
  `Timestamp` input the sequential and concurrent explorers both admit it; for a deleting outcome
  the sequential explorer performs it and the concurrent explorer names its exclusion (a workload
  drawn before any call runs can name a deleted row). Aligning the explorers' other exclusions
  (stored-row conditions, `affects:`, `replays`, no invoking actor) is
  `story:the-sequential-and-concurrent-explorers-exclude-alike`.
- The explorer decides a guard over `now` against a clock (`ExploreOptions.Now`), not against the
  text "now"; a guard `expiry < now` reaches both outcomes in Go and TypeScript.
- Tests are modules of existing test binaries; no new `tests/*.rs` file.
