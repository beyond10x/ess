---
format: aep.planning-md/3
id: review-result:adversary-5w-leaves-pass-1
kind: review-result
status: active
title: Adversary pass 1, leaves (the-5-waves)
relations:
- reviews: story:nested-struct-per-leaf-comparison
revision: 1
---
Adversary pass 1 against story:nested-struct-per-leaf-comparison (#179), aep:adversary, 2026-09-27, the-5-waves wave 1.

verdict: NEEDS-CHANGE
cases: added 3, red 1
origin: introduced 1, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_leaves_shapes.rs`. Red: a leaf read whole from a struct-typed input is stored under a non-scalar dotted path that admission refuses. Green: two nested levels, the second transition source row.

Coordinator routing: to the implementor.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4113, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "collect_leaves stores a whole-struct leaf read from a struct-typed input under a non-scalar dotted path, which leaf_payloads::admit_format refuses, so the synthesizer emits an unadmittable suite"}]
```
