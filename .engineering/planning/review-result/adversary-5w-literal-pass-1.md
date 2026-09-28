---
format: aep.planning-md/3
id: review-result:adversary-5w-literal-pass-1
kind: review-result
status: active
title: Adversary pass 1, literal (the-5-waves)
relations:
- reviews: story:literal-fallback-after-else
revision: 1
---
Adversary pass 1 against story:literal-fallback-after-else (#163), aep:adversary, 2026-09-27, the-5-waves wave 1.

verdict: NEEDS-CHANGE
cases: added 6, red 2
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_literal_pass1.rs`. Red: a payload literal after `else:` naming an input, or misspelling `input.`, compiles as text. Green: ess/16 gate on `sets:` alone, typed Decimal/Boolean/Optional/Integer literals, updating outcome over an arranged value, filtered view.

Coordinator routing: finding 1 to the implementor; finding 2 (Struct arm owned by unit leaves) checked on the integration branch after both merge.

```findings
[{"file": "crates/specify/ess-domain/src/command/value_expression.rs", "line": 408, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A payload literal after else: skips the misspelled_reference rules a top-level payload literal gets, so else: rank compiles as text although E4 says the literal is checked exactly as one written there."},
 {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4031, "category": "mutant", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "A literal fallback inside a struct beside a generated leaf is never asserted while its input is omitted, so a different-default mutant survives there; the Struct arm belongs to unit leaves."}]
```
