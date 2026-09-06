# Standalone TypeScript normalization

This design extends the existing source-pinned normalization contract to a native
TypeScript execution target. The reference remains the checked recipe and retained
source schemas described in `source-pinned-data-normalization.md`; structural type
aliases do not implement execution or validation.

## Implemented target (ESS 0.20.0)

`Plan::typescript(package)` and `normalize-generate --target typescript` now emit
and execute the contract below for recipe formats 1–6. Historical preparation
receipts later in this document describe their named source snapshots; their
pending-dispatch statements do not describe this implementation.

The qualified package is private ESM, version `0.0.0`, with strict ES2022/NodeNext
build inputs and only the root public export. It has no runtime dependencies.
Package identities use one optional `@scope/` prefix, at most 214 ASCII bytes,
and parts beginning with `a`–`z` followed by `a`–`z`, digits, `-`, `_` or `.`.
Generation emits files without installing packages or compiling them.

The existing checked Plan remains the first admission boundary. The fixed target
profile then qualifies boolean schemas, local references and their siblings,
`type`, `const`, `enum`, object/required/additional/property-name constraints,
`items`, `prefixItems`, minimum/maximum and exclusive numeric bounds, Unicode
string length, array length, `anyOf`, `oneOf` and `allOf`, plus exactly the frozen
Bytes pattern. This keyword list does not broaden Plan shape admission. For
example, the numeric type-plus-enum intersection
`{"type":"number","exclusiveMinimum":-1,"exclusiveMaximum":1.0,"enum":[-0.0,0,1]}`
and `{"allOf":[{"type":"number","minimum":1},{"type":"number","maximum":-1}]}`
remain `normalization_shape` refusals before target selection. Ordinary string
enums and model enums execute. Private conjunction/const validator probes are
internal validation evidence, not public Plan/target support.

`uniqueItems: true`, every unqualified pattern and any other unsupported
constraint refuse as `typescript_schema_profile` at their original qualified
schema pointer before Realization/report creation. `uniqueItems: false` is inert.
Formats and content encodings are annotations; schema defaults do not execute.
The fully hashed `schema-profile.json` states these limits. Source schemas retain
all annotations and their original bundle/model identities.

Native qualification uses Node 22.23.1 (V8 12.4.254.21-node.56) and TypeScript
6.0.3. It executes all 3,091 shared text/helper vectors, 31 public schema controls,
56 Unicode/prototype/lexical/arity controls, seven private schema conjunctions,
25 equality controls and 26,966 numeric cases derived from 13,455 finite bit
patterns plus 28 midpoint/huge-exponent tokens. Applicable runtime vectors repeat
against the same Normalizer to check call isolation. The 13 named decoded-value
API cases do not apply to the text-only API; none of the 3,091 text/helper cases
is skipped. These measurements qualify the tested engine and cases, not every
binary64 value or another engine. Equality measurements compare the reference
and native TypeScript; no new Rust/Go native equality run is implied.

Before shared edits, twelve complete Rust/Go emission maps for formats 1–6 were
captured at actual generator 0.19.0: 218 files including exact report bytes.
The literal witness does not rewrite the generator version. A later release
projection requires separate qualification; this target changes no old map byte.

## Input and result boundary

The generated library accepts an exact branch name and one complete JSON text.
Successful execution returns complete JSON text; refusal exposes the existing
located `pointer`, `rule`, `detail` findings. It performs no filesystem or network
lookup. A JavaScript object input API is not initially offered, because its caller
may already have lost integer precision, duplicate members or token identity.

This JSON-text boundary is an engineering choice preserving the existing exact
integer contract. Integral input tokens in the reference's signed/unsigned 64-bit
range use internal bigint values; floating tokens and explicitly selected binary64
inputs use number values. `1`, `1.0` and `1e0` therefore retain the distinctions the
reference's integer operations observe. The target does not broaden admission to
arbitrary-size integers. No internal numeric wrapper becomes an output field.

## Parsing, evaluation and serialization

The text parser checks complete JSON grammar before recursive decoding, preserves
duplicate-member detection, rejects invalid Unicode and follows the reference's
depth and exact-number policies. Binary64 paths remain authored branch data.
Missing values use a private sentinel distinct from JSON null. User member names
are never resolved through JavaScript prototypes.

Evaluation preserves every existing recipe operation and condition, including
ordered requirement checks, lazy choices/fallbacks, lexical item/index scopes,
original indices after filtering and enclosing-scope find fallbacks. Checked
signed arithmetic uses bigint; wrapping explicitly reduces to signed 64 bits.
Floating steps retain source order and checked finite constants, emitted from
their binary64 bit patterns rather than reparsed approximate decimal literals.

