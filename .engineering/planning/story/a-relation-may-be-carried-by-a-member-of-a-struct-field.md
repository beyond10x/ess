---
format: aep.planning-md/3
id: story:a-relation-may-be-carried-by-a-member-of-a-struct-field
kind: story
status: draft
title: A relation may be carried by a member of a struct field
summary: 'via: is one field name, so a relation cannot be carried by a member of a struct field (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
- informed_by: story:via-and-instance-may-name-a-member-of-a-struct-input
revision: 2
---
## Need

An entity stores a structured value whose member is a reference to another entity: the record's own shape is fixed by a published
document, and the link sits inside it. The relation cannot name that member. `via:` is one field name
(`crates/specify/ess-domain/src/entity.rs:955-963`, `pub via: String`, read with `deserialize_field_name`), so a dotted path is refused
before the relation is looked at, and the model either states no relation (a reader out of `x-ess-relation` finds none, nothing
refuses a dangling reference, synthesis arranges no referenced row) or copies the reference into a second stored field.

`docs/design/ess-entity-relations-design-v0.1.md:297` puts "relations to value types" out of scope (`via` resolves against entities only;
a struct is not an identity). That sentence is about the target of a relation. The carrier is not the target, and the same design
asks that "the property carrying the relation says so" (§ 5, the `x-ess-relation` key on the carrying property); a member's property
can carry it.

## Reproduction

Brand-free, `format: ess/37`, domain `demo.run`: `Project`, `Evidence {project: ProjectId, note: String}`, `Run {evidence: Evidence}`.
Installed 0.98.0:

| relation on `Run` | result |
|---|---|
| `via: evidence.project` | `system.yaml: invalid field name identifier "evidence.project": contains '.'; a field name has to survive into generated code as an identifier` |
| `via: project`, with `project: ProjectId` stored beside `evidence` and `invariants: [project == evidence.project]` (the idiom) | valid |

The idiom works and keeps the check, at the price of a second stored field in the record's schema and the wire shape of the entity.

## Findings covered

- Gap 4 of the thirteenth report: a relation's `via:` cannot name a member of a struct field.

## Fit review

1. Need: a relation whose carrier is a member of a struct field of the source (`references`) or of the target (`owns`). Domain fact: the
   place a reference lives follows the wire shape of a stored value.
2. Class: gap against the wire shape, convenience against the model: the idiom above is valid and checked, but adds a field the
   published record does not have. The store already accepted the same argument for inputs: "flattening the input changes a wire shape a
   published protocol fixes" (`story:via-and-instance-may-name-a-member-of-a-struct-input`, D48), which opens a struct input's member path
   at `ess/38`.
3. Expressible today: yes by the idiom, with the price named; no without it. FM-D4 puts both to the owner.
4. Fit: grid R, "what can be read where": the member path grammar of `ess/38` (`input_path.rs`, `address`, `reads_member`) names a member
   through struct members and refuses a step into a list, a map, a union or an `Optional` struct. The same grammar names a carrier
   (`via: evidence.project`). The relation then reads like a top-level carrier everywhere: the `references`/`owns` type rules (`entity.rs`
   1560-1596), the one-relation-per-carrier rule (`duplicate_declaration`), the projection key on the carrying property
   (`ess-gen/src/types.rs:374, 827`), navigation by the relation's name in views, values and guards (`story:row-queries-navigate-a-reference`),
   the enforced-reference derivation. A `when_related: {via: <stored field>}` over the same carrier is the stored-field half of the
   input stories and borders this one (no new construct there: it names the same path).
5. Second adopter: an order record whose address block holds the customer id; an event envelope entity whose `source` struct names
   the producing component.
6. Cost: the format that holds the member-address family (`ess/38`, D77); the IR carries the carrier as the dotted path, absent for
   a top-level carrier so existing models keep their bytes; one projection key on a nested property; the diff reports a moved carrier
   as it does a moved top-level one; synthesis arranges the referenced row through the nested field (`reference_integrity`); generated
   Rust and Go entity types document the member as they document a field. No new diagnostic code (`undeclared_reference`,
   `type_mismatch`, `duplicate_declaration`).
7. Alternatives: change nothing (the idiom); a relation declared on the struct type (refused: a struct type is reached from several
   entities and from inputs, so the relation would not know its source).

## Decisions

Accept, redesigned (the requester wrote only the path). FM-D4, a relation carried by a struct member. Decided: A, 2026-11-24 16:36 CET (the coordinator, under the owner's delegation), taken on the default: `via:` takes the `ess/38` member path (`evidence.project`) through struct members. B (no construct, the idiom stays) and C (the relation declared on the struct type) not taken.

## Acceptance

Format (D77, 2026-11-24): this construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; a reproduction model keeps the format it declares.

- the_nested_carrier_validates_at_the_open_format: the reproduction with `via: evidence.project` validates at `ess/38`; below it the
  dotted path is refused as `unsupported_format_version` naming `ess/38`.
- nested_carrier_types_are_checked_like_a_field: a member typed as something other than the target's identity (wrapped in `Optional`
  or `List` as for a field) is refused as `type_mismatch`; an undeclared member as `undeclared_reference`, naming the struct's members;
  a path through a list, a map, a union or an `Optional` struct as `type_mismatch` naming the type; two relations on one member as
  `duplicate_declaration`.
- top_level_carriers_keep_their_bytes: every model with a top-level `via:` keeps its IR bytes and digest.
- the_projection_marks_the_member: the JSON Schema of `Run` writes `x-ess-relation` on the `project` property inside `evidence`, and
  on no top-level property.
- synthesis_arranges_the_referenced_row: the suite for a command that sets `evidence` arranges a `Project` row and sends its
  identity at `evidence.project`, and a scenario whose member names no row is refused where a top-level reference would be.
- navigation_follows_the_nested_carrier: a view field or guard reading `project.title` through the relation is decided on the row
  `evidence.project` names.
- diff_reports_a_moved_carrier: `ess verify diff` reports a relation whose `via` moved from `project` to `evidence.project` in the
  kind it uses for a moved top-level carrier.

## Scope

- cited: `crates/specify/ess-domain/src/entity.rs` (955-963, 1560-1596), `command/input_path.rs` (the shared path helpers)
- inferred: `crates/specify/ess-compiler/src/ir.rs`, `resolve.rs`; `crates/generate/ess-gen/src/types.rs` (374, 827)
- inferred: `crates/generate/ess-synth/src/rust/entity.rs` (80-90) and the Go counterpart
- inferred: `crates/verify/ess-conformance/src/synthesize/reference_integrity.rs`, `crates/verify/ess-diff/src/`
- inferred: `docs/design/ess-entity-relations-design-v0.1.md` (§ 6 names "relations to value types" as the target only)
- Depends on: `story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a`
  (the shared path helpers).
