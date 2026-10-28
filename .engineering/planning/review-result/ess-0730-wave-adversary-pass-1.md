---
format: aep.planning-md/3
id: review-result:ess-0730-wave-adversary-pass-1
kind: review-result
status: active
title: The 0.73.0 adopter wave adversary pass 1
summary: 'NEEDS-CHANGE then fixed: F1 to F3 fixed in 3daa68993; an older outcome-arrangement gap filed for a later wave'
relations:
- reviews: story:a-set-effect-binding-refusal-names-each-cause
- reviews: story:a-binding-no-step-triggers-leaves-a-row-set-arrangeable
- reviews: story:a-predicate-refusal-policy-is-forced-through-an-arranged-row
- reviews: story:affects-witness-with-the-adopters-remaining-condition
revision: 1
---
## Adversary pass 1, the 0.73.0 adopter wave (one pass over the integrated branch)

Integration tree ess-0730-int at f887a894b: U4 85ec20e51, U2 4aa19000a, U3 e6207f59e and U1
a5c9f602f merged, with the tables re-pinned. The adversary added
`crates/verify/ess-conformance/tests/adversary_0730_pass1.rs`, with six cases (three red) and
fixtures under `tests/fixtures/adversary-0730/`.

Attacked and held, and kept as passing cases:
- a bounded retry of a predicate refusal counts every attempt (U3);
- a binding set off through a two-binding chain leaves the selector unfollowed (U2);
- two creators with opposite predicates leave the reset arranged (U4).

Also tried without a finding: chain variants (U2), a refused row reached through an update and a
predicate comparing a stored field to an input (U3), and a predicate over two stored fields or a
negation no creator input satisfies (U4).

Outcome:
- F1 to F3 are fixed in 3daa68993, and the six cases are green. The targeted gate passes (lib plus
  38 binaries) with the tables, and no pinned row moves. Every reduced reproduction keeps its count.
- F2's outcome scenario `Give/outcome/not-flagged` already failed on the released 0.72.0. Its
  cause is in the shared related-guard arrangement, which this wave did not touch. It is pinned in
  the test as the one known failure and filed as
  story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off.

```findings
- {file: crates/verify/ess-conformance/src/synthesize/binding_effects.rs, line: 1865, category: acceptance, severity: blocker, verdict: NEEDS-CHANGE, origin: introduced, message: "a bare-path trigger input guard was not counted by keeps_branch_sent, so the trigger was re-sent naming a generated row and took its refusal; fixed in 3daa68993"}
- {file: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs, line: 299, category: boundary, severity: warning, verdict: NEEDS-CHANGE, origin: undecided, message: "the arranged refused row was undone by another binding its own arrangement sets off; the refusal-policy cell is fixed in 3daa68993, the older outcome-scenario part is filed as story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off"}
- {file: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs, line: 339, category: contract-drift, severity: note, verdict: NEEDS-CHANGE, origin: introduced, message: "the unbounded-retry path reported RelatedRefusalUnarranged without trying an arrangement; fixed in 3daa68993"}
```
