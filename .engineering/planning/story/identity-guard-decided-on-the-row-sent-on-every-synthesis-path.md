---
format: aep.planning-md/3
id: story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path
kind: story
status: implemented
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
  path: crates/verify/ess-conformance/tests/identity_guard_on_the_row_sent.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:58Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:59Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-12T22:15:34Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
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

## Decisions

- **Both paths steer the row, none is refused.** `OpenPass` takes `pass_id` as input, so a pass
  `"frozen"` or `"main"` can be opened and every branch here can be witnessed; the two paths get the
  steering `identity_steered` already gives the subject-addressed paths (D343 U3), and
  `ESS-SYNTH-003` stays the answer only where no row the creator can make meets the guard, as it is
  there.
- **F2 is not in this story.** It is a different family (the related-boundary run,
  `synthesize.rs:4068` at the review's commit), inferred and never run, and its acceptance would be
  "reproduce or close"; it needs its own backlog draft with a reproduction.

## Acceptance

- `cargo test -p ess-conformance --locked --test identity_guard_on_the_row_sent`,
  `addressed_row_sends_the_row_its_identity_guard_selects`: on the `complete.yaml` model above it
  fails on the base commit (the evidence records the run); after the change the
  `demo.passes.ClosePass/outcome/frozen` scenario opens a pass `"frozen"` and sends it, and the
  interpreted run of the suite has no failed scenario.
- Same target, `prepare_state_input_sends_the_row_its_identity_guard_selects`: on the `state.yaml`
  model above it fails on the base commit; after the change the scenarios
  `demo.passes.ClosePass/outcome/closed`, `demo.passes.ClosePass/outcome/not-open`,
  `demo.passes.Pass/state/Closed/refuses/demo.passes.ClosePass` and
  `demo.passes.Pass/transition/close/by/demo.passes.ClosePass/closed` each open a pass `"main"` and
  send it, and the interpreted run has no failed scenario.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, and every row it
  moves is a model with an identity guard on a branch reached through `addressed_row` or
  `prepare_state_input`.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs (`addressed_row` :5589, `addressed_held` :5650, `identity_steered` :5744, `reach_in_state` :8615, `prepare_state_input` :8697)
- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs (`onto_identity` :4984, `identity_parts` :5007)
- inferred: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`search_within` :5534)
- inferred: crates/verify/ess-conformance/tests/identity_guard_on_the_row_sent.rs (new)

confidence: high

Shared surfaces: proposes its `CHANGELOG.md` sentence and re-pins only the suite-bytes rows of models
with an identity guard on those two paths.
