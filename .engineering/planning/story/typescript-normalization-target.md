---
format: aep.planning-md/1
id: story:typescript-normalization-target
kind: story
status: draft
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
- confidence: inferred
  path: crates/edge/ess-cli/tests/normalization_typescript.rs
- confidence: cited
  path: crates/generate/schema-contract/Cargo.toml
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/go_target.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/target.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_collection.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_condition.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_expression.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_input.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_numeric.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_retained.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_runtime.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_schema.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/ts_value.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/typescript_schema.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/normalize/typescript_target.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript_legacy_maps.json
- confidence: inferred
  path: crates/generate/schema-contract/tests/fixtures/normalization_typescript_tests.ts.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_typescript.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_typescript_legacy_bytes.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_typescript_native.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/normalization_typescript_schema.rs
- confidence: inferred
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
revision: 6
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

- **cited — Source authority:** production baseline is ESS `6c6620b3783cbd41ca31a998805bc8e51e0c3515`; story and positional binding are read at coordinator commit `2bbb0128ab10f2623675b3b2418ca507b1f4d4b2`. Their production trees, Cargo.lock and Taskfile.yml are identical. No moving positional/structural implementation was used as frozen proof. This is preparation for `story:typescript-normalization-target`, with a mandatory post-positional source refresh.
- **cited — Binding authority:** `docs/design/typescript-normalization.md`, `docs/design/positional-array-normalization.md`, and the TypeScript story require a standalone ES2022 JSON-text execution library, checked finite Binary64 and format-6 positional policies. The checked-arity appendix requires private `position_arities` keyed by full escaped static expression pointers. Format 6 is bound but is not implemented at the public source baseline.
- **inferred — Primary surface and size:** one normalization target in `schema-contract`, four narrowly edited existing production files, one feature-gate registration, two new Rust production modules, nine TypeScript runtime templates and nine new test/fixture/support files. The proposed 31 paths also include six documentation paths, three of which remain coordinator-owned. This is a parser/evaluator/validator plus target-packaging unit; structural TypeScript aliases are not its runtime.
- **inferred — Confidence: medium:** existing constructors, call sites and compatibility seams are cited below; new filenames are an explicit proposed split. Final positional symbols, frozen template layout and complete corpus counts still require refresh after that dependency integrates.

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
- **cited — Exact report version dispatch:** recipe 1 or 2 emits `ess-normalization-target/1`; recipe 3 emits `/2`; recipes 4, 5 and bound 6 emit `/3`. `target.rs::finish` already implements the first five selections; positional integration owns the addition of 6. A third execution language does not change root meanings, recipe/file digest inputs, field order or attestation guarantees. This serializer-only report contract has no additional meaning that would justify a new common report version solely for TypeScript. If implementation changes any of those semantics, stop and bind that separate format consequence.
- **cited — Source/file accounting:** reuse `target::sources` and `finish`. Preserve `source.recipe.json`, every `schemas/{source-key}-{root-name-digest}.schema.json`, every `sources/{bundle-digest}.bundle.json` and `sources/model-{projection-digest}.schema.json`, complete Root/ModelIdentity coordinates and recipe canonical digest. `Report.files` hashes all emitted inputs except the report itself; `normalization-report.json` is then added to Realization.files. No fixture branch, model selection, schema refinement or provenance coordinate disappears from this accounting.
- **inferred — Complete new package paths:** emit `package.json`, `tsconfig.json`, `README.md`, `schema-profile.json`, `src/index.ts`, `src/value.ts`, `src/numeric.ts`, `src/input.ts`, `src/retained.ts`, `src/schema.ts`, `src/expression.ts`, `src/condition.ts`, `src/collection.ts`, generated `src/bindings.ts` and `src/schemas.ts`, plus the retained files and report above. All exact bytes enter the existing digest map. `schema-profile.json` is fixed typed capability/refusal metadata emitted by the qualifier, not an authored policy or configurable profile. Compiled `dist/` is a consumer build output and is not preclaimed in generated-file digests.
- **cited — Native API:** export Finding and the discriminated `NormalizationResult` union, and a no-argument Normalizer exposing `normalize(branch, jsonText)` and `normalizeBase64Json(branch, encoded)`. Success contains compact JSON text without final LF; refusal contains complete located findings, never partial output. There is no object-input API or caller-replaceable recipe, filesystem lookup or network dependency.
- **inferred — Package implementation:** private version-0.0.0 ESM metadata, explicit exports/types to `dist/index.js`/`dist/index.d.ts`, strict ES2022 compilation, declaration emission and `.js` import specifiers in generated TypeScript. Use no Node type package or numeric/schema runtime dependency. Validate package identity before source generation; publish the exact accepted name grammar and negative controls. The binding has not established whether scoped npm names are required, so that grammar is a bounded implementation choice to record, not an excuse to infer JavaScript identifiers from package or wire names.

