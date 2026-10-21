---
format: aep.planning-md/3
id: story:aggregate-group-keys-are-expressions
kind: story
status: active
title: A group key is an expression, with one group per List element
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#565
relations:
- serves: vision:O2
- decomposes: epic:generic-aggregates
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:02Z", actor: "human:timo", revision: 3}
---
## Outcome

An aggregate view's group key is an expression; a `List` field as a key yields one group per
element, so a row counts once in each group its list names.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 3, group keys.
Parent: `epic:generic-aggregates`.

## Fit review

Finding covered: https://github.com/beyond10x/ess/issues/565 (a downstream incident-ledger
specification, ess 0.59.0, `ess/24`).

1. **Need.** Downtime by service, where a window names the services it affects as a list. Refusal:
   `ESS-VIEW-002 "… groups by service_ids of type List<…>, which has no value equality a group could
   be keyed on"` (`crates/specify/ess-domain/src/view.rs:1654`). Requester's syntax (theirs):
   `group_by: [service_ids]`.
2. **Class.** Gap.
3. **Already expressible?** No: one owned row per (window, service) would state it, at the cost of a
   second entity the domain does not have.
4. **Fit.** The refusal is right for grouping by the whole list; per-element grouping is a distinct
   operation and must be spelled so (`each(service_ids)` or the binding design's form), not inferred
   from the type. Composes with `story:row-queries-quantify-over-a-list-of-references` (the same list
   of references) and with list-typed view parameters (`story:feature-request-438`). Targets:
   validation, interpreter, synthesis (a row with two elements counted in both groups), generated
   view queries, diff.
5. **Second adopter.** Articles counted per tag; tickets per affected component.
6. **Cost.** A source format version; one diff class.
7. **Alternatives.** (a) Change nothing. (b) Infer per-element grouping from a list type: hides a
   different count. (c) An explicit per-element key expression (chosen).

## Decisions

Accept, redesigned: per-element grouping is spelled explicitly.

## Acceptance

- `group_per_list_element_counts_a_row_in_each_group`: a window naming two services counts once
  under each.
- `group_by_whole_list_stays_refused`: today's refusal holds for a bare list key.