Object traversal and emitted member ordering follow Rust scalar/UTF-8 ordering,
including non-BMP keys and numeric-looking names. Serialization preserves exact
integers, negative zero and integral-float identity. Native JSON.parse/stringify
alone cannot satisfy these boundaries. Number formatting and decimal admission
require differential evidence at rounding ties, subnormals and exponent limits.

## Source-schema validation

A Rust feasibility pass examines each retained selected schema before returning
any artifacts. The runtime validates against those source schemas, not against
generated TypeScript aliases. References use generated local bindings only.

The initial target qualifies a closed schema profile covering its scalar, object,
array, union and model fixtures. Required/additional fields, enums, numeric bounds,
lengths and collection refinements must either be implemented with reference
evidence or refused at their qualified schema pointer during generation. No
constraint is discarded because a library cannot represent bigint. Numeric schema
equality is distinct from recipe integer eligibility. Unqualified regex profiles
and numerical refinements remain explicit generation refusals until qualified;
the frozen base64 profile is required for the model Bytes projection.

Ajv is not a drop-in validator for this boundary: its locally inspected numeric
type checks require JavaScript number. A new runtime dependency needs evidence
that it preserves the exact contract rather than coercing bigint values.

## Artifacts and compatibility

The target emits a standalone ES2022 TypeScript library, its package/build metadata,
all retained recipe/model/bundle schemas and the deterministic normalization report.
The native package identity is validated before artifacts are returned. Generation
is pure: it emits build inputs and does not run npm, a compiler or a network client.

Reuse `target::sources` and `target::finish` for source acquisition and file digests.
Normalization-specific configuration may represent TypeScript package identity
while retaining the exact existing Rust/Go serialized configuration. Do not alter
the separate structural TypeScript configuration or previously generated Rust/Go
bytes. If a persisted envelope change is required, name and test the format
consequence before implementing it; a language implementation alone is not one.

The CLI exposes the target through normalize-generate, rejects Go-only module
options and preserves complete preflight, source-input protection and read-only
drift checks. The generated package requires no hidden local path or runtime
compiler service.

## Evidence required

Native TypeScript compilation and Node execution use the existing v1, v2, numeric,
model and base64 fixtures against the reference. Every current operation executes;
fixtures must not be silently filtered to the easiest subset. Additional cases
cover prototype-sensitive keys, scalar ordering, duplicate keys, Unicode, depth,
signed/unsigned limits, integral floats, negative zero, lazy failure precedence and
qualified versus refused schema constraints. Reports retain complete source pins,
and the existing Rust/Go artifact digest maps remain unchanged.

The target must execute the checked lexical-capture contract introduced by format
4, including retained-document helpers. It must also consume the final integrated
Binary64 and positional-input contracts before its implementation is accepted.
Those units define their semantics once; TypeScript must not derive a second
policy from broad JSON Schema types. Consumer dispatch remains outside the target.

The selected configuration is a private normalization-local serde enum tagged
`language`: `Rust { package }`, `Go { package, module }` and
`Typescript { package }`. Existing Rust/Go field order and report bytes remain
unchanged. The separate structural fieldless TypeScript configuration is untouched.
Recipe formats1/2 retain target report1, format3 report2, and formats4/5/6 report3.
The new API is additive; the report's configuration field remains private.
There is no authored or configurable schema-profile identifier. Deterministic
runtime inputs and a fixed `schema-profile.json` enter the ordinary file-digest map;
unsupported profiles refuse before a Realization or report is returned.

## Concrete library and CLI contract

The additive API is `normalize::Plan::typescript(package: &str) -> Result<Realization, Refused>`.
The existing structural `realize::Plan::typescript()` remains a separate API.
`normalize-generate --target typescript --package <name>` emits a checked package;
`--module` remains a Go-only option and refuses here before any output is written.

The emitted public API is:

```typescript
export interface Finding {
  readonly pointer: string;
  readonly rule: string;
  readonly detail: string;
}
export type NormalizationResult =
  | { readonly ok: true; readonly json: string }
  | { readonly ok: false; readonly findings: readonly Finding[] };
export class Normalizer {
  constructor();
  normalize(branch: string, jsonText: string): NormalizationResult;
  normalizeBase64Json(branch: string, encoded: string): NormalizationResult;
}
```

