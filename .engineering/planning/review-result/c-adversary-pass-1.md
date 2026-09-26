---
format: aep.planning-md/2
id: review-result:c-adversary-pass-1
kind: review-result
status: active
title: C adversary pass 1 (stored-field guards)
relations:
- reviews: story:stored-field-guards
revision: 1
---
Adversary pass 1 on wave unit C (#75 stored-field guards, ess/9). Tree `ess-wave-20260925-c` on
`09984c9d7` plus the implementor's uncommitted diff.

Header as returned: verdict NEEDS-CHANGE; cases executed 495→508 (ess-conformance), red 4, plus 2
red of 4 added in ess-entity-runtime; origin introduced 3, pre-existing 0, undecided 2.

Coordinator routing: F1, F2, F5 back to the implementor. F4 (undecided) decided as a fix: Timestamp
ordering lowers to `before`/`after`; text ordering is refused at lowering. F3 (undecided) deferred to
`story:count-guards-above-one-are-synthesized`; its cases re-pinned to today's refusal.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":439,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"An implementation that refuses every parcel over 20 kg regardless of service passes the whole generated parcels suite, because the default is witnessed by one row only."},{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":1023,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"around() adds a post-state view observation to ess/6 field-equals models, changing their generated suite (13 to 15 steps) although the design and format docs promise unchanged bytes."},{"file":"crates/verify/ess-conformance/src/witness.rs","category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"A .count ordering over a stored list is refused with ESS-SYNTH-001 and a misleading hint, inherited from the input witness search, which also cannot satisfy count > 1."},{"file":"crates/generate/ess-entity-runtime/src/lib.rs","line":3039,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"Timestamp and text orderings lower to an untyped compare that Entity Runtime answers Unknown, so a stored-field ordering guard fails every call with OutcomeUnobservable."},{"file":"website/docs/guides/write-a-specification.md","category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"No user guide documents when_subject predicate."}]
```
