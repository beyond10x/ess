---
format: aep.planning-md/1
id: story:typescript-normalization-target
kind: story
status: active
title: Execute source-pinned normalization in native TypeScript
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
- depends_on: story:normalize-positional-array-input
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/normalize.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization_typescript.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/normalization_typescript_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_collection.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_condition.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_expression.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_input.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_numeric.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_retained.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_runtime.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_schema.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/ts_value.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/typescript_schema.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/typescript_target.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript_adversary.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript_legacy_maps.json
- confidence: cited
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript_tests.ts.txt
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_typescript.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_typescript_adversary.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_typescript_legacy_bytes.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_typescript_native.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/normalization_typescript_schema.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/support/normalization_typescript.rs
- confidence: cited
  path: docs/design/source-pinned-data-normalization.md
- confidence: cited
  path: docs/design/typescript-normalization.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 36
---
## Outcome

Implement the native TypeScript execution target of source-pinned-data-normalization
under docs/design/typescript-normalization.md. JSON text and internal bigint preserve
the existing signed/unsigned integer boundary without caller-side precision loss.
This is the coordinator's stated engineering choice, not a claim of a new operator
answer to the optional preference question.

## Evidence

The existing normalize.rs exposes only Rust/Go execution targets. normalize/target.rs
owns source retention and reports. crates/edge/ess-cli/src/normalize.rs:49 declares
only Rust and Go in its normalization target enum. Structural TypeScript generation
already exists and is a separate capability. The read-only scoper identified the
native JSON parser/serializer and number-only schema validators as insufficient for
exact integer parity.

## Acceptance

- Execute every current recipe operation through a standalone ES2022 TypeScript
  library with JSON-text input/result and located reference-compatible refusals.
- Retain token-kind numeric distinctions, signed wrapping, binary64 input policy,
  missing/null distinction, ordering and lazy evaluation.
- Validate the admitted source-schema profile without lossy bigint coercion;
  refuse every unqualified constraint before returning a partial adapter.
- Preserve complete bundle/model pins and deterministic report/file accounting;
  keep existing Rust/Go output bytes stable.
- Expose CLI generation and read-only drift checking with source-input protection.
- Native TypeScript/Node execution covers the shared fixtures, prototype-sensitive
  names and numeric/schema/error-order boundaries described by the binding design.

## Scope

- **cited — Source authority:** positional production baseline is ESS `eb2e5d60e9e803993417df39563bc744dbcd36fc`; the TypeScript binding is coordinator `81f1551f1373bc66e9b4dabf59d6557ec4350a43`. Immutable Git blobs, not moving adversary additions, supply the confirmed source lines/hashes. This is a read-only refresh of the prior 31-path packet; positional review and final integration still precede TypeScript dispatch.
- **cited — Binding authority:** the coordinator TypeScript design and frozen positional implementation establish the standalone ES2022 JSON-text target, checked finite Binary64 and recipe-6 positional policies. Private `position_arities` and source-tuple union distinction are now implemented in the frozen dependency. No TypeScript target implementation or execution is claimed.
- **inferred — Primary surface and size:** one normalization target in `schema-contract`, four narrowly edited existing production files, one feature-gate registration, two new Rust production modules, nine TypeScript runtime templates and nine new test/fixture/support files. The proposed 31 paths also include six documentation paths, three of which remain coordinator-owned. This is a parser/evaluator/validator plus target-packaging unit; structural TypeScript aliases are not its runtime.
- **inferred — Confidence: medium:** the final positional symbols, template routing and constructor inventory are now cited. New TypeScript filenames remain the same bounded proposed split. Exact native/schema qualification and the twelve-map pre-edit byte capture still belong to implementation, after any positional adversary correction is refreshed.

### Typed implementation write scope

