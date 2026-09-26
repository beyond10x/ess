---
format: aep.planning-md/2
id: story:schema-describes-serialized-strings
kind: story
status: draft
title: The generated schema describes ExternalRef and Period as the strings they serialize to
relations:
- serves: vision:O2
revision: 1
---
# The generated schema describes ExternalRef and Period as the strings they serialize to

## Outcome

`schemas/generated/ess.schema.json` describes `ExternalRef` as a `provider:key` string and `Period` as an ISO
`PT…S` duration string, which is what ESS serializes. A document that validates against the schema is one ESS
reads, and the reverse.

## Why

`ExternalRef` serializes as a string, while the generated schema (`schemas/generated/ess.schema.json` about line
400) still describes an object with `provider` and `reference` fields. A fix was written on 2026-09-15 in the
retired ESS-evolution scope tree and never landed: hand-written JSON schemas in `crates/specify/ess-domain/src/refs.rs`
(+40), `crates/specify/ess-domain/src/binding/periodic.rs` (+149) and `crates/verify/ess-conformance/src/web_replay.rs`
(+55), with tests and a `jsonschema` dev-dependency. The design note is
`docs/design/consumer-wire-behavior-period-parity.md`. The code is kept in the archived bundle
`.ess-evolution/archive/20260926-old-trees/` (patch `ess-scope-dirty.patch`).

## Acceptance

- The schema for both types is generated, not copied: regenerate with the repository's schema xtask.
- A test validates the serialized form of each type against the generated schema, and fails when the schema
  describes an object.
