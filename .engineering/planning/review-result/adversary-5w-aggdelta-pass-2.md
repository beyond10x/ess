---
format: aep.planning-md/3
id: review-result:adversary-5w-aggdelta-pass-2
kind: review-result
status: active
title: Adversary pass 2, aggdelta (the-5-waves)
relations:
- reviews: story:ungrouped-aggregate-views-are-witnessed
revision: 1
---
Adversary pass 2 against story:ungrouped-aggregate-views-are-witnessed, aep:adversary, 2026-09-28, the-5-waves wave 3, after correction round 1.

verdict: APPROVE
cases: added 6, red 2 (executed 956→962)
origin: introduced 1, pre-existing 1, undecided 0

New cases in `adversary_aggdelta_pass2{,_admission}.rs`. Red: a target truncating Decimal amounts passes (Decimal arranged on the Integer ladder, pre-existing); a changed_by amount 1e40 is admitted. Pass-1 correction holds.

Ledger against pass 1: carried 0, new 3, resolved 2. Coordinator routing: a short final round (Decimal ladder if committed suites stay identical, admission of inexact amounts, one doc line); the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 125, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "Decimal aggregate inputs are arranged on the Integer ladder, so a target that truncates Decimal passes"},
 {"file": "crates/verify/ess-conformance/src/aggregate_delta.rs", "line": 126, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a changed_by amount with no exact decimal spelling (1e40) is admitted although it can never be compared"},
 {"file": "docs/design/aggregate-views.md", "line": 687, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the page does not state that a concurrent writer between snapshot and read fails a correct shared target"}]
```