### Operation and value coverage

- **cited — Exhaustive operation matrix:** the pinned Expr has 23 variants: Null, Boolean, String, Integer, Read, Field, Record, List, Fallback, Choose, Arithmetic, Map, DistinctCount, Concat, Join, IntegerString, ConcatLists, ItemIndex, SelectMap, Find, Binary64Literal, Binary64 and Binary64ToInteger. Format 6 adds Position, for 24. Conditions are Present, Boolean, Equal, Greater, StartsWith, All, Any and Not. Integer arithmetic has Add/Multiply and Reject/Wrap; binary64 steps are Multiply/Minimum/Maximum and conversion range Reject. Use exhaustive Rust matches so a new checked operation cannot silently disappear.
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
- **inferred — Step 1, final freeze:** after positional integration, refresh exact recipe/Plan/checker/parser/evaluator symbols, arity plumbing, all schemas/fixtures and legacy template routing. Record all twelve complete Rust/Go format-1–6 file maps using identical package/module identities and one fixed generator version before any shared TypeScript edit. Retain exact source paths/bytes and report bytes, not only selected template hashes. Existing public `normalization_legacy_bytes.rs` covers formats 1–4 and normalizes its generator field; retain that old guard, but it is not the complete same-version witness required here. Do not normalize the new witness or regenerate expected hashes to hide a failure.
- **inferred — Step 2, qualifier/package:** implement the normalization-local configuration and additive API, exact package/profile emission, source closure traversal and preflight refusals. Establish deterministic complete-file accounting and invalid-package/source/refinement controls before returning executable artifacts.
- **inferred — Step 3, value/input core:** implement private values, scalar ordering, exact number reader/formatter/comparator, two-phase token parsing, strict retained helpers and positional preparation together around one token-span representation. Qualify midpoint/limits, prototype names, escaped/literal surrogate and BOM boundaries, duplicate/depth/error precedence, capture bytes and discarded tails.
- **inferred — Step 4, execution:** lower all 24 expressions and eight conditions, preserve stage/context/lazy behavior, checked Binary64 constants/equality and static private Position arities; add all operation and defensive-arity controls. Do not infer Position type from selected index or runtime array contents.
- **inferred — Step 5, schema and complete native corpus:** compile/run the standalone package, retain independent expected JSON/bit/refusal observations and compare the complete stage-schema finding multiset. Add mixed bigint/float bounds/const/enum and tuple min/max/prefix/tail multiplicity probes; freeze actual Node/TS versions and full fixture inventory. No easiest-subset branch filtering or uniqueness-bug emulation.
- **inferred — Step 6, CLI and integration:** prove module refusal, source protection, all-file drift and preflight failure atomicity through the actual CLI; recheck all twelve fixed-generator Rust/Go file maps after shared edits; run focused native/offline checks, then repository task check and the required site gate for documentation changes. These are future required checks, not work executed in this scoping task.

### Collisions and remaining refresh boundaries

- **cited — Would collide with:** positional work edits `normalize.rs`, `normalize/target.rs`, `normalize/go_target.rs`, CLI `src/normalize.rs` and `docs/design/source-pinned-data-normalization.md`. It also changes the recipe/check/input/retained/eval/execute read dependencies and current template selection. Those are actual shared seams, so TypeScript implementation must start after a frozen positional integration and refreshed scopes. Do not claim two units are disjoint merely because the eventual TS runtime templates are new.
- **inferred — Structural unit separation:** the proposed local configuration and new test names leave structural `realize.rs`, `realize/{ts,rust,go}.rs` and their codec tests outside the TS write scope. A later required widening must be explicit before overlapping writers begin. Shared changelog and public formats/generation guidance remain serial coordinator work.
- **inferred — Not yet established:** final positional symbol names/layout/corpus totals and expected tail-diagnostic multiplicity; exact accepted npm package-name grammar; current engine/compiler requalification; a complete fixed-generator format-1–6 legacy witness. None is evidence of a completed implementation. The source-hash packet distinguishes the immutable public baseline from the bound but pending positional contract.

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
