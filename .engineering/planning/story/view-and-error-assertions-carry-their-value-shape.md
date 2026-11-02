---
format: aep.planning-md/3
id: story:view-and-error-assertions-carry-their-value-shape
kind: story
status: draft
title: View and error assertions carry their value shape
summary: Typed shapes on view and error steps, so runners fold null into absent only for struct members and a Map key holding null stays distinct (row 48 follow-up, 0.70.0).
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
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
revision: 10
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

## Acceptance

- View and error steps carry the value shape the way `PayloadShape` does, under a new suite format
  version, and each runner folds `null` into absent only for struct members.
- The 0.69.0 test `a_map_key_holding_null_reads_as_absent_until_assertions_carry_their_type` is
  turned round: a Map key holding `null` differs from an absent key in all three runners.
- A suite in the earlier format keeps the 0.69.0 behaviour.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (view and error comparison in the Rust, Go and TypeScript runners) — cited.
- **Files, Rust:** `src/runner.rs:3516` (`matches`), `:3559` (`same_value`, its Map arm at 3561-3566 treats absent and `null` keys as equal), `:3590` (`mismatch`), `:1821` (`expect_error`), `:2780` (`expect_view`) — cited.
- **Files, Go:** `src/go/runtime.go:3803-3880` (`matches`, `rowMatches`, `sameValue`, `sameMember`), `:2837` (`errorCarries`), `:3076` (`expectView`) — cited.
- **Files, TypeScript:** `src/ts/runtime.ts:6046-6110` (`matches`, `rowMatches`, `sameValue`), `:4414` (error check) — cited.
- **Files, vocabulary:** `src/scenario.rs:2748` (`ExpectView`), `:2776` (`EventuallyView`), `:2519` (`ExpectError`), `:2934` (`ViewExpectation`); `PayloadShape` at `:2004` is carried only by event and await steps — cited.
- **Shape source:** `ResolvedView.fields` (`crates/specify/ess-compiler/src/ir.rs:2349`), walkable as `payload_shape` (`synthesize.rs:9523`) does for events — inferred.
- **Also likely:** `src/authored.rs:4274, 4669`, `src/synthesize.rs:9677` (`view_expectations`), a new version module beside `src/member_paths.rs`; `tests/adversary_u28_pass1.rs:287` is the 0.69.0 test to turn round (cited); `src/ts/runtime.test.ts` — inferred.
- **Confidence:** medium — the comparators and the failing test are cited; the shape source and the Map gap are inferred.
- **Would collide with:** any unit touching the view and error comparators in all three runners, the `ViewExpectation` and `ExpectError` vocabulary, the Go and TS admission tables (`go/runtime.go:945`, `ts/runtime.ts:1795`), and the next suite major number — inferred.
- **Suite major:** the next free ordinary major is 64 (coverage 65); `SUPPORTED_SUITE_FORMATS` ends at 63 (`scenario.rs:558-563`); probes at `tests/suite.rs:765`, `scenario.rs:3611` and `ts/predicate.test.ts:1139` assert `ess-conformance/64` is unsupported — cited.
- **Map gap:** `Holds::Map` describes no values (`scenario.rs:2114`); folding inside Map values is a decision — inferred.
- **Size:** large, about 400-650 lines plus a version module; a new suite major — cited.
