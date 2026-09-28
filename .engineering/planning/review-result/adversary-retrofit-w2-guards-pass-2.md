---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w2-guards-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: fold-refuting candidates never reach a scenario'
relations:
- reviews: story:subject-guard-input-and-case-folding
revision: 1
---
Adversary pass 2 against story:subject-guard-input-and-case-folding (beyond10x/ess#157, #140), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE
cases: executed 708→712, red 2
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_guards_pass2.rs`. Red: no scenario sends a
text that only Unicode folding equates with the literal; the default branch is witnessed on the base
text, not on a one-character change. Green: `in_ignore_case` where every one-character change is
another member; literals without letters witnessed both ways. Held: only Entity Runtime reads the new
input environment; an entity field named `input` handled consistently; precomposed vs decomposed é.

Coordinator routing: both to the implementor in the final correction round.

```findings
[{"file": "crates/verify/ess-conformance/src/witness.rs", "line": 1279, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the unicode_refuting candidate never becomes a scenario witness, so a Unicode-folding target still passes the suite"},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 1278, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the default branch of a fold guard is witnessed on the base text, not on the one-character change the acceptance names"}]
```
