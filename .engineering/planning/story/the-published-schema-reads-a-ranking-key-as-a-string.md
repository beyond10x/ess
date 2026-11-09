---
format: aep.planning-md/3
id: story:the-published-schema-reads-a-ranking-key-as-a-string
kind: story
status: draft
title: The published schema reads a ranking key as the string the parser reads
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/published_name_aliases.rs
- confidence: cited
  path: schemas/generated/ess.schema.json
revision: 5
---
## Finding

Found by U4 of wave D350 (out of scope there): the `Ranking` view key carries a struct-level
`#[schemars(with = "String")]` at `crates/specify/ess-domain/src/view.rs:238`, which
schemars_derive 0.8.22 ignores, so the published document schema describes the key as an object
while the parser reads a string. The alias corpus test of D350 names it as one of its two
exceptions (18 documents).

Reproduced on 0.92.0 (`ess/35`), with the schema committed at main:

- A view with `order_by: [n desc, id]` validates with `ess specify validate` (exit 0).
- The same document checked against `schemas/generated/ess.schema.json` (JSON Schema draft 7) has
  2 errors: `'n desc' is not of type 'object'` and `'id' is not of type 'object'` at
  `/views/0/order_by/*` (`definitions/Ranking` is `{"type": "object", "required": ["direction",
  "field"], ...}`).
- Written as the schema describes it, `order_by: [{field: n, direction: desc}, ...]`, the parser
  refuses the document: `invalid type: map, expected a string`.

So no `order_by` document is admitted by both the parser and the published schema, and an editor
or CI step that checks documents against the schema flags every valid ranking.

## Acceptance

- `schemas/generated/ess.schema.json` `definitions/Ranking` is a `string` (with the
  `<field>` / `<field> asc|desc` shape stated in its description or as a `pattern`), and the
  published schema admits every committed document that writes a ranking key as the parser reads
  it.
- The `Ranking` exception (`is_ranking_exception` and its count) is removed from
  `crates/specify/ess-domain/tests/published_name_aliases.rs`, and
  `every_committed_document_the_parser_reads_is_admitted_by_the_published_schema` passes without it.
- A red case first shows a committed ranking document the schema refuses today.
- `cargo xtask schema --check` passes with the regenerated file.

## Scope

- cited: crates/specify/ess-domain/src/view.rs
- cited: schemas/generated/ess.schema.json
- cited: crates/specify/ess-domain/tests/published_name_aliases.rs
- inferred: crates/edge/ess-xtask (the `schema` regeneration, if the fix is a manual `JsonSchema` impl)

confidence: high