- **cited — `crates/generate/schema-contract/src/realize/normalize.rs`:** `module declarations`, `Plan::typescript`. Register private TypeScript emitter/schema modules; add Plan::typescript(&str) -> Result<Realization, Refused>. Preserve: Recipe admission, all public existing methods, Plan private position_arities from integrated positional work, source identities, reference execution.
- **cited — `crates/generate/schema-contract/src/realize/normalize/target.rs`:** `Report::configuration`, `new normalization-local Configuration`, `rust`, `finish`. Replace private report configuration type with normalization-local enum retaining exact Rust/Go language-tagged serialization; add package-bearing Typescript arm; use local Rust arm; retain integrated format-family selection in finish. Preserve: Report and Realization public types/reexports; root/source paths and all digests; generated Rust runtime_sources bytes; report field order and complete legacy JSON bytes.
- **cited — `crates/generate/schema-contract/src/realize/normalize/go_target.rs`:** `configuration import`, `generate -> target::finish configuration construction`. Switch only the configuration import and Go variant construction to the normalization-local enum. Preserve: All emitted Go strings, schema support, expression/condition lowering, positional bindings and runtime_sources.
- **cited — `crates/edge/ess-cli/src/normalize.rs`:** `Target`, `GenerateArgs::module documentation`, `generate`. Add Typescript enum arm; call plan.typescript(package); reject module; describe recipes 1 through 6 accurately. Preserve: Sources::plan and source pinning; preflight/write/check ordering; canonical input/link protection; check_files behavior.
- **cited — `crates/generate/schema-contract/Cargo.toml`:** `[[test]] normalization_typescript_native`. Register the new native test with required-features = [typescript-typecheck]; reuse the existing feature. Preserve: No runtime dependency or lockfile change; existing structural native test registration and Go feature.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/typescript_target.rs`:** `generate`, `package_name`, `expression`, `condition`, `numeric_step`, `schema_bindings`, `input_policies`. Validate package, preflight schemas, lower every checked operation and policy, embed constant bits and private position_arities, assemble deterministic files with target::sources/finish. Preserve: No replacement authored-recipe runtime parser; no runtime network or source acquisition; no structural alias emitter reuse as validation.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/typescript_schema.rs`:** `qualify`, `visit_schema`, `lower_schema`, `numeric_literal`, `source_pointer`. Walk the complete selected retained schema closure, qualify closed profile, emit private typed schema data and original-source findings. Preserve: Reference siblings and all map-key/combinator/tuple positions; annotation/literal values are not schemas; compiler-owned authority is not inferred.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_value.ts.txt`:** `Value`, `MISSING`, `scalarCompare`, `serialize`, `pointer`. Private value union with Map records, scalar ordering, exact serializer and located findings. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_numeric.ts.txt`:** `readNumber`, `decimalKey`, `formatFloat`, `fromBits`, `numericCompare`. i64/u64 kind admission, checked i128 decimal keys, finite binary64, signed wrapping/truncation, bit-exact constants and schema numeric comparison. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_input.ts.txt`:** `scanDocument`, `decode`, `preparePositional`, `validateToken`. Complete grammar/token-span scanner, ordinary sorted decoding, raw capture, positional first-boundary preparation and lexical-only tail validation. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_retained.ts.txt`:** `decodeBase64Json`, `decodeUtf8`, `encodeUtf8`, `encodeBase64`. Strict canonical standard base64 and Unicode conversion, preserving BOM and capture token spelling. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_schema.ts.txt`:** `Schema`, `validate`, `schemaEqual`. Closed source-schema execution, exact numeric comparisons, tuple validation and reference finding multiset. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_expression.ts.txt`:** `Expression`, `Context`, `read`, `field`, `record`, `fallback`, `choose`, `position`. Private typed expression closures and located lazy evaluation including checked positional arity. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_condition.ts.txt`:** `Condition`, `equal`, `greater`, `startsWith`, `all`, `any`, `not`. All eight condition variants with correct short-circuit and typed Binary64 equality. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_collection.ts.txt`:** `map`, `distinctCount`, `selectMap`, `find`, `concatLists`, `itemIndex`. Ordered lexical item/index scopes, original indices and enclosing-scope find fallback. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/src/realize/normalize/ts_runtime.ts.txt`:** `Finding`, `NormalizationResult`, `Normalizer`. Public JSON-text API, first-entry preparation, stage validation/requirements/evaluation/output validation and retained-document delegation. Preserve: New TypeScript template only; do not edit or fork old Rust/Go frozen template bytes.
- **inferred — `crates/generate/schema-contract/tests/normalization_typescript.rs`:** `determinism`, `configuration`, `complete_file_accounting`, `source_pins`. Offline generation/report tests for each recipe format, valid/invalid package identity, all-source closure accounting and source-order invariance. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/normalization_typescript_schema.rs`:** `closed_profile`, `refusal_paths`, `numeric_schema_boundaries`, `tuple_profile`. Rust-owned feasibility/refusal cases including nested propertyNames, refs, tuple alternatives, uniqueItems true and annotation decoys. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/normalization_typescript_native.rs`:** `native_all_corpora`, `native_engine_identity`, `raw_and_numeric_boundaries`. Feature-gated Rust driver compiles emitted ES2022 package and runs independent Node assertions across all applicable fixture branches. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/normalization_typescript_legacy_bytes.rs`:** `all_six_rust_go_file_maps`. Compare complete exact path/hash maps for recipes 1 through 6 at one fixed generator version, recorded before TypeScript shared edits. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/fixtures/normalization_typescript.rs`:** `cases`, `prototype_cases`, `schema_cases`, `numeric_bits_cases`. New public literal cases and raw input strings; reuse existing fixtures without filtering branches or computing expected values through the new runtime. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/fixtures/normalization_typescript_tests.ts.txt`:** `runCases`, `assertResult`, `assertBits`, `assertFindings`. Compiled native assertion program; keep large expected values in exact JSON text/bigint/bit strings and actual input token text. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/fixtures/normalization_typescript_legacy_maps.json`:** documented target capability and evidence. New exact fixed-generator compatibility witness for twelve target-format combinations; never overwrite historical expectations to accept drift. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/generate/schema-contract/tests/support/normalization_typescript.rs`:** `compiler`, `node_identity`, `write_package`, `compile`, `run`, `fixture_inventory`. Rust-only harness support; reuse ESS_TYPESCRIPT_COMPILER convention and worktree-local CARGO_TARGET_TMPDIR. Preserve: Existing public fixture bytes and legacy baseline files remain unchanged; no production checker in Python/shell/JS.
- **inferred — `crates/edge/ess-cli/tests/normalization_typescript.rs`:** `generate_and_check`, `preflight_atomicity`, `source_and_link_protection`. Exercise TS CLI generation, module refusal, complete-file drift, stale/missing late paths, file/link conflicts and bundle/model/recipe protection. Preserve: Existing normalization.rs tests and CLI main dispatch remain unchanged.
- **cited — `docs/design/typescript-normalization.md`:** documented target capability and evidence. Describe only implemented TS package/API/profile/refusals and actual measured qualification; remove superseded pre-format-5/6 preparation claims after refresh. Preserve: Existing format semantics and source pin meaning.
- **cited — `docs/design/source-pinned-data-normalization.md`:** documented target capability and evidence. Describe only implemented TS package/API/profile/refusals and actual measured qualification; remove superseded pre-format-5/6 preparation claims after refresh. Preserve: Existing format semantics and source pin meaning.
- **cited — `website/docs/reference/cli.md`:** documented target capability and evidence. Describe only implemented TS package/API/profile/refusals and actual measured qualification; remove superseded pre-format-5/6 preparation claims after refresh. Preserve: Existing format semantics and source pin meaning.

