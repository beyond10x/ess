---
format: aep.planning-md/3
id: story:a-distinct-invariant-is-projected-into-its-schema
kind: story
status: active
title: 'A distinct: invariant is asserted by the schema where JSON Schema says it exactly, and annotated with a documented limit where it cannot'
summary: distinct over scalars gives uniqueItems; by an enum or Boolean key gives one bounded contains per variant; an open key stays x-ess-invariants
refs:
- provider: github
  reference: beyond10x/ess#686
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d138-schema-invariants-as-scoped
- depends_on: story:an-entity-invariant-is-projected-into-its-schema
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/agreement.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/distinct_projection.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/model_ergonomics.rs
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: inferred
  path: website/docs/guides/specify/fields-and-invariants.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:21Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:22Z", actor: "human:timo", revision: 7}
---
## Fit review

1. **Need.** A list whose elements must not share a key is not enforced by the schema: two `Train` splits, or a repeated token. Repro: a minimal specification outside the repository (not committed). Its generated list-bearing struct document has the rule as an annotation only, with no `uniqueItems`. The entity case has nothing at all (#688). Requester's proposal: `uniqueItems` for scalars; per-variant `contains` with `minContains: 0`, `maxContains: 1` for an enum key; annotation plus documented limit for an open key.
2. **Class.** Defect, by the module's principle that a contract's job is refusing and a keyword beats an annotation where exact (`types.rs:61-65`). Distinct is evaluated today (`ess-primitives/src/predicate.rs:2032-2051`), but no keyword expresses it.
3. **Already expressible?** No. No authored construct puts uniqueness into the projection.
4. **Fit.** Every key kind (`predicate.rs:1948-1963`) has equality at least as coarse as JSON equality, so `uniqueItems` never refuses a list ESS admits. It is exact for Boolean, Integer, String, Uuid and Enum, and sound but incomplete for Decimal (strings `"1"` and `"1.0"`) and Timestamp (two spellings of one instant). The requester's enum form is unsound as written: a `contains` without `type: object` and `required` counts an element lacking the key. Proven on the 0.74.0 validator, outside the repository: `naive: [{}, {}]` is refused, while ESS gives an absent key Unknown (`predicate.rs:2037-2038`). The corrected form admits role-less elements, scalar short forms and empty lists, and refuses `[Train, HeldOut, Train]`. Boolean is a finite key too, so it is a sibling and is included. An open key has no JSON Schema 2020-12 keyword. Realization: on model input `contains` is refused (`realize.rs:1251-1253`), and `allOf` becomes `Shape::Intersection` (`realize.rs:1390-1408`), which Rust and Go refuse (`realize/rust.rs:400-405`, `realize/go.rs:153-158`). The struct reaches realize through `types::body` (`model_types.rs:119`), so `ess generate types` would refuse every struct that uses the enum form unless realize changes. Bundle import already accepts and walks `contains` (`bundle.rs:597`, `:604-605`). Bundle realization keeps refusing it, as it refuses `x-ess-*` (`bundle.rs:607-613`).
5. **Second adopter.** An order's `lines` must not repeat a `sku` (open key, annotation only). A schedule's `slots` hold at most one per `weekday` enum (asserted).
6. **Cost.** No format, no new authored keyword, and no new `x-ess-*` annotation: `x-ess-invariants` is the annotation, and a new `x-ess` key would need `bundle.rs` and `realize.rs` to admit it. Node gains five keywords. `ess import openapi` records gaps. Realize changes for model input only. No example uses `distinct` (grep), so `generated/` is not expected to move.
7. **Alternatives.** (a) Change nothing. (b) As requested: refuses data ESS admits. (c) A new `x-ess-distinct` annotation: adds surface for what `x-ess-invariants` already carries. (d) Chosen: the corrected per-variant form, Boolean added, open key documented.

## Decisions

- Accept, redesigned (2026-10-28). For `distinct: {in: F, as: x, by?: x.m1…mk}` where F is a top-level List field of the struct or entity, or a List newtype's `value`, required or Optional, with a non-Optional element:
  - No `by`: `"uniqueItems": true` on F's node.
  - `by` through required struct members to an Enum (through newtypes) or a Boolean: on F's node, `"allOf": [ {"contains": C_w, "minContains": 0, "maxContains": 1} … ]`, one entry per variant wire spelling in declaration order (`true`, `false` for Boolean). C_w nests `{"type":"object","properties":{<wire m_i>: …},"required":[<wire m_i>]}` per segment, ending in `{"const": w}`.
  - Any Optional on the key path: skipped, stays annotation.
  - An open key (String, Integer, Decimal, Uuid, Timestamp): `x-ess-invariants` only, and documented as the limit. Decimal and Timestamp element-equality incompleteness is documented the same way.
- Changed from the request: `type: object` and `required` added; Boolean added; no new annotation keyword.
- realize.rs, model input only: an `allOf` whose every branch holds only `contains`/`minContains`/`maxContains` adds no shape term and records one runtime obligation at `<pointer>/allOf`. Bundle input is unchanged.
- If moving the committed `generated/` files changes what a consumer sees, D135 applies (`approval-record:d135-a-breaking-unit-leaves-the-batch`).

## Acceptance

- A distinct list of scalars is unique items: `cargo test -p ess-gen --locked --test distinct_projection a_distinct_list_of_scalars_is_unique_items`.
- `a_distinct_by_an_enum_key_is_one_bounded_contains_per_variant` (exact JSON, nested key path, entity and struct): `cargo test -p ess-gen --locked --test distinct_projection a_distinct_by_an_enum_key_is_one_bounded_contains_per_variant`.
- `a_distinct_by_a_boolean_key_is_bounded_like_an_enum`: `cargo test -p ess-gen --locked --test distinct_projection a_distinct_by_a_boolean_key_is_bounded_like_an_enum`.
- `an_open_key_or_an_optional_path_stays_an_annotation`: `cargo test -p ess-gen --locked --test distinct_projection an_open_key_or_an_optional_path_stays_an_annotation`.
- `a_validator_refuses_two_elements_of_one_variant_and_admits_keyless_ones`: `cargo test -p ess-gen --locked --test distinct_projection a_validator_refuses_two_elements_of_one_variant_and_admits_keyless_ones`.
- Rust, Go and TypeScript realize a distinct-by-enum struct with no refusal, and it carries one obligation: `cargo test -p schema-contract --locked --test model_ergonomics issue_686_a_distinct_by_enum_struct_realizes_with_one_obligation`.
- `a_bundle_with_contains_is_still_refused`: `cargo test -p schema-contract --locked --test model_ergonomics a_bundle_with_contains_is_still_refused`.
- The new keywords are classified as assertions: `cargo test -p ess-gen --locked --test agreement`.

## Scope

- `crates/generate/ess-gen/src/types.rs`: Node gains uniqueItems, allOf, contains, minContains, maxContains; distinct lowering beside the bounds; module doc limit.
- `crates/generate/schema-contract/src/realize.rs` `:1180-1253` and `:1390-1408`: model-only allOf of contains.
- `crates/generate/ess-gen/tests/distinct_projection.rs`, `crates/generate/schema-contract/tests/model_ergonomics.rs`, `crates/generate/ess-gen/tests/agreement.rs` `:332`. The new file is inferred; the others are cited.
- `docs/design/types-only-realizations.md`, `website/docs/guides/specify/fields-and-invariants.md`, `CHANGELOG.md`: the documented open-key limit. The guide is inferred.
- Confidence: medium. The schema form is proven; the realize collapse is designed but not run.
- Shared with: #688, #687, #685 (`types.rs`, `CHANGELOG.md`, design page); `realize.rs` and `model_ergonomics.rs` with #685.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
- Shared with U2 (`story:defined-and-not-defined-branches-are-exhaustive`, lines 82-99 of `website/docs/guides/specify/fields-and-invariants.md`): this story adds to the schema-projection prose of the same page and leaves those lines alone. On a conflict the integration branch keeps both edits.
