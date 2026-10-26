---
format: aep.planning-md/3
id: story:a-view-field-from-names-the-related-spelling
kind: story
status: implemented
title: 'A view field from: names the related spelling'
summary: 'Adopter report N22: from: on a view field gives no hint (ess#675).'
refs:
- provider: github
  reference: beyond10x/ess#675
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:57Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:58Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T17:50:13Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

A view field written `from: book.cover` is refused with ``unknown field `from` ``, and the adopter concluded joins are unsupported. Joins exist since ess/28 (`field: {related: {via, field}}`) and row-computed values since ess/27 (`expr:`). Adopter report N22 (ess#675), ess 0.68.0.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/row_queries_navigate.rs`:
  `a_view_field_from_names_the_related_spelling` (the reproduction's `from: book.cover` is
  `unsupported_construct` at `view.<view>.fields[1].from`, hint naming
  `field: {related: {via: book_id, field: cover}}`),
  `a_view_field_from_that_names_no_reference_names_both_spellings` (hint names `field: {related:
  …}` and `expr:`), and
  `a_view_field_from_below_ess_28_names_ess_28`.
- `crates/edge/ess-cli/tests/validate_parse_refusals.rs`:
  `a_view_field_from_is_refused_at_its_line_beside_other_refusals` (code, line, and a second
  refusal in the same file are reported) and `from_outside_a_view_field_keeps_its_refusal` (a
  `from:` on an entity field still reads ``unknown field `from` ``).

## Fit review

Reviewed 2026-10-26 against main `a3bc2df3d` (0.69.2). Reproduction: `fit/repro/n22-view-from.yaml`
(a shelf view reading its book's cover as `{name: cover, type: Optional<String>, from: book.cover}`),
refused on 0.69.2 as ``n22-view-from.yaml: unknown field `from` `` (whole file, no line, no code).
The idiom, `fit/repro/n22-idiom.yaml` (`field: {related: {via: book_id, field: cover}}`), is
`valid` on 0.69.2.

1. **Need.** A view field reads a field of the row its source references. That is expressible;
   the need is that the refusal of a guessed spelling leads to it. Requester's syntax:
   `from: book.cover` (issue N22).
2. **Class.** Convenience (the join exists from `ess/28`, `RawViewField.field`,
   `crates/specify/ess-domain/src/view.rs:793-796`; a row-computed value from `ess/27`, `expr:`,
   `:787-792`). The diagnostic half is a defect of discoverability: the refusal stops the whole
   file and names nothing (`RawViewField` is `deny_unknown_fields`, `view.rs:772-774`).
3. **Already expressible?** Yes, and the idiom keeps the figure in the specification and the
   suite: the related value is a view field the suite reads (grid R row "view field", "part: one
   hop", `docs/design/language-grids.md:32`). `via:` names the reference field, not the relation:
   `via: book` is refused as ESS-VIEW-001 (`fit/repro/n22-idiom-relname.yaml`). `expr:` reads the
   source row only (`expr: book_id` over a `Uuid` newtype is ESS-VIEW-002, `fit/repro/n22-expr.yaml`).
4. **Fit.** No grid cell opens. The refusal joins the parse-time "take the key, refuse it at its
   line" pattern already used for `undeclared_fields:` and `shorthand:`
   (`crates/specify/ess-domain/src/spec.rs:191-195`, `undeclared_fields.rs:22-33`). `from:` stays
   an unknown field everywhere else (transitions, bindings), where it means something else or
   nothing.
5. **Second adopter.** An order view showing the customer's display name next to each order:
   the same guess, the same idiom.
6. **Cost.** No format, no keyword, no IR change; no new diagnostic class (the family number of
   `unsupported_construct` in `ESS-VIEW`, `website/docs/reference/diagnostics.md:21`). A refusal
   moves from a file-stopping parse error to a located one, so the file's other refusals now show
   beside it. Refusal-only: not counted under policy 3.
7. **Alternatives.** (a) Change nothing: the adopter concluded joins are unsupported. (b) Admit
   `from:` as a synonym: a second spelling of one idea, rejected. (c) Rewrite serde's
   `unknown field` text after the fact: no line, no source entity, rejected. (d) Take `from:` out
   of view fields at parse and refuse it at its line with the idiom (chosen).

## Decisions

- **Decline, with the idiom.** `from:` is not admitted. The idiom is
  `field: {related: {via: <reference field>, field: <field>}}` (`ess/28`); `expr:` for a value
  computed from the source row's own fields (`ess/27`). This story ships the idiom in the refusal;
  the idiom's `specification` artifact (skill table, "decline, with the idiom") is
  `specification:a-view-field-reads-a-referenced-row-through-field-related`, with
  `fit/repro/n22-idiom.yaml`'s output as its evidence.
- `from:` on a view field is refused as `unsupported_construct` at its line. Where it is
  `<x>.<field>` and `<x>` is a relation or a reference field of the view's source, the hint is the
  rewrite with the reference field resolved (`from: book.cover` -> `field: {related: {via: book_id,
  field: cover}}`); otherwise the hint names both spellings. Below `ess/28` the hint names `ess/28`.
- Every other position keeps ``unknown field `from` ``.

## Scope

| unit | files | confidence |
|---|---|---|
| take and refuse | `crates/specify/ess-domain/src/spec.rs` (`RawSpecFile::parse` `:188-212`, beside `undeclared_fields::Written::take` `:193`); a new `from:` taker modelled on `crates/specify/ess-domain/src/undeclared_fields.rs` (`Written::take` `:131`) or `shorthand.rs` (`:60`) | cited |
| hint with the reference resolved | `crates/specify/ess-domain/src/view.rs` (`RawViewField` `:772-797`, view validation) | inferred |
| docs | `website/docs/reference/diagnostics.md` (generated by `cargo xtask diagnostics` if the catalogue text moves) | inferred |

No format gate (refusal-only). Crates: ess-domain, ess-cli (tests only). Size S. Shares
`spec.rs::parse` with any other story adding a parse-time taker; no file shared with
`story:a-struct-that-ignores-undeclared-fields-may-declare-none` (`types.rs`) or the lifecycle story.

confidence: high
