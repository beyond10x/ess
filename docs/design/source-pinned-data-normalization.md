# Source-Pinned Data Normalization

## Boundary

A permitted nominal conversion is not an executable data transformation. A stored
record, a decoded DTO and a normalized runtime record can have different fields and
meaning. They must remain separate typed roots; do not attach stored settings to a
similarly named normalized field or infer behavior from JSON Schema annotations.

Use an explicit, offline normalization recipe over replay-checked schema bundles.
The author supplies an external dispatch key; the recipe selects exactly one declared
branch. Each branch is a nonempty ordered sequence of stages. A stage names its input
and output by canonical bundle digest and admitted root identity. Adjacent stage
identities must agree. Each stage validates its input, evaluates explicit requirements
and a pure expression, then validates its output. No stage mutates its caller's input,
and failure returns no successful intermediate result.

## Concrete Operations

The bounded expression vocabulary is derived from actual adapter operations:

- Read a declared object field path from the current stage input or current collection item.
- Construct a record, list, null, boolean, string or signed integer literal.
- Substitute an explicit fallback for absence and, only when declared, null.
- Choose a value using ordered conditions; only the selected branch evaluates.
- Add or multiply signed 64-bit integers with an explicit checked or wrapping policy.
- Map a collection with a lexical item binding.
- Count distinct string keys of collection items satisfying an explicit condition.

Conditions are presence, boolean value, scalar equality, signed-integer ordering and explicit
all/any/not composition. Missing is distinct from JSON null. An absent field copied
into a record remains absent; an absent root/list element is refused. Reading an
undeclared field, an unbound item or traversing a nonobject is refused during checking.
Collection item scope does not leak across expressions. No arbitrary code strings,
generic behavior property bag, network lookup or implicit source-language parsing.

Equality accepts null, boolean, string and signed-integer operands, including optional
or union scalars. Different scalar kinds are unequal; either missing operand makes
the condition false, including when both are missing. Number operands follow the
exact signed-integer representation policy below. Object, array, unrestricted JSON
and general-number comparisons refuse during checking. Structural JSON equality is
not delegated to target libraries, whose numeric equality policies can differ.

`starts_with` compares two present non-null strings by exact case-sensitive UTF-8
prefix. It neither normalizes text nor trims it, folds case or interprets a URI. An
empty prefix matches every string. Source adapters use prefix selection to choose
different target representations; equality against a finite sample is not equivalent.

Signed integer operations accept exact integral JSON tokens representable by `i64`;
decimal/exponent spellings are not silently coerced. Overflow behavior is a required
operation property: reject or two's-complement wrapping. This is a concrete decoder
policy, not a new interpretation of the schema `integer` keyword. A future exact
unbounded arithmetic mode requires its own semantics and target support.

## Checking And Execution

Reuse the existing shared structural plan to resolve field types and local references.
Check every dispatch branch, not just the one exercised by a fixture. Check expression
types, required versus missing fields, collection item scopes and output structure.
Schema refinements remain runtime validation at stage boundaries; a structural type
check must not claim it proves bounds, patterns, exclusive unions or formats. Existing
bundle format-annotation policy remains unchanged. Unsupported structural expressions
refuse with recipe pointers and source-root identities, never a partial checked plan.

The initial library implementation establishes typed recipe identity and reference
execution. The story remains active until Go, Rust and TypeScript realization consumes
this same sealed checked plan, retains source provenance and passes equivalent positive
and negative behavior fixtures. Library checks or a Rust evaluator alone do not satisfy
the three-target acceptance, nor prove parity with every source adapter.

## Persistence

The new authored behavior envelope is `ess-normalization/1`. It is separate from schema
bundles because operations, stage order, overflow and external dispatch are not schema
annotations. Roots are pinned by full canonical bundle digest, which includes original
source bytes and qualification. Strict parsing refuses unknown fields and versions,
duplicate dispatch keys and duplicate constructed record keys; no last-key-wins behavior.
Reload always rechecks the supplied bundles and every branch; there is no trusted
serialized checked flag. No `ess-ir/2`, lifecycle entity, service or format change to
existing schema/model artifacts is introduced.

## Command Boundary

`ess generate schema normalize-check --recipe FILE --bundle FILE...` checks every
branch and prints the canonical authored recipe, or writes it with `--out FILE`.
`normalize-run` takes the same recipe/bundles plus an explicit `--branch NAME` and
`--input FILE`, and emits only the resulting JSON value. No branch is inferred from
data. Both commands reload and replay-check every supplied bundle. Output cannot
replace any recipe, bundle or instance input; output containment uses the existing
CLI preflight. A failed check or run writes no output value.

