---
format: aep.planning-md/3
id: review-result:interpreted-command-execution-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:interpreted-command-execution
relations:
- reviews: story:interpreted-command-execution
revision: 1
---
# Adversary pass 2 — story:interpreted-command-execution

Dispatched as `aep:adversary` against `ess-chc-interp` after the pass-1 correction, 2026-09-27/28.
Header and findings verbatim.

unit: story:interpreted-command-execution, uncommitted working tree of `impl/interpreted-command-execution` over base 472d35fbe
verdict: NEEDS-CHANGE
cases: executed 797→802, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (`<unit dir>/adversary-pass-2-suite.log`, next to `brief.md`; the build went to the assigned build dir)
needs-coordinator: decide whether the correction (findings 1–3) happens in this unit or moves to `story:linearizability-checker-over-the-interpreter`. `Generated::Given` has no caller outside tests yet; its only intended consumer is that checker.

Cases (tests/interpreted_command_execution_adversary_pass2.rs), all RED on first run: a branch minting
more than was recorded hides the recorded branch; given values reach event fields in the documented
order; a given identity already held never replaces its instance; an ill-typed given value is not
assigned; a published optional generated field does not shift the next value.

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` 797 passed, 5 failed, EXIT=101.

Not broken: replay-test edit is setup only; step de-dup compares the next Store, so distinct outcomes
never merge; branches read Given from the start; CLI `Interpreted` still uses Counter and never
invents a refusal; `act` answers wrong-state before any write.

Coordinator routing: all five `introduced` → same implementor, this unit. Decided: `Given` keyed by
observable slot (created identity, published event field); `sets: {generated}` from the counter; an
unsatisfiable, ill-typed or identity-reusing branch yields no step.

```findings
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 352
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a branch that runs out of Given values fails the whole step, so under Externals::Open a recorded SendEmail/failed history (no value published) is never an allowed outcome"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 133
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Generated::Given documents identity-then-event-fields order, but sets {generated: true} (admitted since ess/14) consumes values before the events and takes the recorded event values"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 461
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "creates with a Given identity the store already holds silently replaces that instance, so a history reusing an identity is accepted as one the model allows"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 704
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a Given value is assigned without checking its declared type, so a non-UUID identity is held and published as a Uuid"
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 700
  category: property
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an optional generated field consumes no Given value, so a published optional value shifts every later value and the recorded event is never among the steps"
```