Each instance embeds one checked recipe and local validators. The caller cannot
replace its recipe at runtime. Results expose neither partially evaluated values
nor reusable mutable internal state. Successful JSON is compact and has no final
line feed. Non-string JavaScript arguments are outside the typed API, not implicit
coercions. There is no decoded-object input API.

Emit a private, version `0.0.0`, ESM package with explicit exports/types, strict
TypeScript configuration, extension-correct imports and ES2022 output. Retained
recipe, model, bundle and root-schema files use the existing source-accounting
machinery. Generated build inputs enter the report's file map; compiled `dist/`
is a consumer build product. Generation never installs dependencies or executes
the compiler. The native qualification lane uses TypeScript 6.0.3 and records the
actual Node/V8 identity. Browser and unmeasured engine support are not implied.

Lower checked expressions into private typed function bindings. Runtime parsing
of a replacement recipe is unnecessary. Private values use a missing sentinel,
null, boolean, string, bigint, number, arrays and `Map<string, Value>` records.
Branch/member access and emitted bindings must never consult object prototypes.
Keys such as `__proto__`, `constructor`, empty strings and numeric-looking names
remain ordinary data. Use one Unicode-scalar comparator for ordered object
traversal, serialization and target-specific diagnostic ordering; neither default
UTF-16 sorting nor locale-dependent comparison implements the reference order.

## Bound numeric implementation strategy

The first implementation uses native `Number(token)` only after complete JSON
grammar and integer-kind admission. It retains bigint for the existing signed-i64
and unsigned-u64 integer fast paths, with a lexical size check before construction.
An integer token outside those ranges falls through to the existing exact float
admission policy; it does not become an unrestricted bigint or automatically fail.
A bare exact `-0` becomes integer zero, while selected floating zero retains sign.

Require finite conversion. Explicit binary64 inputs may round, underflow to signed
zero and admit subnormals. Unselected floating tokens additionally require equality
between the original and shortest-result decimal keys. Preserve the reference's
zero special case, coefficient normalization and checked i128 exponent arithmetic;
unbounded bigint exponent arithmetic must not broaden that input language.
Signed recipe arithmetic requires both bigint representation and the i64 range.

For floating output, preserve `-0.0` and `0.0`. Extract the shortest coefficient and
decimal exponent from native `Number.toString()`, then use the pinned formatter's
layout: fixed notation for scientific exponents -5 through 15, with `.0` for
integral fixed values; scientific notation elsewhere with an explicit exponent
sign and no unnecessary zeros. Bigint output uses exact integer decimal text.
Emit recipe floating constants from their checked bit patterns, with explicit
DataView byte order, instead of a second generated decimal-literal conversion.

This strategy is supported by focused preparation at ESS source `6c78676`: Node
22.23.1 / V8 12.4.254.21-node.56 matched 13,455 finite bit patterns, 28 midpoint
cases and the existing 35-case numeric reference corpus. The 29 scalar pipelines
also matched ordered Node arithmetic. These experiments are not proof over every
binary64 value or a completed target. Their independent bit/byte cases must become
Rust-owned native qualification tests, including the final format-5 model cases.

The active serde_json 1.0.151 reader has `float_roundtrip` enabled through
jsonschema 0.52.1. Its inactive fallback parser is not a porting target. The
observed serializer uses zmij 1.0.23. Any dependency or engine change must be
qualified against the actual build graph and corpus, rather than inferred from
one direct dependency declaration. No new numeric runtime dependency is selected.

Static checking admits the additional Binary64 equality case only when both
operand kinds are Binary64 in formats 5/6. This does not change general Number
admission or allow Integer/Number/Decimal values to fill Binary64 outputs.
The frozen reference evaluator has a separate runtime limitation: formats 5/6
compare any two floating representations before integer eligibility, including
integral floating tokens admitted by Integer source schemas. TypeScript preserves
that behavior. Mixed integer/floating pairs still require integer eligibility and
refuse; two integer representations retain the signed-64 check. Compiler-owned
metadata remains authoritative for static checking, lowering and output provenance.
This compatibility choice adds no target-only refusal or core evaluator change;
a separately governed core-contract follow-up must assess any semantic tightening.
Missing explicit input paths and unsupported model path shapes remain planning refusals.

## Lexical input and retained bytes

Use two phases. First recognize one complete JSON value and original token spans
without numeric decoding, duplicate collapse or premature semantic depth checks.
Only JSON whitespace separates tokens. Malformed grammar or trailing data anywhere
returns the existing input-syntax finding before branch lookup or located semantic
errors. An iterative scanner must handle syntax beyond the semantic depth limit
without an accidental JavaScript call-stack failure changing error precedence.

