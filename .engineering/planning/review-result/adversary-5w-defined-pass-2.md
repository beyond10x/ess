---
format: aep.planning-md/3
id: review-result:adversary-5w-defined-pass-2
kind: review-result
status: active
title: Adversary pass 2, defined (the-5-waves)
relations:
- reviews: story:defined-over-optional-aggregates
revision: 1
---
Adversary pass 2 against story:defined-over-optional-aggregates (#176), aep:adversary, 2026-09-27, the-5-waves wave 1, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 4, red 3 (executed 807→811)
origin: introduced 1, pre-existing 0, undecided 1

New cases in `crates/verify/ess-conformance/tests/adversary_defined_pass2_views.rs`. Red: view filters `defined(metrics.detail)`, `missing(metrics.detail)`, `defined(metrics.waiting)` below an Optional struct are decided wrongly because `shows` marks only the top-level path. Green: `defined(metrics)` control; runtime agreement, quantifier binders, refusal text.

Ledger against pass 1: carried 0, new 2, resolved 1 (1 routed to the coordinator). Coordinator routing: both findings (one fix) to the final correction round; the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4718, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "shows() marks only the top-level settled struct and never walks it, so a view filter over an aggregate nested in an Optional struct is decided wrongly"},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4713, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "shows() binds no leaf of a settled struct, so a view filter defined(metrics.waiting) is decided false while the runner sees the leaf"}]
```
