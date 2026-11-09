---
format: aep.planning-md/3
id: story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path
kind: story
status: draft
title: Synthesis decides an identity guard on the row sent on the stored-field and addressed-row paths
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/
revision: 5
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u3-pass-1`, F3 and F2, out
of scope there): synthesis decides a branch's identity guard on the row it sends on the
subject-addressed paths D343 U3 fixed (`addressed_held` → `identity_steered`), but not on the
held-state path `prepare_state_input` nor on `addressed_row` (row-set accepting branches, complete
input-guarded refusals), which calls `addressed_held` with no input
(`crates/verify/ess-conformance/src/synthesize.rs`). F2 (inferred): a related-boundary run for one
conjunct takes the first answering branch, so under an identity guard it can be answered by a
branch that never reads the related row and isolates nothing.

## Problem

Both paths write scenarios the reference interpreter fails, so a correct implementation fails its
own suite. Reproduced on ess 0.92.0.

`addressed_row`: a pass is opened by `OpenPass(pass_id, note)` (creates `Pass`); `ClosePass(pass_id,
reason)` declares, in order, `frozen` (`when: 'pass_id == "frozen"'`, `error: Frozen`) and `closed`
(`deletes: Pass`, `instance: pass_id`). `frozen` is a complete refusal, so its scenario arranges a
pass and sends it; the pass is created as `pass_id = "pass_id"`, so the guard fails:

```
$ ess verify conform run --path complete.yaml --target interpreted --report-format 2
  failed demo.passes.ClosePass/outcome/frozen
  ClosePass(pass_id = "pass_id", reason = "reason") | expected outcome = frozen | observed outcome = closed
```

`prepare_state_input`: the same entity with states `[Open, Closed]` and transition `close`;
`ClosePass` declares `frozen` (`when: 'pass_id != "main"'`, `error: Frozen`), `closed`
(`when_subject_state: Open`, `moves: Pass.close`) and `not-open` (`when_subject_state: Closed`,
`error: NotOpen`). The input is chosen with `pass_id = "main"`, the row is created as `"pass_id"`
and sent in its place:

```
$ ess verify conform run --path state.yaml --target interpreted --report-format 2
  failed demo.passes.ClosePass/outcome/closed
  failed demo.passes.ClosePass/outcome/not-open
  failed demo.passes.Pass/state/Closed/refuses/demo.passes.ClosePass
  failed demo.passes.Pass/transition/close/by/demo.passes.ClosePass/closed
  ClosePass(pass_id = "pass_id", reason = "reason") | expected outcome = closed | observed outcome = frozen
```

## Acceptance

- Over the two models above, `ess verify conform run --target interpreted` passes every scenario it
  writes: the `frozen` scenario sends a pass created as `"frozen"`, and the `closed` and `not-open`
  scenarios (and the lifecycle scenarios) send a pass created as `"main"`; or the branch is refused
  `ESS-SYNTH-003` naming the guard. A red case per path comes first.
- F2 is reproduced by a run or closed as not reproducible, with the run named.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`addressed_row`, `prepare_state_input`, `addressed_held`, `identity_steered`, `reach_in_state`)
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs (`onto_identity`, `identity_parts`)
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`search_within`)
- inferred: crates/verify/ess-conformance/tests/

confidence: high
