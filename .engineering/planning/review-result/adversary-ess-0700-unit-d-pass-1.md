---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-d-pass-1
kind: review-result
status: active
title: Adversary pass 1, unit D (public minting and ordering, denied grant)
relations:
- reviews: story:identity-minting-and-view-ordering-are-public
- reviews: story:a-denied-grant-over-a-command-that-writes-nothing-is-not-an-error
- reviews: story:a-refused-grant-scenario-runs-on-the-interpreter
revision: 1
---
## Adversary pass 1, unit D (ess-0700 wave)

Worktree ess-0700-d, base 6cd9a059b, implementor's changes uncommitted; adversary added
`crates/verify/ess-conformance/tests/adversary_d_values.rs` (4 cases, 2 red) and one guard in
`tests/grant_denied.rs` (the reported lamp model's denied scenario in Go and TypeScript).

Runs, one binary at a time with `ESS_TYPES_NODE` set: `adversary_d_values` 2 passed, 2 failed;
`grant_denied` 18 passed; `interpreted_views` 9 passed; `interpreted_typed_identities` 13 passed.

Held: mint parity with the interpreter for every minted type; order parity for every `order_by`
shape (descending Optional, ties, identity tie-break, enum, Decimal, Timestamp); no panic reachable
from a compiled IR; the tick contract; the mutant run turns 7 of 17 grant cases red in all three
runners.

Coordinator disposition: one bounded correction round verified by the coordinator, no second pass.

```findings
[
{"file":"crates/verify/ess-conformance/src/interpret/values.rs","line":31,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the rustdoc promises a Uuid for any u64 tick; from tick 10^12 the last group has 13 or more digits"},
{"file":"crates/verify/ess-conformance/src/interpret/values.rs","line":35,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the rustdoc says only few-valued types repeat; Timestamp repeats every 28 ticks and Integer and Decimal saturate past u32::MAX"},
{"file":"crates/verify/ess-conformance/tests/interpreted_views.rs","line":372,"category":"property","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the 6-row tie test stays green under a sort_unstable_by mutant; the new 64-row guard catches it"},
{"file":"crates/verify/ess-conformance/src/interpret.rs","line":79,"category":"judgement","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the public interpret::values and the private interpret::execute::values share a name"}
]
```
