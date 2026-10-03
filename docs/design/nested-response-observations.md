# Nested response observations

Design candidate for the held ESS consumer bundle. The measured defect is a source-admitted
`event.packet.value: {response: value}` mapping whose healthy and unequal response/event values
all pass the synthesized suite. An independent generated sibling still fails on a wrong type.
The missing obligation is equality against the same invocation's actual response.

## Authority

Extend `response::Observation` with an omitted optional `nested` block. Preserve the meaning,
ordering and serialization of the existing `fields`, `declarations`, `mappings` and `targets`.
The block contains ordered event root fields, their complete reachable representation declarations,
and mappings from a checked array of destination field segments to one top-level response field.
Paths are not dotted strings, JSON pointers, collection indexes or wildcard expressions.

The structural declarations use the representation variants of `selection::Declaration` through
dedicated closed structural DTOs. Root and struct-member fields permit exactly `name` and `type`;
they reject naming, presence, reading and other metadata. Each declaration permits only the
members of its selected kind. Producer omission alone is insufficient: every runtime must reject
the same forged extra keys before projection can discard them. Convert these checked
representations into registry types internally.
They prove declared traversal and terminal types; they do not claim to execute invariants or
readings. Collect complete representation bodies, including unrelated siblings, without imposing
the response-value profile on those siblings. Validate all references, declaration forms, names,
and exact reachable closure. Resolve structural graphs with a visited set; an untraversed sibling
cycle is finite schema data. A transparent alias cycle encountered by a selected path is invalid.

The existing response fields and declarations remain the authority for validating actual returned
values. Their existing restrictions remain explicit. This change does not implement Binary64 or
other currently unsupported response profiles; those remain required backlog work. In particular,
the existing whole-model Binary64 refusal must not be described as successful conformance.

Use checked private path/name wrappers and closed raw DTOs. An absent block preserves legacy
bytes; explicit null, an empty block, unknown fields and duplicate original JSON keys are rejected.
Each path has 2 through 33 segments using the source field-name grammar. Length one belongs in
the existing flat mapping. Its source names one declared response field, without traversal.

## Admission and bounds

Recompute the terminal TypeRef from the structural root. At intermediate positions, unwrap Optional
and named newtypes until a Struct supplies the next declared member. Lists, maps, unions, enums
and primitives cannot be intermediate traversal operations. Preserve nominal identity at the
terminal and use the existing source-to-target assignability rule. Do not infer conversions.

Require unique paths and roots, exactly the roots used by paths, no prefix-overlapping paths, and
no overlap between a flat mapped root and a nested root. Multiple leaves may reference the same
response field. Order generated mappings lexicographically by segment arrays; preserve source
event-field ordering for roots and ordered maps for declarations.

Keep at most 256 response fields and 256 combined flat/nested relationships. Bound the union of
nominal names across value and structural declarations to 4096. Where a name occurs in both,
require the same representation while ignoring only metadata the structural projection deliberately
does not carry. Count both copies within the existing whole-observation 1 MiB byte bound.
Preserve the existing response-value resource limits and original-byte admission protections.
Memoize structural resolution so shared graphs do not expand exponentially per path.

For observations carrying `nested`, all runtimes measure one explicit compact UTF-8 JSON profile
of the complete admitted observation: preserve ordered field arrays, sort map keys, serialize only
the active members of each declaration kind, use `[]` and `{}` for empty collections, and include
declared response-field presence. Use minimal JSON escaping matching Rust serialization: escape
quotes, backslashes and control characters; do not HTML-escape or escape U+2028/U+2029. Count
UTF-8 bytes, not characters. Go's legacy struct marshaling and TypeScript's legacy Go-shaped
`marshalShape` are not the nested profile. Implement equivalent normalization/counting and pin
exact boundary fixtures across all runtimes, including admitted escaped/non-ASCII strings.
The member order is `command,outcome,event,fields,declarations,mappings,targets,nested`;
outcome is `command,outcome`; nested is `roots,declarations,mappings`; each mapping is
`target,source`. A field is `name,type` followed by `presence` when declared. Declaration member
orders are `kind,of`, `kind,fields`, `kind,variants`, and `kind,tag,variants` for newtype, struct,
enum and union respectively. Sort dictionary keys by UTF-8 lexical order. Use ordinary short
escapes for backspace, tab, newline, form feed and carriage return; other escaped control bytes
use lowercase four-digit Unicode escapes. Do not normalize away supplied unknown metadata.
Nested-less observations retain their historical serialization and byte-check behavior. This
avoids silently changing a historical admission boundary while making the new boundary exact.

The 33-segment boundary follows 32 source-admitted Struct constructions followed by a terminal
leaf. Source and admission tests must demonstrate both sides of that boundary. This is not an
extension of the existing three-segment AccessorPlan contract.

## Synthesis and comparison

Recursively inspect resolved Struct payload mappings. Record only ResponseField leaves; refuse
an executable conversion on the leaf or any traversed ancestor. Carry the complete structural
schema only for roots that contain a nested relationship. Do not change caller-state synthesis.

Ordinary event shape assertions remain for every unrelated generated, input-derived or literal
sibling. Remove only the response-mapped terminal subtree from those assertions; the response
observer validates and compares that complete value. Filtering must use whole field segments.

Use the existing selected command/outcome and its direct event association. Validate the actual
response using the current value profile before comparing each nested destination. Every ancestor
must be an actually present object, including a declared Optional ancestor constructed by the
mapping. Missing, null or scalar ancestors fail. At the terminal retain current Optional
absence/null equivalence and enforce the response field's explicit presence policy first.
Compare complete exact values; never round integers through JavaScript Number. Each TypeScript
lookup must be an owned property, including valid names such as `__proto__` and `toString`.

## Compatibility

This work amends the held, unreleased `ess-conformance/34` and `/35` authority. It introduces no
authored ESS key, new suite pair, report version, or independent release. Raw presence of `nested`
under any older suite major is refused before target callbacks, even when null or empty. Apply
the same gate to in-memory suites. Preserve every nested-less historical observation byte and
existing flat feature gate. Capture an actual frozen pre-amendment reader refusing the unknown
closed field under `/34` and `/35`; envelope numbers alone do not demonstrate that refusal.

Original input/coverage parent bytes, selected scenario identity, lineage, counts and provenance
remain immutable authority. No relabeling or regeneration of historical parents is permitted.

## Verification and ownership

Share independent healthy and wrong-response/wrong-event targets across native, actual generated
Go and TypeScript, and actual WASM execution. Keep the generated-sibling fault as a separate
control. Cover mixed flat/nested mappings, multiple roots/leaves, deep paths, Optional and
newtype ancestors, complete aggregate leaves, exact large integers, stale invocations and mutated
results. Malformed authority must fail before every target callback. Exercise resource boundaries,
shared/cyclic schemas, legacy bytes and ordinary sibling assertions.

Production scope is `response.rs`, new `response/path.rs`, `admission.rs`, Go
`response.go`/`runtime.go`, and TypeScript `response.ts`/`runtime.ts`, plus focused Rust-driven
tests. Existing embedded Go/TypeScript runtime assets retain the repository exception; new
executable implementation and harness code is Rust. The browser product execution bridge remains
a separate required story: library WASM execution does not complete it.

The full ESS bundle remains held until the remaining accepted work and integration gates pass.
No component PR or extra full remote gate is needed for this unit.