### Coordinator documents and explicit read dependencies

- **cited — `CHANGELOG.md`:** Coordinator integrates truthful TS target/package/profile/refusal and validation documentation after accepted implementation. The current publication story assigns this path to the coordinator; this proposal does not transfer that ownership.
- **cited — `website/docs/guides/generate-artifacts.md`:** Coordinator integrates truthful TS target/package/profile/refusal and validation documentation after accepted implementation. The current publication story assigns this path to the coordinator; this proposal does not transfer that ownership.
- **cited — `website/docs/reference/formats.md`:** Coordinator integrates truthful TS target/package/profile/refusal and validation documentation after accepted implementation. The current publication story assigns this path to the coordinator; this proposal does not transfer that ownership.

- **cited — Reference-only production seams:** `normalize/recipe.rs` owns closed DTOs/version gates; `normalize/check.rs` owns type admission; `normalize/source.rs::selection` retains compiler-owned Binary64 metadata and refuses model invariants; `normalize/input.rs::{parse,decode,number,decimal}` owns text admission; `normalize/retained.rs::{decode,capture,validate}` owns retained bytes; `normalize/numeric.rs::{literal,finite,constant,convert,floating}` owns finite arithmetic; `normalize/eval.rs::Context` and `normalize/execute.rs::run` own runtime ordering. Consume these contracts without changing reference behavior to make TypeScript parity easier.
- **cited — Structural configuration remains unchanged:** `crates/generate/schema-contract/src/realize.rs::TargetConfiguration` is public and fieldless for Typescript; its `name`, `Plan::report` match, and constructors in `realize/ts.rs`, `realize/rust.rs` and `realize/go.rs` belong to the structural report. They are read dependencies, not TypeScript-normalization write scope. `src/typescript.rs` is another structural projection, also outside scope. No `ess-gen`, compiler, conformance, CLI main, xtask, workflow, lockfile or Taskfile edit is presently established as necessary.
- **cited — Existing CLI forwarding:** `crates/edge/ess-cli/src/main.rs` already delegates normalization GenerateArgs/generate; adding the enum arm in `src/normalize.rs` requires no top-level command or report-rendering change. Existing CLI file preflight helpers retain their paths and behavior.

### Public API, package and report contract

- **cited — Additive public Rust API:** add only `normalize::Plan::typescript(&self, package: &str) -> Result<Realization, Refused>`. `normalize::Report` is reexported, derives Serialize, and has private fields; its configuration is not a public constructor argument or accessor. `Realization { files, report }` and existing methods/reexports remain unchanged. `target::finish` is `pub(super)`. Existing normalization configuration construction occurs only in `target.rs::rust` and `go_target.rs::generate`; both call `finish`. The new emitter supplies the third construction. There is no normalization-report reader in the inspected production call chain; existing structural configuration matches therefore need no new arm.
- **inferred — Selected normalization-local configuration:** use a private serde enum tagged `language`, with `Rust { package }`, `Go { package, module }`, and `Typescript { package }`. The coordinator selected this minimal package arm, with no configurable profile identifier. Preserve exact field order and serialized Rust/Go objects. Unsupported schemas return `Refused` before any Realization or report. Keep the structural fieldless TypeScript arm untouched.
- **cited — Exact report version dispatch:** recipe 1 or 2 emits `ess-normalization-target/1`; recipe 3 emits `/2`; recipes 4, 5 and 6 emit `/3`. The frozen positional `target.rs::finish` at line 163 now implements all six. A third language changes neither root meaning nor report/file identity; preserve the selected serializer-only report families and exact old Rust/Go arms.
- **cited — Source/file accounting:** reuse `target::sources` and `finish`. Preserve `source.recipe.json`, every `schemas/{source-key}-{root-name-digest}.schema.json`, every `sources/{bundle-digest}.bundle.json` and `sources/model-{projection-digest}.schema.json`, complete Root/ModelIdentity coordinates and recipe canonical digest. `Report.files` hashes all emitted inputs except the report itself; `normalization-report.json` is then added to Realization.files. No fixture branch, model selection, schema refinement or provenance coordinate disappears from this accounting.
- **inferred — Complete new package paths:** emit `package.json`, `tsconfig.json`, `README.md`, `schema-profile.json`, `src/index.ts`, `src/value.ts`, `src/numeric.ts`, `src/input.ts`, `src/retained.ts`, `src/schema.ts`, `src/expression.ts`, `src/condition.ts`, `src/collection.ts`, generated `src/bindings.ts` and `src/schemas.ts`, plus the retained files and report above. All exact bytes enter the existing digest map. `schema-profile.json` is fixed typed capability/refusal metadata emitted by the qualifier, not an authored policy or configurable profile. Compiled `dist/` is a consumer build output and is not preclaimed in generated-file digests.
- **cited — Native API:** export Finding and the discriminated `NormalizationResult` union, and a no-argument Normalizer exposing `normalize(branch, jsonText)` and `normalizeBase64Json(branch, encoded)`. Success contains compact JSON text without final LF; refusal contains complete located findings, never partial output. There is no object-input API or caller-replaceable recipe, filesystem lookup or network dependency.
- **inferred — Package implementation:** private version-0.0.0 ESM metadata, explicit exports/types to `dist/index.js`/`dist/index.d.ts`, strict ES2022 compilation, declaration emission and `.js` import specifiers in generated TypeScript. Use no Node type package or numeric/schema runtime dependency. Validate package identity before source generation; publish the exact accepted name grammar and negative controls. The binding has not established whether scoped npm names are required, so that grammar is a bounded implementation choice to record, not an excuse to infer JavaScript identifiers from package or wire names.

