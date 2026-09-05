---
format: aep.planning-md/1
id: story:schema-document-root-import
kind: story
status: implemented
title: Retain the root of an imported JSON Schema document
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema_bundle.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/schema_bundle.rs
- confidence: cited
  path: crates/generate/schema-contract/src/bundle.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/bundle.rs
- confidence: cited
  path: docs/design/schema-document-root-import.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 5
---
## Evidence

The existing bundle importer in crates/generate/schema-contract/src/bundle.rs::import selects only /components/schemas. A root JSON Schema object with typed properties outside that map has no admitted input path. Keeping only definitions drops a real data record; manually copying it into a fabricated service is not an import.

## Outcome

An adopter imports a complete JSON Schema document root and its transitive local definitions with original-byte retention, explicit dialect and checked source identity, and consumes the result through the existing schema and data-library projections.

## Acceptance

The document root and selected definitions survive import, persisted replay, root projection and Go/Rust/TypeScript realization; incompatible dialects, root-name collisions, unsupported references and tampered provenance refuse without partial output. Existing component-bundle v1 bytes remain unchanged, and old strict readers refuse the explicit v2 document-root envelope.

## Design And Scope

docs/design/schema-document-root-import.md binds root identity, local reference rules and v1/v2 persistence. Scope: schema-contract bundle and realization input locations, CLI import-document, targeted library/CLI tests, public generation docs and changelog. No runtime decoder coercion or fabricated domain lifecycle. This is one standalone gap under the existing source-import and types-only work, not an epic decomposition.

## Local Implementation Evidence

The sealed document importer retains the original root alongside its local definition closure. `ess-schema-bundle/2` carries document-root identity; old component imports retain `/1` bytes exactly. Replay checks the complete envelope. Root/definition source pointers survive type planning; unsupported references, contradictory dialects, name collisions and output/source replacement refuse.

`cargo test --offline -p schema-contract --test bundle -p ess-cli --test schema_bundle --test command_surface` passed: 12 bundle tests, 8 CLI bundle tests and 5 command-surface tests. Generic document fixtures exercise root recursion, required/nullable fields, escaped definition names, boolean roots, schema projection and all three type targets. Strict old-reader and tampered-envelope cases passed.

`CARGO_NET_OFFLINE=true task check` exited 0, covering workspace format, strict Clippy, tests, rustdoc, smoke and boundary checks. `task site-build` exited 0; npm still reports 30 existing dependency vulnerabilities (9 moderate, 21 high), not repaired by this feature. This is local unreleased implementation evidence, not a release or deployment claim.


## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 6 and status implemented. Source artifact SHA-256: 8af9bcfa1728393f663a843fed66a24af8ef7398049f3add7079d59136090f1e. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.
