---
format: aep.planning-md/1
id: story:normalize-positional-array-input
kind: story
status: draft
title: Normalize declared positional arrays without guessing decoder policy
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
- depends_on: story:model-binary64-fields
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/eval.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/execute.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_expression.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_input.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/go_retained.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/input.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v1_v5/go_expression.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/go_input.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/go_retained.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/input.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/retained.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v5/eval.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v5/execute.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v5/go_runtime.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v5/recipe.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/legacy_v5/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/retained.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_legacy_maps.json
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_positional.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_positional_go_tests.go.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_positional_rust_tests.rs.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_legacy_bytes.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_positional.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_positional_targets.rs
- confidence: inferred
  path: docs/design/positional-array-normalization.md
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
revision: 9
---
## Evidence

At ESS 9899faed9eb10eb2e4d9e6dc946f6d732e87c6b6,
crates/generate/schema-contract/src/realize/normalize/check.rs:198 refuses
Shape::Array with positional prefixes as unsupported_shape. recipe.rs declares
field reads and list operations but no explicit fixed-array input decoder.
The structural realization already retains positional schema arrays; the gap is
normalization admission/execution, not missing evidence that such schemas exist.

A source-contract audit found fixed-width decoded string arrays represented later
as named operand records. The downstream named record describes already-decoded
positions but does not implement source padding, truncation, null behavior or
position-specific element errors. Adopter identities and immutable source
citations remain in the adopter's private specification.

## Outcome

Bind and implement explicit positional-array normalization where the source
contract supplies arity and element policies. Do not silently flatten a tuple into
a homogeneous list or replace source decoding with stricter schema validation.

## Acceptance

- A binding design distinguishes checked tuple mapping from source decoder policy,
  including absent/null values, short/long inputs, invalid elements and error order.
- Input/output source identities and each positional read remain checked.
- A generic fixed-pair fixture demonstrates exact supported behavior in reference
  execution and generated targets; unsupported decoder policies refuse with source
  locations before adapter publication.
