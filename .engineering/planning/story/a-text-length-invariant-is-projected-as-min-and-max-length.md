---
format: aep.planning-md/3
id: story:a-text-length-invariant-is-projected-as-min-and-max-length
kind: story
status: draft
title: A text-length invariant (.count against a literal) is published as minLength and maxLength
summary: value.count and field.count bounds on a String lower to minLength and maxLength; utf8_bytes stays an annotation
refs:
- provider: github
  reference: beyond10x/ess#685
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d138-schema-invariants-as-scoped
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/text_length_bounds.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/model_ergonomics.rs
- confidence: cited
  path: docs/design/string-alphabet-and-length.md
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: inferred
  path: website/docs/guides/specify/fields-and-invariants.md
revision: 3
---
## Fit review

1. **Need.** A fixed-length or bounded text rule (`value.count == 64`, or `value.count >= 1` with `value.count <= 10`) is not enforced by the schema. Repro: a minimal specification outside the repository (not committed). Its generated newtype document has a pattern but no length bound, and a second newtype carries the same rule as an annotation. Requester's proposal: `minLength`/`maxLength` for a `value.count` literal bound on a String newtype; `utf8_bytes` annotation-only.
2. **Class.** Defect. The only documented refusal is a non-goal, `docs/design/string-alphabet-and-length.md:561-562` ("Invariants stay annotations in every schema this repository publishes"). Its premise was retired by #394, so the page is now wrong (`docs/design/types-only-realizations.md:96-101`). Strictly read, this is a gap; the decision is the same either way.
3. **Already expressible?** No. `pattern` "bounds no length" (`types.rs:806-807`).
4. **Fit.** `.count` on text is the number of Unicode scalar values (`facts.rs:1145-1162`). `minLength` counts characters, which `string-alphabet-and-length.md:213-215` equates with code points. A lone surrogate never reaches a Rust `str`, so this is exact for every text ESS admits. Siblings: a struct or entity field of primitive String (`name.count >= 1`) lowers onto its property. The map form `{gte, lte}` parses to `all` of comparisons (`ess-primitives/src/predicate.rs:3533-3544`), so the lowering reads the conjuncts of a top-level `all`. List/Map `.count` (`minItems`/`maxItems`/`minProperties`/`maxProperties`) is deferred to its own story: realize turns `minItems`/`maxItems` into the Array shape (`realize.rs:1517-1518`), and the Rust/Go behaviour for a bounded non-tuple array was not read. A field typed by a String newtype stays annotation only, because its schema is a `$ref` (`types.rs:652`). Realization already enforces `value.count` from the annotation (`TextConstraint`, `realize.rs:281-338`).
5. **Second adopter.** A country code newtype with `value.count == 2` over an uppercase alphabet. A validator should refuse `"D"`.
6. **Cost.** No format, no keyword. Generated schemas gain `minLength`/`maxLength`. `ess import openapi` records gaps (`ess-openapi/src/lib.rs:1158-1180`). The types-only report must not grow a duplicate obligation (see Decisions). No example uses `.count` (grep), so `generated/` is not expected to move.
7. **Alternatives.** (a) Change nothing. (b) As requested, newtype only: leaves struct and entity fields behind. (c) Chosen: newtype plus String fields, all forms.

## Decisions

- Accept, redesigned (2026-10-28). `<s>.count <op> <integer>`, where `s` is a newtype's `value` of String or a top-level primitive String field (required or Optional), lowers to:
  - `>=` minLength, `>` minLength+1, `<=` maxLength, `<` maxLength-1;
  - `==` both (on Optional and nullable fields too, since length keywords ignore non-strings);
  - a literal on the left is mirrored; conjuncts of a top-level `all` are read one by one;
  - negative results are clamped or skipped, never published.
- `utf8_bytes`, `!=`, dotted parents and newtype-typed fields stay `x-ess-invariants` only.
- Following the #592 precedent (`realize.rs:1222-1225`), on model input a minLength/maxLength equal to the node's TextConstraint adds no obligation beyond `model_invariants`.
- Amend `docs/design/string-alphabet-and-length.md:561-562` so the stale non-goal is removed.
- If moving the committed `generated/` files changes what a consumer sees, D135 applies (`approval-record:d135-a-breaking-unit-leaves-the-batch`).

## Acceptance

- `value.count == 64` gives minLength 64 and maxLength 64: `cargo test -p ess-gen --locked --test text_length_bounds a_count_equality_on_a_string_newtype_becomes_min_and_max_length`.
- `the_mapping_form_a_pair_and_a_mirrored_literal_project_alike`: `cargo test -p ess-gen --locked --test text_length_bounds the_mapping_form_a_pair_and_a_mirrored_literal_project_alike`.
- `a_count_bound_on_a_struct_or_entity_string_field_lands_on_its_property`: `cargo test -p ess-gen --locked --test text_length_bounds a_count_bound_on_a_struct_or_entity_string_field_lands_on_its_property`.
- `utf8_bytes_and_a_newtype_typed_field_stay_annotations`: `cargo test -p ess-gen --locked --test text_length_bounds utf8_bytes_and_a_newtype_typed_field_stay_annotations`.
- 64 × U+1F600 is admitted and 63 refused: `cargo test -p ess-gen --locked --test text_length_bounds a_validator_counts_scalar_values_as_the_model_does`.
- The types-only realizer reports the length once: `cargo test -p schema-contract --locked --test model_ergonomics issue_685_the_types_only_realizer_reports_the_length_once`.

## Scope

- `crates/generate/ess-gen/src/types.rs`: Node gains minLength/maxLength; the lowering gains the `.count` path and `all` conjuncts; newtype arm `:874-896`.
- `crates/generate/schema-contract/src/realize.rs` `:1180-1250`: no extra obligation for a TextConstraint-equal length.
- `crates/generate/ess-gen/tests/text_length_bounds.rs`, `crates/generate/schema-contract/tests/model_ergonomics.rs`: tests.
- `docs/design/string-alphabet-and-length.md` `:561`, `docs/design/types-only-realizations.md`, `website/docs/guides/specify/fields-and-invariants.md`, `CHANGELOG.md`. The first two are cited; the guide is inferred.
- Confidence: high.
- Shared with: #687, #688, #686 (`types.rs`, `CHANGELOG.md`, design page); `realize.rs` and `model_ergonomics.rs` with #686.