Second, decode using the checked branch policies. Outside captured tokens, validate
decoded object names and uniqueness before traversing children in scalar-key order;
visit arrays in index order. Root depth is zero and the existing maximum is 64.
At a capture boundary, validate Unicode and depth through the untouched token in
source-member order, then base64-encode its exact UTF-8 bytes. Preserve internal
whitespace, escape and numeric spelling, duplicate members and quoted delimiters;
exclude surrounding transport whitespace. Missing values stay missing and terminal
null becomes `bnVsbA==`. Do not create missing intermediate objects or lists.

A JavaScript input string can contain literal lone UTF-16 surrogates. Validate both
literal and escaped surrogate pairing at the located decoding phase. Do not first
encode the whole document with a replacement-based UTF-8 encoder. Valid surrogate
pairs become their scalar's UTF-8 bytes; escaped spellings remain literal ASCII
inside captured tokens. Malformed grammar elsewhere still takes precedence.
Captured Unicode/depth errors point to the capture instance, not inside duplicate
members. Unknown branches use empty policies and retain parse-before-dispatch order.

`normalizeBase64Json` checks the standard alphabet, required padding, zero unused
bits and exact re-encoding equality first. It then decodes strict UTF-8, rejecting
overlong, truncated, surrogate and out-of-range sequences without replacement, and
delegates unchanged text to `normalize`. A leading BOM must remain U+FEFF and fail
JSON grammar; a BOM within a JSON string remains content. Empty decoded bytes reach
the syntax error. Preserve the existing input-base64/input-UTF-8 findings and order.

This helper's canonical encoding is stricter than the frozen model Bytes regex:
the schema pattern admits `AB==` and `AAB=`, while the retained helper refuses their
nonzero unused bits. Do not substitute one validation contract for the other.
Composition creates a new document boundary; it neither proves where arbitrary
caller-provided bytes originated nor supplies a transaction across recipes.

## Schema diagnostics and compatibility evidence

Bind TypeScript schema-finding presentation to the existing Go convention: preserve
the complete reference finding multiset, sorted stably by scalar-value instance
pointer. Required-field multiplicity remains observable. Other findings retain
their exact reference order, rule and detail. The validator feasibility pass must
inspect every schema-valued position, including referenced definitions, map keys,
`propertyNames` and combinators, before returning an artifact set.

Qualify numeric enum/const/uniqueness/bound behavior independently of recipe integer
eligibility. No `Number(bigint)` shortcut may lose exact values. Unsupported
refinements refuse at the source-qualified pointer; the required base64 and existing
normalization corpora cannot be dropped to make a narrower implementation pass.

Reuse the v1, v2, numeric, model, base64, raw and adversary fixture constructors,
then the final Binary64 and positional corpora. The raw fixture's seven decoded-
value API cases are inapplicable to this text-only API and must be named separately;
all applicable text/helper cases execute. Add prototype keys, scalar ordering,
literal/escaped surrogates, BOM, strict UTF-8 and helper padding-bit mutations.
Expected large integers remain lossless text or typed observations in the harness,
never ordinary JavaScript-parsed numbers. Compare numeric bits, integer/float
re-entry eligibility, captured bytes and deterministic output independently.

Before shared target/report edits, freeze complete Rust/Go file maps for every
previously integrated recipe format. A normalization-local configuration enum may
add the TypeScript package identity while preserving old Rust/Go serialized arms
and the fieldless structural TypeScript arm. Use the final integrated per-recipe
report versions; adding an execution language alone changes no authored meaning.

## Source-schema profile and numeric refinement binding

This binding supplements `docs/design/typescript-normalization.md`. Its measured
baseline is ESS source `6c78676c35193423fe326b9dde21b8fc21681b8a`, with
jsonschema/jsonschema-value 0.52.1 and num-cmp 0.1.0. This is a **pre-format-5/6
minimum**, not the final TypeScript acceptance profile. The target must refresh
against the final integrated Binary64 and positional-input contracts. In
particular, format-6 tuple schemas require `prefixItems`, `minItems` and
`maxItems`; they cannot remain generation refusals in the accepted target.

The inspected shared native corpus contains 40 v1, 14 v2, 35 numeric, 66 model
including aliases, 2490 base64, 149 raw text/helper and 42 raw-adversary cases:
2836 applicable stored cases. Use the actual shared constructors and every branch,
including base64 `model_keys`. The raw fixture's seven decoded-value entrypoint
cases remain explicitly inapplicable to the text-only API. These counts establish
the reviewed baseline; they do not claim completed TS execution or include the
format-5/6 additions; the frozen positional refresh below records their current inventory.

