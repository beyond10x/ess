---
format: aep.planning-md/3
id: review-result:adversary-5w-compose-pass-1
kind: review-result
status: active
title: Adversary pass 1, compose (the-5-waves)
relations:
- reviews: story:consumer-imports-owner-types
revision: 1
---
Adversary pass 1 against story:consumer-imports-owner-types (#162), aep:adversary, 2026-09-28, the-5-waves wave 5.

verdict: NEEDS-CHANGE
cases: added 10, red 2 (executed 24→34)
origin: introduced 2 (+2 notes), pre-existing 0, undecided 0

New cases in `crates/specify/ess-composition/tests/adversary_compose_conformance.rs`. Red: enum variant wire spellings and Optional field presence are not compared. Green: field wire names, newtype of another type, union tag/variants, map key, kind mismatch, recursion, cross-component isolation.

Coordinator routing: findings 1-3 to the implementor; finding 4 is the documented design limit.

```findings
[{"file": "crates/specify/ess-composition/src/conformance.rs", "line": 145, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "enum conformance compares variant names only, not wire spellings"},
 {"file": "crates/specify/ess-composition/src/conformance.rs", "line": 118, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "struct field comparison ignores naming.presence"},
 {"file": "crates/edge/ess-cli/src/main.rs", "line": 120, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "ess specify compose help still documents ess-composition/1"},
 {"file": "crates/specify/ess-composition/src/conformance.rs", "line": 14, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "newtype alphabet/prefix/invariants are not compared, so a stricter local type conforms"}]
```
