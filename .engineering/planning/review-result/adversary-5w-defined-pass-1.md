---
format: aep.planning-md/3
id: review-result:adversary-5w-defined-pass-1
kind: review-result
status: active
title: Adversary pass 1, defined (the-5-waves)
relations:
- reviews: story:defined-over-optional-aggregates
revision: 1
---
Adversary pass 1 against story:defined-over-optional-aggregates (#176), aep:adversary, 2026-09-27, the-5-waves wave 1.

verdict: NEEDS-CHANGE
cases: added 13, red 1 (executed 1688→1701)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `adversary_defined_{admission,witness,runner}.rs`. Red: a view filter `defined(x)` over an Optional aggregate is decided false in synthesis (`shows`), so the suite expects no row. Green: ess/15 refusal in every position, witnesses for list/map/missing/nested, the `RowAndInput::present` patch, Optional<List> invariant, null agreement across runtimes.

Coordinator routing: finding 1 to the implementor (edit of `shows` allowed); finding 2 wired by the coordinator on the integration branch after unit leaves registers /26–/27.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4709, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "shows() drops a settled struct or list literal without a presence mark, so a view filter defined(x) over an Optional aggregate is decided false and synthesis expects no row"},
 {"file": "crates/verify/ess-conformance/src/presence.rs", "line": 17, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "a suite using defined() over an aggregate keeps the old suite format, so a base-release runtime reads defined(metrics) as false and passes the invariant without checking it"}]
```