The current minimum validates boolean/empty schemas, local definition references,
primitive and array-valued `type`, `properties`, `required`, boolean/schema-valued
`additionalProperties`, `propertyNames`, homogeneous `items`, string enums,
inclusive numeric bounds and the exact frozen Bytes pattern. Include `anyOf` for
the published nullable-map/fallback checks and `maxLength` for the raw-capture
refinement check. Every reference sibling remains conjunctive. Only the exact
local reference forms admitted by the checked ESS source plan are supported.

Validate every schema-valued position in the retained selected closure before
returning artifacts, including map keys, referenced definitions and every
combinator alternative. Do not interpret values inside default/examples/enum/const
or provenance as schemas. Preserve complete source/model/root pins and report
accounting; imported annotations do not confer compiler-owned model authority.

Draft202012 validation keeps formats disabled, as in `bundle.rs::compile` and
`normalize/source.rs::validate_model`. Format, contentEncoding and
contentMediaType remain annotations; defaults do not execute. The v1 format branch
must accept `"not an email"`. Preserve known source/model metadata and ordinary
annotations without inventing validation behavior.

The frozen Bytes pattern is:

```text
^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$
```

Implement its full ASCII alphabet/group/padding shape. Native JS `$` semantics
must not admit a trailing line terminator. `AB==` and `AAB=` satisfy this pattern;
the retained-document helper separately rejects their nonzero unused bits. Count
string length in Unicode scalars. Capture occurs before schema validation, so a
length constraint on a captured value measures its encoded base64 string.

### Exact numeric semantics

Schema `number` admits represented bigint integers and finite Number values.
Schema `integer` admits bigint and finite Number values with no fractional part,
including -0.0 and 1e20. This does not authorize recipe integer operations:
those still require signed-i64 integer representation. Preserve the distinction
from the final compiler-owned Binary64 type contract.

For numeric bounds, const and enum, bigint operands compare exactly. Number
operands compare by finite IEEE numerical value, with +0 equal to -0. Mixed
bigint/Number comparison must compare the integer with the exact represented
binary value. Never round bigint to Number or compare it with the floating
value's shortest printed decimal. Preserve schema-literal numeric kind when
embedding canonical source schemas.

One direct implementation decodes Number bits using DataView. A normal magnitude
is `(2^52 + fraction) * 2^(exponentBits - 1075)`; a subnormal is
`fraction * 2^-1074`. Apply sign and compare integer numerators by shifting the
appropriate side. Intermediate width is bounded by finite binary64's exponent
range. Reuse this comparison for inclusive/exclusive bounds and numeric
const/enum. Preserve negative-zero transport separately from schema equality.

Generic schema equality is recursive: strings and booleans compare exactly,
null equals null, arrays compare ordered elements, and records compare decoded
key sets and associated values independent of member order. No scalar coercion
is allowed. Numeric const/enum must use this schema equality, not recipe operand
eligibility or strict host-language type equality.

Independent source-6c CLI probes confirmed that minimum integer
`9007199254740993` rejects `9007199254740992.0`, maximum floating
`9007199254740992.0` rejects integer `9007199254740993`, and numeric const/enum
preserve the same distinction. Enum `[0,9007199254740993]` accepts -0.0. The
validator's ordinary non-arbitrary-precision path uses NumCmp; its conditional
arbitrary-precision implementation is not the measured source CLI contract.

Numeric/deep const/enum, exclusive bounds and combinators require qualification
whenever the final selected corpus uses them. A profile that only implements
string enums must explicitly refuse other enum members. The pre-5/6 minimum
does not itself require numeric const, oneOf or allOf; this is not permission to
omit a construct subsequently required by the integrated contracts.

### Findings and multiplicity

Preserve the complete reference schema-finding multiset and sort it stably by
Unicode-scalar instance pointer, following Go's presentation convention. Retain
duplicate identical findings. Other finding order remains exactly reference
ordered. Every runtime schema finding has rule `schema_validation` and detail
`value does not satisfy the selected stage contract`.

Prefix the escaped instance pointer with
`/branches/<escaped branch>/<stage>/input` or `/output`. Escape `~` as `~0` and
`/` as `~1`; never add schema-reference or keyword segments to instance pointers.

- Each missing required name produces one finding at its containing object.
  A present null satisfies presence and may separately fail its field schema.
