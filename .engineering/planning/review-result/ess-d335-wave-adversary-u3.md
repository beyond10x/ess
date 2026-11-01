---
format: aep.planning-md/3
id: review-result:ess-d335-wave-adversary-u3
kind: review-result
status: active
title: The D335 wave adversary pass over U3
summary: 'NEEDS-CHANGE then fixed: F1 and F2 fixed in ccecfc75b; out of scope: 1 adopted as a draft story, 1 declined'
relations:
- reviews: story:a-binary64-value-round-trips-through-a-conformance-suite
revision: 1
---
## Pass

Adversary pass over U3 on `integration/ess-d335` at f085bff06 (unit diff `f085bff06^1..f085bff06`), reviewing the diff against story:a-binary64-value-round-trips-through-a-conformance-suite's Acceptance only (rule of 2026-10-31). Verdict NEEDS-CHANGE: 2 red cases; cases executed 15 to 20. Tests: `crates/verify/ess-conformance/tests/adversary_d335_u3_pass1.rs`, `crates/verify/ess-conformance/src/ts/adversary_d335_u3.test.ts`.

## Attacked and not broken

Go and TypeScript `binary64Major` cover shape, capture, response and fixtures; `Map<…, Binary64>` exact observers in Go and TypeScript; union content keys; the witness stays below 2^53; Go and TypeScript `asNumber` refuse `1e400`; a model without Binary64 keeps its format and bytes. Kept green pins: `1e400` in an `establish_entity` step is refused by name at its path; a Binary64 entity identity is refused at admission and as ESS-AUTHOR-036; a respelled Binary64 passes the Rust runner.

## Outcome

Fixed in ccecfc75b. F1: TypeScript reads a number in a view row, an error's fields, an event's payload and an invocation's input as Go and Rust do (an integer exactly, any other number by its nearest binary64); the exact observers keep comparing a Decimal by exact value. F2: an authored Binary64 literal of magnitude 2^53 or more is refused as ESS-AUTHOR-015, because view and event assertions carry no field types; synthesis never writes one. The F2 case is restated as that refusal, its assertion message naming the story.

## Out of scope (decided by the wave lead at close, rule of 2026-10-31 23:4xZ)

| finding | impact | evidence | decision |
|---|---|---|---|
| A binding's delivery-context field typed `Binary64` directly is not walked by `owner_uses`, so `uncarried_uses` cannot refuse it (looks pre-existing, not run). | An uncarried Binary64 position may be admitted and reach a runner that cannot carry it. | `crates/specify/ess-compiler/src/binary64.rs`, `owner_uses` | adopt: draft story:a-binding-context-binary64-is-refused-by-name |
| A typed record on view and event assertions, so a Binary64 integer token at or past 2^53 compares by its nearest binary64. | None reachable: synthesis never writes such a value and authoring now refuses it. | F2 above; `crates/specify/ess-primitives/src/facts.rs` integer comparison | decline: unreachable after the authoring refusal; reopen if a hand-written suite needs it |

```findings
[
{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":6118,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"TypeScript view-row equality compares a Binary64 by exact decimal while Rust and Go use its nearest binary64, so 0.1 vs 0.10000000000000001 passes in two runners and fails in TypeScript"},
{"file":"crates/specify/ess-primitives/src/facts.rs","line":574,"category":"acceptance","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"a Binary64 integer token at or past 2^53 is compared exactly, so an f64 target answering its nearest binary64 fails although the contract calls them one value"},
{"file":"crates/specify/ess-compiler/src/binary64.rs","line":null,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"out of scope: a binding delivery-context field typed Binary64 directly is not walked by owner_uses, so uncarried_uses cannot refuse it (not run)"}
]
```
