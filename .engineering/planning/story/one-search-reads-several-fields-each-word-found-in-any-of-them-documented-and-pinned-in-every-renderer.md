---
format: aep.planning-md/3
id: story:one-search-reads-several-fields-each-word-found-in-any-of-them-documented-and-pinned-in-every-renderer
kind: story
status: draft
title: One search reads several fields, each word found in any of them, documented and pinned in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A list page wants one search box that finds a row when every word typed appears in some field of
the row: the title, the customer or the city. The reference documents `contains` with one
field on its left (`website/docs/reference/ess-ui.md:2054`). `<a> or <b>` of two `contains` needs
every word in one field. The requester concluded it cannot be said.

By reading, it can, though not as a documented or tested contract:

- The filter grammar parses a bracketed list as an operand (`crates/ui/ess-ui/src/filter.rs:331-345`),
  and `contains` takes any operand on its left (`:313-323`). Only `in` requires a list on its right.
- `contains` reads its left side as text (`filter.rs:155-160`). A list reads as its items' text
  joined with `,` (`filter.rs:140-153`, line 150). So `[row.a, row.b] contains state.q` keeps a row
  when every word of `q` occurs in `"<a>,<b>"`.
- React does the same: a list operand evaluates to an array (`expr.ts.tmpl:330-331`), and
  `containsWords` reads it with `String(…)`, which also joins with `,` (`expr.ts.tmpl:318-326`).
- `ess ui check` admits it: `filter_expr` needs a boolean top-level form, and `filter_paths` checks
  each path, list items included (`crates/ui/ess-ui-check/src/filter.rs:142-188`).

Two faults remain. Nothing documents or pins it, so a renderer could drop it. A word can also match
across two fields: the search `h,n` keeps a row whose fields end in `h` and start with `n`.

The requester's proposal, restated: `contains` over several fields, written
`filter: "[row.title, row.customer, row.city] contains state.search"`.

## Outcome

`[<a>, <b>, …] contains <q>` keeps a row when each whitespace-separated word of `<q>` occurs, in
any case, in the text of at least one item. An item that is a list is read as its items, each
separately. The reference states it beside the one-field form, and a filter bar search bound to
`<q>` is drawn as it is for one field. Both renderers and the checker share the rule.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/read_filter.rs` (which compares the React runtime
  with `ess_ui::filter::keeps`, `read_filter.rs:80`): over rows `{a: build, b: north}` and
  `{a: test, b: south}`, `[row.a, row.b] contains state.q` with `q = "north build"` keeps the
  first row only, and `q = "d,n"` keeps none. The second case fails before the change in both
  `ess_ui::filter` and the React runtime.
- A list field item is read item by item: `{a: [run, fast]}` is kept for `q = "fast"` and dropped
  for `q = "n,f"`.
- The terminal keeps the same rows (`crates/ui/ess-ui-tui/tests/read_filter.rs`).
- `ess ui check --model` reports `filter_paths` for an item naming a field the view lacks.
- The reference's `Reads.filter` note states the list form and the any-item rule.

## Fit review

Tree `bcfc51555`, read-only. Second report, G8.

1. **Need.** One search over a row's several text fields. Reproduction: the filter above, over a
   placeholder fixture with the two rows.
2. **Class.** *Convenience*, by reading: it is expressible today, undocumented. The cross-field
   match is a small defect of the joined-text reading. I could not run `ess ui check` or a renderer
   to confirm, so it is not shown running here. The first acceptance shows it.
3. **Already expressible?** Yes, by reading (lines above). By the skill an idiom is an answer only
   when shown running. This story turns it into a pinned contract rather than a new construct.
4. **Fit.** No grid applies. It adds no word: the list operand and `contains` already exist, and
   the requester's spelling is the grammar's own.
5. **Second adopter.** A contacts list searched by one box over name, email and company.
6. **Cost.** No format version and no key. The meaning changes only for a search word that holds
   the item separator `,`, which today matches across two fields.
7. **Considered.** (a) Document the joined-text reading as it is: a word can span two fields.
   (b) A `search: {over: [fields]}` key on the filter bar: a second spelling of the predicate.
   (c) Chosen: the existing form, each word matched against each item.

## Decisions

**Decline the new construct, with the idiom, and make the idiom a contract.**

- **SL-D3. How is a word matched against a list?** (A) Against each item separately, so a word
  never spans two fields. (B) Against the items' text joined with `,`, today's reading, documented
  as it is. **Recommended: A.**

## Scope

- cited: `crates/ui/ess-ui/src/filter.rs` (`text` 140-153, `contains` 155-160)
- cited: `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl` (`containsWords` 318-326)
- cited: `crates/ui/ess-ui-react/tests/read_filter.rs`, `crates/ui/ess-ui-tui/tests/read_filter.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml` (`Reads.filter` note), `website/docs/reference/ess-ui.md`
- Size: S, about 160 changed lines.

## Depends on

Nothing. Its files are disjoint from the value-format stories, so it can be built beside them.

## Source

A downstream application, 2026-11-24, second report, G8.
