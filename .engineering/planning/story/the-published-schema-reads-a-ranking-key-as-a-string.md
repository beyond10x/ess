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
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/published_name_aliases.rs
revision: 8
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

## Decisions

- `definitions/Ranking` is `{"type": "string"}` with the `<field>` / `<field> asc|desc` shape in
  its description and no `pattern`, because `Ranking::parse` splits on any Unicode whitespace and
  admits `ascending`/`descending` too, and a pattern would be a second grammar that drifts from it.
- The schema is fixed by a hand-written `JsonSchema` impl on `Ranking` in `view.rs`; `cargo xtask
  schema` regenerates the file unchanged in its own code.

## Acceptance

- `cargo xtask schema --check` passes, and in the regenerated `schemas/generated/ess.schema.json`
  `definitions/Ranking` has `"type": "string"`, no `properties` and no `required`.
- `cargo test -p ess-domain --locked --test published_name_aliases
  every_committed_document_the_parser_reads_is_admitted_by_the_published_schema` passes with
  `is_ranking_exception` and its `rankings` count removed from the test. Failing before: with only
  the exception removed, the same test fails listing the `Ranking` refusals (18 documents) at
  `/definitions/Ranking/type`.
- Same target, a new case `a_ranking_key_validates_against_the_published_schema`: a document with
  `order_by: [n desc, id]` has 0 errors against the published schema, where 0.92.0 gives the 2
  above.

## Scope

- cited: `crates/specify/ess-domain/src/view.rs`
- cited: `crates/specify/ess-domain/tests/published_name_aliases.rs`

confidence: high

Regenerates `schemas/generated/ess.schema.json` (coordinator-reconciled shared surface).
