---
format: aep.planning-md/1
id: story:raw-json-normalization-provenance
kind: story
status: draft
title: Preserve declared raw JSON token bytes during normalization
relations:
- derived_from: story:source-pinned-data-normalization
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/execute.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_input.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/input.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_raw_json.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_go.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_raw_json.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_rust.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 4
---
## Evidence

Source-driven settings normalization found a concrete lexical boundary: an
embedded JSON document is copied as raw token bytes and those bytes contribute
to a later cache identity. Whitespace, member order and numeric spellings remain
observable even when the parsed JSON values are equal. Absent raw data and an
explicit JSON null also differ.

The current normalization input/evaluation boundary produces serde_json::Value
and emits canonical JSON. The closed recipe operations do not capture original
token bytes. Copying a parsed value therefore does not preserve this source
contract. A Bytes model type can describe retained bytes but does not implement
their extraction from the enclosing input document.

## Outcome

Provide explicit, source-pinned lexical JSON preservation where the source
contract requires it, without treating arbitrary JSON objects as opaque by default
or silently changing the existing exact/binary64 numeric policies.

## Acceptance

- Bind capture scope, validation order, byte representation, source locations and
  format compatibility in the normalization design before implementation.
- Preserve exact selected token bytes, including whitespace inside the token,
  property order, escapes and numeric spelling; distinguish absence from null.
- Ordinary unselected fields retain the declared normalization/input semantics.
- Raw data is not interpreted as a replacement recipe or executable source.
- Reference and generated targets preserve the same bytes and fail consistently
  at unsupported boundaries. Generic fixtures demonstrate equal parsed values
  whose original byte identities differ.
- Keep complete source/provenance accounting and prevent partial successful
  adapters when a selected lexical boundary cannot be realized.

## Relation

This is a concrete input-fidelity requirement of source-pinned-data-normalization.
Adopter field names and immutable implementation citations remain in the adopter
specification. It is not permission to introduce a generic domain property bag or
to reinterpret all JSON objects as raw byte buffers.

## Scope

Derived 2026-09-06 by aep-drive:story-scoper during session recovery.

- Primary surface: schema-contract normalization admission, raw-input decoding and standalone Rust/Go targets (cited).
- Recipe admission: normalize.rs, normalize/recipe.rs and normalize/check.rs own format admission, input paths, root identity and typed path validation (cited).
- Raw-input boundary: normalize/input.rs encounters RawValue tokens but discards their original bytes into serde_json::Value (cited).
- Generated runtimes: normalize/rust_runtime.rs.txt, go_input.go.txt and go_runtime.go.txt own text/value entrypoints, token decoding and stage validation (cited).
- Generation/provenance: normalize/go_target.rs and target.rs emit input paths and retain recipe/source identity (cited).
- Additional evaluation modules: normalize/eval.rs and execute.rs may be needed depending on capture representation (inferred).
- Tests: normalization_go.rs, normalization_rust.rs and CLI tests/normalization.rs (cited); new normalization_raw_json.rs and fixtures/normalization_raw_json.rs are proposed corpus homes (inferred).
- Documentation: docs/design/source-pinned-data-normalization.md, website/docs/guides/generate-artifacts.md and website/docs/reference/formats.md (cited); CHANGELOG.md needs an entry for implemented behavior (inferred).
- Confidence: medium; token-loss boundaries are established, capture representation/order/format remain design decisions (inferred).
- Collisions: go_target.rs, normalization_go.rs, normalization_rust.rs and the binding design overlap the recovered base64 qualification (cited).

## Design Decisions Still Required

The scoper identified capture representation versus first-stage schema validation;
root/field/array reach and overlapping paths; extraction from retained embedded
documents; duplicate-key, Unicode, depth and numeric checks inside captured tokens;
interaction with binary64 paths; absence versus explicit null; refusal by decoded
value APIs without lexical provenance; format/report compatibility; and located,
deterministically ordered diagnostics. These are unresolved design questions, not
observed runtime failures. TypeScript normalization has no target yet.

The coordinator will finish the existing base64 qualification before scheduling
this overlapping implementation. This scope update claims no implementation.

## Retained-document sequencing

The source audit requires retaining an enclosing document's exact bytes before a
later, separately typed decode of that document and capture of nested token bytes.
An input-edge capture alone does not implement that second boundary. The binding
design must declare either composition of separately checked decoding steps or an
explicit parse-retained-bytes operation, preserving provenance and error order.
Do not claim complete external decoding from a capture-only implementation.
