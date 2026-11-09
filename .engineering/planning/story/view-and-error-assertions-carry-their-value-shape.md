---
format: aep.planning-md/3
id: story:view-and-error-assertions-carry-their-value-shape
kind: story
status: draft
title: View and error assertions carry their value shape
summary: Typed shapes on view and error steps, so runners fold null into absent only for struct members and a Map key holding null stays distinct (row 48 follow-up, 0.70.0).
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_u28_pass1.rs
revision: 12
---
## Need

Left open by `story:a-null-member-matches-an-absent-optional-in-a-nested-assertion` (row 48,
0.69.0). Its rule makes an absent struct member and an explicit `null` one value below the top level
in view assertions and expected errors, in the Rust, Go and TypeScript runners. None of the three
runners knows a field's declared type there: `ExpectView`, `EventuallyView` and `ExpectError` carry
raw values (Rust `Runner` holds no IR; Go and TypeScript read only the suite JSON). So a
`Map<String, Optional<T>>` value `{a: null}` also reads as `{}` and the reverse, although a map key
is data, not a member. Event and await steps already carry a typed `PayloadShape` whose leaves
include a `Map` kind (`scenario.rs`).

Reproduction, still green on 0.92.0: the test
`crates/verify/ess-conformance/tests/adversary_u28_pass1.rs::a_map_key_holding_null_reads_as_absent_until_assertions_carry_their_type`
stores `{a: null}` in a `Map<String, Optional<Integer>>` view field and asserts `{}` (and the
reverse); both scenarios pass, so a target that drops or invents a map key holding `null` passes
the suite. The control `adv_u28_a_map_key_holding_a_value_is_not_the_empty_map` (`{a: 3}` against
`{}`) fails as it should. The fold sits in each comparator:

- Rust `same_value`, `src/runner.rs:3559-3567`: its Map arm treats `None | Some(Node::Null)` on both
  sides as equal for every key.
- Go `sameValue`/`sameMember`, `src/go/runtime.go:3843-3883`: the same, for every `map[string]any`.
- TypeScript `sameValue`, `src/ts/runtime.ts:6090-6110`: the same.

## Acceptance

- View and error steps carry the value shape the way `PayloadShape` does, under a new suite format
  version, and each runner folds `null` into absent only for struct members.
- The 0.69.0 test `a_map_key_holding_null_reads_as_absent_until_assertions_carry_their_type` is
  turned round: a Map key holding `null` differs from an absent key in all three runners (the
  scenario fails with a mismatch naming the key).
- A suite in the earlier format keeps the 0.69.0 behaviour.

## Scope

- cited: `crates/verify/ess-conformance/src/runner.rs` (`matches` :3516, `same_value` :3559, `mismatch` :3590, `expect_error` :1821, `expect_view` :2780)
- cited: `crates/verify/ess-conformance/src/go/runtime.go` (`errorCarries` :2840, `matches` :3806, `rowMatches` :3811, `sameValue` :3843, `sameMember` :3876; admission table)
- cited: `crates/verify/ess-conformance/src/ts/runtime.ts` (`matches` :6055, `sameValue` :6094; admission table)
- cited: `crates/verify/ess-conformance/src/scenario.rs` (`PayloadShape` :2004, `ExpectError` :2525, `ExpectView` :2754, `EventuallyView` :2782, `SUPPORTED_SUITE_FORMATS` :558 ending at 63)
- cited: `crates/verify/ess-conformance/tests/adversary_u28_pass1.rs` (the test to turn round, :287)
- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (`view_expectations`, shape walked as `payload_shape` does for events)
- inferred: `crates/verify/ess-conformance/src/authored.rs` (authored view and error steps)
- inferred: `crates/verify/ess-conformance/src/lib.rs` and a new version module beside `src/member_paths.rs`
- inferred: `crates/verify/ess-conformance/src/ts/runtime.test.ts`, `tests/suite.rs` (the probes asserting `ess-conformance/64` unsupported)
- inferred: `crates/specify/ess-compiler/src/ir.rs` (`ResolvedView.fields`, the shape source)

confidence: medium — the comparators and the test are cited; the shape source and whether values
inside a Map are folded (`Holds::Map` describes no values, `scenario.rs:2114`) are a decision.

Size: large, about 400-650 lines plus a version module; a new suite major (the next free ordinary
major is 64). Collides with any unit touching the view and error comparators in all three runners,
the `ViewExpectation` and `ExpectError` vocabulary, the Go and TypeScript admission tables, and the
next suite major number.
