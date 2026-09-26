---
format: aep.planning-md/2
id: review-result:a2-adversary-pass-1
kind: review-result
status: active
title: A2 adversary pass 1 (predicate reference page)
relations:
- reviews: story:predicate-reference-page
revision: 1
---
Adversary pass 1 on wave unit A2 (`website/docs/reference/predicates.md`, #92). Tree
`ess-wave-20260925-a2` on `bb7718889` plus the implementor's uncommitted diff.

Header as returned: verdict NEEDS-CHANGE; cases executed 5→10, red 5; origin introduced 5,
pre-existing 1, undecided 0. Added `crates/edge/ess-cli/tests/predicate_reference_page_adversary.rs`.

Coordinator routing: F1 (Duration ordering) and F3 (compact `&&`/`||` in an unquoted operand) are
code fixes routed to unit A1, which owns the checker and parser; the page keeps its claims. F2, F4,
J1, J2 back to the A2 implementor as page/test changes. F5 is pre-existing; the page documents the
difference, no code change.

```findings
[{"file":"website/docs/reference/predicates.md","line":419,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page says Duration has no ordering, but validate admits `wait > \"PT5M\"` on a Duration input (checker treats Duration as Text), and after #94 it would order byte-wise."},{"file":"website/docs/reference/predicates.md","line":391,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page says list elements are reachable only through a quantifier, but validate admits the ordinal path `tags.0 == vip`."},{"file":"website/docs/reference/predicates.md","line":191,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page implies compact `&&`/`||` are refused, but on a text field validate admits `sku == A1 && gift` by reading everything after `==` as one text literal."},{"file":"website/docs/reference/predicates.md","line":166,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The page says a right-hand side without a dot is always text, but validate refuses `sku == gift` with ESS-COMMAND-002 because the word names a declared field, and the page omits that refusal."},{"file":"crates/specify/ess-primitives/src/predicate.rs","line":989,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"The map equality shorthand `{sku: A.1}` reads a bare dotted word as a literal, while `{sku: {eq: A.1}}` and `sku == A.1` read it as a fact path, contradicting the page's claim that the spellings parse to the same value."},{"file":"crates/edge/ess-cli/tests/predicate_reference_page.rs","line":385,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"`ess-pending` skips an example entirely, including validate, and nothing fails when A1/B1 merge and the marker stays."},{"file":"website/docs/reference/predicates.md","line":17,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The placement table omits newtype `invariants` over `value` (types.rs:597), and the page does not mention that quantifiers and `.count` also accept a Map."}]
```
