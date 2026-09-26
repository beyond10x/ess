---
format: aep.planning-md/2
id: review-result:a1-adversary-pass-1
kind: review-result
status: active
title: A1 adversary pass 1 (Optional, list and text guards)
relations:
- reviews: story:optional-guards-mean-what-they-say
- reviews: story:list-and-text-guards-are-synthesized
revision: 1
---
Adversary pass 1 on wave unit A1 (#93, #94, plus Duration-ordering and compact `&&`/`||`
refusals). Tree `ess-wave-20260925-a1` on `bb7718889` plus the implementor's uncommitted diff.

Header as returned: verdict NEEDS-CHANGE; cases executed 463→471 (ess-cli only; run stopped at
11G free disk), red 4; origin introduced 6, pre-existing 0, undecided 0. Added three test files.

Coordinator routing: 1, 2, 3, 5, 6 back to the implementor; 4 (INFEASIBLE, nothing reaches it)
recorded as a CHANGELOG note and its case rewritten to assert the refusal. Three
`observed_bindings` failures attributed to stray `.git` directories in the home directory, not
to this unit.

```findings
[{"file":"crates/specify/ess-primitives/src/predicate.rs","line":1170,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"validate refuses an unquoted null comparison without any ESS code, while the story requires a stable code"},{"file":"crates/verify/ess-conformance/src/input.rs","line":114,"category":"property","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"stores built by new_facts order a declared Timestamp by its spelling, so entity setup accepts an offset instant that breaks its invariant"},{"file":"crates/verify/ess-conformance/src/input.rs","line":35,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the doc says list ordinal reads are decided, but a guard such as tags.0 == vip validates and is refused ESS-SYNTH-002 as a collection"},{"file":"crates/specify/ess-primitives/src/predicate.rs","line":275,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"canonical predicate strings the base build wrote for quoted null or && text no longer read back, and no format version records it"},{"file":"crates/specify/ess-primitives/src/error.rs","line":84,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the NullComparison message always says null, even when the input was ~"},{"file":"crates/verify/ess-conformance/src/witness.rs","line":275,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"list element alternatives multiply while the list is empty, giving identical candidates that use up the candidate budget"}]
```
