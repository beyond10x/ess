---
format: aep.planning-md/1
id: story:binary64-structural-codecs
kind: story
status: active
title: Preserve finite Binary64 in standalone model data libraries
relations:
- derived_from: story:model-binary64-fields
- depends_on: story:model-binary64-fields
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/go.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/go_binary64.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/rust_binary64.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/binary64_structural.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/binary64_wire_tests.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/binary64_wire_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_binary64.rs
- confidence: inferred
  path: docs/design/binary64-structural-codecs.md
- confidence: cited
  path: docs/design/model-binary64.md
revision: 6
---
## Outcome

Emit standalone Rust and Go model data libraries with checked finite Binary64
wrappers and JSON codecs, while preserving the older non-Binary64 artifact maps.
This is distinct from whole-system synthesis and conformance, whose unsupported
Binary64 surfaces retain their located publication refusals.

## Evidence

The in-progress model-binary64-fields unit introduces compiler-owned Binary64
identity and executable normalization, with bounded structural Rust/Go refusals.
An all-type model selection therefore cannot emit those native libraries after
adding a Binary64 field. TypeScript structural obligations alone do not close
this native adoption boundary. The first unit must integrate before this scope
is refreshed and implemented; no successful codec is claimed here.

At published ESS6c78676, schema-contract realize/rust.rs maps numeric shapes to
serde_json::Number; realize/go.rs emits token-preserving EssNumber. The Rust
union decoder trials use serde_json::Value/from_value. A new finite wrapper must
preserve original numeric tokens through every path that reaches Binary64;
ordinary Value parsing can lose lexical negative-zero sign before construction.
Imported type:number or format:double annotations do not create model authority.

## Acceptance

A compiler-owned Binary64 model selection emits standalone Rust and Go libraries
whose checked construction and original-JSON decoding preserve finite nearest-even
values, signed zero, subnormals and signed underflow while refusing nonfinite and
wrong-kind inputs, including nested/union paths, with old non-Binary64 output maps
unchanged and every unsupported codec boundary explicitly reported.

## Boundaries

Use private finite value storage, a checked constructor and a read-only value
accessor. Do not expose a mutable raw float or serialize nonfinite values as null.
Reject quoted numbers, null and private-number object encodings at a numeric source
boundary. Preserve integral floating lexical identity in serialized output.

Rust must qualify raw-token decoding through optional/nullable fields, collections
and union trials. Do not claim that from_value can reconstruct a sign already lost
by earlier Value parsing; document unsupported non-text deserializers explicitly.
Conditional helpers and manifest features must preserve complete old generated
paths and bytes under the same generator version. Structural TypeScript remains
an obligation-bearing declaration, not an invented runtime decoder.

## Scope

Scope refreshed from frozen Binary64 unit bf16e504ccad68b2ee67607ba39606aadf07f627.
This is a read-only source audit, not an implementation or final integrated gate.
Refresh every cited seam after the dependency review and before dispatch.

The coordinator-owned story:normalization-followup-publication now owns
CHANGELOG.md and the shared formats/generate-artifacts guidance. Those three
files were removed from implementation write scopes because they have a separate
serial integration owner and acceptance, not because their work was dropped.
Planning and release metadata also remain coordinator-owned.

The bounded implementation write scope is:
- **cited** `crates/generate/schema-contract/src/realize.rs`.
- **cited** `crates/generate/schema-contract/src/realize/rust.rs`.
- **cited** `crates/generate/schema-contract/src/realize/go.rs`.
- **inferred** `crates/generate/schema-contract/src/realize/rust_binary64.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/go_binary64.go.txt`.
- **inferred** `crates/generate/schema-contract/tests/binary64_structural.rs`.
- **inferred** `crates/generate/schema-contract/tests/fixtures/binary64_wire_tests.rs.txt`.
- **inferred** `crates/generate/schema-contract/tests/fixtures/binary64_wire_tests.go.txt`.
- **cited** `crates/generate/schema-contract/tests/normalization_binary64.rs`.
- **cited** `crates/edge/ess-cli/tests/model_types.rs`.
- **cited** `docs/design/model-binary64.md`.
- **inferred** `docs/design/binary64-structural-codecs.md`.

The source audit found no need to mutate shared Shape, compiler authority or CLI
main dispatch. The following symbol constraints preserve the other unit's read
dependencies; a necessary widening requires re-scoping before overlapping writes.

### crates/generate/schema-contract/src/realize.rs

May edit: Plan::binary64_codec (322); Plan::rust (335); Plan::go (341); target-specific accounting in Plan::report (346), if not handled in emitters; new codec reachability helper.

Preserve: Report fields (22); InputIdentity (38); TargetConfiguration (53); Plan fields and binary64 inventory meaning (93); Node (107); Shape/Shape::Array (113/127); Field (149); Plan::from_bundle (166); Plan::from_model (207); Plan::build (236); Builder::kind array lowering (673); children (755); reference/recursion checking.

Normalization reads the shared shapes, definitions and compiler inventory, and constructs Types through from_model/from_bundle. It does not call structural Plan::rust/go/report.

### crates/generate/schema-contract/src/realize/rust.rs

May edit: emit (17), conditional manifest/helpers and reservations; Emitter::account (69), target-only finite obligation accounting; Emitter::named (93), record (160), union (217), ty (234), anonymous (256) where needed for affected layouts.

Preserve: package_name (315); field_name (285) and existing no-Binary64 spelling behavior; unconditional HELPERS bytes; ordinary no-Binary64 output path.

normalize/target.rs::rust_package calls structural rust::package_name; no positional edit needs Emitter or its output.

### crates/generate/schema-contract/src/realize/go.rs

May edit: emit (14), conditional helper/import/name handling; Emitter::account (75), target-only finite obligation accounting; Emitter::ty (340) and necessary affected named allocation wiring.

Preserve: package_name (414); module_name (451); existing record/raw field/list/tuple/union decode behavior; unconditional HELPERS bytes and imported general numbers.

normalize/go_target.rs::generate calls package_name/module_name. Conditional finite mapping uses the existing raw-token container path.

### crates/generate/schema-contract/tests/normalization_binary64.rs

May edit: compiler_metadata_and_structural_refusals_cannot_be_minted_by_schema_annotations (195): replace structural refusal expectations and rename if appropriate.

Preserve: compiler metadata assertions; imported annotation cannot mint identity assertions; TypeScript finite-codec obligation assertion; all other tests, including inaccessible_map_and_union_input_policies_refuse_before_generation (282).

The frozen test asserts the blanket structural refusal that this unit removes. Positional does not own this file, despite its normalization filename.

### docs/design/model-binary64.md

May edit: Implemented boundary and compatibility: standalone structural Rust/Go refusal paragraph at 199–209.

Preserve: whole-system synthesis and conformance refusals; format-5 numeric semantics, compiler authority and map-key refusal.

Structural changes only the completed standalone target boundary. Root edits shared public summaries.

## Scheduling

Depends on model-binary64-fields. This separate bounded follow-up is necessary
before a complete adopter model containing Binary64 can keep its standalone
three-language generation path. It overlaps shared structural source and native
fixture infrastructure; do not infer disjointness from separate language names.
The coordinator alone owns planning, integration and publication.
