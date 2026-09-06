# Finite Binary64 model fields

Binding design for `story:model-binary64-fields`, selected by the coordinator while
continuing the authorized source-pinned normalization work. Implementation follows
raw JSON format 4 integration as a separate serial unit; this page alone claims no
implemented primitive, passed gate or release. The scope originated at source
`60ffcb2`; the implementor must check exact final raw-format source before editing.

## Bound decision

Add one concrete model primitive, authored `Binary64`, serialized in the typed compiler enum as `binary64`. It denotes a finite IEEE-754 binary64 value, including both signed zeros and subnormals. `Decimal` continues to denote the existing exact, string-carried decimal. Integer, Decimal and Binary64 are distinct model types; assignment and nominal conversions do not silently cast between them.

This is a data representation and input-decoding contract, not new general floating arithmetic. Existing explicitly ordered normalization conversion operations remain the executable arithmetic boundary.

### Value and wire policy

- The value stores the finite binary64 bit pattern. NaN and positive/negative infinity refuse construction and decoding; serialization never substitutes null for them.
- Its JSON wire form is a JSON number token. Decode with nearest representable binary64, ties to even, after validating JSON numeric grammar. Preserve the sign of zero, including negative underflow. Admit subnormals and signed underflow-to-zero. Refuse overflow-to-infinity. A quoted number is not a Binary64 value.
- Serialization emits a finite round-tripping numeric spelling, preserves negative zero as `-0.0`, and retains floating lexical identity for integral floating values where crossing back into normalization would otherwise make them integer-operation operands. This is value round-trip preservation, not preservation of the original token spelling. Cross-language byte-identical float formatting is not claimed without a separately qualified formatter.
- Numeric equality uses finite IEEE comparisons: +0 and -0 compare equal. Signed-zero storage and serialized sign remain observable independently. No implicit epsilon comparison. Do not introduce hashing or a total-order promise in the initial primitive API. Where an adapter cannot maintain the required distinction, it refuses rather than normalizing the sign away.
- Do not invent defaults. Required Binary64 stays required; Optional retains existing omission/null rules. A Go checked wrapper's zero value may mean +0, but a missing JSON property must still be rejected if required. Binding and outcome literals remain text-only under current rules; adding this primitive does not make a literal string into a numeric default.
- Refuse `Map<Binary64, V>` in authored validation and programmatically constructed type validation until key spelling, collision, equality and ordering semantics are bound. Never let `types::key` fall through to `{type:number}` under `propertyNames`, which would reject every JSON object key. Binary64 map *values* are valid model data; normalization support depends on whether its explicit path language can reach them.

### Schema and checked model authority

The portable JSON Schema/OpenAPI/AsyncAPI projection can describe a numeric wire value. `type:number`, optional format hints, and even numeric bounds cannot by themselves enforce binary64 rounding, signed-zero preservation or original token admission. Do not label that projection a complete finite-binary64 validator.

The sealed compiler model must retain the primitive identity independently of that broad wire schema. Extend `ModelTypes` with concrete compiler-minted Binary64 location/type metadata, derived through checked type handles and actual wire names, or an equivalently concrete typed model view. This is not an imported annotation and not an arbitrary property bag. Plain imported `type:number` remains a general numeric schema; it does not acquire model Binary64 semantics.

Structural type projections may continue to emit wire representations with explicit source-located finite/precision/decoder obligations. They must not claim their `number`, `serde_json::Number`, or `EssNumber` type enforces Binary64. Executable codecs either implement the new contract or refuse the selected unsupported type before returning artifacts.

### Normalization policy

Keep the existing branch-specific `binary64_inputs` declaration. A model-owned external Binary64 leaf must be explicitly covered by a declared binary64 path before its raw JSON input can be normalized; a missing declaration must produce a located planning refusal, not hidden insertion of a path or a runtime-dependent loss of `-0` through the exact-integer reader. Unrelated numeric fields retain the existing exact policy, even when unused by the recipe.

Use compiler-owned Binary64 metadata to retain a distinct numeric type in checking. Permit it as input to the existing `binary64_to_integer`; keep the conversion's ordered multiply/minimum/maximum, finite intermediate checks, truncation toward zero, explicit range rejection and subsequent integer reject/wrap semantics unchanged. A plain exact integer or general number must not silently fill a modeled Binary64 output. The explicit literal and value-producing conversion proposed below are required for decoded defaults and newly computed floating outputs; read-only floating inputs would not satisfy that boundary.

