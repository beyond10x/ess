---
format: aep.planning-md/3
id: story:binding-rewrites-are-followed-for-subject-guards-and-decoy-rows
kind: story
status: draft
title: Synthesis follows binding rewrites for when_subject rows and decoy rows
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard/rewritten.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs
revision: 7
---
## Finding

Found by U2 of wave D350 (out of scope there, from reading). D350 row 6 makes synthesis pass over a
`when_related:` row that a step's binding rewrites (0.81.0, `synthesize/related_guard/rewritten.rs`,
used only from `with_row_where`, `crates/verify/ess-conformance/src/synthesize/related_guard.rs:4816-4936`).
Two neighbours are not covered, and both are still open on 0.92.0:

- `when_subject:` guards, whose rows are arranged in
  `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`, which has no rewrite check.
  Reproduced. Take the fixture `tests/fixtures/adversary-0730/restoring-binding` (binding
  `raise-on-lowered`: `Lowered` invokes `Raise`, which sets `flag: true`) and add:

  ```yaml
  - name: demo.d.Use
    input: [{name: grant_id, type: demo.d.GrantId}]
    outcomes:
      - {name: not-found, unknown_instance: true, error: demo.d.Refused}
      - name: not-flagged
        when_subject: {predicate: flag == false}
        error: demo.d.Refused
      - name: used
        updates: demo.d.Grant
        instance: grant_id
        emits: [demo.d.Used]
        payload: {demo.d.Used: {grant_id: input.grant_id}}
  ```

  On 0.92.0 the `when_related:` twin, `Give/not-flagged`, is refused as `ESS-SYNTH-003` ("... arranged
  through `demo.d.Lower`, which sets off `raise-on-lowered`, and its `demo.d.Raise` writes `flag`
  ..."). `Use/outcome/not-flagged` is synthesized instead: `DefineGrant`, `Lower(grant)`, then
  `Use(grant)`, expecting `not-flagged`. It fails the interpreted target with `ESS-CF-OUTCOME`
  expected `not-flagged`, observed `used`, because the binding raised `flag` before the send
  (`18 scenarios: 17 passed, 1 failed`).
- Decoy rows, which are not checked for binding rewrites (only the selected row, the branch's
  earlier steps and the rows beside it are). From reading: `decoy`
  (`related_guard.rs:4064`) and `row_at` arrange decoys without `rewritten::rewriting`. Not
  reproduced by a run.

## Decisions

- `Use/not-flagged` is refused as `ESS-SYNTH-003`, not sent on another row: in this model `flag` is
  `false` only after `Lower`, which always sets off `raise-on-lowered`, so no row the guard selects
  survives to the send, and the `when_related:` twin on the same model is refused the same way.
- A decoy row a binding rewrites is left out at that witness (`decoy` returns no row, as when the
  stored-row search finds none), not replaced by another search.

## Acceptance

- `cargo test -p ess-conformance --locked --test adversary_d350_u2_pass1`,
  `a_subject_guard_row_a_binding_rewrites_is_passed_over`: on the model above, synthesis writes no
  `demo.d.Use/outcome/not-flagged` and refuses it as `ESS-SYNTH-003`, the text naming `demo.d.Lower`,
  `raise-on-lowered` and `demo.d.Raise`; the interpreted run of the suite has no failed scenario.
- Same target, `a_subject_guard_row_a_binding_removes_is_passed_over`: with `demo.d.Raise`'s outcome
  replaced by one that `deletes: demo.d.Grant`, `Use/outcome/not-flagged` is refused as
  `ESS-SYNTH-003` naming `demo.d.Lower`, `raise-on-lowered` and `demo.d.Raise`, and the interpreted
  run has no failed scenario.
- Same target, `a_subject_guard_row_a_binding_moves_is_passed_over`: with the `Grant` lifecycle given
  a state `Retired` (transition `retire`), `demo.d.Raise` replaced by an outcome that `moves:
  demo.d.Grant.retire`, and `Use` given `{name: retired, when_subject_state: Retired, error:
  demo.d.Refused}` before `not-flagged`, `Use/outcome/not-flagged` is refused as `ESS-SYNTH-003`
  naming the same step, binding and command, and the interpreted run has no failed scenario.
- Same target, `a_decoy_row_a_binding_rewrites_is_not_used`, on a fixture whose `when_related:`
  branch's decoy row is arranged through a step that sets off a binding rewriting a field the guard
  reads: it fails on the base commit (the evidence records the failing run), and after the change
  the scenario arranges no decoy at that witness and the interpreted run has no failed scenario.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, and every row
  it moves is a model with a binding that writes a field a subject guard or a decoy reads.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard/rewritten.rs
- cited: crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs

confidence: medium

Shared surfaces: proposes its `CHANGELOG.md` sentence and re-pins only the suite-bytes rows of models
with such a binding.
