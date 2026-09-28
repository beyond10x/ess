---
format: aep.planning-md/3
id: review-result:adversary-5w-whensubj-pass-2
kind: review-result
status: active
title: Adversary pass 2, whensubj (the-5-waves)
relations:
- reviews: story:when-subject-witness-and-diagnostics
revision: 1
---
Adversary pass 2 against story:when-subject-witness-and-diagnostics (#172, #173), aep:adversary, 2026-09-27, the-5-waves wave 1, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 3, red 2 (executed 804→807)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_whensubj_pass2.rs`. Red: the wrong-state input does not refute the `when:` half of a `when_subject:` refusal sibling; an update writing the held value is awaited through an eventual view. Green: an eventual moving branch awaits the state it arrives at (kills the `leaves_changed -> false` mutant). Pass-1 corrections hold.

Ledger against pass 1: carried 0, new 2, resolved 3. Coordinator routing: final correction round; the coordinator verifies it by reading the diff (budget of two attacks spent).

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 531, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "refusal_input refutes siblings input guards only through when, which skips the when: half of a when_subject: refusal, so the wrong-state scenario sends an input where that refusal also holds"},
 {"file": "docs/design/cross-record-and-stored-field-guards.md", "line": 265, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the doc says a row a branch left unchanged is never awaited through an eventual view, but the generic view assertion still emits an eventually block after an update that wrote the held value"}]
```