Existing `NumberPath` supports field/items segments, not arbitrary map values or branch-discriminated paths. Initially support scalar roots, declared fields and lists where those paths are sufficient. Refuse a Binary64 map-value/ambiguous union boundary the declaration cannot represent. Do not add a wildcard segment incidentally; that is an authored normalization-vocabulary decision coordinated with the raw-input work.

Decoded-value APIs must clearly state that rounding/duplicate keys/token spelling lost before the call are unrecoverable. Their explicit Binary64 preparation converts available numeric values and preserves signed zero still present in the caller value; it cannot certify the original JSON token. Text APIs remain the evidence-bearing decoding edge.

## Format and old-reader consequences

Authored-language decision: reserve `ess/2` for Binary64, continue admitting unchanged `ess/1`, and refuse Binary64 use under `ess/1` during model validation. This is the selected authored-format boundary; its implementation and compatibility tests remain required. The repository requires a version consequence when names/meaning change; simply adding a new enum arm is not a sufficient compatibility argument.

Read evidence: `ess-domain/src/system.rs:53` supports only `[1]`; `Assembly::check_format` at :750 already emits a located unsupported-format diagnostic. `TypeRef::parse` at `types.rs:145` lacks header context, so format-dependent admission belongs in model/manifest validation, not an ad hoc global parser switch. Include both parsed documents and direct typed construction in the guard.

Old-reader tests must show the published old reader refuses a new-format document, the new reader refuses a Binary64 declaration in old format, and old fixtures retain exact canonical artifacts/digests. Test newtype, struct, command/event/error fields and nested Optional/List/Map paths so a format gate cannot be bypassed outside ordinary types.

`EssIr` is currently serialize-only and unversioned (`ess-compiler/src/ir.rs:1254`); do not invent `ess-ir/2` or add a global persisted field merely for this task. The new typed primitive changes source/contract identity naturally. Old IR/projection bytes need not change for models that do not use it. Source-model snapshots, full selected root identity and projection digests remain accounted through the existing model pin.

Reusing the existing explicit binary64 declaration and unchanged operations alone would not require a new normalization recipe/report version. However, that alone cannot construct source-required floating defaults or newly computed Binary64 output values. The two proposed new expression variants below therefore require a new recipe version, coordinated after raw-input format 4: ess-normalization/5, after verifying no concurrent unit has reserved it. Versions 1–4 must reject their use. The implementation must refuse a conflicting concurrent reservation rather than reuse its meaning. The authored ess/2 primitive decision is a separate format boundary.

### Explicit defaults and floating output construction

Current operations are insufficient: `Expr::Integer` constructs an integer representation, `binary64_inputs` only decodes values actually present in external input, and `binary64_to_integer` always returns an integer. None constructs a missing default `0.0`, negative zero or an arbitrary finite floating constant. Using the integer literal 0 as a Binary64 field default would require precisely the hidden coercion this design forbids.

Add two concrete, closed expressions in the coordinated next recipe format:

1. `binary64_literal { value: <JSON numeric token as a string> }` constructs a finite Binary64. Example: `{"op":"binary64_literal","value":"-0.0"}`. Retain the authored token in canonical recipe/source identity. Validate its JSON numeric grammar and finiteness through the existing checked numeric-literal path, including when it appears in an unused branch. Round once to binary64, ties to even, preserving signed zero and subnormals. A numeric JSON field instead of a token string would allow the recipe reader to lose information before checking it.
2. `binary64 { value: <required numeric expression>, steps: [...] }` explicitly converts a numeric value to finite binary64 and applies the existing ordered multiply/minimum/maximum steps, returning Binary64 without truncation. An empty steps list is the explicit integer/general-number-to-Binary64 conversion. Refuse nonnumeric, missing, null and nonfinite inputs; check finiteness after every step. Its floating steps have the same literal admission, evaluation order and signed-zero clamp semantics as `binary64_to_integer`. It does not add general floating addition, division or an unbounded expression language; an observed operation outside this closed set remains a concrete gap.

An explicit default is then:

```json
{
  "op": "fallback",
  "value": {"op": "read", "scope": "input", "path": ["ratio"]},
  "fallback": {"op": "binary64_literal", "value": "0.0"},
  "on_null": false
}
```

