---
format: aep.planning-md/3
id: review-result:adversary-5w-overlap-pass-2
kind: review-result
status: active
title: Adversary pass 2, overlap (the-5-waves)
relations:
- reviews: story:input-guard-overlap-precedence
revision: 1
---
Adversary pass 2 against story:input-guard-overlap-precedence (#178), aep:adversary, 2026-09-27, the-5-waves wave 2, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 4, red 2 (executed 891→895)
origin: introduced 1, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_overlap_pass2.rs`. Red: a shadowed external branch and a shadowed when_subject branch are refused without naming the shadowing refusal. Green: stored-only and enum when_subject branches held to the precedence. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 2, resolved 3. Coordinator routing: both to the final correction round (the undecided one is the same Shadow step); the coordinator verifies the diff.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":3491,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"an external branch fully claimed by a sibling input-guarded refusal is refused as no candidate satisfying its own satisfiable guard, never naming the refusal"},
 {"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":1011,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"a when_subject branch fully claimed by a sibling input-guarded refusal is refused blaming the stored field, never naming the refusal"}]
```
