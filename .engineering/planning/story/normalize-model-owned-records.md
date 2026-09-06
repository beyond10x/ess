---
format: aep.planning-md/1
id: story:normalize-model-owned-records
kind: story
status: implemented
title: Normalize model-owned records without duplicating their schemas
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/source.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_model.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_go.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_model.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_rust.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 10
---
## Evidence

Normalization Plan currently selects roots only from qualified schema Bundles.
ESS model projections carry x-ess-provenance, x-ess-name and x-ess-kind, with
additional model wire/nominal/invariant annotations where applicable. Importing
an ESS-generated model schema through generate schema import-document refuses
those keywords before a normalization source can be formed. This was verified
against the current CLI, not inferred from a type declaration.

The structural realization Plan::from_model already understands model-owned
identity. The normalization boundary must reuse that authority rather than
requiring an adopter to hand-author a second schema or strip annotations from a
generated file. Both would separate the conversion contract from its typed home.

## Outcome

Normalize between model-owned records and qualified source contracts with exact
source identity, preserving model names, wire semantics and explicit obligations.

## Acceptance

- Establish the persisted root/source identity and compatibility consequences
  in the binding design before adding a normalization reader or CLI source option.
- Consume a checked model projection through an explicit typed API; do not accept
  unverified x-ess claims as compiler-minted authority or blanket-ignore unknown
  schema annotations.
- Preserve source model identity and selected root closure in canonical recipes,
  standalone target reports and every generated target's retained source inputs.
- Preserve model newtypes, wire mappings and invariant obligations. Structural
  validity alone is not proof that a model invariant was executed.
- Generate and run a source-to-model or model-to-model normalization without a
  hand-maintained duplicate schema, and refuse stale/mismatched source identity.
- Existing bundle-owned normalization recipes and their canonical bytes retain
  their declared compatibility guarantees.

## Relation

This closes the model-owned input/output boundary required by
source-pinned-data-normalization and uses the existing structural model realization
capability. It does not introduce a generic facet registry or merge unrelated IRs.

## Model Enum Projection Finding

The actual adopter conversion reached a further model-owned shape refusal: a
compiler-projected string enum carries both type:string and enum:[...]. The
shared structural plan correctly retains their intersection, but normalization
previously refused every intersection, including this finite model enum.

This is part of the model-owned-records boundary, not permission to flatten
arbitrary allOf or imported intersection shapes. Admit only the checked model's
string-enum projection, retain each literal and validate membership at stage
boundaries. General intersection and tuple mapping remain explicit refusals.

## Implemented Model-Owned Normalization

The binding design now defines ess-normalization/3 and its model root identity:
system, specification version, resolved source digest, contract digest, projection
digest and exact selected root set. Root::pin_model admits only explicitly
selected roots; Plan::check_with_models accepts sealed ModelTypes selections
beside checked bundles. No imported annotation or parallel authored schema is
accepted as compiler authority. Existing bundle-only recipe bytes are unchanged.

Model wire schemas retain provenance, nominal definitions, requiredness, field
renames, constraints and optional-property semantics. The checked model's finite
string-enum projection is handled without flattening arbitrary intersections.
Selected model invariant statements refuse planning until an evaluator exists.

Rust and Go target reports use ess-normalization-target/2 for version 3 recipes,
retain each complete selected model projection and identify every root schema.
Root schema identifiers include complete selection identity: distinct root sets
that share projection bytes cannot collide in the Go validator registry. Source
projection bytes may still be retained once when they are identical.

The CLI accepts repeated --model inputs with or without bundles, compiles them,
recreates the declared selections and checks every identity coordinate. Model
input files and directory trees are protected from generated or canonical output.
Stale identities, closure-only roots and old recipe versions refuse explicitly.

Verification on the implemented tree:
- Full offline task check exits zero; task site-build exits zero.
- Six model normalization checks cover wire mappings, presence, every identity
  coordinate, old-reader/version refusal, invariant refusal, enum membership,
  and qualified-source-to-model normalization.
- Native Go 1.26.5 runs old, ordered, numeric and model recipes with -race,
  including two distinct selections sharing the same projection. Four tests pass.
- Five Rust target tests pass, including model records and both serde_json feature
  configurations. The old version 1 canonical recipe digest remains unchanged.
- Twelve CLI normalization tests pass, including model acquisition, source-tree
  protection, generation/check and stale model refusals. git diff --check is clean.

An adopter's actual decoded transfer conversion now checks and runs from its
model, and a generated standalone Rust adapter passes the source-grounded mapping
case. Its Go generation correctly remains refused at the model Bytes base64
pattern, tracked by go-normalization-pattern-semantics. Adopter data and evidence
remain outside this public store.

This closes model-owned normalization, not the complete parent objective.
TypeScript normalization, selected lexical JSON capture, bounded Go pattern
qualification and remaining source-driven mappings still require work and a
subsequent integrated release.
