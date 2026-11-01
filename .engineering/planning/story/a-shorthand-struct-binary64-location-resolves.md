---
format: aep.planning-md/3
id: story:a-shorthand-struct-binary64-location-resolves
kind: story
status: draft
title: A shorthand struct's Binary64 locations resolve in the published schema
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by adversary pass 1 of wave D350 U1 (`review-result:adversary-d350-u1-pass-1`, F3, out of
scope, not probed): a struct with a shorthand (`ess/27`) is published as `oneOf [short, object]`
(`crates/generate/ess-gen/src/types.rs:995-1003`), but `body_numeric_locations`
(`crates/generate/ess-gen/src/model_types.rs:529-549`) still points its Binary64 fields at
`/$defs/X/properties/...`, so those locations may not resolve in the published schema.

## Acceptance

- A probe first shows whether a shorthand struct with a Binary64 field realizes and decodes in
  Rust, Go and TypeScript; if it does not, its Binary64 locations name the object member of the
  `oneOf` and the struct realizes, or it is refused by name.
