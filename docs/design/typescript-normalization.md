# Standalone TypeScript normalization

This design extends the existing source-pinned normalization contract to a native
TypeScript execution target. The reference remains the checked recipe and retained
source schemas described in `source-pinned-data-normalization.md`; structural type
aliases do not implement execution or validation.

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

## Concrete library and CLI contract

Add `normalize::Plan::typescript(package: &str) -> Result<Realization, Refused>`.
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

Modeled Binary64 remains a checked type distinction from general Number even when
both use a private JavaScript number representation. Consume compiler-owned
metadata and preserve it in lowering. Format-5 equality accepts two typed
Binary64 operands and uses finite IEEE equality, including +0 equal to -0.
Runtime `typeof number` is insufficient authority. Integer/general Number/Decimal
values do not silently fill Binary64 outputs. Missing explicit input paths and
unsupported model path shapes remain planning refusals.

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
pending format-5/6 additions.

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
