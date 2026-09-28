---
format: aep.planning-md/3
id: review-result:adversary-5w-compose-pass-2
kind: review-result
status: active
title: Adversary pass 2, compose (the-5-waves)
relations:
- reviews: story:consumer-imports-owner-types
revision: 1
---
Adversary pass 2 against story:consumer-imports-owner-types (#162), aep:adversary, 2026-09-28, the-5-waves wave 5, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 9, red 1 (executed 34→43)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/specify/ess-composition/tests/adversary_compose_pass2.rs`. Red: a drift in a nested type used by two fields names only the first path. Green: pair-keyed visiting, enum wire/name, presence on the tolerance path, tolerance inside a list, extra local field, mutual recursion. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 2, resolved 3. Coordinator routing: final correction round; the coordinator verifies the diff.

```findings
[{"file": "crates/specify/ess-composition/src/conformance.rs", "line": 93, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a pair of types is never compared again once visited, so a drift in a type used by two fields names only the first path"},
 {"file": "docs/design/composition-type-conformance.md", "line": 61, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "struct invariants are not compared, but the Not-compared list names only newtype invariants"}]
```
