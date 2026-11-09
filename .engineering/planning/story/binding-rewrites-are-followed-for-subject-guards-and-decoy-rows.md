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
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs
revision: 5
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

## Acceptance

- `a_subject_guard_row_a_binding_rewrites_is_passed_over`, on the model above: `Use/not-flagged` is
  sent on a row no binding rewrites, or refused as `ESS-SYNTH-003` naming the step (`Lower`), the
  binding (`raise-on-lowered`) and the command (`Raise`), as `when_related:` is since 0.81.0. No
  synthesized scenario fails on the interpreted target. The same holds where the binding moves the
  subject's state or removes the row.
- `a_decoy_row_a_binding_rewrites_is_not_used`: a red case first shows the gap on a fixture. Then a
  decoy row that a binding rewrites is not used as a decoy, and the suite passes the interpreter.
- Models whose bindings write nothing a subject guard or a decoy reads keep their suite bytes
  (`synthesis_suite_bytes_table`).

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- cited: crates/verify/ess-conformance/src/synthesize/related_guard/rewritten.rs
- inferred: crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs
- inferred: CHANGELOG.md

confidence: medium