- AdditionalProperties false produces one containing-object finding for all
  unexpected names. Schema-valued additionalProperties reports each invalid
  value at its member pointer.
- PropertyNames validates each decoded key as a string. Each underlying failure
  reports at the containing object, not a synthetic key pointer.
- Type-specific refinements skip other kinds. A wrong object type does not
  produce missing-field errors. An enum/const failure and a sibling type failure
  remain separate findings.
- AnyOf emits one current-pointer finding if no alternative validates. OneOf
  emits one if zero or multiple alternatives validate. Their child diagnostics
  stay nested upstream and are not exposed in ESS's flat finding list.
- AllOf retains the combined child finding list, including repeated pointers.
  A type array emits one type failure, not one per option.

The format-6 refresh must qualify tuple item locations and min/max-item
multiplicity against the integrated reference and native positional corpus.
Do not infer a source positional decoder from JSON Schema length keywords.

### Explicit unsupported refinements

Preflight-refuse `uniqueItems:true` in every selected schema until its dependency
semantics are corrected and qualified. False imposes no constraint. Report at
the original source-qualified `uniqueItems` pointer and emit no partial artifact
set. None of the inspected shared baseline schemas requires true. Reassess that
fact at the final source refresh; do not drop a newly required case to retain
this refusal.

This refusal tracks a separately evidenced pre-existing jsonschema-value 0.52.1
inconsistency: semantic uniqueness rejects equal +0/-0 through array length 15,
but raw floating-bit hashing can admit them from length 16. TypeScript must
neither silently emulate the defect nor claim a mathematically corrected
validator is exact parity with that old behavior.

Other refinements may refuse at their qualified schema pointer until required
and individually qualified: multipleOf, non-frozen patterns, unsupported
reference forms, conditional/dependency/unevaluated keywords and unrecognized
keywords. Retain existing model-invariant and structural-source refusals.
MultipleOf is not plain Number remainder: the pinned implementation combines
exact integer modulo, shortest-decimal i128 arithmetic and a fraction fallback.
No constraint may disappear because the target cannot represent it.

These are generation-time capability boundaries, not runtime data coercions.
Format-6 `prefixItems`, `minItems` and `maxItems` are expressly required additions
to this measured minimum, not final-profile refusals. No new runtime dependency
is selected by this binding.

## Frozen positional implementation refresh

This source/design refresh is pinned to positional commit `eb2e5d60e9e803993417df39563bc744dbcd36fc`
and coordinator TypeScript binding `81f1551f1373bc66e9b4dabf59d6557ec4350a43`.
It defines the implementation dependency; it does not claim TypeScript execution.
A positional adversary correction or final integration delta requires refresh
before TypeScript implementation begins.

## Actual type and checked-arity boundary

`normalize/check.rs:20` now has `Kind::Tuple(Vec<Type>, bool)` and
`Kind::SchemaUnion(Vec<Type>)`, separate from expression `Kind::Union`.
`from_node` at line 465 admits a tuple representation only for recipe 6. Exactness
requires a nonempty prefix, `Shape::Never` tail, and minimum/maximum equal to the
prefix length. A homogeneous array is still an array. `assignable` at line 1118
does not flatten tuples into homogeneous collections and rejects a nonexact source
tuple. Compiler-owned Binary64 input values under a tuple still refuse with
`model_binary64_path`; no numeric tuple selector was added to NumberPath.

`schema_union` at line 84 retains distinct source alternatives containing tuples,
even when they are structurally equal. `has_tuple` sees nested object, array and
union positions. `member` at 1062, `array_item` at 1096 and `without_null` preserve
that provenance; fallback does not erase it. Consequently duplicate source
`anyOf` tuples, tuples inside alternative objects, and tuples reached through
alternative arrays do not become one proved positional source. An expression
`choose` over the same independently exact tuple remains admissible. The public
regression at `tests/normalization_positional.rs:202` covers both sides.

TypeScript consumes the sealed Plan rather than reproducing this type admission.
Its schema validator still validates every retained source alternative; target
preflight must not reinterpret a union as permission for positional decoding.

`normalize.rs:125` stores the private `BTreeMap<String, u64>` named
`position_arities`. `check::stage` at 415 gathers it; `Context::position` at 633
records the operand's exact positive arity at the full escaped static expression
pointer, after proving the index fits. Nested item scopes share this map. It is
not serialized into `source.recipe.json` (the format-6 regression asserts this at
`tests/normalization_positional.rs:524`).

