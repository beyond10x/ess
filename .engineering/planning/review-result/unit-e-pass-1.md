---
format: aep.planning-md/3
id: review-result:unit-e-pass-1
kind: review-result
status: active
title: Unit E adversary pass 1
relations:
- reviews: story:a-view-may-publish-a-decimal
- reviews: story:a-binary64-field-is-admitted-by-a-conformance-run
revision: 1
---
## Adversary pass 1, unit E (ess-0700 wave)

Worktree ess-0700-e, base 79bbf9a3c, implementor changes uncommitted; adversary added 9 cases in retained_replay.rs, suite.rs, retained-replay-runtime.go and response.test.ts (4 red). Cases executed 335 to 344.

Held: Decimal spellings 1/1.0, 1.10/1.1, -0/0, 1.5e0, 15e-1, 1E2, 1e-7 compare equal in Rust, Go and TypeScript; the format rises to /58 through Optional, Map values, a newtype, a list of structs and top level, and not when a Decimal is published but not observed; coverage is written at /59; a Binary64 reached through every declared root still refuses; the unreached note is deterministic.

```findings
[
{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":4034,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Go compares non-integer Decimals through binary64 while the docs say it compares the exact value, so Rust, Go and TypeScript disagree on one suite"},
{"file":"crates/specify/ess-primitives/src/node.rs","line":56,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the Rust reader makes two different Decimals inside the exact range compare equal (0.1 vs 0.10000000000000001, a 30-digit integer pair)"},
{"file":"crates/specify/ess-compiler/src/binary64.rs","line":144,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"a Binary64 declared directly as an actor attribute is neither refused nor noted, while the same attribute through a declared type is refused"},
{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":550,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the unreached-Binary64 note names five reaching roots and omits actors, bindings and conversions"}
]
```
