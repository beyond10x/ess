---
format: aep.planning-md/3
id: review-result:adversary-5w-narrow-pass-1
kind: review-result
status: active
title: Adversary pass 1, narrow (the-5-waves)
relations:
- reviews: story:optional-input-narrowed-after-refusal
revision: 1
---
Adversary pass 1 against story:optional-input-narrowed-after-refusal (#169), aep:adversary, 2026-09-27, the-5-waves wave 2.

verdict: NEEDS-CHANGE
cases: added 13, red 2 (executed 1898→1911)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `adversary_narrow_{domain,compile,synthesis}.rs`. Red: under ess/16 narrowing replaces the read type, so a declared `Optional<X> -> Y` conversion no longer applies (domain and compiler). Green: nested leaves, enum/Decimal, subject-state guards, any: and when_subject: scoping, synthesis sends/omits the input.

Coordinator routing: both to the implementor.

```findings
[{"file": "crates/specify/ess-domain/src/command.rs", "line": 2793, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Under ess/16 the narrowed type replaces the declared read type, so a declared Optional<X> -> Y conversion no longer applies, and the refusal misstates the input type."},
 {"file": "docs/design/optional-input-narrowing.md", "line": 72, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The note claims every conversion-workaround model keeps validating under ess/16, which holds only for Optional<T> -> T."}]
```
