---
format: aep.planning-md/3
id: story:a-reader-side-conformance-admits-reader-widening
kind: story
status: active
title: A consumer's reader-side conformance admits the widening a reader may do
refs:
- provider: github
  reference: beyond10x/ess#191
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/adversary_compose_conformance.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/owner-consumer
- confidence: cited
  path: crates/specify/ess-composition/tests/owner_types.rs
- confidence: cited
  path: docs/design/composition-type-conformance.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T11:29:55Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-09-28T11:29:56Z", actor: "human:timo", revision: 13}
---
# Story: a consumer's reader-side conformance admits the widening a reader may do

## Outcome

A consumer that mirrors a producer type to read it off the wire can assert that mirror with
`conformances:` even where it reads more loosely than the producer writes: a newtype by its
primitive, an enum as `String`, enum variants spelled differently with the same wire name, an
opaque JSON value as a structure that accepts any JSON, and a subset of the producer's fields.
Anything that could reject a value the producer can send stays `type_conformance_drift`.

## Why

beyond10x/ess#191 (operator, 2026-09-28): in one real consumer 14 of 15 mirrored types are
refused by `ess-composition/2` `conformances:` (#162) for these five reader-side differences, so
field-by-field scripts remain the only check.

## Design (coordinator decisions, 2026-09-28)

1. A per-entry `reader: true` on a `conformances:` entry, in a new source format
   `ess-composition/3`; `/1` and `/2` refuse an authored `reader` key whatever its value, `null`
   included (the released `/2` keeps its meaning and bytes). `/3` is registered in `FORMAT_RELEASES`
   for the next release.
2. Newtype read as its primitive: through any chain of newtypes to the same primitive;
   `alphabet`/`prefix` stay uncompared (the known limit already admits a stricter consumer).
3. Enum read as `String`: a required producer enum may be read as `String` or `Optional<String>`;
   an `Optional` producer enum only as `Optional<String>`.
4. Variants compared by wire name: the consumer's wire set must include every producer wire name
   (a superset cannot reject a producer value); a producer variant whose wire name the consumer
   lacks is drift.
5. JSON read as a map — revised 2026-09-28 after adversary pass 1 (`review-result:adversary-n-reader-pass-1`, F1): an object-shaped producer (a struct, or a map with `String` keys) may be read as `Map<String, Json>`; a producer `Json` (bare, or through a newtype, `Optional`, `List` or `Map`) read as a map is drift, because a map rejects arrays and scalars. The issue's `Map<String, String>` example stays drift.
6. Field subset: the consumer may omit producer fields; a field the consumer requires that the
   producer marks optional or lacks is drift; an extra consumer field is matched by wire name against every producer field, omitted or renamed ones included, and compared by type (F2). `reader: true` asserts that the consumer's reader ignores unknown fields; ESS-generated closed types (`additionalProperties: false`, `deny_unknown_fields`) do not, so a consumer reading through them must not use `reader` for a subset (F4).

## Acceptance

- Each of the five widenings conforms under `reader: true` in `/3`, one test per case, on the
  owner/consumer fixtures.
- For each, the narrowing counterpart stays `type_conformance_drift` (a narrower primitive, a
  missing producer wire name, a consumer-required field the producer may omit, a map that rejects
  some JSON).
- Without `reader: true` nothing changes; `/2` refuses `reader:`; formats.md, spec-versions.md,
  the composition design page and CHANGELOG describe `/3`.

## Scope

From `story-scoper` on `7b21e59d1`, 2026-09-28 — cited unless marked.

- `crates/specify/ess-composition/src/conformance.rs` (`Comparison::bodies`/`references`, `variant_spelling`)
- `crates/specify/ess-composition/src/lib.rs` (`TypeConformance` :331, `CONFORMANCE_COMPOSITION_FORMAT` :44, `SUPPORTED_COMPOSITION_FORMATS` :47, `validate_format` :1279)
- `crates/specify/ess-composition/tests/owner_types.rs`; fixtures `tests/fixtures/owner-consumer/` (inferred); `tests/adversary_compose_conformance.rs` (inferred)
- `crates/edge/ess-xtask/src/docs.rs` `FORMAT_RELEASES` (inferred)
- `docs/design/composition-type-conformance.md`, `docs/design/review-format-catalog.md`, `website/docs/reference/formats.md`, `website/docs/reference/spec-versions.md`
