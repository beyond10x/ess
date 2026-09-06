# Explicit positional-array normalization

Binding decisions selected by the coordinator for
`story:normalize-positional-array-input`. Implementation and qualification are
pending. This document introduces no released capability and makes no claim of
complete compatibility with a native JSON decoder.

The selected boundary is a freshly constructed fixed string array: missing stays
missing, null becomes a zero array, short arrays are padded, excess values are
discarded, and null consumed elements become empty strings. A separate checked
expression reads a statically known position. Neither the source schema alone
nor the positional-read expression silently selects a decoder policy.

The public source baseline is ESS
[`6c78676c35193423fe326b9dde21b8fc21681b8a`][baseline]. Its
[capture contract][capture-design] supplies the original-token and retained-document
boundaries. The separately selected Binary64 work owns normalization format 5;
this work follows its integration and uses **`ess-normalization/6`**, retaining
**`ess-normalization-target/3`**. Inspection of the frozen Binary64 unit
`bf16e504ccad68b2ee67607ba39606aadf07f627` establishes the report contract below;
refresh those source seams after integration before implementing this extension.

## Why two distinct mechanisms are necessary

The [structural plan][shape] already retains array prefixes, element schemas and
length bounds. The [normalization checker][checker] refuses positional prefixes
instead of flattening them. The [text decoder][input] decodes all ordinary array
elements before the [stage executor][execute] validates input and evaluates
expressions. Consequently, an expression that merely chooses the first two
elements cannot suppress numeric conversion of an ignored third element. An exact
tuple schema would also reject a short input before such an expression ran.

Format 6 therefore adds exactly:

1. `positional_inputs`: an explicit transformation at a branch's external JSON
   text boundary, before its first input schema is validated.
2. `position`: a typed read of an already checked, fixed tuple position.

No new model primitive, source-schema annotation, arbitrary JSON property bag,
generic decoder registry, model IR envelope or wildcard path segment is added.
Existing homogeneous collection operations retain their admission and behavior.

## Authored envelope and fixed-string profile

This is an envelope excerpt. Existing branches, stages and checked source roots
remain mandatory:

```json
{
  "format": "ess-normalization/6",
  "positional_inputs": {
    "decode": [{
      "path": [{"kind":"field","name":"operands"}],
      "kind": "fixed_string_array",
      "length": 2,
      "missing": "preserve",
      "null": "zero",
      "short": "zero_pad",
      "extra": "discard",
      "null_element": "zero"
    }]
  }
}
```

The optional `positional_inputs` member is a unique-key map from exact branch
names to ordered lists of declarations. It may be absent, an empty map or contain
empty lists; null is not a map. Absent members remain omitted in canonical recipe
serialization. Branch-map order follows the existing deterministic map contract;
declaration-list order is retained.

Every declaration field shown above is required. The declaration is a closed
record. `kind` and the five policy fields are closed string enums admitting only
the displayed values in this version. Missing fields, unknown fields, unknown
enum spellings and wrong JSON kinds are strict recipe syntax errors; no defaults
or alternate policies are guessed. Direct typed construction must obey the same
semantic constraints when the recipe is checked.

`length` is an unsigned integer and must be greater than zero. It must exactly
match the checked terminal tuple arity before any allocation based on it. Reject
zero arity. Do not add a hidden architecture-dependent truncation or convert a
large authored length to a platform index before checking it against the finite
schema prefix.

`zero` means the empty string for each slot. It is an explicit policy, not the
application of a JSON Schema `default`. The profile does not merge into caller
state. Repeated-member merging, nonzero preexisting array elements, heterogeneous
element decoding, coercion from numbers/bools, custom unmarshalers and replacement
of malformed Unicode remain outside this profile.

## Checked paths and source schemas

`path` retains the existing typed selector grammar:

- `{"kind":"field","name":"operands"}` selects an exact declared wire field.
- `{"kind":"items"}` traverses every item of a declared homogeneous collection.
- `[]` selects the external document root. It is valid only when that root has
  the required fixed-tuple schema.

The closed form of `items` accepts no extra fields. Preserve the existing path
depth limit. Resolve references through checked source identities. Use exact
decoded wire names, including escaped JSON names; different escape spellings of
one field are the same selector. Do not resolve internal model field names when
the checked projection supplies another wire name. Undeclared object keys,
additional-properties wildcards, tuple-index path segments and inferred union
discriminators are not admitted.

