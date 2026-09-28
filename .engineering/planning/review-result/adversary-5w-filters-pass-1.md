---
format: aep.planning-md/3
id: review-result:adversary-5w-filters-pass-1
kind: review-result
status: active
title: Adversary pass 1, filters (the-5-waves)
relations:
- reviews: story:view-filters-witnessed-on-matching-rows
revision: 1
---
Adversary pass 1 against story:view-filters-witnessed-on-matching-rows, aep:adversary, 2026-09-27, the-5-waves wave 2.

verdict: NEEDS-CHANGE
cases: added 8, red 2 (executed 880→888)
origin: introduced 1, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_filters_pass1.rs`. Red: a filtered view without the identity gets both holds-no-rows and holds-a-row; a fold filter beside a creating branch guarded on the literal is witnessed only over the byte-exact value. Green: in_ignore_case, two-field, enum, Optional, state-change and ranked filtered views; committed suites byte-identical.

Coordinator routing: both to the implementor (the undecided one is the story acceptance).

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4011, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A filtered view that does not project the identity gets Excludes{} and Contains{} in one block, so every implementation fails."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5228, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "arrange_toward accepts the first creating branch that lands a match, so a fold filter is witnessed only over the byte-exact literal when a branch is guarded on it."}]
```
