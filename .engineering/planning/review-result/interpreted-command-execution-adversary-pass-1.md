---
format: aep.planning-md/2
id: review-result:interpreted-command-execution-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:interpreted-command-execution
relations:
- reviews: story:interpreted-command-execution
revision: 1
---
# Adversary pass 1 — story:interpreted-command-execution

Dispatched as `aep:adversary` against `ess-chc-interp` (uncommitted tree on
`impl/interpreted-command-execution`, base `472d35fbe`), 2026-09-27. Header and findings verbatim.

unit: story:interpreted-command-execution, uncommitted working tree of `impl/interpreted-command-execution` over base 472d35fbe
verdict: NEEDS-CHANGE
cases: executed 793→797, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (`<unit scratch>/adversary-suite.log`)
needs-coordinator: decide which story owns non-deterministic generated identities (finding 2): this one or `story:linearizability-checker-over-the-interpreter`

Cases (tests/interpreted_command_execution_adversary.rs): created identity published as generated is
the identity held (RED); determined refusal through two open branches is answered (RED); a sequential
history of the unfaulted reference replays through the step (RED); a step leaving an instance
violating its invariant is not answered (green, mutant-killer for `into:`/`at_rest`).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` 794 passed, 3 failed, EXIT=101.

Not broken: re-pinned tests pin exact sets (stronger); `execute` takes `&Store`, no interior
mutability; CLI digest refusal for missing and foreign `--path`; billing agreement on `sets:`,
payload mappings, rejected-before-wrong-state order.

Coordinator routing: all three `introduced` → same implementor. Finding 2 decided: this story owns
caller-supplied generated values in `execute`.

```findings
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 715
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a created identity whose payload field is declared {generated: true} is minted twice, so the event announces an id the store does not hold and the next command on it gets an undeclared wrong-state"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 378
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "generated identities are one fixed counter value per step, so a sequential history recorded against the unfaulted Billing target is rejected by the step the linearizability checker is to call"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 296
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "identical steps from overlapping branches are not removed, so Interpreted refuses as unsupported an outcome the model determines"
```
