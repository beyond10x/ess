---
format: aep.planning-md/3
id: review-result:adversary-5w-leaves-pass-2
kind: review-result
status: active
title: Adversary pass 2, leaves (the-5-waves)
relations:
- reviews: story:nested-struct-per-leaf-comparison
revision: 1
---
Adversary pass 2 against story:nested-struct-per-leaf-comparison (#179), aep:adversary, 2026-09-27, the-5-waves wave 1, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 18, red 2 (executed 800→818)
origin: introduced 0, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_leaves_pass2.rs`. Red: a view row omitting the generated leaf, or holding it at the wrong type, passes. Green: Optional structs, lists and maps of structs, enums, unions, Json, newtypes over structs, several generated leaves over two levels, row-only dotted keys and their format.

Ledger against pass 1: carried 0, new 1, resolved 1. Coordinator decision on the undecided finding: the row asserts presence of the generated leaf; its type where the existing expectation vocabulary allows, otherwise payload only, stated in E5. Final correction round; the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4174, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "the story scope requires presence and type for the generated leaf of a nested sets: mapping, but a view row carries no shape, so a row omitting lead.rank or holding it as text passes"}]
```
