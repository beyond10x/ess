---
format: aep.planning-md/3
id: story:a-distinct-invariant-takes-several-members-of-an-element-as-its-key
kind: story
status: draft
title: A distinct invariant takes several members of an element as its key
summary: distinct names one member of the element as its key, so uniqueness over a tuple of members cannot be stated (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:an-identity-may-be-a-composite-of-fields
- informed_by: architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first
- informed_by: story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant
revision: 2
---
## Need

A list of structs is unique on a pair of members, not on one: no two pieces share both a kind and a name, though two may share the
kind. `distinct: {in: <list>, as: <name>, by: <name>.<member>}` names one member as the key (`crates/specify/ess-primitives/src/predicate.rs:3395-3398`:
"`by` names one member of the element, a fact path"; `website/docs/reference/predicates.md` § "From `ess/22`: distinct list members"). A
struct key is refused as `type_mismatch` (the same page), and an invariant has no way to build one key from two members, so the model
either claims a stronger rule (unique on one member) or states none.

## Reproduction

Brand-free, `format: ess/37`, domain `demo.run`: `Piece {kind: String, name: String}`, `Bag {pieces: List<Piece>}`, installed 0.98.0:

| written on `Bag` | result |
|---|---|
| `distinct: {in: pieces, as: piece, by: piece.name}` (control) | valid |
| `distinct: {in: pieces, as: piece, by: [piece.kind, piece.name]}` (the requester's need, written as a list) | `[unparsable_predicate] ... cannot parse predicate "distinct: as: ...": `by` names one member of the element, a fact path` |

The proposed spelling is mine, not the requester's: the report states the need only ("`distinct` takes one member as key, no composite key").

## Findings covered

- Gap 3 of the thirteenth report.

## Fit review

1. Need: uniqueness of list elements over a tuple of their members, checked in the specification. Domain fact.
2. Class: gap. No idiom keeps the check: a derived key needs concatenation in an invariant, which the language lacks
   (`docs/design/generic-language-features.md` text functions are values, not invariant operands, `story:text-functions-compute-values`),
   and the one-member rule weakens what the reader sees.
3. Expressible today: no.
4. Fit: grid O row "distinct". The sibling is the entity-level tuple of `story:an-identity-may-be-a-composite-of-fields`
   (`unique: [a, b]`, ADR `a-compound-key-is-a-uniqueness-constraint-first`): one spelling for "a tuple of members", a list of names, so a
   reader learns it once. A single member written `by: x.a` keeps meaning what it means, and its IR bytes. Every target that decides
   `distinct` decides the tuple: the shared evaluator (`ess-primitives`), the Go and TypeScript predicate runners, the generated Rust
   and Go struct decoders (`rust/invariant.rs`, `go/invariant.rs`, story `generated-rust-and-go-decoders-check-a-structs-distinct-invariant`),
   synthesis witnesses and mutation. `uniqueItems` cannot publish it: as for an open single key, it stays in `x-ess-invariants`
   (`predicates.md`, "An open key ... stay in `x-ess-invariants` only").
5. Second adopter: a manifest listing files unique on (directory, name); an order whose lines are unique on (product, warehouse).
6. Cost: a new form of an existing predicate, so the format that holds it (`ess/38`, D77); canonical form writes one `kind` per key; the
   shared evaluator, two predicate runners, two generated decoders, the witness (`witness.rs` duplicate and distinct ladders, 3466-3508),
   one diff classification of an existing kind (a changed key is a changed invariant). No new diagnostic code.
7. Alternatives: change nothing and state the weaker one-member rule; a struct-typed key (refused: a struct key is `type_mismatch`
   by design, and every consumer would need a struct equality); a `unique:` on the list type (a third spelling).

## Decisions

Accept, redesigned (the requester proposed no syntax). FM-D3, spelling of a composite `distinct` key. Decided: A, 2026-11-24 16:36 CET (the coordinator, under the owner's delegation), taken on the default: a list of fact paths, `by: [piece.kind, piece.name]`; one member written as before keeps its bytes. B (a `key:` list beside `by:`, two spellings) and C (a struct-typed key, refused by design) not taken.

## Acceptance

Format (D77, 2026-11-24): this construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; a reproduction model keeps the format it declares.

- the_tuple_key_validates_at_the_open_format: the reproduction with `by: [piece.kind, piece.name]` validates at `ess/38`; below it the
  list is refused as `unsupported_format_version` naming `ess/38`, where today the refusal is the parse error above.
- members_are_refused_by_name: a member that is no member of the element, one listed twice, one whose type is no admitted key type
  (the single-key list of `predicates.md`), and the whole element as a member are each refused by name; an empty list is refused.
- single_key_keeps_its_bytes: every model with `by: x.a` keeps its IR bytes and digest, and a suite carrying it keeps its bytes.
- tuple_semantics: two elements equal on every member make the invariant false; elements differing on any one member make it true
  (the first member only, then the second member only); a tuple with an absent or invalid member is an unknown key, so the
  invariant is unknown, as for a single key.
- synthesis_witnesses_each_member: `ess verify conform synthesize` sends a list repeating the whole tuple (refused), one differing in
  the first member only and one in the second only (accepted), and `ess verify conform mutate` kills a mutant that compares only
  the first member.
- decoders_decide_it: the generated Rust and Go decoders refuse the list repeating the whole tuple, naming the invariant, and accept
  the two that differ in one member.
- schema_publishes_it_as_an_annotation: `x-ess-invariants` carries it and no `uniqueItems` is written for a tuple key.

## Scope

- cited: `crates/specify/ess-primitives/src/predicate.rs` (`Distinct`, parser 3380-3405)
- inferred: `crates/specify/ess-domain/src/expression.rs` (key typing)
- inferred: `crates/verify/ess-conformance/src/witness.rs` (3466-3508), `mutate.rs`, `go/predicate.go`, `ts/predicate.ts`
- inferred: `crates/generate/ess-synth/src/rust/invariant.rs`, `go/invariant.rs`
- inferred: `crates/generate/ess-gen/src/types.rs` (the invariant annotation), `crates/verify/ess-diff/src/`
- inferred: `website/docs/reference/predicates.md`, `docs/design/language-grids.md`