The text execution edge parses raw JSON values before numeric conversion, rejects
duplicate object keys and requires exact mathematical equality between every source
numeric token and its reserialized representation. It compares normalized decimal
significands/exponents, not floating-point equality; integers within signed range
retain integer representation, including `-0`. Fractional/exponent tokens remain
nonintegral representations for signed-integer operations even when their value is
mathematically integral. Numbers that underflow, overflow or lose decimal precision
refuse before execution. This adds no arbitrary-precision feature to existing bundle
serialization. The value-based library API cannot recover bytes lost by a caller's
decoder; consumers of raw JSON use `Plan::run_json` for the checked text boundary.

The command output is data, not a conformance report or a three-language adapter.
Go/Rust/TypeScript generation remains required by the story's full acceptance.

## Standalone Rust Realization

The Rust target emits an offline Cargo library from the sealed checked plan. It
retains the canonical recipe, referenced qualified bundles and independently rooted
schemas. A new `ess-normalization-target/1` report records target/package identity,
generator version, canonical recipe digest, root identities and schema locations,
plus the digests of all generated files other than the report itself. A data-types
report is not repurposed to claim executable normalization.

Extract the existing recipe types, diagnostics and pure execution modules into shared
source modules and emit those exact bytes into the target. Do not maintain a second
Rust evaluator or load an unchecked caller-provided recipe at runtime. The generated
`Normalizer` owns compiled validators for every pinned stage root, using the same
offline JSON Schema validator version, draft and format-annotation policy as ESS.
Its public operation consumes raw JSON through the checked text decoder and returns
only a fully validated result. Callers can then deserialize into separately generated
data types. Source-specific types are not renamed into normalized types.

The generated manifest pins Serde, serde_json and jsonschema, with no ESS/AEP runtime
dependency and no network-enabled schema retrieval. The text decoder's numeric
admission must remain unchanged if Cargo feature unification also enables
serde_json arbitrary_precision in the consumer. Normalizer construction compiles the
embedded schemas; repeated runs reuse them without mutable caller state.

Compile and run the actual generated crate against the same positive and negative
fixtures as the reference executor, including every operation, runtime schema failure,
integer boundaries, missing/null distinctions, dispatch, lexical item scope and lazy
selection. Exercise the consumer with and without arbitrary_precision. This proves
the Rust target only; Go, TypeScript and complete source-driven adoption remain open.

## Verification

Use generic fixtures for external dispatch, preinitialization/default ordering,
absence/null/zero distinctions, unit scaling and overflow, branch precedence,
mapping and distinct-category counts. Refuse wrong digests, unselected roots,
stage-identity mismatch, unknown paths, invalid expression types, unsupported
operations and invalid intermediate/final values. Preserve caller data on success
and failure. Verify deterministic recipe bytes and replay; test every target against
the reference behavior rather than merely compiling the declarations.

## Ordered Collection and Text Extension

The next authored envelope is `ess-normalization/2`. Readers continue to admit
version 1 with its original operations and numeric policy. Version 1 recipes that
use a version 2 operation refuse at that operation's recipe pointer; relabelling
an extended recipe as version 1 is not a compatibility mechanism. Version 2
preserves all existing operation meanings and adds the following concrete
operations. Canonical version 1 bytes remain unchanged.

- `concat {parts}` evaluates required strings in order and concatenates exact UTF-8
  text. An empty parts list yields the empty string. No escaping, interpolation,
  Unicode normalization or regular-expression interpretation occurs.
- `join {list, separator}` requires a list of present strings and a present string
  separator. It preserves order, duplicate and empty entries. An empty list yields
  the empty string; separators occur only between entries.
- `integer_string {value}` renders an exact signed 64-bit integer as ordinary base-10
  digits, with a leading minus only for negative values. No locale, padding, plus
  sign or floating conversion is used; zero renders as `0`.
- `concat_lists {lists}` concatenates required lists in expression order. It does
  not flatten their elements recursively, sort or deduplicate. An empty expression
  list yields an empty list.
- `item_index` reads the original zero-based index of the current collection item
  as a signed integer. It is bound inside map, distinct-count, select-map and
  find expressions only, and is rebound by nested collections. It never refers to
  a filtered output index or a global counter. An unbound use refuses statically.
- `select_map {list, condition, value}` examines items in source order, evaluates
  the condition with item and original index bound, and evaluates the value only
  for selected items. A selected missing value refuses rather than silently
  shortening the result. Rejected items neither evaluate the value nor change
  later source indices. All value and condition expressions still check upfront.
