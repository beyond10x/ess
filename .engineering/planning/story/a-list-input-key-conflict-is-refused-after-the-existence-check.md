---
format: aep.planning-md/3
id: story:a-list-input-key-conflict-is-refused-after-the-existence-check
kind: story
status: draft
title: A list-input key conflict is refused after the existence check, and a same-key-different-value guard is witnessed
summary: An input refusal always answers before existing_instance:, distinct also refuses equal duplicates, and the nested-exists form validates but gets no scenario (ESS-SYNTH-003) (adopter item 19, 2026-10-22, related to beyond10x/ess#554); same on 0.67.0.
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 19 of the recheck batch, second team's list L16, relayed by the orchestrator;
adopter names kept out). A create command answers an identity a record already carries first (`existing_instance:`), and only
for a new record checks its list input for a key conflict: the same key twice with different values. Equal duplicates are not a
conflict.

Two parts:

1. Order. Under the one precedence rule an input-guarded refusal answers at step 1 and `existing_instance:` at step 2, so a
   key-conflict refusal cannot be declared to answer after the existence check. Needed: a way to state that an input refusal
   answers after the addressed record's existence, or a recorded decision that it never does (the rule is beyond10x/ess#554's;
   this asks the opposite order for one input refusal).
2. "Same key, different value". `distinct` refuses equal duplicates too. A nested `exists` states the need and validates, but
   synthesis finds no witness for it, so the refusal has no scenario.

## Reproduction

Neutral fixture, `format: ess/29`, domain `demo.server`: struct `Entry {key, value}`, entity `Batch`, command
`StoreBatch(batch_id, label, entries: List<Entry>)` with outcomes `exists` (`existing_instance: true`), `conflict` (input
refusal) and `stored` (`creates:`). 0.68.0 and 0.67.0 byte-identical outputs and suites:

| variant | result |
|---|---|
| `exists` above `conflict: {not: {distinct: {in: entries, as: e, by: e.key}}}` | ESS-COMMAND-004 `conflicting_declaration`: "`conflict` answers at step 1 of the precedence order and `exists`, declared above it, at step 2; where both hold, the declaration order and the precedence order disagree" |
| `conflict` above `exists` (control) | valid; 3 scenarios, 3 passed on the interpreter |
| `conflict: {exists: {in: entries, as: a, that: {exists: {in: entries, as: b, that: {all: [a.key == b.key, a.value != b.value]}}}}}`, above `exists` | valid; synthesis ESS-SYNTH-003 "no candidate of the 2 tried satisfies `exists a in entries: (exists b in entries: ((a.key == b.key and a.value != b.value)))`"; 2 scenarios, 2 passed |
| the same nested guard below `exists` | ESS-COMMAND-004 as in the first row |

## Where it lives

- `crates/specify/ess-domain/src/command.rs:3898`: the declaration-order refusal against the precedence plan.
- `crates/verify/ess-conformance/src/synthesize.rs:1607`: the "no candidate of the N tried satisfies" refusal for the nested
  `exists`.
- Related, not the same: `story:refusal-overlap-witnessed-for-distinct-guard` (overlap witness of two input refusals),
  `story:every-guard-combination-validates-under-the-rule` (combinations refused for want of an order, beyond10x/ess#554).

## Regression status

Not a 0.68.0 regression: byte-identical validation output and suites on 0.67.0. Gap (order) and defect (synthesis witness for a
pair over one list). Does not block 0.68.0.
