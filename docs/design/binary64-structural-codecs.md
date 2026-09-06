# Standalone structural Binary64 codecs

Binding draft for a bounded follow-up to [finite Binary64 model fields](model-binary64.md).
Implementation and qualification are pending. This document defines the selected
contract; it claims no passing native tests, released codec or whole-system support.

The public structural source baseline is
[`6c78676c35193423fe326b9dde21b8fc21681b8a`][baseline]. It establishes the existing
emission and decoding seams cited below. It does not contain the subsequent
Binary64 compiler authority. Before implementation, pin the integrated Binary64
commit and refresh the actual sealed model inventory, Plan admission/refusal,
report accounting and old-output baselines. Do not substitute moving line numbers
or infer primitive identity from the broad schema projection.

## Bound scope and authority

Standalone Rust and Go model data libraries must support compiler-owned Binary64
through scalar roots, nominal wrappers, closed record fields, presence/nullable
wrappers, lists, existing supported exact tuples, references, unions and typed
map values. This includes named maps, newtypes over maps and nested combinations.
Every accepted path must deliver its original JSON numeric token to the scalar
decoder. A declaration that compiles while losing that token is not support.

The integrated `ModelTypes::binary64_locations` inventory, consumed by
`Plan::from_model`, is the intended authority. Refresh the final names and location
contract at the integrated base. Exact marked-leaf membership selects the native
scalar representation; transitive reachability selects affected container codecs.
Traversal must terminate on recursive references without overlooking an affected
path. A marked location must resolve to a numeric wire node or refuse explicitly.

Imported `type:number`, numeric bounds, format hints, names and untrusted annotations
do not mint Binary64 identity. Imported general numbers retain their existing
`serde_json::Number`/`EssNumber` mapping and obligations. Integer and Decimal stay
distinct. Binary64 map keys retain the compiler's prohibition, including direct
typed model construction; this work adds no spelling or equality policy for keys.

A Rust mixed record whose additional-property values reach Binary64 may refuse at
that additional-property location. Pure typed maps must work and cannot be swept
into this exception. Whole-system synthesis and conformance keep their existing
refusals; normalization, transport generation, arithmetic and decoder compatibility
outside this concrete source-token profile are not extended.

## Checked native API

