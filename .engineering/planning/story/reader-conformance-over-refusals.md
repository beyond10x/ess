---
format: aep.planning-md/3
id: story:reader-conformance-over-refusals
kind: story
status: draft
title: Reader-side conformance does not refuse what a reader accepts
refs:
- provider: github
  reference: beyond10x/ess#191
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/tests/adversary_reader_conformance_pass2.rs
revision: 3
---
# Story: reader-side conformance does not refuse what a reader accepts

## Outcome

Under `reader: true`, two consumer shapes that accept every value the producer sends are admitted instead of refused as `type_conformance_drift`.

## Why

Adversary pass 2 on the #191 unit (`review-result:adversary-n-reader-pass-2`), both notes, fail-safe:

- `crates/specify/ess-composition/src/conformance.rs:386` `object_shaped` counts only `String`-keyed maps; `Map<Integer, _>` (and other non-String keys) travels as a JSON object with text keys, so it can be read as `Map<String, Json>`.
- `conformance.rs:160`: a local field sharing a producer field name is matched by name and refused for its wire name, while extra fields are matched by wire name; a local field that reads another producer field's key with a conforming type is refused.

## Acceptance

- The two cases in `crates/specify/ess-composition/tests/adversary_reader_conformance_pass2.rs` that assert today's refusal (`reader_reads_an_integer_keyed_map_as_a_map_of_json`, `reader_matches_a_shared_field_name_by_wire_name_as_it_does_an_extra_field`) are flipped to admission, and every existing reader drift case stays drift.

## Scope

- `crates/specify/ess-composition/src/conformance.rs` — cited
- `crates/specify/ess-composition/tests/adversary_reader_conformance_pass2.rs` — cited
