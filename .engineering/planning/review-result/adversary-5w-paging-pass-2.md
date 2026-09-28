---
format: aep.planning-md/3
id: review-result:adversary-5w-paging-pass-2
kind: review-result
status: active
title: Adversary pass 2, paging (the-5-waves)
relations:
- reviews: story:view-paging-and-caller-filters
revision: 1
---
Adversary pass 2 against story:view-paging-and-caller-filters (#174), aep:adversary, 2026-09-28, the-5-waves wave 5, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 6, red 5 (executed 1337→1343)
origin: introduced 4, pre-existing 0, undecided 1

New cases in `adversary_paging_pass2.rs` (ess-conformance, ess-gen). Red: an OFFSET-by-page-number target passes; distinct_by empties when one projected field is unset; paging drops the deletion witness; the paged operation omits the filter parameter; the description names declared rather than wire parameter names. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 5, resolved 5. Coordinator routing: all five to the final correction round (the undecided OpenAPI one included); the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/paging.rs", "line": 75, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Every page read has size 1 or the first page, so a target offsetting by the page number passes."},
 {"file": "crates/verify/ess-conformance/src/synthesize/paging.rs", "line": 156, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "distinct_by empties when any projected field lacks a known literal."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 6581, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Declaring paging on an otherwise parameterless view drops its deletion witness."},
 {"file": "crates/generate/ess-gen/src/openapi.rs", "line": 508, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "The paged view operation lists page and size but not the declared filter parameter."},
 {"file": "crates/generate/ess-gen/src/openapi.rs", "line": 1485, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The paging description names declared parameters while the query keys are wire names."}]
```