Rust exposes a private finite value with this API:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EssBinary64(f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EssBinary64Error { NonFinite }

impl EssBinary64 {
    pub fn new(value: f64) -> Result<Self, EssBinary64Error>;
    pub fn get(self) -> f64;
}
// EssBinary64 implements serde::Serialize and serde::Deserialize.
// EssBinary64Error implements Display and std::error::Error.
```

Go exposes:

```go
type EssBinary64 struct { value float64 }
func NewEssBinary64(value float64) (EssBinary64, error)
func (v EssBinary64) Float64() float64
func (v EssBinary64) MarshalJSON() ([]byte, error)
func (v *EssBinary64) UnmarshalJSON(data []byte) error
```

Constructors reject NaN and either infinity, otherwise preserving the input bits.
Backing fields remain private; there is no unchecked mutable float access, implicit
cast, arithmetic, hashing or total-order promise. Native numeric equality makes +0
and -0 equal. Their sign remains observable through the read accessor and JSON.
Existing nominal wrappers may contain this checked value as their public member.

Go's zero value is +0, not an authored default. Missing required fields still
refuse. A missing optional field stays absent; a nullable enclosing type can admit
null without constructing a Binary64. The scalar never treats null as zero.
Reserve new helper/API names only in output that actually needs those helpers.

## Numeric source and serialization contract

Admit one actual JSON number token after complete JSON grammar validation. Round
that original token once to the nearest finite binary64, ties to even. Preserve
negative zero from `-0` and `-0.0`, admit subnormals and signed underflow-to-zero,
and refuse overflow-to-infinity. No quoted-number coercion is introduced.

Null, booleans, strings, arrays and objects refuse at the scalar. In particular,
objects containing serde_json private Number or RawValue marker keys must not
masquerade as numeric source tokens. NaN, Inf/Infinity, leading plus signs,
hexadecimal floats, comments and malformed numeric spellings fail JSON grammar.

Serialization checks finiteness defensively and never substitutes null. Emit a
finite round-tripping JSON number, preserving negative zero as `-0.0`. Integral
floating values retain a decimal point or exponent marker so later normalization
does not reinterpret their lexical form as an integer-operation operand. Original
numeric spelling and cross-language byte-identical float formatting are not promised.

Go uses the existing `essIsNumber` grammar/kind gate before
`strconv.ParseFloat(token, 64)`, rejecting conversion errors and nonfinite results.
The marshaller uses shortest round-tripping formatting, handles negative zero
explicitly and appends `.0` when no decimal/exponent marker is present. Rust parses
the validated original token with `str::parse::<f64>` and checks finiteness. Its
finite `serialize_f64` path must satisfy the signed-zero/integral-marker corpus;
if the pinned serializer does not, implement that token requirement explicitly.

Decode into temporary state and assign only after success. Preserve the existing
Go whole-record atomic assignment. Do not promise common Rust/Go error wording,
error ordering, full schema validation or parity with a permissive native decoder.

## Complete Rust token path

The baseline [Rust emitter][rust] maps generic numbers to `serde_json::Number` and
tries union branches via `Value` plus `from_value`. Under its pinned serde_json
1.0.151 arbitrary-precision parser, lexical `-0` can become integer zero before
such a scalar is called. `Number::deserialize` also recognizes a private map form.
Neither is an acceptable source-token boundary for the new primitive.

The required affected path is:

1. The scalar deserializes `Box<serde_json::value::RawValue>`. RawValue has checked
   the complete JSON value; after whitespace trimming, require a minus/digit
   token kind, parse the original numeric text and check finiteness. Do not route
   it through Number or Value.
2. Each union transitively reaching Binary64 captures its original RawValue and
   retries existing alternatives, in their existing order, with
   `serde_json::from_str::<Alternative>(raw.get())`. This includes a branch that
   reaches Binary64 only through a record, reference, list or map.
3. Direct record fields, ordinary sequence elements and reference/newtype wrappers
   continue direct deserialization. Existing [EssPresence and ess_required][rust-support]
   forward to `T::deserialize`; Serde's nullable Option forwards `visit_some` too.
   These wrappers do not require a new Value intermediate.
4. Every fieldless typed-extra object reaching Binary64 retains its public
   `ess_extra: BTreeMap<String, T>` layout and existing flatten serialization, but
   implements Deserialize by directly deserializing `BTreeMap<String, T>` and
   wrapping it. Map values must receive the original deserializer, including
   nested maps, lists, references and unions. Do not derive flatten Deserialize
   for this affected layout.
5. A mixed declared-field/additional-property record whose extra values reach
   Binary64 refuses before successful output unless a complete raw-preserving
   object visitor is separately implemented and qualified. Known fields containing
   Binary64 are not themselves a reason to refuse a mixed record whose extras
   cannot reach Binary64: those fields deserialize directly.

The direct map step is mandatory. [Model Map and newtype lowering][model-types]
produces an object with typed `additionalProperties`, even for a named map with
no explicit struct fields. The existing Rust record emitter stores that shape in
flattened `ess_extra`. Serde flatten buffers extras as Content; adding RawValue
only at the scalar therefore does not solve this source path.

Retain union first-structurally-decodable selection and its independent membership
obligations. Another explicitly permitted string/object/general-number variant
may win in declared order; strict Binary64 admission does not erase that variant.
Retain existing map-key validation obligations, duplicate-key behavior and
serialization collision checks. Do not broaden this work into a generic open-object
decoder or claim a complete native object-decoding policy.

Conditionally add serde_json's `raw_value` feature to the pinned generated manifest
only for a Binary64 selection. Keep serde 1.0.229, serde_json 1.0.151 and
`arbitrary_precision` unless the integrated base explicitly changes their pins.
Other general-number fields in the same library still need their old representation.

The source-token guarantee covers serde_json text/slice/reader input and equivalent
raw-capable JSON deserializers. `serde_json::from_value` can reconstruct raw JSON
from an already parsed Value, but cannot recover a sign or token kind lost before
the call. Document it as value conversion, not source decoding. Non-JSON or
non-raw-aware Deserializer implementations may refuse. Checked programmatic
construction remains available without any claim about original token provenance.

## Go containers and conditional output

The baseline [Go emitter][go] retains `json.RawMessage` through record fields,
additional properties, lists, supported tuples and every union trial.
[EssNullable][go-support] delegates the same source bytes. Map values therefore
need the checked scalar mapping, not a replacement container decoder.

Select the new helper at exact compiler-marked Number leaves. Add conditional
helper text and standard-library imports before declarations; keep the old helper
literal untouched. Separate import declarations are valid only before types and
functions. No new Go dependency or minimum-language-version change is needed.
Rust likewise adds conditional helper text without rewriting unconditional helpers.

## Admission, reporting and exact refusals

Replace the integrated blanket structural Binary64 guard only when every selected
source path is supported or returns a located target refusal. Do not return a
partially successful declaration/manifests/report set. Keep model metadata, source
identity, primitive assignment and normalization's Plan construction unchanged.

| Rule | Location | Exact detail |
| --- | --- | --- |
| `model_binary64_layout` | Compiler-marked primitive pointer | `compiler-owned Binary64 location does not resolve to a numeric wire node` |
| `rust_binary64_open_record` | Mixed record's additionalProperties pointer | `Binary64 additional values on a record with declared fields require raw-token object decoding, which this Rust target does not support` |

Use existing helper-collision rules, extending reservations only for affected
output. Wrapper kind errors say `Binary64 requires a JSON number token`; finite
errors say `Binary64 requires a finite value`. Malformed JSON can retain its native
parser error. Constructor failures use the finite-value error.

Successful Rust/Go reports must distinguish the implemented finite codec from
remaining schema constraints. Discharge the corresponding target finite-codec
obligation while retaining numeric bounds, predicates, union membership, map-key
grammar and storage obligations. Record the Rust source-deserializer limitation.
Do not globally change compiler obligations or imply TypeScript now has this codec.

This unit adds no authored model/recipe semantics and requires no new recipe format.
Use the final integrated structural report contract; a new report number is not
justified solely by supporting a previously refused target. Structural
`ess-types-report` accounting is distinct from positional normalization's separately
versioned executable target report. Neither unit may select the other's format.

## Required independent qualification

Use literal expected IEEE bit patterns or independently specified results, not
expected values calculated by the same parser under test.

| Group | Required cases |
| --- | --- |
| Admission | Actual numeric tokens; wrong JSON kinds and quoted numerics; private Number/RawValue marker objects; malformed grammar and nonfinite spellings. |
| Rounding | `9007199254740993` to `9007199254740992`; `9007199254740995` to `9007199254740996`; negative counterparts and fractional midpoint neighbors. |
| Extremes | Maximum finite `1.7976931348623157e308`; overflow `1.7976931348623159e308`; smallest subnormal `5e-324`; half-subnormal neighbors; `1e-999` and `-1e-999`. |
| Construction/serialization | Finite input bits retained; NaN/Inf rejected; Go zero value; signed-zero equality separate from sign tests; exact bit round-trip, `-0.0` spelling and integral floating markers. |
| Presence | Missing required versus absent optional; explicit null refused at scalar; nullable container null remains a separate admitted value. |
| Complete paths | `-0` and overflow through scalar/newtype, fields, lists, supported tuples, references, nested unions, named maps, map newtypes, maps of lists/unions/maps and optional map values. |
| Boundaries | Mixed-open-record refusal location/no partial output; compiler map-key refusal; independently permitted union variants; documented prior-Value loss and raw-capable source entrypoints. |
| Selection/publication | All selected model types succeed within profile; excluded Binary64 roots do not activate helpers; imported numbers remain unchanged; every unsupported selected path refuses before publication. |

Before changing generation, freeze complete no-Binary64 output maps from the final
integrated base at fixed roots, target configuration and generator identity. Compare
all emitted bytes and paths, including declarations, manifests and reports. Do not
strip whitespace, omit helper files, normalize source or forge generator versions.
An actual release's truthful version fields and dependent file digests are separately
accounted changes. Conditional reservations must also preserve old admitted names.
Retain positional and older normalization output bytes by leaving their emitters,
templates and common package-validation functions unchanged.

Native corpus execution and CLI publication checks are acceptance work for the
implementation. This draft records no executed gate, generated fixture or wave run.

## Concrete scope and positional-unit collision assessment

The companion binding is `docs/design/positional-array-normalization.md`. Its
`fixed_string_array` policy and `position` expression change normalization input,
checker/evaluator and executable target paths. They do not require changing native
structural tuple layout, the shared Shape definition or compiler Binary64 authority.

| Surface | Structural codec scope | Positional scope / scheduling consequence |
| --- | --- | --- |
| `crates/generate/schema-contract/src/realize.rs` | Structural `Plan::rust`, `Plan::go`, finite-codec guard/accounting and a cycle-safe reachability helper. | Positional reads the existing Shape/Node and compiler metadata; it must not modify this file under the bounded scope. Verify that constraint on the final Binary64 base. |
| `realize/rust.rs` | `emit`; `Emitter::{ty,record,union,account}`; conditional manifest/helper handling. | Positional uses `normalize/target.rs` and `normalize/rust_runtime.rs.txt`; these are different files. Preserve `rust::package_name`, which normalization reads. |
| `realize/go.rs` | `emit`; marked scalar selection in `Emitter::ty`; conditional helper/import/name handling and accounting. | Positional uses `normalize/go_target.rs`, `go_input.go.txt`, `go_expression.go.txt` and `go_runtime.go.txt`; different files. Preserve `go::{package_name,module_name}`, which normalization reads. |
| New `realize/rust_binary64.rs.txt`, `realize/go_binary64.go.txt` | Conditional finite-value codecs. | Positional's input/retained lexical helpers and frozen normalization templates are disjoint. Do not extract a new common decoder as incidental cleanup. |
| `realize/normalize.rs` and `realize/normalize/{recipe,check,input,eval,target,go_target,retained}.rs` | Read-only dependency boundary; no structural-codec edits. | Positional owns its recipe format, tuple type checking, text/value entrypoints, policy/evaluation, emission and any qualified lexical-tail reuse. |
| Structural tests and fixtures | `tests/{rust_realization,go_realization}.rs`, new finite wire fixtures, CLI `tests/model_types.rs`. | Positional uses normalization-specific fixtures, legacy output maps and CLI `tests/normalization.rs`; disjoint bounded test files. Shared test-support changes require re-scoping. |
| Design documents | This design, `docs/design/model-binary64.md`'s standalone target boundary and the structural realization design. | Positional owns its design and source-pinned-normalization design. Cross-links can be integrated by one owner. |
| `website/docs/reference/formats.md`, `CHANGELOG.md` | Structural capability/compatibility publication. | Both full scopes include these files. Their final edit ranges are not proved disjoint; assign one integration owner or sequence edits. |
| Planning store and release metadata | Coordinator-owned. | Shared mutation surfaces must remain serial; this draft does not authorize competing journal/release edits. |

All relative `realize/` and `tests/` paths above are within
`crates/generate/schema-contract/` (`realize/` is under `src/`); CLI tests are under
`crates/edge/ess-cli/`. New helper/fixture names are proposed paths. Existing
production symbols are grounded in the public baseline, while compiler inventory
and blanket-guard names must be refreshed after Binary64 integration.

There is no demonstrated intrinsic implementation dependency between the two
follow-ups after the common Binary64 base. Their bounded production edits can be
disjoint by **file**, which is stronger than an assertion about nearby line ranges.
However, the current full scopes overlap publication files, and the final common
base has not yet been inspected for this scheduling decision. Do not declare an
already-disjoint wave. Either sequence the full units, or first assign shared
documentation/release/planning edits to one coordinator, refresh exact file/symbol
scopes on the integrated base, and verify neither implementation needs to widen
them. Any shared Shape, compiler metadata, package validator, common fixture or
report-contract change invalidates that narrowed independence and requires
coordinated sequencing. Integration and its required gates remain serial.

[baseline]: https://github.com/beyond10x/ess/commit/6c78676c35193423fe326b9dde21b8fc21681b8a
[rust]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/rust.rs
[go]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/go.rs
[rust-support]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/rust_support.rs.txt
[go-support]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/go_support.go.txt
[model-types]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/ess-gen/src/types.rs