### Operation and value coverage

- **cited — Exhaustive operation matrix:** the frozen Expr has 24 variants: Position, Null, Boolean, String, Integer, Read, Field, Record, List, Fallback, Choose, Arithmetic, Map, DistinctCount, Concat, Join, IntegerString, ConcatLists, ItemIndex, SelectMap, Find, Binary64Literal, Binary64 and Binary64ToInteger. Conditions remain Present, Boolean, Equal, Greater, StartsWith, All, Any and Not. Keep exhaustive Rust matches; recipe 6 inherits Binary64 operations/equality while Position and positional declarations remain recipe-6-only.
- **cited — Execution order:** parse/preparation precedes branch dispatch; input schema precedes ordered requirements, then value evaluation, missing-output detection and output schema. Only external entry applies input policies. Later stages receive the previous prepared result with identical source-root identity. Preserve lazy fallback/choose/predicates, input/item lexical scope, original indices after filtering and Find's enclosing-scope otherwise. Record construction omits missing members; list construction refuses missing elements; null/false/zero/empty string are distinct.
- **cited — Private values:** missing sentinel, null, boolean, string, bigint, finite number, arrays and Map<string, Value>. Exact decoded keys including `__proto__`, `constructor`, empty and numeric-looking names are ordinary data. One Unicode-scalar comparator controls record traversal, serialization and schema-finding presentation; neither object enumeration nor UTF-16/default/locale sorting is sufficient.
- **cited — Exact numeric reader:** validate JSON grammar first; preserve the existing signed-i64/unsigned-u64 bare-integer domain as bigint, with lexical size checks. Exact bare -0 becomes integer zero. Other numeric lexemes use finite Number conversion; selected binary64 may round/underflow with sign. Unselected floats require normalized decimal-key equality with the shortest result, including zero special handling and checked i128 exponent limits. An out-of-range integral token falls through to exact float admission; neither arbitrary bigint admission nor unconditional refusal is equivalent.
- **cited — Number operations and output:** require signed-i64 bigint representation for integer operations, explicit two's-complement wrapping and [-2^63, 2^63) truncation bounds. Preserve binary64 operation order, signed-zero clamp tie rules, finite-overflow checks and literal constants lowered from checked bits with explicit DataView byte order. Format floats with native shortest digits and zmij layout: fixed scientific exponent -5 through 15, `.0` for integral floats, signed scientific exponent otherwise, and explicit -0.0/0.0. Typed modeled Binary64 authority comes from the checker, not typeof number or broad schema number. General numeric/Integer/Decimal values require explicit conversion to fill Binary64 outputs.

### Raw text, retained documents and format-6 positions

- **cited — Text parser priority:** perform iterative complete-document grammar/token-span scanning before semantic decoding, duplicate handling, depth checks or dispatch. Only JSON whitespace is allowed. The second phase checks ordinary decoded keys/uniqueness before scalar-sorted children, arrays in index order, root depth zero and maximum depth 64. A malformed suffix must win over an earlier located semantic error. Literal and escaped lone UTF-16 surrogates refuse at the proper decoding boundary; do not replace them while eagerly UTF-8 encoding the input string.
- **cited — Capture and helper:** captured tokens retain exact UTF-8 source bytes, internal whitespace, escapes, duplicate members and numeric spelling, excluding transport whitespace. Validate capture Unicode/depth in source-member order and locate errors at the capture boundary. Base64 helper admission requires standard alphabet, padding, zero unused bits and re-encoding equality before strict UTF-8; reject overlong/truncated/surrogate/out-of-range encodings. Preserve a leading BOM so JSON syntax rejects it; BOM inside a string is content. Empty decoded input reaches syntax. Helper strictness differs from the frozen Bytes pattern, which admits AB== and AAB=.
- **cited — Positional external policy:** consume the integrated closed positional_inputs declarations, with field/items/root selectors and checked positive N. Missing stays missing; terminal null becomes N empty strings; consumed null becomes empty string; short arrays pad; excess tokens discard; wrong consumed kind refuses without numeric conversion or recursive interpretation. Intermediate missing/null remains unchanged. Decode once, then retain every tuple schema refinement. Do not infer this policy from prefixItems or min/maxItems.
- **cited — Discard traversal and error order:** discarded tokens receive lexical-only Unicode/depth validation from the original root, with objects in source order, arrays in index order, no number conversion or duplicate collapse. Report deeper failures at the discarded element pointer. Earlier consumed kind errors precede later discarded Unicode/depth errors, while complete grammar errors have already won. Check depth before kind/Unicode; a wrong boundary-kind subtree is not recursively interpreted. Reuse the same original spans as capture rather than create untyped tail values.
- **cited — Private position_arities:** the final checker recomputes `BTreeMap<String,u64>` from replay-checked stage roots and each Position operand's exact tuple type. Lower the private binding into `src/bindings.ts` and retain it privately in Normalizer/context. Full escaped static expression pointers include branch/stage/nested locations; runtime collection indices never alter the lookup key. Evaluate the operand once and propagate missing. A present operand requires positive metadata, array kind, exact N length and index < N. Missing/zero metadata, nonarray, shorter/longer arrays even when the selected index exists, and out-of-range index yield `position_value`: `position encountered a value outside its checked tuple contract` at the expression pointer. No authored arity field, runtime-length inference or public API change.

