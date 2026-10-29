---
format: aep.planning-md/3
id: story:a-binary64-bound-is-projected-as-a-numeric-keyword
kind: story
status: implemented
title: A Binary64 bound an invariant states is published as minimum, exclusiveMinimum, maximum or exclusiveMaximum
summary: Binary64 comparisons with a canonical literal lower to numeric keywords on struct, newtype and entity schemas
refs:
- provider: github
  reference: beyond10x/ess#687
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d138-schema-invariants-as-scoped
- depends_on: story:an-entity-invariant-is-projected-into-its-schema
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/agreement.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/binary64_bounds.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/model_ergonomics.rs
- confidence: cited
  path: docs/design/types-only-realizations.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:18Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:19Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-10-29T00:13:48Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Fit review

1. **Need.** A Binary64 rule (`selection_rate > 0`, `selection_rate <= 1`, a newtype's `value > 0`) is not enforced by the schema. Repro: a minimal specification outside the repository (not committed). Its generated struct document has `selection_rate: {"type":"number"}`, and a second struct carries the rule as an annotation only. Requester's proposal: `>` to `exclusiveMinimum`, `>=` to `minimum`, `<` to `exclusiveMaximum`, `<=` to `maximum`, "exact for every finite binary64 literal".
2. **Class.** Defect. The documented reason for not lowering is that a Decimal is projected as a string (`types.rs:136-138`). That reason does not apply to Binary64, which is projected as a JSON number. The lowering skips it only because `is_integer` filters on the primitive (`types.rs:1045-1052`).
3. **Already expressible?** No. No authored construct puts a numeric bound on a `type: number` node.
4. **Fit.** The same lowering, extended to `Primitive::Binary64`. Predicates already compare Binary64 "as the existing Number scalar … including numeric literals" (`docs/design/model-binary64.md:201-203`). That Number is the value's shortest round-trip decimal (`crates/specify/ess-primitives/src/facts.rs:400-413`). It reaches struct fields, `value` on a newtype, and entity fields (via #688). The requester's exactness claim is too broad: for a literal that is not its binary64's shortest spelling (for example `9007199254740993`), a validator that reads numbers as binary64 would refuse `v = 2^53` under `< 9007199254740993` while ESS accepts it. Established by walking the case, not run. `model-binary64.md:26` ("do not label that projection a complete finite-binary64 validator") still holds: the `model_binary64` obligation stays (`realize.rs:548-551`). Realization: the new keywords are already classified as runtime obligations, not refused (`schema-contract/src/realize.rs:1242-1250`). Inferring the integer width reads only `"integer"` nodes (`realize.rs:1460`).
5. **Second adopter.** A sensor reading with `humidity >= 0` and `humidity <= 100` on Binary64. An ingest validator should refuse `101.5`.
6. **Cost.** No format, no new authored keyword. `Node.minimum`/`maximum` (`types.rs:264-269`, `i64`) become number-valued bounds. Integer bounds keep their bytes (`"minimum": 1`, not `1.0`). `ess import openapi` records the new keywords as coverage gaps, not refusals (`ess-openapi/src/lib.rs:1158-1180`). No corpus example uses Binary64 (grep of `examples/`), so `generated/` is not expected to move.
7. **Alternatives.** (a) Change nothing. (b) As requested, every literal: unsound at the edges, as above. (c) Chosen: as requested, but only for literals that round-trip.

## Decisions

- Accept, redesigned (2026-10-28). The lowering applies to a comparison of a Binary64 field (required or Optional), or a newtype's `value`, with a numeric literal L, only when L is the canonical decimal of its own binary64 (`Number::new(L.get())` equals L). Then:
  - `>` becomes `exclusiveMinimum`, `>=` becomes `minimum`, `<` becomes `exclusiveMaximum`, `<=` becomes `maximum`;
  - `==` becomes `minimum` and `maximum` (no new `const` variant);
  - `!=` and non-canonical literals stay annotation only.
- The tightest bound of each keyword wins. An integral bound is written as an integer token.
- Changed from the request: the canonical-literal condition, and `==` added. The shared comparison reader also reads a literal on the left as its mirror (`0 < x`), for Integer too, as `realize.rs:350-380` already does for `value.count`.
- Proven on 0.74.0's validator: `exclusiveMinimum: 0` refuses 0 and -0.0, and `maximum: 1` admits 1 and refuses 1.5. This agrees with ESS: -0.0 is carried as 0 (`facts.rs:404-405`).
- If moving the committed `generated/` files changes what a consumer sees, D135 applies (`approval-record:d135-a-breaking-unit-leaves-the-batch`).

## Acceptance

- Each operator lands on struct, newtype and entity: `cargo test -p ess-gen --locked --test binary64_bounds a_strict_binary64_bound_becomes_an_exclusive_keyword`.
- `binary64_equality_becomes_minimum_and_maximum`: `cargo test -p ess-gen --locked --test binary64_bounds binary64_equality_becomes_minimum_and_maximum`.
- `a_literal_that_is_not_its_own_shortest_spelling_stays_an_annotation`, using `9007199254740993`: `cargo test -p ess-gen --locked --test binary64_bounds a_literal_that_is_not_its_own_shortest_spelling_stays_an_annotation`.
- `a_literal_on_the_left_lowers_as_its_mirror`: `cargo test -p ess-gen --locked --test binary64_bounds a_literal_on_the_left_lowers_as_its_mirror`.
- `a_validator_refuses_what_the_binary64_invariants_refuse`: 0, -0.0 and 1.5 refused; 0.5 and 1 admitted: `cargo test -p ess-gen --locked --test binary64_bounds a_validator_refuses_what_the_binary64_invariants_refuse`.
- Integer bytes unchanged: `cargo test -p ess-gen --locked --test integer_bounds` passes unmodified.
- `ess generate types` realizes a bounded Binary64 newtype in Rust, Go and TypeScript without refusal, and the bounds are obligations: `cargo test -p schema-contract --locked --test model_ergonomics issue_687_a_bounded_binary64_realizes_with_its_bounds_as_obligations`.

## Scope

- `crates/generate/ess-gen/src/types.rs`: Node bounds `:264-273` (new exclusiveMinimum/exclusiveMaximum, number-valued bound), integer_comparison `:1055-1074`, apply_bound `:1077-1118`, `is_integer` `:1045`, module table `:30`.
- `crates/generate/ess-gen/tests/binary64_bounds.rs`: new.
- `crates/generate/schema-contract/tests/model_ergonomics.rs`: one test.
- `docs/design/types-only-realizations.md` `:96-108`, `CHANGELOG.md`.
- Confidence: high. Defect site, number semantics and realize admission have all been read.
- Shared with: #688, #685, #686 (`types.rs` lowering and Node, `CHANGELOG.md`, design page); `model_ergonomics.rs` with #685 and #686.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
- `crates/generate/ess-gen/tests/agreement.rs` `:332-347` (inferred): the keywords this story publishes must be classified there, as the U1 entity-invariant story does for `minimum`, or `:861-891` fails on them.
