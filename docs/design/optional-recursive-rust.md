# Optional recursive structs in the Rust target

Decision: story:optional-recursive-rust, adopter report beyond10x/ess#400.

A compiler-admitted struct may contain an optional child of its own type. Rust's `Option<T>` is
not an indirection, so the previous Rust target correctly refused that representation as
`recursive-layout`. This decision extends representation coverage without changing the authored
language, neutral synthesis plan, wire format, or previously successful generated output. It
supersedes the optional-self refusal in `review-rust-target-feasibility.md`; that design's other
refusals remain.

## Representation

A named struct with an exact `Optional<Self>` field is a recursive struct candidate. The Rust
layout records those resolved identities once. Every reference to such a named type is represented
as `std::boxed::Box<T>`, including references in optional fields, collections, command inputs,
events, view rows and accessor signatures. The declaration itself remains `struct T`, with its
ordinary constructor. Keeping references context-free makes an accessor's semantic input agree
with the representation of a field carrying the same type. Field-only boxing would require a
second accessor type algebra and conversions at every traversal edge.

The wire decoder for a named declaration still returns its unboxed `T`; decoding a reference to
a recursive struct wraps that result in `Box::new`. Encoders borrow the declaration through
Rust's dereference coercion. The JSON shape, field names, optional absence/null handling and enum
wire labels are unchanged. Fully qualified `std::boxed::Box` avoids capture by an authored `Box`
declaration. Allocation is target representation, not authored domain semantics.

Feasibility removes only exact optional-self struct edges from the source size graph, because
these are the edges this decision intentionally admits. Mandatory self cycles, mutual cycles,
newtype cycles and union cycles retain deterministic source-addressed refusal, even if a candidate
also has an optional-self edge. List/Map-mediated recursion retains its existing behavior. This
bounded policy prevents incidental global boxing from silently admitting additional cycle classes.

## Composition and compatibility

The layout's relative and absolute type renderers share the representation choice. Declaration
paths remain raw names for definitions, constructors and named codec entry points; use-site type
references carry the box. Shared HTTP and Web decoding uses the same layout decision. Generated
bindings, accessor traversal, stored fields, guards and collections must preserve this distinction;
a reached unsupported construction must be refused before rendering rather than emitted as
uncompilable Rust.

An already admitted model has no exact optional-self edge, so its set of boxed types is empty and
its generated artifacts remain byte-for-byte unchanged. No format version, new source key,
semantic IR field or failure code is needed. Generated Rust for newly admitted models requires
callers to construct boxed references; these models had no successful Rust artifact API to migrate.

Enum format labels are a separate authoring matter: `variants: [{name: DocumentV1, wire:
"demo.document/1"}]` already expresses a valid Rust identifier and its exact serialized spelling.
No identifier normalization or wire rewrite is part of this change. Adopting that idiom can change
source and contract identities even when serialized values stay unchanged.

## Verification

A minimal optional-self fixture must first fail on the old target, then compile from exact fresh
emission in both workspace and single-crate layouts. Nested values round-trip through generated
codecs, retain wire labels, and traverse generated binding accessors. A shared Web codec fixture
compiles with its Rust dependencies. Existing acyclic artifact controls, deterministic generation,
List/Map recursion and all unsupported-cycle refusals stay covered. Code compilation is necessary:
source substring assertions alone cannot establish agreement between constructors and use sites.

The implementation reports measured limitations, exact commands and runner counts with the AEP
story. Package tests, formatting and strict Clippy are local checks; repository CI owns the full PR
gate, and release verification follows AGENTS.md.

## Implementation measurements

The initial acceptance fixture failed with `recursive-layout` in both Rust and Web. A
context-free reference-layout change admitted it and compiled its nested codecs and binding
accessors. Extending that same fixture with a structured event payload exposed the remaining
constructor seam: using `absolute_type` in a struct literal emitted `Box<T> { ... }`, which Rust
rejected with `comparison operators cannot be chained`. Structured payloads now construct the raw
declaration and pass it through the same reference-value helper as wire decoding. The fixture
executes that generated behavior and checks its nested child as well as the codec round-trip.

The accessor and selection emitters already derive their typed signatures through
`Layout::absolute_type`; their field reads preserve the boxed representation. Scalar literal,
newtype conversion and increment constructors cannot construct a recursive struct. Event, command,
view and entity constructor paths name their own declarations, which are not members of the boxed
struct set. These source checks bound the constructor class alongside the executable witness.

## Multiline authored documentation prerequisite

The adopter's actual fresh generated crate also exposed a pre-existing emitter defect: domain
summaries containing paragraph breaks placed their later lines outside `//!` comments, so valid
source produced invalid Rust. System, component and item summaries use the same single-line
interpolation pattern; human display names and conversion explanations are the same free-text
class. Prefix each continued line with the current documentation marker and indentation. Keep
single-line text byte-identical and leave source text, wire labels and generated APIs unchanged.
A separate minimal fixture compiles multiline summaries and display names through the real Rust
emitter before the adopter's exact generated crate is checked again.