The `on_null` value is an authored source-semantic choice, not a new default. A computed floating output can use `binary64` over that fallback with a declared multiply step. Float-to-integer conversion remains a separate explicit operation; existing `binary64_to_integer` recipe meanings and bytes are unchanged.

Checking gives both new expressions a concrete Binary64 result. They may fill modeled Binary64 output fields and feed later numeric operations, but cannot become eligible for signed-integer arithmetic merely because their numeric value is integral. Literal `0.0`, an integer converted explicitly to Binary64, and a computed integral float must all retain floating runtime representation.

These expressions do not broaden input-token admission. A source field whose original token needs binary64 rounding still requires its explicit `binary64_inputs` declaration; a later conversion cannot rescue a token the exact input reader already refused.

Reference, generated Rust, generated Go and eventual TypeScript must lower both variants or refuse before artifact publication. Reuse the numeric step evaluator, but preserve the old truncating operation's behavior. Old readers must reject the new recipe format, and new readers must reject the new operations in versions 1–4 even in unused branches. New recipe tokens naturally change the recipe digest; report/root identity need no new field merely for these expressions. Coordinate target report version dispatch with raw format 4 rather than using the current equality check for only one model-root recipe version.

Qualification must cover explicit +0.0/-0.0 and fractional defaults, presence versus null, lazy unused invalid-at-runtime branches, literal admission in all branches, nearest-even conversion, overflow at an intermediate step, integral floating output followed by integer-operation refusal, and native target output bits/sign. Do not claim unchanged old generated Rust file bytes without verifying the shared recipe/runtime templates: adding enum variants currently changes their included source. Preserve versioned legacy templates if byte identity is required, or record a coordinated generated-artifact migration with new file digests; do not silently promise both.

Conformance is a separate format seam: Rust suite records serialize `Primitive`, while the emitted Go runner switches on primitive strings and currently falls through successfully for an unknown spelling (`ess-conformance/src/go/runtime.go:1558`). Do not publish `binary64` under an existing suite version and assume every old reader refuses. Either qualify it with a separately coordinated suite-format change or explicitly refuse affected conformance synthesis in the first implementation. Existing conformance format work may already reserve the next number.

## Exhaustive matches and adjacent consumers

The lexical scan found 22 files mentioning the enum/variants, not 22 exhaustive production matches. There are 15 exhaustive production match expressions in eight files, plus two exhaustive matches in the primitive-map class test. The distinction matters: adding enum arms will not automatically expose wildcard/runtime-string fallthroughs.

| Source | Evidence and required review |
|---|---|
| `crates/specify/ess-domain/src/types.rs:77` | Exhaustive `Primitive::as_str`; add enum arm, ALL, parsing/display/serde tests, map-key refusal and old-format admission coverage. |
| `crates/generate/ess-gen/src/types.rs:483` | Exhaustive `primitive`; `key` at :524 has a fallback and must explicitly reject Binary64 keys upstream. This mapping feeds Schema, OpenAPI and AsyncAPI. |
| `crates/generate/ess-synth/src/rust/feasibility.rs:80` | Exhaustive primitive helper inventory; the feasibility walk is also a candidate for explicit selected-Binary64 refusal. |
| `crates/generate/ess-synth/src/rust/layout.rs:293` | Exhaustive native type mapping. Raw f64 would break existing Eq derives; use a checked local wrapper if supported. |
| `crates/generate/ess-synth/src/rust/wire.rs:574,587,680,707` | Four exhaustive value/key encode/decode matches. Value parser must consume raw number tokens; unsupported key arms must be unreachable through a checked refusal. |
| `crates/generate/ess-synth/src/go/layout.rs:863` | Exhaustive Go primitive mapping; checked wrapper or target refusal. |
| `crates/generate/ess-synth/src/go/http.rs:534,545,697,720` | Four exhaustive value/key encode/decode matches; a further constructor match at :623 has a wildcard. |
| `crates/verify/ess-conformance/src/witness.rs:496,523` | Two exhaustive kind/witness matches. Binary64 witnesses must be finite and distinguishable after rounding, not just different decimal spellings. |
| `crates/generate/schema-contract/tests/normalization_go.rs:305` | Primitive::ALL class plus two exhaustive test matches; add an explicit Binary64-key refusal case, not an accidental numeric propertyNames schema. |
| `crates/generate/ess-synth/src/go/mod.rs:198` | Constructor match has `other => panic`; wrapper registration/emission at :521/:554 also needs explicit handling if supported. |
| `crates/generate/ess-synth/src/go/refusal.rs:415` | Current map-key refusal names Bytes only; cannot assume a new float key is representable because Go happens to permit float64 keys. |
| `crates/generate/ess-synth/src/plan.rs:1024` | Mechanical literal construction recognizes String; retain non-text numeric refusal rather than broadening literals. |
| `crates/verify/ess-conformance/src/input.rs:563` | Primitive/node match has `_ => None`; add finite typed binding or explicit unsupported coverage. |
| `crates/verify/ess-conformance/src/scenario.rs` | Direct Primitive occurrences are test fixtures, but serialized suite `Holds`/shape records carry Primitive and are an old-reader boundary. |
| `crates/specify/ess-domain/src/binding.rs:1212` | Representation walk treats non-String primitives as non-text; retain text-literal refusal. |
| `crates/specify/ess-domain/src/system.rs` | Current Primitive occurrences are test setup, but format validation is a real new implementation surface. |
| `crates/specify/ess-domain/src/domain.rs`, `entity.rs`, `view.rs`, `command.rs` | Direct Primitive occurrences are existing test setup, not exhaustive primitive mappings. Audit generic field/identity/literal paths and add only necessary version/position tests. |
| `crates/specify/ess-domain/tests/billing.rs:425`, `crates/verify/ess-conformance/tests/suite.rs` | Primitive::ALL and fixture coverage must receive an explicit expected result for the new primitive. |