The TypeScript emitter can access this parent-private field from its new child
module. Emit a deterministic private arity table in `src/bindings.ts`; use the
same static branch/stage/requirement/expression pointers during evaluation.
Runtime collection indices do not enter arity keys. `eval.rs:123` and generated
Go `go_expression.go.txt:111` evaluate the operand once, propagate missing before
checking metadata, and otherwise require present positive arity, array kind,
exact length and `index < arity`. Preserve the exact `position_value` finding at
the expression pointer, including for an overlong array where the selected index
exists. Null slots remain null for a pure Position read. Source null-to-empty
conversion belongs only to the explicit input policy.

## Closed policy, parser and retained-token behavior

`recipe.rs:21` exposes FORMAT_V6; `PositionalInput` at 29 is a closed DTO. Its
required fields are path, kind, length, missing, null, short, extra and
null_element. Every policy enum has the one bound variant. The private path
deserializer at 526 accepts only exact field/items objects; the map reader at
546 rejects duplicate branch names. An empty path selects the document root.
Present null or malformed declaration data is a recipe syntax failure, not an
absent declaration or a default. `position` uses a checked u64 index; there is
no authored arity field and no replacement recipe parser in the TS runtime.

`check::input_positions` at 295 checks positive length, duplicate declarations,
earlier positional overlaps, raw-capture overlap, numeric overlap, then terminal
schema feasibility. `positional_path` at 371 requires one exact tuple of the
declared arity with string-shaped positions; optional ancestors may be traversed
without creating them, while a nullable terminal tuple does not prove this
profile. Preserve these Plan refusals unchanged and lower only checked path/length
bindings into the new TypeScript target.

`input.rs:13` parses the complete grammar first. `decode` at 34 checks capture
boundaries, then original-root depth, then a selected positional boundary before
ordinary value conversion. Overlap checks make capture/positional conflicts
unreachable from a checked public Plan. Ordinary objects retain decoded-key
uniqueness validation and scalar-sorted child traversal; arrays retain index order.
`positional` at 147 accepts only array or null, converts consumed null slots to
empty strings, checks consumed depth before kind/Unicode, pads short arrays and
does not recursively interpret a rejected consumed object or number. Missing
members and null ancestors are preserved; zero padding is synthetic and is not
another input traversal.

Every excess token is walked by `retained::validate` at 50 with the detail
`discarded positional token contains invalid Unicode`, original depth and the
discarded element pointer. This is source-order Unicode/depth validation without
number conversion or duplicate collapse. Huge exponents and duplicate keys in a
valid discarded token are therefore admissible. A later grammar failure anywhere
still wins before preparation; otherwise an earlier consumed kind error wins over
a later discarded Unicode/depth failure. New TypeScript `ts_input.ts.txt` and
`ts_retained.ts.txt` must share the original spans and this validator parameter,
including the distinct capture error detail. Strict base64/UTF-8/BOM behavior
remains the existing retained-document boundary.

`normalize.rs:259`, `execute.rs:6` and generated runtime entrypoints apply input
policies once before first-stage validation. Position remains usable later in the
stage pipeline without another input decode. The Rust decoded-value APIs refuse
capture provenance before positional provenance; the TS text-only API has no
decoded-value entrypoint to add.

## Format families, source accounting and immutable older output

The frozen implementation now admits recipe 6 in envelope, model-root, raw-input,
Binary64 construction/equality, generated Rust-runtime and CLI gates. Position and
positional declarations remain recipe-6-only, even for an empty declaration map
in older formats. `execute.rs:28` enables the existing format-wide floating equality branch for
formats 5 and 6, including the Integer-expression runtime limitation stated above.
`target.rs:163` actually selects report 1 for recipes 1/2, report 2 for recipe 3,
and report 3 for recipes 4/5/6. Keep this selection; a third target adds no common
report version.

Report configuration is still a private field (`target.rs:23`); `finish` remains
`pub(super)`. Existing configuration construction is in `target::rust` and
`go_target::generate`. The selected normalization-local Rust/Go/Typescript package
enum remains an additive public-API implementation: expose only
`normalize::Plan::typescript(&self, package: &str) -> Result<Realization, Refused>`.
Preserve old serialized Rust/Go arms and report field order. Keep the structural
configuration enum unchanged. The fixed schema profile and every new deterministic
TS runtime input enter the existing file map; unsupported profile refusals precede
Realization/report creation. No profile selector is added.

