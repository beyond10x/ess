---
format: aep.planning-md/3
id: story:view-and-error-assertions-carry-their-value-shape
kind: story
status: draft
title: View and error assertions carry their value shape
summary: Typed shapes on view and error steps, so runners fold null into absent only for struct members and a Map key holding null stays distinct (row 48 follow-up, 0.70.0).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
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
