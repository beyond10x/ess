---
format: aep.planning-md/3
id: review-result:adversary-5w-now-pass-2
kind: review-result
status: active
title: Adversary pass 2, now (the-5-waves)
relations:
- reviews: story:current-time-guard-operand
revision: 1
---
Adversary pass 2 against story:current-time-guard-operand (#171), aep:adversary, 2026-09-28, the-5-waves wave 3, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 5, red 5 (executed 1756→1761)
origin: introduced 4, pre-existing 0, undecided 0

New cases in `adversary_now_pass2.rs` (ess-conformance) and `adversary_now_pass2_hints.rs` (ess-domain). Red: a fixed bound after the 2019 reference beside a now guard is witnessed the wrong way; an equality with a fixed instant is rewritten as now_offset; i64::MIN overflows admission; two refusal hints suggest the operand where it is refused. Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 4, resolved 3. Coordinator routing: final correction round (finding 1: refuse by name); the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/now_offset.rs", "line": 52, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "values are decided against a 2019 reference, so a fixed bound between the reference and the run beside a now guard is witnessed the wrong way"},
 {"file": "crates/verify/ess-conformance/src/now_offset.rs", "line": 105, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an equality with a fixed instant is not recognised as a fixed bound"},
 {"file": "crates/verify/ess-conformance/src/admission.rs", "line": 289, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "seconds.abs() overflows on i64::MIN"},
 {"file": "crates/specify/ess-domain/src/expression.rs", "line": 1040, "category": "contract-drift", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "refusal hints suggest the operand at sites that refuse it"}]
```
