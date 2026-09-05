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
