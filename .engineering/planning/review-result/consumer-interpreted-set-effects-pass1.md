---
format: aep.planning-md/3
id: review-result:consumer-interpreted-set-effects-pass1
kind: review-result
status: active
title: Set effects independent adversarial review first pass
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
needs-revision

Bounded source review of ess-backlog-one-time-response-20261002/target/backlog-input/interpreted-set-effects.patch, SHA256 a3005406b4c131a9b9822ae6c9723b2ebe1b03fc3abcdd77f626a027c96a9063. All four source hashes matched interpreted-set-effects-source-sha256.txt when reviewed. Own test/build executions: 0. No implementation, test, AEP or remote changes. This report is the only file written by this review.

Frozen patch statistics: four files, 819 insertions and 22 deletions: interpret/execute.rs, interpret/execute/set_effects.rs, tests/interpreted_set_effects.rs and tests/set_effects.rs, all under crates/verify/ess-conformance. Unrelated working-tree changes are outside this review.

One source-level acceptance finding requires correction: set_effects.rs57 evaluates the affected-row filter over every row before lines58–59 exclude the same-entity primary subject. A primary that lacks a field read by this filter can therefore produce Undetermined::Undecidable at select's Truth::Unknown arm, even though the primary is categorically outside the secondary set. The exclusion promised by docs/design/set-effects-over-filtered-instances.md, Secondary effect, and the adopted interpreted-set-effects-design.md item3 must happen before evaluating that row as a candidate.

Concrete reproduction recipe, not executed by this reviewer: reuse tests/interpreted_set_effects.rs MODEL and its admitted missing-team arrangement at lines185–189 (remove Open's `team: input.team` assignment); add a typed team input to Invite; replace its affects filter `team == subject.team` with `team == input.team`; open only the primary row, then invoke Invite with that identity and team `one`. The primary's own on_hold assignment and Invited event are determined, and there are no secondary candidate rows. Expected result is the invited outcome/event and the primary update. Current source instead visits the excluded primary, reads its missing team and returns Undecidable. This is a reachable source arrangement supported by the existing missing-fact test, not an invalid injected Store. The exact new model and outcome remain to be measured by the implementor. Required correction: exclude the same-entity primary by entity plus identity before evaluating its filter; retain cross-entity same-key inclusion and retain refusal for genuinely eligible rows with unknown filter facts. Add the described primary-only control and a companion eligible-unknown refusal control.

No additional concrete finding in the other reviewed boundaries. All selectors use the original Store; subject facts include original identity/state; each entry's writes apply in declaration order to the working clone. The applied-row counter includes equal assignments and excludes from-state-ineligible moves, matching the recorded coordinator decision. Instances and affects cannot coexist under the admitted source grammar, so the internal common counter does not introduce an affects count into an instances outcome. Typed writes, recursive Struct values and InputOrGenerated preserve present false/zero/empty while absent/null use the specified fallback. Every secondary row written is checked for final invariants, primary final invariants are checked by take, and errors return without publishing the cloned Store or event list. This is source assessment, not independent execution proof.

The existing eleven independent wrong-set-effect modes at tests/set_effects.rs437–460 remain unchanged, as do the #288 identity/conjunct mutants. The added actual Interpreter suite requires all14 scenarios Passed, while the new direct regressions cover counts, original snapshots, write ordering, typed fallback, cross-entity equal identities and invariant failure. Those tests do not currently exercise unknown facts on an excluded primary. Author log interpreted-set-effects-final-green.log records 58 passing tests across nine binaries, with zero failures/ignores; that is author evidence, not reviewer execution. This review does not claim complete nontext Store identity support or other separately recorded Interpreter capabilities.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/interpret/execute/set_effects.rs",
    "line": 57,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Same-entity primary exclusion happens after filter evaluation, so an excluded primary with an unwritten filter field makes an otherwise determined affects operation Undecidable; exclude the primary by entity and identity before evaluating candidates, add a primary-only missing-field regression, and preserve refusal for eligible rows with unknown facts and inclusion of cross-entity equal keys. This is a source-derived counterexample; own executions0 and implementor reproduction is pending."
  }
]
```
