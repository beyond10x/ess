---
format: aep.planning-md/1
id: story:types-only-realizations
kind: story
status: active
title: Consistent types-only realizations for Go Rust and TypeScript
relations:
- informed_by: specification:existing-struct-generation
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/model_types.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema_bundle.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/command_surface.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/model_types.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/schema_bundle.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/schema.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/fixtures/model-types.yaml
- confidence: inferred
  path: crates/generate/ess-gen/tests/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 12
---
## Evidence

- crates/generate/ess-synth/src/lib.rs:83,271 defines target selection and full synthesis.
- crates/generate/ess-synth/src/go/items.rs owns Go type emission; crates/generate/ess-synth/src/rust owns Rust emission.
- crates/generate/schema-contract/src/typescript.rs:65 and crates/edge/ess-cli/src/schema.rs:39 expose a separate TypeScript input/CLI path.
- examples/billing/domains/invoice.yaml provides existing typed nouns; this work extends realization, not domain identity.

## Outcome

An adopter can select named ESS type roots and obtain deterministic, consumable Go, Rust and TypeScript type libraries without generating handlers, services, lifecycle executors or a transport application.

## Proposed Contract

Reuse resolved ESS type identity and existing emitter logic. Include each selected root's complete transitive type closure, preserving cross-domain references and deterministic ordering. Define package/module identity and naming collision policy explicitly. Provide provenance naming source/model digest, generator version, target and options. A types-only result must not imply business behavior or runtime validation.

Publish an explicit target mapping for primitive precision/range, nominal identities, enums, records, lists/maps, field wire names, optional versus nullable values and union discrimination. Distinguish target-language syntax from JSON wire identity. Defaults, read aliases, flattened fields, untagged dispatch and unknown-field policy must either have concrete supported semantics or yield named source-located obligations/refusals; never silently discard them while claiming parity. JSON Schema import coverage is not authority to invent ESS lifecycle semantics.

The interface spelling is a design decision, not an invented supported command in this story. Extend the existing generation surface after reviewing compatibility. Preserve existing full synthesis behavior. TypeScript is a peer types realization, not the existing Web target renamed.

## Acceptance

For the declared supported subset, one selected root set yields standalone compileable libraries in Go, Rust and TypeScript with deterministic names/provenance and equivalent documented wire shapes, while unsupported semantics produce explicit diagnostics before partial successful output is claimed.

## Scope

- Cited: crates/generate/ess-synth, crates/generate/schema-contract, crates/edge/ess-cli/src.
- Inferred: a shared type-selection/mapping layer or a bounded types-only mode; choose in the binding design before coding.
- No application runtime, infrastructure deployment, credential selection, adopter-specific names or automatic consumer migration.
- Verification focuses on compiler acceptance, structural serialization boundaries and determinism; it is not application conformance.

## Qualified Structural Inputs

`story:schema-bundle-import` provides a locally implemented input boundary: `crates/generate/schema-contract/src/bundle.rs` retains original source bytes and dialect claims, explicit selected dialect, named roots, component closure and adapter accounting. Realization consumes the sealed, replay-checked `Bundle`, not independently deserialized wire definitions. `story:schema-document-root-import` now also preserves a complete JSON Schema root and local definitions in a versioned document bundle.

Both imported contracts and resolved ESS type roots use one package, naming, deterministic provenance and target accounting contract across Go, Rust and TypeScript. Structural import coverage does not prove language realization coverage. The separate legacy TypeScript projector retains its older supported subset; the accounted types-only path checks siblings, combinators, tuples and object openness explicitly.

Earlier planning inferred that a missing enclosing-record field necessarily required an ESS-model-field-to-imported-root binding. Source review established a different case: the exporter had dropped the actual enclosing schema root, and a similarly named runtime descriptor represented already-normalized data. Retaining the source root restores that existing typed field. Do not bind a normalized field to a pre-normalization contract merely because their names resemble each other.

An ESS record still cannot name an imported structural root directly as a field type. Any future binding must establish equal semantic representation, source digest, root identity and closure ownership in a concrete design. This remains an unimplemented capability, not a substitute for recovering existing source structure or modeling an actual transformation. Decoder aliases, null-to-default coercion and external dispatch remain separate from JSON Schema annotations and structural validation and still need supported semantics or explicit source-located accounting.

Domain-semantic synthesis and JSON data realization remain distinct: existing Go newtypes have private storage and default JSON encoding is not the primitive wire value. Extracting declarations from full synthesis is not proof of equivalent serialization.

## Implementation Design

`docs/design/types-only-realizations.md` binds the shared qualified structural plan,
root closure, naming, structural siblings, target accounting and publication surface.
Implement the shared layer and language emitters under this story; TypeScript alone
does not satisfy its Go/Rust/TypeScript acceptance. Preserve the separate full
synthesis and legacy TypeScript command while adding the accounted bundle path.

## Model Root Realization

The local implementation supports explicit resolved model roots and all-type selection through `ess generate types`. `crates/generate/ess-gen/src/model_types.rs` seals the selected closure and reuses the existing `schema::types` wire mapping; `schema-contract::realize::Plan` consumes it alongside qualified bundles. No synthetic component or service import is used.

The `ess-types-report/3` envelope distinguishes model system/version/source/contract/projection identity from qualified-bundle identity. `source.schema.json` retains selected model definitions; `source.bundle.json` remains specific to imported bundles. Model invariants and map-key grammars remain runtime obligations, and TypeScript nominal-identity weakening is explicit per newtype. Native typed wire fixtures cover all primitives, recursive records, wire renames, field absence, collection nullability and adjacent tagged unions.

Current command and library gates pass, but this story remains active: source-pinned application decoder semantics and normalization are not implemented. Do not equate two separately generated libraries with a checked transformation or binding. The original missing-envelope case is addressed by retaining its schema document root under `story:schema-document-root-import`; the qualified-input section records why directly binding a normalized descriptor would have been incorrect. The broader compiler wire-key collision gap remains `story:unique-wire-field-identity`; model realization already refuses collisions locally.

## Wire Identity And Normalization Follow-up

`story:unique-wire-field-identity` is now locally implemented: specification validation checks every field-bearing object namespace before compilation and ordinary projections, while the selected-model guard remains defense in depth. The full workspace gate and documentation build pass.

The remaining decoder/normalization capability is concretely tracked by `story:source-pinned-data-normalization`: separately typed stored and normalized representations, explicit source provenance and ordered decoding/defaulting/validation/branch/conversion semantics. This story remains active; recording the follow-up does not establish a working adapter or shrink the required end state to structural type emission.