Nullable/optional **intermediate** records and homogeneous lists are allowed when
their non-null alternatives permit the exact existing field/items traversal.
An ambiguous tuple union cannot become a terminal by ignoring one alternative.
The terminal may be optional, but its present type must be one non-null, exact
closed tuple of the declared arity. Resolve ordinary refs before deciding that
type; do not flatten arbitrary intersections or select one branch by trial.

For the input decoder, the terminal must have:

- A nonempty explicit positional prefix with N entries.
- No admitted tail (`items: false`).
- Minimum and maximum length both N in the checked schema representation.
- Every position string-shaped, including existing checked string refs/newtypes
  and supported refinements. A nullable string, Any/unknown position or numeric
  position does not satisfy this profile.

The schema describes the value **after** the declared external-input policy:

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "operands": {
      "type": "array",
      "prefixItems": [{"type":"string"},{"type":"string"}],
      "items": false,
      "minItems": 2,
      "maxItems": 2
    }
  }
}
```

This optional property accepts absence after preparation. The raw source array
may be null, short or long because the recipe explicitly describes how it reaches
this schema. A caller cannot claim that the displayed schema, used alone, admits
that broader raw input. Required properties remain required: a missing selected
member is not synthesized merely to satisfy their schema.

Retain every existing source identity coordinate. Model-owned roots keep their
compiler-minted identity; qualified tuple roots keep their qualified source pins.
Schema changes require freshly checked identities, not edits beneath a retained
digest. No nominal type is inferred from an imported annotation.

All retained refinements still run at normal stage validation. Padding may fail a
nonempty-string constraint, an enum may reject an input string, and tuple-wide
constraints may reject the prepared value. Such failures remain schema failures;
the decoder does not remove constraints or invent replacement defaults.

## Text-input value semantics

At a selected boundary of length N:

| Original value | Prepared value |
| --- | --- |
| Missing member | Missing member |
| JSON null | N empty strings |
| Empty array | N empty strings |
| Array shorter than N | Consumed values followed by enough empty strings to reach N |
| Array of length N | All N consumed values, in order |
| Array longer than N | First N consumed values, in order; excess tokens are discarded |
| Consumed string | Decoded Unicode string; no trimming, case change or normalization |
| Consumed null | Empty string |
| Any other consumed kind | Located element-type refusal |
| Any non-array, non-null boundary value | Located boundary-type refusal |

String escape spelling is decoded here; this operation is not lexical capture.
It preserves string values, including empty strings and U+0000. It does not
preserve the original string token spelling. A numeric token in a consumed slot
is a type error without numeric conversion, including a huge exponent. Excess
numeric tokens are never converted.

Missing or null intermediate containers are not traversed, replaced or populated.
Their original absence/null remains for ordinary schema handling. A wrong-kind
intermediate container receives no positional coercion; ordinary input decoding
and validation determine its refusal. Requiredness and nullability of those
containers remain authored schema choices. The distinction between a selected
terminal null and an intermediate null is load-bearing.

The policy runs once per external text entry. Later stages see the prepared tuple
and never pad, truncate or decode it again. All ordinary first-stage validation,
ordered requirements and output validation remain in place. Failure returns no
partial result and never mutates caller data.

## Grammar, discarded tokens and deterministic error order

Retain the format-4 complete-document grammar precheck before branch lookup or
located value processing. Malformed JSON, trailing data and malformed excess
elements produce the existing complete-value syntax refusal before a consumed
slot's type error. Comments, trailing commas, NaN and Infinity remain invalid JSON.

After that precheck:

1. Outside selected token-processing boundaries, preserve the established strict
   Unicode, unique decoded key, numeric and depth rules. At each ordinary object,
   collect/check its decoded member names before processing children in decoded-key
   order. Ordinary arrays retain index order. A positional policy does not permit
   duplicate members in its containing object.
2. At a selected positional boundary, check its original-root depth before its
   null/array/type decision. Null constructs the zero tuple. A wrong boundary kind
   stops immediately; its children are not interpreted to manufacture another
   error. An array visits its original positions in ascending order.
3. For each consumed position, check node depth first, then kind, then strict
   Unicode decoding for a string. Null becomes an empty string. Wrong kinds stop
   at that original indexed pointer without numeric conversion or recursive
   object interpretation.
4. For each discarded position, perform a bounded lexical walk of its original
   token: strict UTF-8 and paired surrogate escapes, with the existing global
   depth 64 bound measured from the original input root. Check each node's depth
   before its string/key Unicode, then traverse object members in source order
   and arrays in index order. Do not deduplicate keys, interpret numbers, build
   an untyped value bag or retain base64. Duplicate members and arbitrarily large
   grammatical numeric lexemes are admitted inside the discarded token.
5. Stop at the first finding in this traversal. Therefore an earlier consumed
   type error precedes a later discarded token's Unicode/depth finding; malformed
   JSON anywhere has already won in the grammar phase. Pad after successful
   processing, then use the unchanged stage input/requirements/expression/output
   order.

Locate a discarded token's lexical finding at that original array-element
pointer, including a deeper failure inside it. Internal duplicate members cannot
be addressed unambiguously by a deeper JSON Pointer. This mirrors the existing
capture-boundary location principle without claiming discarded bytes are retained.
Depth 64 is admissible; a visited node at depth 65 refuses. Do not substitute a
token library's larger or smaller default depth profile for this contract.
The input depth bound counts original token nodes. Synthesized empty slots do
not acquire fictitious original-input depths; their prepared values remain subject
to normal schema validation.

These rules preserve canonical strictness on successful inputs. They do not
promise that a rejected wrong-kind subtree will be traversed to enumerate every
Unicode/depth defect. Error order is the declared traversal, not an exhaustive
validation pass or native decoder error wording.

Unknown branches have no positional policy, use the established ordinary text
decoding policy, then receive the existing unknown-dispatch refusal. Do not move
dispatch ahead of parsing as part of this change.

## Checked `position` expression

```json
{
  "op": "position",
  "value": {"op":"read","scope":"input","path":["operands"]},
  "index": 0
}
```

`index` is an unsigned literal integer; it is not an expression or path segment.
The checker requires one exact closed, positive-arity tuple and proves `index < N`.
Open or variable tails, variable-length prefixes, zero-arity tuples, homogeneous
arrays and ambiguous outer tuple unions refuse. Optionality of the whole tuple
is allowed. A nullable tuple must be normalized explicitly before this expression;
null is not silently treated as missing.

Pure statically bounded **heterogeneous** tuple reads are admitted. Retain every
position's own already-supported type and refinements; `[String, Boolean]` read
at index 1 yields Boolean. No union of all element types replaces this proof.
This permission does not add a heterogeneous source decoder. A source slot whose
type is otherwise unsupported remains unsupported; the new operation does not
erase that refusal or turn opaque source data into a concrete type.

Evaluation computes `value` once. If the tuple value is missing, the result is
missing. Otherwise it must be an array of the checked exact arity; a defensive
runtime mismatch refuses and does not invent a value. A successful result is the
selected position with its checked type. A present null **slot** in a pure
heterogeneous tuple can be returned when that slot's checked type admits null;
this differs from a null tuple and from the string decoder's null-to-empty rule.

The expression propagates missing optional tuples. Existing `fallback` can supply
an explicit scalar default without a new tuple constructor:

```json
{
  "op": "fallback",
  "value": {
    "op": "position",
    "value": {"op":"read","scope":"input","path":["operands"]},
    "index": 0
  },
  "fallback": {"op":"string","value":""},
  "on_null": false
}
```

Input/output structural assignment remains checked. Missing cannot fill a required
output without an explicit construction or fallback. Boolean, string, integer,
general number and the integrated format-5 Binary64 type retain their distinctions.
There is no implicit cast merely because a position contains an integral-looking
number. Whole-tuple reads retain their position-specific identity. Do not broaden
existing `list` construction into a new unchecked tuple constructor.

Do not admit tuples to `map`, `find`, `join`, `select_map`, `items` traversal or
other homogeneous operations by flattening their types, even if all positions
currently happen to be strings. Those operations retain their old contracts.

## Admission ordering and exact diagnostics

The closed recipe parser retains the existing `recipe_syntax` finding at `/` for
malformed JSON, missing mandatory fields, duplicate map keys, unknown fields,
unknown enum values and wrong JSON field kinds. Its existing parser detail text
is retained; this design does not introduce a second parser-message format.
In particular, `positional_policy` is not a new catch-all that bypasses closed
enum parsing. The following **new semantic/runtime diagnostics** have fixed
messages; substitutions are marked in braces.

Let `D` be `/positional_inputs/<escaped-branch>/<declaration-index>`, `P` a located
original input pointer and `E` the expression pointer.

| Phase / condition | Pointer | Rule | Exact detail |
| --- | --- | --- | --- |
| Declaration present in format 1–5, even an empty map | `/positional_inputs` | `operation_version` | `positional input declarations require ess-normalization/6` |
| `position` in format 1–5, including unused branches | `E` | `operation_version` | `position requires ess-normalization/6` |
| Declared branch does not exist | `/positional_inputs/<escaped-branch>` | `unknown_branch` | `positional input declaration names an unknown branch` |
| Zero length | `D/length` | `positional_length` | `fixed string array length must be greater than zero` |
| Terminal schema/arity/slot type does not meet the fixed-string profile | `D/path` | `positional_schema` | `fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions` |
| Exact repeated positional path | `D/path` | `duplicate_positional_path` | `positional path duplicates {earlier-D}/path` |
| Positional ancestor/descendant path | `D/path` | `overlapping_positional_path` | `positional path overlaps {earlier-D}/path` |
| Overlap with capture or numeric policy | `D/path` | `input_policy_overlap` | `positional path overlaps {conflicting-policy-pointer}` |
| `position` operand is not one exact supported tuple | `E/value` | `position_type` | `position requires one exact closed tuple` |
| Literal index is outside the checked arity | `E/index` | `position_index` | `position index is outside the declared tuple` |
| Active positional policy through decoded-value API | `/input` | `input_positional_provenance` | `positional input decoding requires original JSON text; use a text input entrypoint` |
| Selected present boundary is neither array nor null | `P` | `positional_input_type` | `fixed_string_array requires an array or null` |
| Consumed position is neither string nor null | `P/<index>` | `positional_element_type` | `fixed_string_array element must be a string or null` |
| Consumed string has invalid Unicode | `P/<index>` | `input_syntax` | `positional string contains invalid Unicode` |
| Discarded token has invalid Unicode anywhere inside it | `P/<index>` | `input_syntax` | `discarded positional token contains invalid Unicode` |
| Visited node exceeds original-root depth 64 | Boundary or original slot pointer, as defined above | `input_depth` | `JSON input exceeds 64 levels` |
| Defensive non-tuple/wrong-length value at `position` runtime | `E` | `position_value` | `position encountered a value outside its checked tuple contract` |

Retain existing complete-value syntax, unknown-dispatch, source-identity,
schema-validation, requirement and ordinary input findings. Existing path helpers
retain `unknown_field`, `object_type`, `collection_type` and path-depth findings
at `D/path/<segment-index>` where applicable. Missing source identities must never
be bypassed to infer the terminal schema.

Preserve the existing checker phases. New positional envelope checks follow the
older envelope checks; new positional input checks follow existing numeric and
capture checks for the first stage. Within positional checks use branch map order
and declaration order. For each declaration: reject zero length first; then exact
duplicate, earlier positional overlap, raw capture overlap, numeric overlap; then
resolve path and check terminal schema. Raw/numeric conflicts select the first
conflict in that policy's declaration order. One of these local failures ends
checks for that declaration, preventing a redundant leaf-type diagnostic.

An older-format declaration receives its version finding without new positional
path/type diagnostics; an older-format `position` receives its version finding
without speculative operand/index findings. Existing independent checker findings
may still accumulate according to the older phase contract. Syntax errors retain
their existing pre-check precedence. These guards also apply to directly
constructed recipe values, not just serialized JSON.

## Policy overlap and decoded-value entrypoints

Compare decoded field names and typed `items` segments. Equal paths and ancestor
relationships in either direction conflict. A root positional path excludes all
other positional, raw and numeric paths in that branch. Disjoint siblings are
allowed. Do not let declaration order choose which incompatible policy wins.

No positional index segment is added to raw or Binary64 selectors. Nested mixed
policies that require such a segment refuse rather than applying one policy to
every heterogeneous slot. Existing raw/raw and raw/numeric conflict behavior
remains unchanged.

Branches with one or more positional declarations require original JSON text,
even when the selected members would be absent in a supplied decoded value.
`Plan::run` and generated Rust `normalize_value` refuse before value preparation,
numeric conversion or schema validation. Empty maps/lists do not activate this
requirement. Pure tuple mapping without a positional policy retains existing
decoded-value behavior and makes no claim about lost input token history.

When a branch has both disjoint raw capture and positional declarations, retain
the existing raw-capture value-API provenance refusal first; otherwise return the
new positional provenance refusal. This preserves the established raw failure
contract and gives one deterministic refusal. Unknown branches have no active
policy and retain their established unknown-dispatch behavior.

The [retained-document helpers][retained] remain the caller's explicit composition
boundary:

- Reference: `Plan::run_base64_json(branch, encoded)`.
- Generated Rust: `Normalizer::normalize_base64_json(branch, encoded)`.
- Generated Go: `Normalizer.NormalizeBase64JSON(branch, encoded)`.

They validate canonical standard base64, then UTF-8, then delegate unchanged JSON
text to the selected recipe. This work adds no base64-decoding expression, nested
recipe interpreter or new CLI input flag. A caller may capture an enclosing
document with recipe A, retain its bytes, and invoke positional decoding in recipe
B through the helper. Both plans retain their own checked identities. A single
branch cannot combine an enclosing capture with positional decoding beneath it.

The caller owns which field is retained, dispatch, invocation order, error
composition and any cross-call atomicity/attestation requirement. Each call is
atomic independently. A helper is not proof that arbitrary supplied base64 came
from another checked recipe. Depth starts at zero for the new retained-document
entry, after the original enclosing document passed its own depth bound.

## Versioning, target publication and old bytes

Use format 6 only after the integrated format-5 baseline is fixed. Formats 1–5
retain their old meanings, refuse the new member/operation and keep their old
tuple-admission behavior. No tuple becomes silently executable in an older recipe
because the shared checker now has a positional type. Old readers must reject
format 6. The new member is version-gated even when empty; absent declarations
must not change old canonical recipe bytes.

No new authored model or schema-bundle format is needed for retained qualified
tuple inputs and existing model outputs. Reuse full checked root identities and
recipe/file digests and retain `ess-normalization-target/3`. In the frozen
Binary64 unit, `normalize/target.rs::finish` hashes the complete canonical recipe
and emitted file set; its root coordinates retain the meaning already established
by raw input preparation. Recipe format 6 declares the new source-decoding policy.
No report field, digest interpretation, root meaning or attestation guarantee
changes here, so a new report version would add no distinct contract.

Runtime admission reads the retained recipe and checks its version; CLI drift
checking regenerates and compares the complete file set. Extend current-family
dispatch deliberately to format 6. Do not copy policy declarations into a second
untyped report field or accidentally emit format 6 through a legacy template.
A later change to the report's own persisted meaning requires its own version.

Reference, generated Rust and generated Go must implement the same input policy,
position type rules, diagnostics, ordering and composition or refuse before any
artifact publication. A structural type projection is not executable normalization
parity. If another target remains unsupported, retain its explicit pre-publication
refusal. Do not silently widen scope into that target's implementation.

Before editing shared emitted sources, preserve complete expected file maps for
representative format-1 through format-5 recipes in both existing targets, using
the final format-5 integration and fixed package/module identities. The baseline
[Rust emitter][target] includes shared recipe/input/evaluator/runtime files, while
the [Go emitter][go-target] emits native policy and operation bindings. Freeze any
additional templates needed to preserve older generated bytes; do not mutate
previously frozen templates to make a new format compile.

For identical admitted inputs, target configuration and generator version, every
legacy emitted path and byte must remain identical, including manifests and report
file-digest maps. Across an actual package release, truthful generator-version fields and
version-bearing generated headers may change, together with their file digests.
Measure legacy byte compatibility at the same generator version; separately
account for release metadata changes. Do not omit emitted files, normalize source
text or forge producer versions to hide differences. A later intentional legacy fix requires its
own recorded generation migration.

The [stage executor][execute] keeps its validation order, but its format-family
selection must change: the frozen unit's `execute.rs::run` enables Binary64
context only for format 5. Format 6 must inherit that context, and the generated
Rust runtime must admit recipe 6. Freeze the old executor bytes before changing
the shared source. Preserve format-5 Binary64 identity, literal/conversion/equality
admission and numeric policy. Do not
inspect a broad `type:number` schema and infer Binary64 for a tuple position.

## Independent qualification corpus

Author a public generic fixture around optional `operands`, a required variant,
one root tuple, a list of containing records and one heterogeneous exact tuple.
Its expected values, rules, locations and ordering are fixed independently of the
new implementation. Share literal cases among reference, CLI and native target
drivers; do not generate expectations by calling the same positional decoder.
Keep a native-language decoder comparison separate and label its intentional
differences; it is not the canonical oracle.

The binding corpus includes:

| Group | Required cases |
| --- | --- |
| Fresh string values | Missing; null; empty; one; exactly N; excess; null in each consumed position; repeated/empty strings; escaped U+0000; multibyte Unicode |
| Discarded values | Every JSON kind; huge integer/exponent; duplicate and escape-equivalent object keys inside a tail token; nested arrays/objects; no numeric conversion or duplicate-member refusal there |
| Consumed kinds | Every wrong kind at every consumed position; huge number consumed versus discarded; first consumed error wins; no partial tuple |
| Grammar/ordering | Earlier type error plus malformed suffix; valid consumed pair plus malformed tail; trailing second JSON value; earlier type error versus later tail Unicode/depth error; competing ordinary object keys |
| Lexical limits | Malformed Unicode in consumed strings and discarded keys/strings; depths 64/65 measured from the original root; deeper invalid nodes located at the discarded element boundary; duplicate containing members still refuse |
| Source schema | Zero arity; length/prefix/bounds mismatch; tail admitted; nullable/nonstring/unknown profile slot; retained refinement rejects padded empty string; stale root/source identities |
| Position typing | Exact heterogeneous tuple yields the selected type; missing whole tuple yields missing; nullable slot remains null; nullable tuple, homogeneous array, variable prefix, open tail and ambiguous tuple union refuse; negative/noninteger index syntax fails; out-of-range unsigned index refuses |
| Paths | Root, field, nested homogeneous items, escaped wire field, missing/null intermediates, wrong intermediate kind, duplicate paths, both overlap directions, disjoint raw/numeric siblings |
| APIs/stages | Active-policy value-API refusal even for absent members; empty policy remains inactive; raw provenance wins for mixed active policies; second stage never decodes again; unknown branch; retained-base64 helper composition and its existing failure precedence |
| Version/targets | Old readers reject format 6; formats 1–5 reject new members/ops including unused branches and typed construction; older tuple recipes remain refused; exact complete legacy file maps; reference/CLI/Rust/Go findings and values agree |

Run the generated Rust corpus with the existing default and arbitrary-precision
serde_json configurations so a feature flag cannot change the declared ignored
tail or numeric-kind semantics. Native target cases exercise their actual token
parsers, not a pre-parsed fixture substituted at the input API. Reuse the existing
normalization harness for code generation and complete artifact accounting.

These are required future checks. No test, adapter generation, release adoption
or native parity result is claimed by this binding draft.

## Implementation seams established by the public baseline

The concrete seams are the closed [recipe][recipe], [tuple checker][checker],
[input preparation][input], [reference evaluator][eval], [plan/entrypoints][plan],
[Rust runtime][rust-runtime], [Go input][go-input], [Go runtime][go-runtime], [Go bindings][go-target]
and [target report/emission][target]. The [retained lexical validator][retained]
is a candidate for shared lexical-only tail validation; preserve its existing
capture messages, locations and source-order traversal when factoring it.

The exact frozen-template layout must be taken from the final integrated
format-5 source. These baseline citations identify existing seams; they do not
claim that format 5 or format 6 is implemented at commit 6c78676.

[baseline]: https://github.com/beyond10x/ess/commit/6c78676c35193423fe326b9dde21b8fc21681b8a
[capture-design]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/docs/design/raw-json-normalization.md
[shape]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize.rs#L126
[checker]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/check.rs#L248
[input]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/input.rs
[execute]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/execute.rs#L20
[retained]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/retained.rs
[target]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/target.rs
[go-target]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/go_target.rs
[recipe]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/recipe.rs
[eval]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/eval.rs
[plan]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize.rs
[rust-runtime]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/rust_runtime.rs.txt
[go-input]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/go_input.go.txt
[go-runtime]: https://github.com/beyond10x/ess/blob/6c78676c35193423fe326b9dde21b8fc21681b8a/crates/generate/schema-contract/src/realize/normalize/go_runtime.go.txt
