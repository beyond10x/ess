---
format: aep.planning-md/1
id: story:raw-json-normalization-provenance
kind: story
status: implemented
title: Preserve declared raw JSON token bytes during normalization
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_input.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_retained.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/input.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/README.md
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/go_input.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/input.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/recipe.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v3/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/retained.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_legacy_maps.json
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_raw.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_raw_go_tests.go.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_raw_rust_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_legacy_bytes.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_raw.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_raw_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_raw_targets.rs
- confidence: cited
  path: docs/design/raw-json-normalization.md
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 12
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

## Design decision disposition

The initial scoper questions are bound by docs/design/raw-json-normalization.md.
The complete lexical, composition, ordering and version decisions below govern the
implementation; TypeScript is a separately scoped target.

## Retained-document sequencing

The source audit requires retaining an enclosing document's exact bytes before a
later, separately typed decode of that document and capture of nested token bytes.
An input-edge capture alone does not implement that second boundary. The binding
design must declare either composition of separately checked decoding steps or an
explicit parse-retained-bytes operation, preserving provenance and error order.
Do not claim complete external decoding from a capture-only implementation.

## Bound implementation

Binding: docs/design/raw-json-normalization.md in managed implementation tree
wt-c12a5474a249 at base60ffcb2. The coordinator selected format4, explicit branch
field/items/root captures into canonical base64 before first-stage validation,
strict Unicode/depth with retained duplicate/numeric lexemes, deterministic path
conflicts and value-API provenance refusals, and explicit base64-to-text helpers
for composition of separately checked recipes.

Five affected emitted templates are frozen as one legacy_v1_v3 family. Existing
formats1–3 preserve complete emitted bytes/file maps under identical generator
version; release-version provenance remains truthful. Native corpus evidence and
old-reader/full-map compatibility checks are required before this unit goes green.
The coordinator owns store updates and integration; the implementor owns its
isolated worktree and reuses the assigned serial build cache. No approval is inferred
from the outstanding optional wrap-up scope question: the original full-gap
authorization remains the current scope absent a correction.

## Implementation and review state

Implemented source unit3e2eb52e84f5ed7b94381076270d4cdc6f2970fd is integrated with
main1667d02 in the coordinator tree. The first adversarial pass added42 lexical
cases and four top-level tests, with no findings; the exact path-normalized report
is review-result:normalization-raw-json-adversary-pass1-public and its recorded
outcome is no-op. Reference/Rust corpus156, Go149 plus UTF-8/composition controls,
six complete legacy file maps and the2,490-case base64 qualification remain green.

The literal task check passed1,894 Rust cases, zero failed/ignored. The corrected
explicit-Go native lane and site build also passed. The initial native harness
failure, original logs, timings, integration conflict resolution, CLI help change,
resource observations and workflow deviations are retained in
verification-report:normalization-raw-json-integrated. Source publication and final
Atlas delivery will be recorded separately. No release is claimed at this stage.

The raw implementation used managed treewt-c12a5474a249, branch
impl/normalization-raw-json at base60ffcb2. Integration is inwt-bf45625a6a50,
branch impl/normalization-base64-resume. Assigned raw scratch is outside both trees.
A serial target cache was reused across these two trees as an explicit deviation
from the wave instruction, with no simultaneous unit builds. Subsequent units use
their own target. Native build artifacts remain owned by the coordinator until
wanted evidence is retained and managed cleanup is safe.

TypeScript, positional arrays and model Binary64 are separate open stories. A
source-observed legacy Go depth-cap difference has not been independently measured
as a legacy native case and is not reported as a confirmed regression.
