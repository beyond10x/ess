---
format: aep.planning-md/2
id: review-result:b0-adversary-pass-1
kind: review-result
status: active
title: B0 adversary pass 1 (entity-core starts_with/ends_with)
relations:
- reviews: story:string-prefix-suffix-substring-operators
revision: 1
---
Adversary pass 1 on wave unit B0 (entity-runtime `starts_with` / `ends_with`, prerequisite of
`story:string-prefix-suffix-substring-operators`). Tree `er-wave-20260925-b0` at base `d67e901`
plus the implementor's uncommitted diff.

Header as returned: verdict NEEDS-CHANGE; cases executed 313→318, red 1; origin introduced 3,
pre-existing 0, undecided 0.

Added cases: `crates/entity-core/tests/string_affix_adversary.rs` — non-string literal refused at
registration (red), subject preload for `$fields` guards (green, kills two surviving mutants),
`$args` routing before load, quantifier binder, escaped `$$` literal.

Coordinator routing: F1 → back to the implementor, decided: refuse a literal operand that is
neither a string nor a reference with `InvalidRule` (a rule that can never hold is refused at
registration, `docs/design/kernel-v0.1.md:261`). F2 → fixed by the adversary's own case. F3 → back
to the implementor.

```findings
[{"file":"crates/entity-core/src/validation.rs","line":1638,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"A literal starts_with/ends_with operand that is not a string (YAML +44 parses as 44) registers and makes the rule false at every evaluation instead of being refused at registration."},{"file":"crates/entity-core/src/runtime.rs","line":2006,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The new condition_needs_subject arms are untested: dropping the StartsWith arm or swapping .any for .all in EndsWith leaves all 316 unit cases green, and only string_affix_adversary.rs:44 catches it."},{"file":"docs/design/service-semantics-v0.1.md","line":1505,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The service-semantics operator list and the section 10.4 mapping table omit starts_with and ends_with."}]
```
