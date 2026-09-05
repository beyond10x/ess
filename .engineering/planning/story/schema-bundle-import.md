---
format: aep.planning-md/1
id: story:schema-bundle-import
kind: story
status: implemented
title: Import structural component bundles with explicit dialect accounting
relations:
- informed_by: story:review-openapi-semantic-accounting
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema.rs
- confidence: cited
  path: crates/edge/ess-cli/src/schema_bundle.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/schema_bundle.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/bundle.rs
- confidence: cited
  path: crates/generate/schema-contract/src/lib.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/bundle.rs
- confidence: cited
  path: docs/design/schema-bundle-import.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
revision: 5
---
## Outcome

Import a schema-only component bundle into the structural-contract path without
inventing HTTP operations, service metadata, lifecycle entities or a source dialect.
The result must support the types-only realization workflow while retaining exact
schema constraints and source provenance.

## Evidence

- `crates/generate/ess-openapi/src/lib.rs:468-485` accepts only OpenAPI 3.1 and
  requires nonempty service metadata, even for an envelope with no operations.
- `crates/generate/ess-openapi/src/lib.rs:723-877` admits a deliberately bounded
  interface subset: scalar types/references and closed objects. Type arrays,
  compositional schemas, tuple arrays and open objects need additional semantics.
- `crates/edge/ess-cli/src/schema.rs:16-56` exposes schema validation and TypeScript
  projection over a pre-existing registry, not component-bundle import.
- An adopter bundle with an OpenAPI 3.0 header, blank service metadata and newer
  JSON Schema component syntax is refused. Relabeling that envelope as a valid
  OpenAPI 3.1 service would conceal the source defect rather than map it.

## Required Contract

Keep the original document, its declared dialect and importer accounting distinct
from an explicitly selected structural-schema dialect. A schema-only extraction
must be an explicit command/mode with exact component roots and local-reference
mapping, not a permissive change to the existing service importer. Preserve
original source pointers and bytes/digests for diagnostics and provenance.

Cover nullable type arrays, enum/const values, allOf/anyOf/oneOf, object openness,
additional-property schemas, arrays including prefixItems tuples, boolean schemas,
local references, numeric/string/array constraints and default annotations. Defaults
are annotations, not automatic decoder behavior. Runtime aliases, flattening and
null-to-default behavior absent from the source remain separate explicit semantics.

Use the existing `InterfaceSchema` and schema-contract validation/projector boundaries
as design inputs, not authority to narrow the source. Settle the typed representation
and format compatibility in a binding design before implementation. Do not introduce
unrelated generic metadata bags or synthesize lifecycles to make schemas importable.

## Acceptance

A checked-in synthetic component bundle exercises every required construct above.
After an explicit schema-dialect selection, all selected components and their
reference closure are represented with equivalent validation behavior and no
unaccounted constraints. Wrong/missing dialect selection is refused, not guessed.
Import, persisted reload and target projection retain source qualification and
diagnostics; unsupported inputs never become an apparently complete successful
artifact. The existing OpenAPI service importer retains its strict behavior.

## Scope

- Cited: `crates/generate/ess-openapi`, `crates/generate/schema-contract`,
  `crates/edge/ess-cli/src/schema.rs` and `src/main.rs`.
- Inferred: typed schema-only import adapter, compatibility design and fixtures.
- Related accounting work remains owned by `story:review-openapi-semantic-accounting`;
  this story supplies schema-only input support, not a duplicate accounting redesign.
- No private adopter source, fabricated service, consumer cutover or application behavior.


## Integration Provenance

Reconciled through AEP from wt-46ef382d9f07 at original revision 5 and status implemented. Source artifact SHA-256: 9a133da4c38e45e68d8d68f0593cf936eb2b7fad5211f469b1922c73984baafd. Original journal history remains with its source recovery snapshot; this store records the reconciliation as new governed operations.
