---
format: aep.planning-md/3
id: story:a-row-filter-tests-membership-in-a-page-state-list
kind: story
status: active
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
- confidence: cited
  path: crates/ui/ess-ui-check/src/filter.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/filter_membership.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/filter_membership.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/filter_membership.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:28Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:29Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
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

All bullets run on the document above, with `picked: [ab]`.

- `ess ui check` over the document with `FILTER = row.id in state.picked` reports
  `0 error(s)`; tested in `cargo test -p ess-ui-check --locked --test filter_membership`.
- `ess ui test` opening `{page: items.list, state: {picked: [ab]}}` and expecting
  `{at: pages/items.list/sections/list, rows: [ab]}` passes; tested in
  `cargo test -p ess-ui-test --locked --test filter_membership`.
- The generated React project, compiled with `tsc` and run under `node` against the stand-in
  `react` (as `crates/ui/ess-ui-react/tests/choice_filter_form.rs` does), keeps rows `[ab]` for the
  same filter and state; tested in `cargo test -p ess-ui-react --locked --test filter_membership`.
- `ess ui check --model` refuses `row.id in state.picked` with a `filter_expr` error naming both
  types when `picked` is `{type: string}` (not a list), and again when it is `{list: integer}`
  while `row.id` is a string; tested in the `ess-ui-check` `filter_membership` target.
- `ess ui check` refuses `state.picked contains row.id` with a `filter_expr` error that names
  `contains` as a text operator and points to `in`; tested in the `ess-ui-check`
  `filter_membership` target.

## Scope

This story moves only the `CHANGELOG.md` shared surface.

- cited: crates/ui/ess-ui/src/filter.rs
- cited: crates/ui/ess-ui-check/src/filter.rs
- cited: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- cited: website/docs/reference/ess-ui.md
- inferred: crates/ui/ess-ui-check/src/rules.rs
- inferred: crates/ui/ess-ui-tui/src/app.rs
- inferred: schemas/ui/ess-ui.schema.yaml
- inferred: crates/ui/ess-ui-check/tests/filter_membership.rs
- inferred: crates/ui/ess-ui-test/tests/filter_membership.rs
- inferred: crates/ui/ess-ui-react/tests/filter_membership.rs

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

Accept, redesigned:

- Widen `in` so its right side may be a state path holding a list, in `ess_ui::filter::parse` and
  in `filterForm`; because `checked` (`crates/ui/ess-ui-check/src/rules.rs`) and the terminal's
  filters (`crates/ui/ess-ui-tui/src/app.rs`) read the same parser, they take the widening with it.
- Refuse `contains` with a list on its left instead of redefining it as membership: `contains`
  keeps one meaning, a text search, everywhere in the grammar, and a document using it over a list
  today gets substring matches, which is the defect the refusal names.
- No format version: the grammar widens within `ess-ui/1`, and the refused `contains` spelling
  never gave the result its author meant.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