- Existing homogeneous-array and already-decoded-record recipes retain their bytes
  and semantics unless an explicitly versioned migration requires otherwise.

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
- **cited** `crates/generate/schema-contract/src/realize/normalize.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/recipe.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/check.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/input.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/eval.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/target.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/execute.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/go_target.rs`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/go_input.go.txt`.
- **cited** `crates/generate/schema-contract/src/realize/normalize/go_expression.go.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/retained.rs`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/go_retained.go.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v5/recipe.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v5/eval.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v5/rust_runtime.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v5/go_runtime.go.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/input.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/go_input.go.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/retained.rs.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v4_v5/go_retained.go.txt`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v1_v5/go_expression.go.txt`.
- **cited** `crates/generate/schema-contract/tests/normalization_legacy_bytes.rs`.
- **cited** `crates/generate/schema-contract/tests/fixtures/normalization_legacy_maps.json`.
- **inferred** `crates/generate/schema-contract/tests/normalization_positional.rs`.
- **inferred** `crates/generate/schema-contract/tests/normalization_positional_targets.rs`.
- **inferred** `crates/generate/schema-contract/tests/fixtures/normalization_positional.rs`.
- **inferred** `crates/generate/schema-contract/tests/fixtures/normalization_positional_go_tests.go.txt`.
- **inferred** `crates/generate/schema-contract/tests/fixtures/normalization_positional_rust_tests.rs.txt`.
- **cited** `crates/edge/ess-cli/src/normalize.rs`.
- **cited** `crates/edge/ess-cli/tests/normalization.rs`.
- **inferred** `docs/design/positional-array-normalization.md`.
- **cited** `docs/design/source-pinned-data-normalization.md`.
- **inferred** `crates/generate/schema-contract/src/realize/normalize/legacy_v5/execute.rs.txt`.

The source audit found no need to mutate shared Shape, compiler authority or CLI
main dispatch. The following symbol constraints preserve the other unit's read
dependencies; a necessary widening requires re-scoping before overlapping writes.

### crates/generate/schema-contract/src/realize/normalize.rs

May edit: format and DTO reexports (20); Recipe::envelope_findings (41); Plan::check_with_models version/input-policy checks (129); Plan::run_json (237); Plan::run (250).

Preserve: source identity/selection semantics; stage ordering and Plan::run_prepared (266); retained-base64 delegation (262); validation contract (272).

Add format 6 and positional preparation locally; all format-5 tests and older byte contracts retain their existing meaning.

### crates/generate/schema-contract/src/realize/normalize/check.rs

May edit: Kind (19): add local exact tuple representation; from_node (315): format-gated exact closed tuple conversion; input_numbers (77), input_captures (197) and local path helpers only as needed to carry version/tuple refusal; new positional policy checker and position checker; Context::version (433), expression (476), condition (759); assignable (897), comparable (828) and related exhaustive Kind handling.

Preserve: compiler-owned Binary64 membership check (335); existing model Binary64 union/input-path refusals; no tuple admission to array_item (880) or homogeneous operations; old format tuple refusal; local contains_binary64 (355) semantics; do not promote into shared structural code.

Shape::Array already contains exact tuple data. Kind::Tuple is local to normalization and does not require a shared Shape variant or Plan mutation.

### crates/generate/schema-contract/src/realize/normalize/recipe.rs

May edit: FORMAT_V6 beside format constants (7–18); Recipe (67) and new positional_inputs accessor/closed DTO; Expr (231): Position; new closed positional syntax visitor using unique_map convention.

Preserve: old canonical omission rules; NumberPath (109): field/items only, no position segment; existing Root/ModelIdentity authority and numeric/capture DTO meanings.



### crates/generate/schema-contract/src/realize/normalize/input.rs

May edit: parse (13); decode (29); descend (98) wiring and dedicated bounded positional helpers.

Preserve: prepare (109): decoded-value numeric semantics; binary64 (143), number (155), decimal (191); grammar-first, unique containing keys, strict Unicode/depth64 and ordinary traversal semantics.

New source policy is lexical preparation before unchanged first-stage validation. Discarded tails do not enter numeric/value decoding.

### crates/generate/schema-contract/src/realize/normalize/eval.rs

May edit: Context::value (20): Position arm and bounded helper.

Preserve: existing expression evaluation and missing/null behavior; existing numeric and collection operations.



### crates/generate/schema-contract/src/realize/normalize/target.rs

May edit: rust (50): format dispatch; finish (150): add format 6 to the target/3 family if coordinator accepts the report analysis; runtime_sources (183): complete old/current source selection.

Preserve: rust_package (39); sources (92) and checked source identity field meanings; old format manifests/report/file maps.

Frozen base maps formats 4/5 to target/3. The bounded analysis recommends retaining target/3 for recipe 6 because report coordinates and their interpretation remain unchanged; root must bind the decision.

### crates/generate/schema-contract/src/realize/normalize/go_target.rs

May edit: generate (15): positional binding and version dispatch; expression (177): Position lowering; new typed positional policy emitter; runtime_sources (323): freeze old templates.

Preserve: existing schema support/qualified base64 pattern rules (120); numeric/condition lowering semantics; shared structural package/module validators.



### crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt

May edit: Normalizer::new: format allowlist at line 30 only; Normalizer::normalize (50); Normalizer::normalize_value (56).

Preserve: Normalizer::new (27) identity/schema setup excluding the version-list extension; normalize_base64_json (63) caller composition; validate (67) boundary.



### crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt

May edit: Normalizer.Normalize (105): positional text preparation.

Preserve: New (67) schema identity setup; validate (149); wireValue (188); existing numeric/collection helpers.



### crates/generate/schema-contract/src/realize/normalize/go_input.go.txt

May edit: parseInput (23); decodeInput (104); typed policy/path traversal helpers.

Preserve: raw decoder token preservation (39–92); inputNumber (191); ordinary number and Unicode/depth behavior.



### crates/generate/schema-contract/src/realize/normalize/go_expression.go.txt

May edit: new bounded positionValue expression helper.

Preserve: all existing expression helper behavior; freeze previous entire source for formats 1–5.



### crates/generate/schema-contract/src/realize/normalize/retained.rs

May edit: validate (45) may be factored for lexical-only tail checking with separate diagnostics.

Preserve: capture (36) bytes and existing error locations; require_text (9) raw refusal precedence; decode (20) base64/UTF-8 ordering.

Inferred optional edit. Freeze exact prior retained source for old captures if live bytes change.

### crates/generate/schema-contract/src/realize/normalize/go_retained.go.txt

May edit: validateCaptured (32) may be factored for lexical-only tail checking.

Preserve: NormalizeBase64JSON (12); captureJSON (23) capture behavior and diagnostics.

Inferred optional edit, confined to positional owner and requiring old template freeze.

### crates/edge/ess-cli/src/normalize.rs

May edit: Sources.recipe help at line 15: currently stale list /1–/4, document admitted /1–/6 accurately; RunArgs.input help at line 42: explain declared lexical policies without promising all nested duplicates refuse.

Preserve: Sources::plan (80); check (126), run (132), generate (146) dispatch/publication safeguards unless a concrete positional issue requires a scoped update.

Existing commands already read/check recipes and call the proper normalization Plan; no new flag or main.rs branch is required.

### crates/generate/schema-contract/src/realize/normalize/execute.rs

May edit: run (6): Context.binary64 format-family predicate at line 27 admits formats 5 and 6.

Preserve: input validation before requirements; condition/value evaluation order; output validation and atomic result; formats 1–5 existing semantics.

Exact frozen equality gate would disable Binary64 condition semantics in recipe 6; this local predicate edit is mandatory and needs a frozen format-5 execute template.

The coordinator binds recipe format 6 to normalization-target/3: existing report
fields, full canonical recipe digest, complete file hashes and prepared-stage
root meanings remain unchanged. Format 6 owns the new source input semantics.
Preserve all older file maps; extend execute.rs Binary64 context and generated
Rust version admission to 6, freezing required old executor/runtime sources.

## Status of the Finding

Filed during resumed source mapping. This is a concrete capability refusal and a
compatibility requirement, not a claim that every native decoder permissiveness
must be reproduced by a typed canonical contract. No implementation is claimed.
