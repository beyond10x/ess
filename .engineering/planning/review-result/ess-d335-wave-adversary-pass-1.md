---
format: aep.planning-md/3
id: review-result:ess-d335-wave-adversary-pass-1
kind: review-result
status: active
title: The D335 wave adversary pass 1
summary: 'NEEDS-CHANGE then fixed: A1-A4 fixed in 7e125d139; no out-of-scope rows'
relations:
- reviews: story:an-authored-claim-may-compare-a-nested-field
- reviews: story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type
- reviews: story:a-text-inequality-against-an-example-is-refuted
revision: 1
---
## Pass

Adversary pass 1 over `integration/ess-d335` at ae14432ce (U4 32d1b8058 and f352bf79b, U1 fecca8240, U2 ae14432ce merged), reviewing each unit's diff against its story's Acceptance only (rule of 2026-10-31). Verdict NEEDS-CHANGE: 4 red cases, cases executed 136 to 145. Tests: `crates/verify/ess-conformance/tests/adversary_d335_u1_pass1.rs`, `adversary_d335_u2_pass1.rs`, `adversary_d335_u4_pass1.rs`.

## Attacked and not broken

U1: the Go, TypeScript and Rust compare rules agree on a null literal at an absent path, a path through a non-object and admission below `/62` (read, not run). U2: no existing model's synthesis bytes moved; a stateless exhaustive accepting partition is refused by the compiler, so the held-state form is the only reachable one. U4: every scalar kind with an example, and an `Optional` text with an example, gets both branches of an inequality against it.

## Outcome

Fixed in 7e125d139 on `integration/ess-d335`: A1 and A2 (U4, `witness.rs`: a second witness the type admits, falling back to the next instance's witness or the invariant ladder), A3 (U2, `typed_input` walks members and elements beside a reference), A4 (U1, `response_shape` refuses a field claimed whole and by its path). All adversary cases pass.

## Out of scope

None.

```findings
[
{"file":"crates/verify/ess-conformance/src/witness.rs","line":null,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"undecided","message":"the own-witness alternative added for an example-valued text leaf is the unrepaired base, which an invariant-constrained type refuses, so a != against the example on a six-byte code has no refuting candidate (ESS-SYNTH-003)"},
{"file":"crates/verify/ess-conformance/src/witness.rs","line":null,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"undecided","message":"an example equal to the leaf's own base witness (Duration PT1S, Bytes AA==, Timestamp 2020-01-01T00:00:00Z) gets no refuting candidate for a != against it"},
{"file":"crates/verify/ess-conformance/src/authored.rs","line":null,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"typed_input reads only whole-literal fields, so an out-of-type literal member beside an $instance reference in a struct or list input is not refused ESS-AUTHOR-015 under ess-scenario/6"},
{"file":"crates/verify/ess-conformance/src/authored.rs","line":null,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"response_shape lacks split_paths' whole-versus-path conflict check, so a response claiming a field whole and a member of it by path with different values compiles into a scenario no return passes"}
]
```
