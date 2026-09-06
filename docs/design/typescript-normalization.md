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

This target does not itself implement lexical JSON capture, source decoder policy
or a consumer's runtime dispatch. Those remain separate, explicitly tracked work.