Additional surfaces not found by enumerating Primitive arms:

- `ess-gen/src/model_types.rs`: currently retains roots, JSON definitions, newtypes and provenance only; add sealed numeric metadata without changing old serialized bytes.
- `schema-contract/src/realize.rs`, `realize/{ts,rust,go}.rs`, `realize/normalize/{check,source}.rs`: preserve typed numeric semantics and report finite/decoder obligations; normalize currently converts model wire shapes into generic `Kind::Number`.
- `ess-synth/src/rust/{mod,json,items,entity,system}.rs`: fixed primitive module, numeric token codec and generated Eq derives; emitting a wrapper only for models that need it avoids changing old emitted files. Go's fixed wrapper package and HTTP helpers have the same old-byte concern.
- `ess-primitives/src/facts.rs:48,89,103` and `node.rs:34`: an available finite-number representation, but not an untouched drop-in Binary64 contract. Equality uses numeric == (signed zeros equal), ordering uses total_cmp (signed zeros distinct), and Node decoding passes through serde_json numeric conversion. Do not silently change these old common semantics or treat their decoded nodes as raw-token provenance.
- `ess-conformance/src/go/{runtime,predicate}.go`: float decoding, primitive string dispatch, equality/ranking and predicate literal parsing escape the Rust enum exhaustiveness checks.
- `schemas/generated/ess.schema.json`, format/reference documentation and old-reader fixtures: regenerate only through existing Rust tooling after the format decision. Do not hand-edit generated schemas.

## Smallest complete implementation boundary

1. Bind the primitive, authored-format gate, finite/signed-zero/equality policy, map-key refusal and old-reader tests.
2. Implement model wire projection plus sealed Binary64 metadata and unchanged source-pin accounting.
3. Support explicit Binary64 model inputs, floating defaults and newly computed floating outputs through reference and generated Rust/Go normalization, including the new version-gated expressions, declared paths, located missing-policy/unsupported-path refusals and finite value/output checks. Run the existing floating conversion corpus through a genuinely model-owned numeric field and add default/output-construction cases.
4. Structural adapters report their representation/decoder obligations. Whole-system Rust/Go/web/clap synthesis and conformance either implement their finite wrappers/codecs/suite contract completely or refuse selected Binary64 surfaces before successful partial artifacts. A refusal is an explicit supported boundary, not a floating implementation claim.
5. Add full-system codec support separately if it is required in the first published outcome. It needs local checked wrappers (private finite value, checked constructor, read access), rather than raw f64 aliases or exposing a mutable field that can hold NaN. Keep numeric equality (+0 == -0) separate from serialized signed-zero preservation. No invented default arithmetic or map-key support.

A source/model/normalization path plus truthful refusals is a substantially smaller unit than completing every synthesized HTTP/browser/conformance surface. The story's final report must state which of those boundaries actually execute.

## Acceptance evidence to require