### Closed schema profile and located refusals

- **cited — Required intersection:** qualify only schemas already admitted by the ESS checked source plan. Baseline branches require boolean/empty schemas, local definitions/refs with conjunctive siblings, primitive and array-valued type, properties/required, boolean or schema additionalProperties, model propertyNames, homogeneous items, string enum, inclusive bounds, anyOf, maxLength and the exact frozen Bytes pattern. Format 6 additionally requires prefixItems, minItems, maxItems and string-length refinements such as minLength so padded empty strings can still fail the retained schema.
- **inferred — Proposed complete first profile:** implement that intersection plus scalar/deep const and enum, exclusive bounds and allOf/oneOf using the same exact comparator and recursive validator. These additions need independent source-validator controls before claiming parity; they are a closed list, not generic JSON Schema support. Resolve references at generation into local schema bindings, preserving percent-decoded fragments and escaped definition names admitted by existing local_definition. Use Bundle::root_definitions, Bundle::source_pointer and ModelTypes::definitions for source-qualified feasibility pointers. Walk refs, every property, additionalProperties, propertyNames, items, prefixItems and every combinator branch. Never walk default/examples/enum/const/provenance values as schemas.
- **cited — Numeric schema meaning:** number accepts bigint and finite number; integer also accepts integral floating representations including -0.0 and 1e20, independently of recipe integer eligibility. Bounds and numeric const/enum compare mixed operands against the exact represented IEEE rational, using bit significand/exponent shifts rather than Number(bigint) or shortest-decimal equality. +0 and -0 are schema-equal. Deep equality compares array order and decoded record key/value sets without scalar coercion. Preserve schema-literal numeric kind in generated bindings.
- **cited — Findings:** source runtime failures use schema_validation and the fixed stage-contract detail. Preserve the complete multiset including duplicate pointers, stably sorted by scalar instance pointer as Go presents it. Required emits once per missing name at the object; additionalProperties false emits one object finding for all extras; schema additionalProperties uses value pointers; propertyNames child errors use the object pointer. Type-specific keywords skip other kinds. AnyOf/OneOf yield one current-pointer failure when invalid, not flattened alternative errors; AllOf retains child multiplicity. Prefix paths with escaped `/branches/<branch>/<stage>/input` or `/output`, never source-schema keyword/reference paths.
- **inferred — Tuple diagnostic qualification:** validate each prefix at its original numeric instance index; items applies only after the prefix; minItems/maxItems each produce a tuple-pointer finding independently of item findings. A false tail schema must preserve the pinned validator's actual aggregate-versus-per-index multiplicity; qualify this with standalone reference observations before freezing TS expected results. Tuple length keywords validate prepared values and never trigger padding or discard themselves.
- **cited — Explicit refusal:** uniqueItems:true preflight-refuses at its original source-qualified keyword pointer in every selected schema; false is unconstrained. No inspected pre-format-5/6 fixture requires true, but recheck the final integrated corpus. This guards a separately recorded pre-existing validator inconsistency around +0/-0 and the array-size-16 hash path; do not silently emulate it or call it a TS-introduced defect. No successful partial report is returned.
- **inferred — Other bounded refusals:** refuse multipleOf, non-frozen patterns, unsupported reference forms, min/maxProperties until qualified/required, conditional/dependency/unevaluated keywords and unknown constraints. Preserve pre-existing structural/model-invariant refusals. $schema/$id/$defs are local schema-envelope metadata; approved format/contentEncoding/contentMediaType and ordinary annotations do not execute validation; defaults do not execute transformations. Do not turn unknown x-ess metadata into model authority. Emit the exact supported/refused profile in schema-profile.json and hash it with the runtime.

### Qualification and implementation sequence