- `find {list, condition, value, otherwise}` returns the value for the first
  selected item. Later items and otherwise do not execute after a match. If no
  item matches, otherwise executes in the enclosing scope, not in a leaked last
  item scope. Both possible results must be present and structurally compatible
  with the stage output. This realizes ordered declarative equality dispatch
  without embedding source-language callbacks or collapsing duplicate keys.

All branches, nested scopes and operand types must check before the sealed plan
exists. Runtime schema refinements still apply at every stage boundary. These
operations must flow through the shared Rust evaluator and every eventual target;
adding them to the reference engine alone does not finish the normalization story.

These collection operations do not change JSON numeric admission. The following
numeric extension makes decoding an explicit declaration; no source float is
silently coerced into the signed-integer arithmetic above. Runtime
callback fields need an authored declarative representation and consumer mapping;
the existence of `find` alone does not establish that mapping.

## Declared Binary64 Conversion

Version 2 is still unreleased. Its concrete numeric extension is bound here before
implementation, alongside the ordered operations above. Version 1 keeps both its
old canonical bytes and its precision-loss refusal. No published version 2 recipe
is being migrated.

An optional `binary64_inputs` map keys exact branch names to lists of typed paths
through that branch's first input root. Each path is an ordered list of
`{kind: field, name: KEY}` and `{kind: items}` selectors; the empty path selects a
numeric root. Items traverses every array element, not a chosen index. Unknown
branches, duplicate paths, undeclared fields, incompatible intermediate shapes
and nonnumeric leaves refuse before execution. Optional/null values retain their
presence: declarations neither create defaults nor turn null into a number.
Intermediate nullable objects/lists may be absent or null and are not traversed.
Version 1 refuses the declaration even when its map is empty; null is not a map.
When absent, serialization omits the field, preserving old canonical bytes.

At raw JSON admission, only numbers at the selected paths decode as IEEE binary64,
rounded to nearest with ties to even. Underflow to signed zero is admitted;
overflow to infinity refuses. Other numbers retain the exact existing admission
policy, including unused fields. Duplicate keys, nesting and syntax guards remain.
Branch identity selects the decode policy; it is not inferred from source values.
The value-based API applies the declared conversion to its already-decoded numeric
values but cannot reconstruct precision its caller discarded. Input schemas check
the decoded value, as at an explicitly declared source decoder boundary; exact
pre-decoding decimal validation is not claimed. Policies apply only at the external
input edge, not silently again at each stage.

`binary64_to_integer {value, steps, out_of_range: reject}` accepts a required
non-null numeric expression, converts its numeric value to binary64 and executes
ordered steps. Each step is `{op: multiply|minimum|maximum, value: TOKEN}`, with a
finite JSON numeric token stored as text so authored decimal provenance is not
rounded by the recipe parser. Every token and every step checks upfront, even in
an unselected expression. Multiplication rounds to binary64 after each step;
there is no reassociation or fused operation. Minimum and maximum preserve the
IEEE signed-zero distinction: min(-0,+0) is -0; max(-0,+0) is +0. Nonfinite inputs,
constants and intermediate results refuse rather than becoming JSON null.

Finally truncate toward zero into signed 64-bit representation. The finite value
must lie in [-2^63, 2^63); checking against a rounded representation of i64::MAX
would incorrectly admit +2^63. Rejection is explicit and is not host-language
saturation, wrapping or an implementation-dependent sentinel. Subsequent integer
scaling uses the existing separately declared reject/wrap policy. This represents
decode, clamp, binary scale, truncate, integer scale in their actual order instead
of substituting exact decimal scaling. Consumers whose source has implementation-
dependent out-of-range conversion must retain that compatibility qualification or
declare a supported input range; ESS does not invent a portable result.

The Rust reference and generated Rust share this code. Go and TypeScript targets
must implement the same declared decoding and ordered operations before their
normalization support can be claimed complete. Floating output serialization,
general floating equality and arbitrary expression-language evaluation are not
introduced by this integer-conversion operation.

