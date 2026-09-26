---
format: aep.planning-md/2
id: review-result:b1a-adversary-pass-1
kind: review-result
status: active
title: B1a adversary pass 1 (string operators design)
relations:
- reviews: story:string-prefix-suffix-substring-operators
revision: 1
---
Adversary pass 1 on wave unit B1a (design page `docs/design/string-predicate-operators.md`, #95).

Header as returned: verdict NEEDS-CHANGE; cases n/a (design unit); origin introduced 11. About 60
citations checked, exact or within 2 lines.

Coordinator routing: all back to the implementor. Decided: F3 — TypeScript refuses suites carrying
the operators (no /12+ reader in this unit); F4 — the #74 field-name refusal applies; F6 — B1b takes
the next free numbers, C and D2 follow; F7 — keep the rev pin, #40 must land with a merge commit;
F10 no-op.

```findings
[{"file":"docs/design/string-predicate-operators.md","line":199,"category":"property","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The composed candidate always puts the path's own text after the contains literals and takes the first positive affix even when negated elsewhere, so all:[contains RE, not starts_with RE] on subject has no witness although subjectRE satisfies it."},{"file":"docs/design/string-predicate-operators.md","line":236,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"With a PhoneNumber invariant starts_with +, both refuting candidates (caller, x44) are dropped by admitted_inputs, so the negation or otherwise branch of starts_with +44 is never witnessed."},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":4203,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page tells the TypeScript lane to follow Go on the new suite pair, but TypeScript admits only suites up to /11 and has no replay."},{"file":"docs/design/string-predicate-operators.md","line":56,"category":"judgement","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page is silent on whether the #74 refusal of a text literal naming a declared field applies, so {starts_with: gift} validates silently as text."},{"file":"docs/design/string-predicate-operators.md","line":384,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The rg recipe claimed to list next-number test literals matches none of them."},{"file":"docs/design/string-predicate-operators.md","line":391,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"B1a assigns ess/8 to B1b while C's page hard-codes ess/8 and D1's page assumes C holds it."},{"file":"docs/design/string-predicate-operators.md","line":396,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The pin targets the head of open PR #40, which never reaches entity-runtime main if the PR is squash-merged."},{"file":"docs/design/string-predicate-operators.md","line":65,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The page cites ParseError::shape, but that site uses ParseError::predicate."},{"file":"docs/design/string-predicate-operators.md","line":165,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The page says an enum is named as an enum, but its example message prints (Text)."},{"file":"crates/verify/ess-conformance/src/witness.rs","line":241,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"With L-prime appended last and the mixed-radix walk capped at 64, the last alternatives of a second text path are never tried once the first ladder has 9 or more positions."},{"file":"docs/design/string-predicate-operators.md","line":132,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The cited vector-corpus precedent is read by Rust, Go and the browser, not TypeScript."}]
```
