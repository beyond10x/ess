---
format: aep.planning-md/3
id: review-result:adversary-5w-whensubj-pass-1
kind: review-result
status: active
title: Adversary pass 1, whensubj (the-5-waves)
relations:
- reviews: story:when-subject-witness-and-diagnostics
revision: 1
---
Adversary pass 1 against story:when-subject-witness-and-diagnostics (#172, #173), aep:adversary, 2026-09-27, the-5-waves wave 1.

verdict: NEEDS-CHANGE
cases: added 7, red 2
origin: introduced 1 (+1 note), pre-existing 1, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_whensubj_pass1.rs`. Red: the wrong-state scenario sends an input that selects an input-guarded refusal sibling; an `unknown_instance:` branch beside `when_subject` branches cascades into ESS-SYNTH-003/004 (same at `e9c437303`). Green: repros with `wrong_state` declared, several guarded branches through an eventual view, filtered-view hint, immediate preferred over eventual.

Coordinator routing: finding 1 and 3 to the implementor; finding 2 (pre-existing) kept in this unit because the acceptance statement covers it.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 521, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "refusal_input takes the first candidate passing only the moving branch input guard, so the wrong-state scenario sends an input that selects the input-guarded refusal instead of a moving branch"},
 {"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 134, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "guarded() does not exclude UnknownInstance, so an unknown_instance branch makes every when_subject branch ambiguous and cascades into ESS-SYNTH-003/004"},
 {"file": "crates/verify/ess-conformance/src/synthesize/subject_fact.rs", "line": 1617, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the unchanged-row check after a refusal goes through an eventual view, which passes on a stale projection"}]
```