Actual Rust template routing (`target.rs:198`): format 6 uses current
rust_runtime/recipe/eval/execute/input/retained sources; format 5 uses `legacy_v5`
runtime/recipe/eval/execute and `legacy_v4_v5` input/retained; format 4 retains
`legacy_v4` runtime/recipe, `legacy_v1_v4` eval/execute and `legacy_v4_v5`
input/retained; older recipes retain their existing legacy families. Actual Go
routing (`go_target.rs:362`) similarly adds `legacy_v5/go_runtime.go.txt`,
`legacy_v4_v5/go_input.go.txt`, `legacy_v4_v5/go_retained.go.txt` and
`legacy_v1_v5/go_expression.go.txt` while format 6 uses current files. TypeScript
must not edit any of these current or legacy template bytes.

The existing `tests/normalization_legacy_bytes.rs:28` now covers formats 1–5
(ten Rust/Go maps), but replaces generator_version with a placeholder at line 64.
That guard remains untouched. Before shared TypeScript edits, the new scoped
`normalization_typescript_legacy_bytes.rs` and its new JSON fixture must freeze
all twelve complete format-1–6 Rust/Go maps at one actual fixed generator version,
same package `normalization_adapter` and Go module
`example.invalid/normalization-adapter`. Include all generated paths and exact
report bytes without normalization. Format 6 can use `normalization_positional::plan()`;
the three-stage mixed source/model plan is an additional accounting/control case,
not a substitute for a missing format family. No snapshot generation was run in
this refresh; the twelve-map pre-edit capture is still required.

## Concrete corpus and profile delta

Retain the earlier 2,836 applicable fixture executions, plus all 70 Binary64
vectors under format 5. Format 6 adds the 99-vector positional constructor,
of which six decoded-value API cases are explicitly inapplicable to TS: four
positional-provenance refusals, the record capture-provenance refusal and the
plain decoded-value control (`normalization_positional.rs` fixture at 324).
The TS text/helper count is therefore 93, as the existing Go harness selects at
`tests/normalization_positional_targets.rs:153`. Do not convert those six cases
to text and pretend their original API assertions executed. The earlier seven raw
decoded-value cases remain separately inapplicable.

Execute all 70 Binary64 vectors a second time with the recipe format promoted to
6, using the same checked model roots and independent expected bits/kinds. This
is a distinct format-family execution, not 70 newly invented inputs. Add all
three `mixed_plan`/`mixed_cases` vectors at fixture lines 98/112:

| External input | Required cross-stage result |
|---|---|
| `{"operands":["a",null,1e999],"raw":{"duplicate":0,"duplicate":1e999},"number":-0}` | true: positional consumed-null/discard handling, raw duplicate capture, signed numeric input and three modeled Binary64 stages coexist |
| `{"operands":null,"number":0.10000000000000001}` | false: positional zero array and explicitly rounded numeric input coexist |
| `{"operands":[]}` | true: zero padding and the explicit Binary64 -0.0 default remain executable |

This accounts for **3,091 planned fixture executions**: 2,836 historical applicable
cases + 70 format-5 Binary64 + 93 positional text/helper + 3 mixed + 70 inherited
format-6 Binary64 + 19 literal positional adversary text vectors. It excludes separate generation/admission/defensive tests and
the new TS-specific engine/numeric/Unicode/prototype controls. These are source
and prior-handoff inventory counts, not a TS test run in this task. Re-enumerate
the constructors at final dispatch, especially if adversary corrections add cases.

The actual positional schemas require prefixItems, items:false, minItems,
maxItems, nullable primitive/array/object types, local roots and references,
properties/required/additionalProperties, and homogeneous arrays containing
tuples. `tests/normalization_positional.rs:398` additionally requires minLength
after padding. Tuple failure pointers are instance indices; min/max violations
are located at the array and remain conjunctive with item/tail failures. Keep
the already proposed full closed profile and separately qualify the exact
diagnostic multiset against the pinned validator; this refresh does not assert a
new measured diagnostic multiplicity. No inspected new positional or mixed
fixture requires uniqueItems:true, so its explicit source-qualified preflight
refusal remains applicable. The fixed schema-profile file must include the tuple
and minLength capability rather than describing the earlier pre-5/6 minimum as
the final profile.

## Next dispatch boundary

Apply the proposed design update only after positional integration; refresh any
changed source hashes/lines from an adversary correction first. Freeze the twelve
old target maps before modifying the four shared TypeScript seams. Requalify the
actual TypeScript 6.0.3 and Node/V8 environment when implementation runs. No engine,
numeric, native or schema diagnostic qualification was rerun here. The same 31
write paths, coordinator-owned document split, fixed package arm, report families,
source pins and no-new-dependency decision remain intact.


## Final integration dispatch delta

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
