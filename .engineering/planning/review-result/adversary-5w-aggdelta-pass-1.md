---
format: aep.planning-md/3
id: review-result:adversary-5w-aggdelta-pass-1
kind: review-result
status: active
title: Adversary pass 1, aggdelta (the-5-waves)
relations:
- reviews: story:ungrouped-aggregate-views-are-witnessed
revision: 1
---
Adversary pass 1 against story:ungrouped-aggregate-views-are-witnessed (#148 follow-up), aep:adversary, 2026-09-28, the-5-waves wave 3.

verdict: NEEDS-CHANGE
cases: added 11, red 2 (executed 944→955)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_aggdelta_pass1.rs`. Red: a count, or a required sum, reported null over no rows passes on an empty target. Green: count+sum only, filtered view, doubled sum, per-field checks, late eventual projection, label and amount admission.

Coordinator routing: both to the implementor.

```findings
[{"file": "crates/verify/ess-conformance/src/aggregate.rs", "line": 184, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "changed_by reads an absent value as zero for every function, so a target reporting a count or a required sum as null over no rows passes on an empty target"},
 {"file": "docs/design/aggregate-views.md", "line": 695, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the Mutants of the change table lists only killed mutants and omits those a change cannot kill"}]
```