- **cited — Reusable corpus paths:** read `tests/fixtures/normalization_v1.rs`, `normalization_v2.rs`, `normalization_numeric.rs`, `normalization_model.rs`, `normalization_base64.rs`, `normalization_raw.rs`, `normalization_binary64.rs`; preserve extra controls and aliases in `tests/normalization_go.rs::{plans,model_aliases}`, raw/base64 adversary tests and Binary64 native fixtures. The pre-format-5/6 inspected inventory was 40 v1, 14 v2, 35 numeric, 66 model including aliases, 2490 base64, 149 raw text/helper and 42 raw adversary cases: 2836. This historical count does not claim a current full-format execution. Add all final Binary64 and positional fixture constructors/branches and enumerate a fresh lossless inventory. Base64 model_keys is mandatory. The raw fixture's seven decoded-value API cases are explicitly inapplicable to the text-only TS API; do not silently filter other cases.
- **cited — Native qualification baseline:** the bound lane requires TypeScript 6.0.3 through ESS_TYPESCRIPT_COMPILER and records Node/V8. Prior strategy probes at source 6c78676 used Node 22.23.1 / V8 12.4.254.21-node.56 across 13,455 finite bit patterns, 28 midpoint cases and 35 numeric corpus cases; 29 scalar arithmetic pipelines matched. These are historical algorithm evidence, not new TS execution or format-5/6 qualification. Cargo.lock pins serde_json 1.0.151, jsonschema 0.52.1 and zmij 1.0.23; jsonschema enables float_roundtrip. Recheck that active feature graph when qualifying the final target.
- **inferred — Test split and commands:** offline generation/schema/CLI tests require no native compiler. Register normalization_typescript_native with the existing typescript-typecheck required feature so normal cargo test cannot accidentally require an unset compiler variable. The explicit lane runs `cargo test -p schema-contract --locked --features typescript-typecheck --test normalization_typescript_native`, with configured compiler and recorded Node identity; compile emitted package using `node <tsc.js> --pretty false --project <tsconfig>` and execute its compiled native assertions with Node. Keep builds and temporary products in the assigned tree/cache and exercise actual JSON text, not a pre-parsed substitute. No dependency installation or native command occurs during generation.
- **inferred — Step 1, final freeze:** after positional integration and any adversary correction, compare the confirmed symbol/hash packet with the integrated tree and freeze twelve complete exact Rust/Go format-1–6 file maps at one actual generator version before shared TypeScript edits. The existing `normalization_legacy_bytes.rs` now covers formats 1–5 and normalizes generator_version; leave that guard intact, but add the separately scoped format-1–6 witness without normalization. Preserve exact report bytes and never regenerate expectations to conceal drift.
- **inferred — Step 2, qualifier/package:** implement the normalization-local configuration and additive API, exact package/profile emission, source closure traversal and preflight refusals. Establish deterministic complete-file accounting and invalid-package/source/refinement controls before returning executable artifacts.
- **inferred — Step 3, value/input core:** implement private values, scalar ordering, exact number reader/formatter/comparator, two-phase token parsing, strict retained helpers and positional preparation together around one token-span representation. Qualify midpoint/limits, prototype names, escaped/literal surrogate and BOM boundaries, duplicate/depth/error precedence, capture bytes and discarded tails.
- **inferred — Step 4, execution:** lower all 24 expressions and eight conditions, preserve stage/context/lazy behavior, checked Binary64 constants/equality and static private Position arities; add all operation and defensive-arity controls. Do not infer Position type from selected index or runtime array contents.
- **inferred — Step 5, schema and complete native corpus:** compile/run the standalone package, retain independent expected JSON/bit/refusal observations and compare the complete stage-schema finding multiset. Add mixed bigint/float bounds/const/enum and tuple min/max/prefix/tail multiplicity probes; freeze actual Node/TS versions and full fixture inventory. No easiest-subset branch filtering or uniqueness-bug emulation.
- **inferred — Step 6, CLI and integration:** prove module refusal, source protection, all-file drift and preflight failure atomicity through the actual CLI; recheck all twelve fixed-generator Rust/Go file maps after shared edits; run focused native/offline checks, then repository task check and the required site gate for documentation changes. These are future required checks, not work executed in this scoping task.

### Collisions and remaining refresh boundaries

- **cited — Would collide with:** positional work edits `normalize.rs`, `normalize/target.rs`, `normalize/go_target.rs`, CLI `src/normalize.rs` and `docs/design/source-pinned-data-normalization.md`. It also changes the recipe/check/input/retained/eval/execute read dependencies and current template selection. Those are actual shared seams, so TypeScript implementation must start after a frozen positional integration and refreshed scopes. Do not claim two units are disjoint merely because the eventual TS runtime templates are new.
- **inferred — Structural unit separation:** the proposed local configuration and new test names leave structural `realize.rs`, `realize/{ts,rust,go}.rs` and their codec tests outside the TS write scope. A later required widening must be explicit before overlapping writers begin. Shared changelog and public formats/generation guidance remain serial coordinator work.
- **inferred — Remaining dispatch work:** refresh any positional adversary correction and final integration delta; capture the twelve fixed-generator old target maps before shared edits; select/document bounded npm package grammar; requalify the actual Node/TypeScript engine and exact schema diagnostic multiplicities. Final positional symbols and current 99-plus-3 fixture split are now confirmed. None of this scoping work is TS implementation evidence.

### Confirmed positional dependency delta

