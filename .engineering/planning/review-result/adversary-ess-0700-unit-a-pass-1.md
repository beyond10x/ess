---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-a-pass-1
kind: review-result
status: active
title: Adversary pass 1, unit A (identity guard beside a row set)
relations:
- reviews: story:an-identity-guard-beside-a-row-set-validates
revision: 1
---
## Adversary pass 1, unit A (ess-0700 wave)

Worktree base d19ce4314, implementor's changes uncommitted; adversary added
`tests/adversary_ess0700a_identity_beside_row_set.rs` under ess-domain, ess-entity-runtime and
ess-conformance (cases 40→48, red 4).

Runs: ess-domain `row_sets` 12 passed; ess-domain adversary 1 passed, 1 failed; ess-entity-runtime
adversary 1 passed; ess-conformance `row_sets` 19 passed; `precedence_plan_answers` 9 passed;
ess-conformance adversary 2 passed, 3 failed.

Green guards kept: Entity Runtime lowers the fixture with a component to `RelatedGuardUnsupported`
and `RowSetUnsupported`; an accepting row set above a named-row refusal is `conflicting_declaration`;
9 models at `ess/30` and `ess/31` keep their suite bytes at `ess/32`; other declaration-order
variants synthesize and pass on the interpreter.

| location | verdict | what was measured |
|---|---|---|
| crates/specify/ess-domain/src/command/row_set.rs:362 | NEEDS-CHANGE | at ess/32 a when_subject guard beside a named row and a row set is refused naming the admitted when_related |
| crates/verify/ess-conformance/src/synthesize/row_set.rs:2970 | NEEDS-CHANGE | no scenario holds a missing named row beside an occupied source; a target reading the row set first passes |
| crates/verify/ess-conformance/src/synthesize/row_set.rs:2970 | NEEDS-CHANGE | step-4 order between a row-set refusal and a named-row refusal is not tested |
| crates/verify/ess-conformance/src/synthesize/row_set.rs:3500 | NEEDS-CHANGE | two row sets over different named rows: synthesis refuses sink-taken although the model validates |
| crates/specify/ess-domain/tests/row_sets.rs:662 | INFEASIBLE | the tree's story copy predates the store correction; a row-set refusal above exists: false validates by decision |

```findings
- file: crates/specify/ess-domain/src/command/row_set.rs
  line: 362
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: at ess/32 a when_subject guard beside a named row and a row set is refused with a message naming the admitted identity-addressed when_related and a hint to move it
- file: crates/verify/ess-conformance/src/synthesize/row_set.rs
  line: 2970
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: no scenario holds a missing named row beside an occupied source, so a target reading the row set first passes the whole suite
- file: crates/verify/ess-conformance/src/synthesize/row_set.rs
  line: 2970
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: step-4 declaration order between a row-set refusal and a named-row refusal is not tested, while the named-vs-named swap at ess/31 is caught
- file: crates/verify/ess-conformance/src/synthesize/row_set.rs
  line: 3500
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: two row sets over different named rows, synthesis refuses sink-taken although the model validates and the format history says each branch is witnessed
- file: crates/specify/ess-domain/tests/row_sets.rs
  line: 662
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the story Decision the adversary read was the tree's stale copy; the store records that a row-set refusal above exists false validates
```
