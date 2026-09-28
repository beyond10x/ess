---
format: aep.planning-md/3
id: review-result:adversary-5w-narrow-pass-2
kind: review-result
status: active
title: Adversary pass 2, narrow (the-5-waves)
relations:
- reviews: story:optional-input-narrowed-after-refusal
revision: 1
---
Adversary pass 2 against story:optional-input-narrowed-after-refusal (#169), aep:adversary, 2026-09-27, the-5-waves wave 2, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 10, red 1 (executed 1033→1043)
origin: introduced 1, pre-existing 0, undecided 0

New cases in `adversary_narrow_pass2_{domain,compile,runtime}.rs`. Red: a `when: true` default declared before the refusal is narrowed, but Entity Runtime tries it first and copies an absent input. Green: every refusal spelling narrows, response reads never narrow, Optional list/struct/map, pass-1 correction holds.

Ledger against pass 1: carried 0, new 1, resolved 2. Coordinator decision: narrow only an `Otherwise` default; the control case asserting that a `when: true` default narrows is rewritten to the decision. Final correction round.

```findings
[{"file": "crates/specify/ess-domain/src/command/narrowing.rs", "line": 64, "category": "acceptance", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A when: true default is narrowed as if taken last, but Entity Runtime lowers it in declared order ahead of the refusal, so an absent input is copied unconditionally; no model in the repository writes when: true."}]
```
