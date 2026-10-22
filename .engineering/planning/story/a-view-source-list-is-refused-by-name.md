---
format: aep.planning-md/3
id: story:a-view-source-list-is-refused-by-name
kind: story
status: implemented
title: 'A view source: list is refused by name, pointing to navigation, instead of a serde error'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- depends_on: story:navigation-reaches-any-depth-in-view-filters-and-guards
scope:
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:00Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-22T01:35:00Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A view whose `source:` is a list (`source: [ledger.ImpactWindow, ledger.Incident]`) is refused by
name, at the `source` key, with a message that a view has one source and a hint naming navigation
from the source row (`field: {related: {via, field}}`, a filter path `<reference>.<field>`) as the
way to read another entity. Today the document fails to load with the serde error "invalid type:
sequence, expected a string" and no code (https://github.com/beyond10x/ess/issues/564, "Refusals";
reproduced on the integration build e44d2b75c, sizing pass 2, 2026-10-21).

The binding design decided no to the list: "A view's `source:` stays one entity
(`dom/view.rs:919-923`); a view reads other entities only by navigation from its source row, never
by a join on arbitrary keys. That keeps the view's row count the source's, which aggregate scoping
depends on" (`docs/design/generic-language-features.md` lines 287-289). This story makes that
decision visible to the author instead of a parser error.

## Fit review

1. **Need.** The requester's need (read another entity's current fields in a view) is met by
   navigation (`story:row-queries-navigate-a-reference`, `ess/28`, and
   `story:navigation-reaches-any-depth-in-view-filters-and-guards`). What is left is the refusal's
   wording.
2. **Class.** Defect in the refusal: an unnamed serde error for a decided-against construct.
3. **Already expressible?** Yes, by navigation; the refusal should say so.
4. **Fit.** Declined as a construct (design lines 287-289); adopted as a named refusal.
5. **Second adopter.** Any author coming from SQL joins.
6. **Cost.** No format version: the document was already invalid at every format and stays
   invalid; only the diagnostic changes. A changed or new diagnostic moves pins outside the
   crate (`task test-xtask`, `cargo test -p ess-compiler --test typed_diagnostics`, AGENTS.md).
7. **Alternatives.** (a) Leave the serde error. (b) Admit a join: declined by the design. (c) A
   named refusal with the navigation hint (chosen).

## Decisions

- Declined as a construct, per design lines 287-289; the refusal reuses an existing
  `ValidationCode` (`unsupported_construct`), no new code.
- Every format: the list is refused at every format, as it is today; only the message changes,
  so no format gate.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/row_queries_navigate.rs`:
  `a_view_source_list_is_refused_by_name_at_every_format`: the #564 shape fails with a coded
  error located at `views.<view>.source`, its hint naming `{related: {via, field}}`, and no
  "invalid type: sequence" text.
- `crates/specify/ess-domain/tests/row_queries_navigate.rs`:
  `a_view_source_string_is_unchanged`: a one-entity source loads with today's bytes.

## Scope

| unit | files | confidence |
|---|---|---|
| raw model and refusal | `crates/specify/ess-domain/src/view.rs` (`RawView.source`, the conversion at `:2097` and `:919-923`) | inferred from design citation |
| schema | `schemas/generated/ess.schema.json` only if `RawSpecFile` changes (`cargo xtask schema`, `task projection-check`) | inferred |
| pins | `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json`, `docs/design/review-typed-diagnostics.md` if a diagnostic is added | inferred |

Crates: ess-domain (sizing: S).

## Order

Lane L4b, last: after `story:navigation-reaches-any-depth-in-view-filters-and-guards`, whose
hint it names.