- **cited — Source versus expression union:** `normalize/check.rs:20,84,465,1062,1096` retains `Kind::SchemaUnion` for source alternatives containing tuples, including nested object/array paths; identical source alternatives cannot mint a Position proof. `Kind::Union` still permits a choose over one independently exact tuple. The target consumes the checked Plan and must not re-admit either case from schema shape or runtime array contents.
- **cited — Private arity plumbing:** `normalize.rs:125`, `check.rs:415,633`, `eval.rs:123`, `execute.rs:6`, `target.rs:86` and `go_target.rs:107` prove full static-pointer arities are checked, shared across nested item scopes, privately lowered and absent from recipe JSON. New TS bindings use the same keys; missing propagates before the positive-arity/exact-length/index checks. No public arity accessor or DTO field is needed.
- **cited — Lexical policy plumbing:** `recipe.rs:29,526,546`, `check.rs:295,371`, `input.rs:13,34,147` and `retained.rs:50` supply the closed policy, admission/overlap order, original-root depth and lexical-only discarded-tail walk. The new TS parser/retained templates share token spans and the distinct capture versus discarded-Unicode details. Padding does not reapply source decoding at later stages.
- **cited — Current old-template families:** format 6 uses current Rust/Go source templates; previous families now also use `legacy_v5`, `legacy_v4_v5` and `legacy_v1_v5`. None enters the TS write scope. Preserve all new runtime_sources routing and positional_bindings bodies when changing only the Go configuration constructor.
- **cited — Exact corpus matrix:** retain 2,836 earlier applicable fixture executions; add all 70 Binary64 vectors under format 5, 93 positional text/helper vectors from the 99-vector constructor, three new mixed source/model vectors, and all 70 Binary64 vectors under format 6. This is 3,072 planned fixture executions before separate TS-specific controls. Name the six positional decoded-value cases and earlier seven raw decoded-value cases as inapplicable; do not silently remove branches. Constructor counts are source/prior-handoff inventory, not tests executed in this refresh.
- **cited — Three mixed vectors:** `fixtures/normalization_positional.rs:98,112` composes bundle input, positional preparation, raw capture and numeric policy with three modeled Binary64 stages. Preserve independent outcomes true for consumed null/discarded 1e999 plus duplicate raw capture and -0, false for null operands plus 0.10000000000000001, and true for empty operands plus explicit -0.0 default.
- **cited — Required profile delta:** execute prefixItems, items:false, minItems/maxItems and minLength after preparation; retain nullable ancestors and arrays of tuples. No inspected new fixture requires uniqueItems:true, so its located preflight refusal remains. Exact instance-pointer/multiplicity qualification remains required during implementation; do not claim it was rerun by this source refresh.
- **inferred — Scope remains bounded:** all refreshed requirements fit the same 31 paths. Recipe/checker/reference/parser/templates and existing positional/Binary64 fixtures stay read-only; the new TS test/support files import those fixtures intact. No additional production path or runtime dependency is requested. Any new need to modify those frozen dependencies must be returned for a separate scope decision before implementation.

## Refreshed binding after raw capture

The binding design now includes the concrete standalone API/package, exact
integer and floating representation, raw capture and retained-document helpers,
Unicode-scalar ordering, prototype-safe records, and stable schema-finding
multiset presentation. The first native lane records TypeScript6.0.3 and the
actual Node/V8 engine; browser or unmeasured engine support is not inferred.

Focused numerical preparation at immutable source6c78676 established a feasible
strategy: JSON grammar first, bigint integer fast paths, guarded Number conversion,
reference decimal-key equality for exact floats, and zmij-compatible layout over
native shortest digits. Node22.23.1 matched13,455 finite patterns,28 midpoint
cases and35 existing numeric cases;29 scalar pipelines also matched. These are
private strategy experiments, not target implementation or format5 qualification.
The actual dependency graph enables serde_json float_roundtrip through jsonschema;
its inactive fallback parser must not be ported.

Implementation follows the integrated Binary64 and positional contracts. It must
refresh their checked metadata, operation variants, input policies, target-report
versions and frozen legacy sources before dispatch. Raw format4 helpers/cases are
required; seven decoded-value API cases are named as inapplicable to the text-only
TypeScript API. No required corpus branch may be silently skipped.

The native runtime validates a closed qualified source-schema profile without
coercing bigint into Number. The independent profile audit is still completing;
unsupported refinements must refuse before any artifacts. A pre-existing pinned
validator discrepancy in numeric uniqueItems is being separately recorded and
must not be silently copied or described as a TypeScript-introduced defect.
No TypeScript adapter, passing full gate, consumer migration or release is claimed.

## Final integration dispatch delta

# Final TypeScript dispatch delta

Pinned integration: `1f8e319cf153c348a6c434c6e74939f4aa587125`.

**The exact 31-path write scope is unchanged.** It matches the persisted TypeScript
story revision 7: 11 existing files and 20 proposed new files. No additional
production, existing-test or template write path is needed. `scope-entries.json`
refreshes every existing path to this integration's exact blob/hash.

The normalization Plan, checker, recipe/input/retained/evaluation code, Rust/Go
runtime families, CLI source and positional/Binary64 fixture constructors match
frozen positional source `eb2e5d60e9e803993417df39563bc744dbcd36fc` exactly.
Structural realization production matches `213d4b8a8338763346cad2bc92cee826430726c9`.
The older packet's structural emitter hashes therefore update to the reviewed
finite codecs; they are read dependencies, not TS write authority. The existing
normalization Binary64 test's changed expectation now checks successful structural
finite wrappers instead of the retired blanket refusal; its normalization fixture
and 70-vector corpus did not change. Structural TypeScript remains separate.

The integrated TypeScript design is byte-identical to the proposed design file
from the positional refresh. Source proof contains 84 current file hashes, the
previous-packet comparisons and the empty protected production/fixture diffs.
Public documentation and planning differences are the recorded integration, not
an unreviewed normalization-runtime change.

## Added positional controls within the same test scope

The new private `adversarial_plan` and `independent_cases` helpers in
`tests/normalization_positional_adversary.rs:53,82` supply **19 additional text
vectors**. The coordinator accepted their addition to the planned native fixture
inventory: **3,072 → 3,091**. Preserve the literal inputs, independent outputs and
ordered findings, repeated fresh-call controls, signed-zero bits, escaped source
paths, source-order depth/Unicode precedence and unknown-branch grammar priority.

