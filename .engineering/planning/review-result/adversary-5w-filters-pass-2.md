---
format: aep.planning-md/3
id: review-result:adversary-5w-filters-pass-2
kind: review-result
status: active
title: Adversary pass 2, filters (the-5-waves)
relations:
- reviews: story:view-filters-witnessed-on-matching-rows
revision: 1
---
Adversary pass 2 against story:view-filters-witnessed-on-matching-rows, aep:adversary, 2026-09-27, the-5-waves wave 2, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 3, red 3 (executed 888→891)
origin: introduced 2, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_filters_pass2.rs`. Red: a sibling view without the identity is asserted empty after a matching row was arranged; the matching row reuses the subject instance name; a negated fold is witnessed only over the byte-exact value. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 3, resolved 2. Coordinator routing: final correction round (F3: fix if contained, else pin todays behaviour naming the story); the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4132, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A matching row arranged for an identity-projecting view lands in a sibling view without the identity that is still asserted Excludes {}, so the scenario is unsatisfiable."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4147, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The matching row is numbered regardless of the subject own distinction, so a fresh-witness subject and the matching row share one instance name."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4133, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "Under a negated fold no case-changed row is arranged to be excluded, so a byte-wise target passes."}]
```
