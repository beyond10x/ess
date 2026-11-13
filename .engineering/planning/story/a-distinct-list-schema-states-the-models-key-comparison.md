---
format: aep.planning-md/3
id: story:a-distinct-list-schema-states-the-models-key-comparison
kind: story
status: draft
title: A distinct list's schema states the model's key comparison
summary: 'Follow-up of story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant: uniqueItems compares JSON values, so the schema refuses identical out-of-range instants the model admits and admits two spellings of one Decimal the model refuses (adversary pass 1, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

The published JSON Schema of a `distinct:` list uses `uniqueItems`
(`crates/generate/ess-gen/src/types.rs`, ~1498), which compares JSON values. It therefore refuses
a list with two identical out-of-range instants that the model admits, and it admits two
spellings of one Decimal or one instant that the model refuses. Found by adversary pass 1 of
story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant (pre-existing).

## Acceptance

1. The schema of a `distinct:` list either states the model's key comparison or does not claim
   `uniqueItems` where it differs from it; the reference page says which, and a test pins one
   case of each direction.
2. A `distinct:` list over a key kind whose JSON equality is the model's keeps its schema bytes.
