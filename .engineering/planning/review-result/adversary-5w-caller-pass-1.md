---
format: aep.planning-md/3
id: review-result:adversary-5w-caller-pass-1
kind: review-result
status: active
title: Adversary pass 1, caller (the-5-waves)
relations:
- reviews: story:caller-value-source-and-guard
revision: 1
---
Adversary pass 1 against story:caller-value-source-and-guard (#168), aep:adversary, 2026-09-28, the-5-waves wave 4.

verdict: NEEDS-CHANGE
cases: added 6, red 5 (executed 1255→1261)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `adversary_caller_values.rs` (ess-conformance, ess-gen). Red: a Boolean caller attribute is compared as text; the swapped run in aggregate scenarios reuses input values so group counts are wrong; a fall-through refusal after a caller-guarded accepting branch is published as 422. Green: control on the #168 repro; enum/Decimal/Integer/Optional attributes.

Coordinator routing: all three to the implementor.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/caller.rs", "line": 263, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "A Boolean caller attribute is written into the guard as a text fact, so the suite expects forbidden for the creator and drops edited."},
 {"file": "crates/verify/ess-conformance/src/synthesize/caller.rs", "line": 383, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The swapped run is appended to aggregate scenarios with the same input values, so its absolute group counts are wrong."},
 {"file": "crates/specify/ess-domain/src/command/caller_value.rs", "line": 164, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "A refusal that falls through after an accepting branch guarded on the caller is not marked decided_by_caller, so OpenAPI publishes 422 instead of 403."}]
```
