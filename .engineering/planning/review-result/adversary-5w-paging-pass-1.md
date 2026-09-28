---
format: aep.planning-md/3
id: review-result:adversary-5w-paging-pass-1
kind: review-result
status: active
title: Adversary pass 1, paging (the-5-waves)
relations:
- reviews: story:view-paging-and-caller-filters
revision: 1
---
Adversary pass 1 against story:view-paging-and-caller-filters (#174), aep:adversary, 2026-09-28, the-5-waves wave 5.

verdict: NEEDS-CHANGE
cases: added 6, red 5 (executed 1330→1336)
origin: introduced 5, pre-existing 0, undecided 0

New cases in `adversary_paging_pass1.rs` (ess-conformance, ess-gen). Red: a view without the identity lets repeated pages pass; no partial last page is exercised; an input named size is sent on unpaged reads; the OpenAPI response closes out the total; rows is still described as every row. Green: eventual paged views.

Coordinator routing: all five to the implementor.

```findings
[{"file": "crates/verify/ess-conformance/src/runner/page.rs", "line": 136, "category": "mutant", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A paged view that does not project its identity gets no distinct_by, so a target returning the first page for every page passes."},
 {"file": "crates/verify/ess-conformance/src/synthesize/paging.rs", "line": 33, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Every synthesized page is full, so a target that never answers a partial last page passes."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5787, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "bound() fills view params from the command input by name, so an input named size is sent on every unpaged read."},
 {"file": "crates/generate/ess-gen/src/openapi.rs", "line": 840, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "With total: true the response schema is closed over rows alone, so a conforming body is invalid."},
 {"file": "crates/generate/ess-gen/src/openapi.rs", "line": 853, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A paged view rows property still says every row."}]
```