The separate `uniqueItems:true` control at that file's line 264 is an explicit
TypeScript **generation-refusal** control under the already bound closed profile.
It does not add runtime uniqueness support or enter the 19 text-vector count.
Assert the original source-qualified keyword pointer and that no Realization or
report is returned. Keep the original reference/adversary test unchanged. This
updates test accounting; it does not silently drop a required refinement.

Concrete copy route, requiring no existing helper visibility edits:

1. In the already scoped new
   `tests/fixtures/normalization_typescript.rs`, add a dedicated nested module
   containing copies of the pinned adversary's small tuple/read/policy/position
   constructors, `adversarial_plan` and `independent_cases`. Preserve those source
   bodies and raw expected strings; expose new fixture wrappers from this new
   module. Cite the immutable source commit and original file hash in the copy.
   Do not include the complete old integration-test module or its native build
   harness. This avoids importing unrelated Rust/Go test executions or editing
   private old helpers merely for reuse.
2. `tests/support/normalization_typescript.rs` inventories the 19 vectors in source
   order; `normalization_typescript_native.rs` executes them through the emitted
   package, using the existing scoped TypeScript assertion fixture. Keep expected
   numbers and raw strings lossless. Add source-union/nested-Position controls from
   the adversary's line 220 to the new schema/generation tests.
3. Reconstruct the two-slot uniqueItems schema in the already scoped
   `normalization_typescript_schema.rs` and assert preflight refusal. Private arity
   fault-injection and missing-propagation controls belong in test-owned generated
   copies or internal test code, without a public mutable metadata API.
4. Adapt the CLI refusal/drift scenarios in the already scoped new CLI TS test.
   Existing decoded-value provenance checks remain reference/Rust controls because
   TS has no value-input API. The Go fixture's invalid raw UTF-8 byte-input cases
   are not a TS text entrypoint; retain separate strict-UTF-8 base64-helper tests
   and located literal/escaped UTF-16-surrogate controls instead of claiming those
   byte-API cases executed unchanged.

The four positional and four structural adversary source/fixture files are retained
as hashed read dependencies. None enters the TS write scope. Full format-1–6
Rust/Go maps still require capture at one actual generator version before any
shared TS edit; no map capture or target build was run in this check.

## Available native tool paths

- Node: `/usr/bin/node`; observed `22.23.1`, V8 `12.4.254.21-node.56`.
- Compiler: set `ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js`.
  `/usr/bin/node` with that exact script and `--version` returned `Version 6.0.3`.
- `/usr/bin/tsc` resolves to `/usr/lib/node_modules/typescript/bin/tsc`; its direct
  version query also returned `Version 6.0.3`.

All four read-only version queries exited 0. `toolchain.json` retains exact commands,
outputs and executable/compiler-file hashes. These identities match the retained
numeric-strategy qualification; no numeric or schema parity run was repeated.

No repository edits, builds, implementation or worktree lifecycle actions occurred.
The coordinator's gates and explicit implementor/worktree dispatch remain pending.
This final source receipt needs only a delta recheck if that dispatch changes the
integration source pin.

## Measured equality compatibility boundary, 2026-09-06

The implementor measured a frozen reference discrepancy before target acceptance. In format 5 or 6 an admitted Integer/Integer equal condition returns true for input {"a":1.0,"b":1.0}, even with no binary64_inputs; format 1 refuses integer_representation at /branches/eq/0/value/condition. The exact source bundle, recipes, commands and outputs are retained in ess-typescript-design/implementation/integer-equality-observation/observations.json. At b55efa4cd6c379217f60b98fb30f28f70e526ee6, crates/generate/schema-contract/src/realize/normalize/check.rs:988 admits checked Binary64 pairs or the existing comparable scalar types. execute.rs:28 enables the runtime flag recipe-wide, and eval.rs:367 takes floating equality whenever both values carry floating tokens.

Coordinator decision: TypeScript preserves this actual reference behavior, with literal qualification and explicit documentation. It does not introduce a target-only generation refusal, alter reference/checker semantics, or change historical Rust/Go output. The design's checked-Binary64 statement describes static typed admission; it must not be presented as a guarantee that Integer inputs in these formats enforce integer-token eligibility during equality. This is a named compatibility limitation and a separate core-contract correction candidate, not a repaired Integer contract or broader typed Binary64 admission. All ordinary integer operations and mixed-token equality retain their measured existing rules. Independent read-only review of this decision is pending.

## Observed scope correction during implementation

The pre-existing CLI test at crates/edge/ess-cli/tests/normalization.rs:578 used TypeScript as the unknown-target example. Its unchanged assertion fails because this unit now admits that target. The recorded after-package run exited 101 with successful generation for that old negative case. The coordinator owns one exact test-only correction: replace the target spelling with unsupported javascript, retaining the original refusal, empty-stdout and untouched-destination assertions. New TypeScript CLI cases independently check successful generation and its supported/refused options. No existing assertion is dropped. Patch SHA256 9a98162c1696851d0b03935df7108f3db4b0026455541cda9c1f7ca1f17f5dab, before hash2c92c41dec2c197819cd7d3ff607a3acd76d534c0e27d215af9797e3ac974e5f.

This cited additional scope makes 32 total paths: implementor25, coordinator6docs plus this existing CLI test. Preserve the original31-path plan as historical; this is the explicit learned correction and does not authorize any broader edit to old cases/templates. The final unit handoff enumerates the coordinator test separately from the implementor manifest.
