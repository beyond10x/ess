---
format: aep.planning-md/3
id: review-result:adversary-5w-search-pass-1
kind: review-result
status: active
title: Adversary pass 1, search (the-5-waves)
relations:
- reviews: story:witness-search-beyond-64-candidates
revision: 1
---
Adversary pass 1 against story:witness-search-beyond-64-candidates, aep:adversary, 2026-09-27, the-5-waves wave 2.

verdict: NEEDS-CHANGE
cases: added 5, red 2 (executed 880→885)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_search_pass1.rs`. Red: a seven-field conjunction (satisfying assignment is the 128th directed candidate); a struct-member comparison beside three literal paths, decided by input name order. Green: four/five-field nested connectives with mutants killed, mixed-type six-field conjunctions, committed suites byte-identical.

Coordinator routing: all three to the implementor.

```findings
[{"file": "website/docs/reference/predicates.md", "line": 736, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the row promises every true/false combination past 64 candidates, but a seven-field conjunction only satisfying assignment is the 128th directed candidate"},
 {"file": "docs/design/string-predicate-operators.md", "line": 291, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "own-literal paths are still cut off when a struct-member comparison is kept whole beside them; the result depends on input name order"},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 510, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "a branch first witnessed in the dropped tail of the walk gets a different input, so a model outside the repository gets a different generated suite"}]
```
