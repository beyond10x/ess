---
format: aep.planning-md/3
id: story:a-row-filter-tests-membership-in-a-page-state-list
kind: story
status: draft
title: A row filter tests membership in a list held in page state
summary: '`row.id in state.picked` is refused; `in` takes literal lists only (workbench report item 3)'
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 5
---
## Need

A collection cannot filter its rows by a multi-select held in page state. Reproduced on 0.92.0:

- `row.id in state.picked` is refused: `filter_expr … a filter must parse as a comparison,
  membership, not, and or or expression`. The parser accepts `in` only before a literal list
  (`crates/ui/ess-ui/src/filter.rs`, the comparison parser, and `filterForm` in
  `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl`). The evaluator already tests membership
  in any list value (`Operator::In` in `filter.rs`).
- `state.picked contains row.id` passes the check. `contains` reads its left side as text, joining
  a list with `,` (`filter.rs`, `text` and `contains`; documented in
  `website/docs/reference/ess-ui.md` under `Reads.filter`), so it matches substrings. With
  `picked: [ab]` it keeps rows `a` and `ab`.

Minimal reproduction. `FILTER` is replaced per variant. `fixtures/items.yaml` holds rows `a`,
`ab` and `c`:

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {dir: fixtures, views: {probe.Items: items.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: items.list
  sections: [{name: all, pages: [items.list]}]
pages:
  items.list:
    kind: list_page
    title: Items
    state:
      picked: {type: {list: string}, class: page_state, store: url, default: []}
    sections:
      - name: list
        component: collection
        reads: {view: probe.Items, paging: none, filter: FILTER}
        columns: [title]
```

The test opens `{page: items.list, state: {picked: [ab]}}` and expects
`{at: pages/items.list/sections/list, rows: [ab]}`.

```text
filter: row.id in state.picked        -> error filter_expr …/reads/filter: a filter must parse as a comparison, membership, not, and or or expression
filter: state.picked contains row.id  -> check: 0 error(s); test: expected rows [ab]; it shows [a, ab]
filter: "row.id in ['ab']"            -> test passed
```

Item 3 of the 2026-11-09 workbench report; examples `ui03`, `ui03b`.

## Acceptance

- `ess ui check` accepts `row.id in state.picked` where `picked` is a page state of type
  `{list: string}`. Over the document above with `picked: [ab]`, `ess ui test` shows rows `[ab]`,
  and the generated React app keeps the same rows (a test of the React filter evaluator over the
  same input).
- `ess ui check` refuses `row.<field> in state.<s>` as `filter_expr`, naming both types, when
  `<s>` is not a list or its element type differs from the field's type (with `--model`).
- `<list> contains <value>` with a page-state list on the left is either the same membership test
  (`picked: [ab]` keeps `[ab]` only) or refused by `ess ui check`, naming `contains` as a text
  operator. It is never accepted with the text reading.

## Scope

- cited: `crates/ui/ess-ui/src/filter.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl`
- cited: `website/docs/reference/ess-ui.md`
- inferred: `crates/ui/ess-ui-check/src/rules.rs` (the `filter_expr` rule and typing)
- inferred: `crates/ui/ess-ui-tui/src/app.rs` (filter call sites)
- inferred: `schemas/ui/ess-ui.schema.yaml` (the `Reads.filter` note)

confidence: medium

## Fit review

1. Need: keep the rows whose key is among the values a user picked into a page-state list. The
   requester proposed `row.id in state.picked`; that is their syntax.
2. Class: gap. The filter grammar has no membership test against a non-literal list. The accepted
   `contains` spelling is a misleading acceptance.
3. Already expressible: no. `contains` reads the list as text and matches substrings
   (reproduction above), so it is no idiom.
4. Fit: `in` already exists with the same meaning over literal lists, and the evaluator already
   handles any list value. Widening its right side to a state path reuses that vocabulary in both
   renderers. The same grammar serves `checked` and `visible` expressions, which the decision
   should cover too.
5. Second adopter: an issue tracker filtering tickets by a set of chosen labels held in the URL.
6. Cost: no format version (the expression grammar widens within `ess-ui/1`). A new check finding,
   or a `contains` refusal that rejects documents loading today.
7. Alternatives: change nothing (the substring trap stays); widen `in`; redefine `contains` over a
   list as membership. Widening `in` reuses the operator that already means membership.

## Decisions

Pending: accept, redesigned is the likely outcome (widen `in`, refuse `contains` over a list).
The fit review above is the input; record the decision here before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
