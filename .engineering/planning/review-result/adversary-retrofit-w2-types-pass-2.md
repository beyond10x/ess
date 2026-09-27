---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w2-types-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: presence omissions on guarded branches and struct members, rule-name collisions'
relations:
- reviews: story:json-values-and-text-patterns
- reviews: story:field-wire-names-and-presence-policy
revision: 1
---
Adversary pass 2 against story:field-wire-names-and-presence-policy (#142, #139) and story:json-values-and-text-patterns (#146, #138), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE
cases: 5 added, red 3 (the package suite could not be linked: /dev/shm quota exceeded)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `adversary_types_pass2.rs` (ess-conformance and ess-entity-runtime tests). Red: a
struct-member policy is never exercised; a policy event on a guarded branch never sees an absent
input; a member `x.y` and a field `x_y` over a two-layer prefix chain lower duplicate rule names.
Green: suites without a carried policy stay below 24; `{generated}` and `{input, else}` policy
fields beside a plain input pass the declared spelling.

Coordinator routing: all three to the implementor in the final correction round.

```findings
[{"file": "crates/verify/ess-conformance/src/witness.rs", "line": 256, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Only the base candidate omits presence inputs, so a guarded branch publishing the policy event never sees an absent value and a swapped implementation passes."},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 369, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "presence_inputs only considers top-level event fields, so a policy on a struct member's leaf is never exercised."},
 {"file": "crates/generate/ess-entity-runtime/src/lib.rs", "line": 2972, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "unique_rule_name adds its suffix once, so two paths that sanitize alike still lower duplicate rule names."}]
```