- Generic model field/newtype/nested-list fixtures produce numeric wire schemas; Decimal still produces its old patterned string and old fixtures/digests remain byte-identical.
- The published old reader rejects the new authored format. New readers reject Binary64 under ess/1 and all Binary64 map keys, including direct typed construction.
- Explicit branch numeric paths preserve +0/-0, signed underflow, smallest subnormal, nearest-even ties at 2^53 and finite extremes; overflow/NaN/infinities/string numbers refuse. Missing path policy and unsupported map-value paths refuse before generation.
- 1.001 × 1000 truncates to 1000 before later integer scaling, and boundary/wrapping expectations remain those of existing ordered numeric fixtures. Unselected/exact fields still reject lossy token conversion.
- Decoded-value and raw-text tests make different provenance claims explicitly; signed-zero tests inspect bits or emitted sign, not ordinary JSON equality alone.
- Required/optional/missing/null behavior, lazy fallback and later-stage output typing are covered. Integer/Decimal-to-Binary64 assignment is not introduced by structural schema widening.
- Supported generated targets execute the cases; unsupported synthesis/conformance routes return source-located refusals with no partial-success artifact set. Old Go conformance readers cannot silently accept a new primitive spelling.

## Scheduling and collisions

Domain type/format and the shared model projection are scoped here; implementation begins only after the overlapping raw JSON unit has passed review and integrated. They are separate from raw JSON token capture at the source-file level until normalization admission is integrated.

Strong collisions with raw normalization: `normalize.rs`, `normalize/check.rs`, `normalize/source.rs`, model/native normalization fixtures and `docs/design/source-pinned-data-normalization.md`. The two explicit value-producing expressions require the next recipe format and shared evaluator changes. Preserve complete old format 1–4 generated file maps before editing shared templates, including the evaluator and numeric helpers. Reuse the final raw format vocabulary; do not add an input path segment incidentally.

Strong collisions with TypeScript normalization: native numeric-kind representation, exact versus binary64 admission, signed-zero serialization, schema runtime obligations and model metadata consumption. TypeScript should consume one compiler-owned policy; it must not derive a second answer from `type:number`.

Primary implementation footprint is approximately 15–20 production files for primitive/model/normalization support with early refusals; full Rust/Go synthesis and conformance support extends beyond that into wrappers, codecs, runtime templates and suite-version work. This is scope, not a time estimate.

## First implementation boundary

The complete first unit includes authored format 2 admission, the Binary64 primitive,
compiler-owned model metadata, structural obligations, and executable reference plus
Rust/Go normalization for explicit inputs, defaults and computed finite outputs.
Whole-system synthesis and conformance must return located pre-publication refusals
for selected Binary64 use unless their complete finite codecs and versioned consumer
contracts are implemented and tested in this same unit. Do not silently emit a raw
mutable float field that can hold NaN or an old conformance suite that accepts an
unknown primitive. This bounded support is sufficient for the observed standalone
model-data and normalization consumer; it does not claim all adapter coverage.

Old authored model and recipe meanings remain unchanged. Freeze the additional
shared emitted sources required by the new expressions, and measure complete old
file maps before live edits. Formats 1–4 reject the new operations in all branches.
Explicit version gating must also cover programmatically constructed model/recipe
values, not only the normal YAML/JSON parse path.

### Admission, equality and conformance publication decisions

Modeled Binary64 normalization requires format 5. Formats 1–4 refuse a selected
Binary64 model closure, including output-only fields, before planning or artifact
publication. Their existing qualified-schema numeric operations remain unchanged.
The refusal does not reinterpret an older recipe against a new model primitive.

In format 5, `equal` may compare two explicitly typed Binary64 values using finite
IEEE equality, so positive and negative zero compare equal. This supports observed
zero-default branches without an implicit integer cast. Mixed Integer, general
Number or Decimal operands do not silently acquire Binary64 identity. Old recipe
versions retain their existing equality admission and runtime behavior; additional
frozen evaluator templates must preserve their emitted files.

A refusal inside conformance synthesis is insufficient if a caller still writes
suite or runner artifacts. Guard both the actual CLI publication path and directly
constructed suite calls to the Go emitter before producing output. If the emitter
needs a checked/fallible API, update every caller and report the public Rust API
change. Do not publish a new primitive through an old suite contract whose Go
reader treats an unknown primitive spelling as successful validation.
