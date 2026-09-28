---
format: aep.planning-md/3
id: review-result:adversary-5w-overlap-pass-1
kind: review-result
status: active
title: Adversary pass 1, overlap (the-5-waves)
relations:
- reviews: story:input-guard-overlap-precedence
revision: 1
---
Adversary pass 1 against story:input-guard-overlap-precedence (#178), aep:adversary, 2026-09-27, the-5-waves wave 2.

verdict: NEEDS-CHANGE
cases: added 6, red 3 (executed 883→889)
origin: introduced 2, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_overlap_pass1.rs`. Red: a when_subject accepting branch never gets its overlap point; a branch fully claimed by a refusal is refused naming only its own guard; beside a subject-fact branch the identity-guarded refusal is required for a non-empty id. Green: Decimal, single-boundary and enum overlaps.

Coordinator routing: all three to the implementor (the undecided one kept in this unit: same model shape).

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":6917,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"overlap_inputs pairs a refusal only with ConstructInput siblings, so a when_subject accepting branch is never sent its overlap point"},
 {"file":"crates/verify/ess-conformance/src/synthesize.rs","line":3529,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"an accepting branch fully claimed by a sibling refusal is refused as no candidate satisfies its own guard, and the shadowing refusal is never named"},
 {"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":108,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"beside a subject-fact branch, the identity-guarded refusal is required for the arranged non-empty ticket id"}]
```