Language evidence: [Go numeric conversions](https://go.dev/ref/spec#Conversions_between_numeric_types)
specify truncation and implementation-dependent out-of-range results;
[Rust binary64 parsing](https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64)
specifies nearest-value rounding. Use the standard binary64 parser after JSON
grammar validation, rather than relying on a JSON library's optional fast float
parser. Verify signed boundaries, subnormal/underflow, halfway decimals, decoded
integer tokens, lexical precision preservation outside declared paths, nested
arrays/nulls and actual generated-library execution under both JSON feature modes.

## Standalone Go Target

`Plan::go(package, module)` emits a standalone Go 1.26 normalization library from
the same sealed plan. Package/module identity uses the existing structural target
rules. The generated program compiles expressions and conditions into typed Go
function bindings; it does not accept a replacement recipe or interpret arbitrary
source strings. The canonical recipe, qualified source bundles, root schemas and
all generated files retain the existing normalization-target/1 provenance envelope,
with the explicit Go target configuration. Rust artifact bytes must not change
merely because packaging helpers are shared.

`New()` prepares reusable offline schema validators. `Normalize(branch, input)`
accepts JSON bytes and returns complete JSON bytes or a typed Refused containing
source-located Findings. No stage result escapes on failure; input bytes are never
modified. Requirements, expression/condition evaluation, lexical item/index scope,
selected-value laziness and every version 1/version 2 operation preserve the
reference semantics. Signed integer arithmetic is not performed through float64.

Use the pinned go-json-experiment/json jsontext decoder for strict JSON grammar,
duplicate names and Unicode handling, with a 64-level boundary imposed by this
adapter. Use standard strconv binary64 parsing only after token validation.
Integral tokens retain int64/uint64 identity; fractional and exponent tokens remain
float64 and do not become eligible for integer operations. Exact-admission decimal
comparison and explicit binary64 input paths preserve the reference policies.
Serialization must retain floating lexical identity, including signed zero, rather
than turning a computed float into an integral token between systems.

Schema validation uses github.com/santhosh-tekuri/jsonschema/v6, pinned at 6.0.2,
with draft 2020-12, no network loader and format assertions disabled, matching the
reference boundary. Source schemas are compiled from embedded data; unresolved or
unsupported schemas refuse at construction rather than retrieving network data.
The initial Go target refuses every retained `pattern` obligation at generation
with `go_schema_pattern`, qualified by bundle digest and schema pointer. RE2 does
not implement the reference validator's ECMA-262 pattern semantics; accepting its
overlapping syntax would silently change some constraints. A proven bounded
compatible matcher is required before removing this refusal. The guard reads the
existing structural plan's obligations, not arbitrary JSON object keys, and checks
the complete referenced closure of every selected root.
Schema failure presentation is target-specific: Go findings are sorted by escaped
instance pointer, retaining every occurrence. The existing Rust validator's
traversal order is preserved unchanged. Cross-target fixtures compare the complete
schema-finding multiset and additionally assert Go's stable order; all non-schema
refusals remain order-sensitive. This does not change which stage runs, which
expression is evaluated first, or where a failed operation is located.
The JSON token decoder is pinned at
v0.0.0-20260601182631-00ed12fed2a6, an exact Go 1.26-compatible revision rather than
a moving experimental API. Dependency identity belongs in the emitted module and
checksums. All generated code must compile and execute through the repository's
explicit go-typecheck lane, including old, ordered and numeric fixtures, malformed
Unicode, duplicate keys, nested schema failures and preservation of signed bounds.

This is a Go runtime library, not a Go reimplementation of the ESS compiler or a
second specification authority. The Rust generator remains the sole plan-admission
edge. Schema refinements are still checked at each stage, never replaced by the
structural type projection. TypeScript and the target-generation CLI remain part
of the original normalization story and are not completed by this target alone.

## Bounded Go Pattern Qualification

The first qualified Go schema-pattern profile is the exact frozen expression
used by the current model `Bytes` projection:

```text
^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$
```

Let A be the 64 ASCII letters/digits/plus/slash characters. Its language is
`A^(4n)` followed by nothing, `AA==` or `AAA=`, for n >= 0. With no flags,
[ECMAScript assertions](https://tc39.es/ecma262/multipage/text-processing.html#sec-assertion)
anchor the entire input, and [Go syntax](https://pkg.go.dev/regexp/syntax) gives
the same start/end behavior. Non-ASCII scalars and all line terminators are outside
the alphabet; no case folding, Unicode classes or lookaround is involved. Empty
strings are admitted. Padding placement is checked, but unused pad bits are not:
this is the declared pattern language, not a stricter binary decoder.

Qualification is exact-pattern identity, not acceptance of arbitrary overlapping
syntax or an inference from `contentEncoding`. Typed structural planning retains
each actual pattern value at its schema pointer. Go generation allows the frozen
expression only; every other selected pattern retains `go_schema_pattern` at
the original source location before publication. Data members or annotation values
named `pattern` cannot enter this accounting. A changed model primitive pattern
requires requalification, not automatic expansion of the supported profile.

Pattern metadata follows schema-valued positions independently of structural type
lowering, including model map-key schemas under `propertyNames`. Those keys keep
the existing `model_map_keys` structural report obligation; their nested patterns
are privately accounted for Go qualification at the exact source pointer. The
primitive-key enumeration covers every current model key shape: no key constraint
for String, enum for Boolean, formats for Timestamp/Duration, and patterns for
Integer/Decimal/Uuid/Bytes. Only the frozen Bytes pattern is qualified. The shared
runtime corpus exercises it as both a value and a property name.

The existing pinned Go validator uses its standard RE2 engine, whose execution
is [linear in input size](https://pkg.go.dev/regexp#hdr-Overview). This fixed pattern
has bounded compiled state and needs no backtracking or wall-clock acceptance
budget. Input byte limits remain a caller policy, not a new schema constraint.
The corpus checks the pinned reference and generated targets for all ASCII
characters, Unicode/line boundaries, padding forms, empty/long inputs and
noncanonical pad bits; separate generation cases refuse ordinary unqualified
patterns, lookaround, backreferences, Unicode classes and pathological nesting.
No existing recipe, schema or target report format changes for this feasibility
extension, and no successful old bundle-only artifact bytes need to change.

## Model-Owned Stage Roots

`ess-normalization/3` admits model-owned stage roots as well as the existing
bundle roots. Versions 1 and 2 keep their existing serialized bundle identity and
meaning; they refuse model roots. A model root contains a `model` identity and a
qualified `root`. The identity records system, specification version, source,
contract and projection digests, and the exact selected root set. Projection bytes
alone do not declare which members of a reachable closure were explicitly selected.
Adjacent stages compare this complete identity, not merely a compatible wire shape.

The library accepts sealed `ess_gen::schema::ModelTypes` selections alongside
checked bundles. It never accepts a JSON document's `x-ess-*` claims as proof that
the compiler minted the model. The existing structural `Plan::from_model` owns
wire names, nominal types, closure and obligation accounting. The normalization
planner reuses it without stripping annotations or manufacturing a parallel schema.
Only roots explicitly admitted by the supplied selection can be chosen. A stale
source digest, contract digest, projection digest, root set or model identity
cannot resolve to a supplied selection.

Model stage validation uses the existing projected definitions and draft-2020-12
wire rules, including requiredness and refinements. Model invariant statements
are retained but are not an executable predicate evaluator: a selected closure
with invariant obligations refuses normalization planning rather than advertising
a successful partially enforced adapter. Nominal names remain source identity;
JSON values themselves do not acquire host-language nominal types.
The compiler's string-enum projection combines `type:string` with a finite `enum`.
Normalize that exact checked-model intersection to its string literals; do not
flatten arbitrary imported intersections or `allOf`. Membership is still validated
at each boundary, and all other intersection/tuple mappings remain refused.

The CLI compiles each supplied `--model` specification and recreates the exact
root selections declared by the recipe before checking their complete identity.
This is source acquisition, not permission to trust authored provenance. Outputs
must not replace any model input or be written inside a model input directory.
Bundle-only command invocations remain unchanged.

Generated targets retain each complete selected model projection, including its
provenance and annotations, plus a root-specific validation schema. The target
report uses `ess-normalization-target/2` for recipes using version 3, preserving
the old report envelope for old recipes. Its root identity distinguishes model
projections from qualified bundles. Rust and Go use the same selected schema
closure; their existing target-feasibility refusals still apply, including Go's
unqualified pattern semantics. Adding model roots does not claim lexical JSON
capture, invariant execution or TypeScript normalization.

## Adapter Generation Command

`ess generate schema normalize-generate` reads `--recipe` and repeated `--bundle`
inputs through the existing sealed-plan boundary. `--target rust|go` selects an
implemented library target; `--package` is required, and `--module` is required
only for Go and refused for Rust. No structural target substitutes for an
unimplemented normalization target. TypeScript joins this command only when its
runtime realization exists.

The command writes the exact library-API file set to required `--out`, including
source recipe, bundles, embedded schemas and normalization-target/1 report.
Admission and target feasibility finish before output preflight. All generated
paths pass the existing shared containment, symlink, hard-link, case-alias and
file/directory checks before any write. Canonical source input paths must not
equal any generated destination. This retains the existing trusted-parent
assumption and does not claim rollback on subsequent I/O failure.

`--check` compares every planned file byte-for-byte without creating directories
or changing files. Missing or stale files are listed deterministically and give a
nonzero exit. It checks the generated file set, not ownership of unrelated files
in an adopter's directory; neither mode deletes obsolete or unowned files.
Source/target refusals still apply in check mode. Tests compare both targets with
the library output and cover input protection, complete preflight, read-only drift,
invalid unused branches and target refusals before publication.
