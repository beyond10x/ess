---
format: aep.planning-md/1
id: specification:existing-struct-generation
kind: specification
status: draft
title: Existing structural generation capabilities
revision: 1
---
## Evidence

- crates/generate/ess-synth/src/lib.rs:83 declares Rust, Go, Web and Clap targets; :271 dispatches target synthesis.
- crates/edge/ess-cli/src/schema.rs:16 and :39 expose a separate schema-to-TypeScript command.
- crates/generate/schema-contract/src/typescript.rs:65 projects structural declarations and explicitly excludes runtime validation.
- CLI 0.18.0 help exposes synthesize --target rust|go|web|clap and schema typescript --root --schemas <schema-id>.

## Finding

Go and Rust type generation already exists inside full system synthesis. TypeScript structural declarations already exist through an independent JSON Schema registry path. Do not file three new emitters as though none existed. The missing adopter capability is a consistent types-only selection and packaging contract across those paths, including exact semantics and target refusals.

This specification records existing capabilities, not a claim that arbitrary adopter schemas preserve all validation, aliases, defaults or serialization behavior. No generator implementation was changed by this finding.

