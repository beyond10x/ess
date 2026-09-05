# Types-Only Realizations

## Boundary

Generate data libraries, not a service, application, lifecycle executor or a claim
of decoder parity. Existing full synthesis remains unchanged. Its nominal wrappers
and runtime contracts cannot simply be extracted and called JSON data types.

The first concrete input is a sealed `schema_contract::bundle::Bundle`. Reload it
through its replay checks. Select explicit roots admitted by that import and their
complete reference closure. Never invent an OpenAPI service to obtain type input.
Resolved ESS type roots use the same target contract, with their
nominal identity and authored wire semantics accounted for explicitly.

## Resolved Model Input

`ess-gen::schema::ModelTypes` is a sealed selection built from a checked `EssIr`
and explicit qualified type roots. It closes references through the compiler's
type handles and reuses `schema::types` for every primitive, field, optional value,
map and tagged union. It refuses missing roots and duplicate wire field names before
serialization; a collapsed property map is not an acceptable projection.

The selection emits a JSON Schema definitions document, not an invented component
bundle or OpenAPI service. Its source is the resolved model, and it carries the
existing source/contract digests. The shared realization plan consumes that sealed
selection; it does not accept unqualified caller-supplied model schemas. Findings
locate schema positions in the retained `source.schema.json`, whose `x-ess-name`,
`x-ess-kind` and `x-ess-field` annotations retain model identity and wire renames.
Invariants stay named runtime obligations. Non-string map-key grammars remain schema
validation obligations; host maps use their established JSON string keys.

`ess-types-report/3` replaces the unreleased v2 envelope with a typed input identity:
qualified bundle digest, or model system/version/contract/projection digests. Both
retain the source digest and target configuration. A model digest must never be
labelled a bundle digest. TypeScript's structural aliases do not enforce nominal
identity; report that limitation for each model newtype. Native wrappers preserve
distinct host declarations but do not discharge model invariants.

Expose model selection as `ess generate types --path SPEC --root QUALIFIED_NAME`
with the same explicit target/package/module options as `schema types-bundle`.
An exclusive `--all-types` selector deliberately selects the complete named type set;
omitting both selectors or combining them is a refusal, never implicit selection.
Publish the checked schema selection alongside the library and report. Neither this
selection nor its output changes the model language or yet binds an ESS field to an
imported schema root; that connection requires its own checked identity contract.

## Shared Structural Plan

A language-neutral, in-memory plan owns selected roots, component identities,
declaration names and concrete structural nodes. Nodes distinguish unrestricted JSON,
impossible values, scalar types, literals, references, objects, arrays, unions and
intersections. Object requiredness is separate from a nullable value. Array prefix
positions are separate from trailing items. A union records whether its source was
`anyOf`, `oneOf` or a type array; it does not invent a discriminator or module name.

Source component names normalize to deterministic ASCII declaration names. A
collision is a refusal, never traversal-order suffixing. References resolve through
the selected closure; anonymous nodes retain source pointers. Reject unguarded
alias cycles that cannot be declared consistently across targets. Recursive object
and array references remain legal. A component reference into a nested schema needs
an explicit nested declaration before it is accepted; do not bind it to the enclosing
component by dropping pointer segments.

Keywords are classified as structural, annotation, runtime constraint or unsupported.
All structural siblings apply together, including beside `$ref`, `const` and `enum`.
Unsupported combinations refuse before a partial successful artifact is returned.
Conditional object/array constraints without an explicit type need a conditional
mapping, not an inferred narrowing to that type. Defaults and formats remain
annotations; they do not enable coercion or infer integer storage widths.

## Target Accounting

The output report `ess-types-report/3` names the source and typed input identity,
selected roots, generator version, target and source/name mapping.
It records source-located constraints and annotations, and named target obligations.
It is an output report, not another authored or reloadable specification format.
Version 2 added typed target configuration, including native package identity, to
the unreleased version 1 envelope; version 3 distinguishes model and bundle input
provenance. Do not silently change a persisted envelope. Carry the original qualified
bundle or checked model schema selection beside generated declarations. Model schema
selections are definitions documents: validation must select a root by its `$defs`
reference, not validate an instance against the unconstrained document root.

A successful type projection is not a zero-obligation runtime implementation.
Patterns, numerical bounds, integer checks, array cardinality and uniqueness,
exact object closure and exclusive unions are still checked by JSON Schema.
Runtime aliases, flattening, coercion and external dispatch absent from the source
remain unimplemented rather than guessed. No implicit root discriminator is added.

## Language Mappings

TypeScript emits declarations only. Required-nullable fields use `T | null` without
`?`; optional fields use `?` independently. Consumers use strict mode and
`exactOptionalPropertyTypes`. Unrestricted source JSON uses a recursive JSON-value
type, never unrestricted `any` or an unexplained `unknown`. Boolean false maps to
`never`. Unions preserve alternatives and intersections preserve sibling constraints.
Prefix tuples preserve required/optional positions; trailing items remain separate.

TypeScript `number` cannot retain arbitrary JSON numeric precision; the report must
name that boundary. An `oneOf` union has an exclusivity validation obligation. An
open object with a constrained additional-value schema and named properties needs
an index-signature weakening because TypeScript cannot subtract literal keys from
`string`; record that weakening rather than reject valid named property values.
Object closure still needs runtime validation even for a closed object declaration.

Go and Rust consume this same plan and preserve wire field names, not the host
language's default name transformation. Their implementation must include explicit
presence/nullable representations and open-object storage. Do not claim completion
using pointers or `Option<T>` that collapse absent and present-null. Recursive values
need size-breaking indirection. Native numeric storage and union decoding choices
must have named target accounting; a first matching untagged alternative does not
establish external dispatch semantics.

Rust emits a standalone Cargo library with an explicit package name, Serde and
`serde_json` with arbitrary-precision numbers. Required fields use a deserialization
hook that does not treat missing nullable fields as null. Optional fields use
`EssPresence<T>` independently of their value type. Open objects retain unknown
properties; serialization refuses additional-key collisions with declared fields.
String literal sets become enums with exact wire renames. Untagged union decoding
tries typed alternatives from a retained JSON value, preserving unknown data and
number precision; the chosen Rust variant does not certify source branch validation.
Fixed prefix tuples become tuple structs. Unsupported native intersections or
variable prefix layouts refuse explicitly rather than becoming an untyped JSON bag.
Anonymous native declarations derive names from source identity, not traversal order.

Go emits a standalone module with explicit package and module identities. Optional
fields use `EssPresence<T>`; nullable values independently use `EssNullable<T>`.
Exact JSON numbers and unrestricted JSON values have distinct checked carriers.
Records decode through raw-member maps, enforce required and closed-object shapes,
and preserve typed additional members. Arrays and tuples have native codecs so a
nil slice never silently becomes a non-nullable JSON null. Untagged alternatives
use sealed native variants with the same documented structural-selection boundary
as Rust. Invalid native zero values or additional-key collisions fail serialization
instead of being silently rewritten to valid-looking data.

## Publication

Use `schema types-bundle` with explicit bundle, root selection, target and output
directory. Complete planning, emission and output containment checks precede writes.
Preserve the existing `schema typescript` command for compatibility until its callers
can adopt this accounted input path. It must not silently gain different defaults.

Verification covers root closure, names, recursion, every structural node, unchanged
source, deterministic output and failure without partial writes. Compile emitted
declarations with pinned language toolchains and check required versus nullable,
wire keys, tuple positions and union alternatives. Runtime conformance is a separate
claim requiring the schema and decoder semantics, not just compiler acceptance.
