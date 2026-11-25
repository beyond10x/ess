---
format: aep.planning-md/3
id: story:a-ui-test-journey-answers-the-rows-it-declares-as-written-chosen-by-by-params-only
kind: story
status: draft
title: A journey of ess ui test answers the rows a test declares as written, chosen by by_params only, on the terminal target as on the Playwright target
summary: The terminal target narrows a test's rows by every same-name param, so a child row with its own id is dropped; JF-D1 option (a), D120
tags:
- defect
- ess-ui
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 2
---
## Summary

A journey of `ess ui test` states the rows a backend really answers. A list read that takes
`params: {id: <parent id>}` is answered with child rows, and each child row carries an `id` of its
own. On the terminal target the journey runner drops every such row, because it narrows the read by
every param that a row also has as a field of the same name. The Playwright target answers the same
rows as written. Both targets of one journey must answer the same rows.

## Acceptance

- A journey test in an existing `ess-ui-test` target declares `fixtures.views` rows whose `id`
  differs from the `id` param of the read, picks them with `by_params: {id: <parent>}`, and asserts
  the rows drawn. It is red today on the terminal target (every row dropped) and green after the
  change.
- A parity case in the `parity` target holds the terminal target and the Playwright target to the
  same rows for that journey.
- A document's own fixtures keep the same-name narrowing: the existing test
  `a_single_value_param_narrows_the_field_of_its_name` in `crates/ui/ess-ui-tui/src/data.rs` still
  passes unchanged, and so do the generated React fixture adapter's cases for `matchesParams`.
- Unchanged: a read is answered by the first answer whose `by_params` all equal the params asked,
  else the answer without `by_params`, else no rows
  (`website/docs/reference/ess-ui-test.md:318`, `:341`).

## Fit review

Written by reading, not reproduced: the build node was down. The reproduction is the first
acceptance line.

1. **The need.** A journey states the rows a backend answers, including a child row with its own
   `id`, and the replay keeps them. Minimal example, generic: a page with param `id` whose list
   reads a view `Lines` with `params: {id: <page id>}`; the backend answers rows `{id: l1, order: o1}`
   and `{id: l2, order: o1}`; the test declares those rows under `by_params: {id: o1}`. The terminal
   target draws none of them. The reporter's two proposals are theirs, and are evidence of the need:
   "filter a read only by the params its view's filter compares", and "a fixture row declares which
   params it answers to".
2. **Class.** Defect: the terminal target contradicts the reference page and the Playwright target
   (`website/docs/reference/ess-ui-test.md:318`, `:341` name `by_params` and no other narrowing;
   `crates/ui/ess-ui-test/src/playwright.rs:362-425` answers test rows as written). A generated
   React runtime in bound mode applies only the read's own filter (`data.ts.tmpl:909-913`), which
   is the reporter's "React does not filter".
3. **Can it be expressed today?** No. A test row cannot avoid the narrowing: test fixtures enter
   the terminal's fixture adapter through `replace_view`
   (`crates/ui/ess-ui-test/src/runner.rs:62-67`), and every read is then narrowed by each param
   that some row carries as a field of the same name (`crates/ui/ess-ui-tui/src/data.rs:533-569`).
   Renaming the child's `id` field changes the row the journey states.
4. **Fit.** The narrowing is intended and tested for a document's own fixtures, and the generated
   React fixture mode narrows the same way (`data.ts.tmpl:238-267`, applied at `:276`), so for a
   document's own fixtures the renderers agree. The change touches only rows a test itself declares.
   The proposal "filter only by the params the view's filter compares" needs a view filter the
   runner does not read: without `--model` it has none, with `--model` it reads enums and fixture
   effects only (`crates/ui/ess-ui-test/src/lib.rs:189-217`, `ess_ui::binding::ModelEnums`). The
   proposal "a fixture row declares which params it answers to" is a new key in the test fixture
   shape and in `FixtureIndex` (`schemas/ui/ess-ui.schema.yaml:1391-1393`, "by_params records which
   params the rows answer"), which already carries that fact as `by_params`.
5. **A second adopter.** Any journey over a master and detail pair whose detail rows are keyed by
   their own `id`: order lines, ticket replies, invoice items.
6. **Cost.** No new key, no format version, no diagnostic, no migration. Terminal target only: the
   same-name narrowing is skipped for a view the test replaced.
7. **Alternatives.** Four, below as JF-D1. Change nothing leaves the two targets disagreeing and
   every journey with child rows unable to state them.

Decision of the fit review: accept, redesigned (the reporter's two proposals are not taken; the
reference's own rule is made true on the terminal target).

## Decisions

- JF-D1: which rows answer a journey read whose param shares a name with a row field.
  - (a) Rows a test itself declares (`fixtures.views` of a ui test) answer as written, chosen by
    `by_params` only, as the reference and the Playwright routes already do; the terminal target
    skips the same-name narrowing for a view the test replaced. A document's own fixtures keep the
    narrowing both fixture adapters share. No model, no new key.
  - (b) With `--model`, both fixture adapters narrow only by the params the view's filter compares,
    on the field it compares them with (a new `ModelEnums` method; the generated React fixture
    adapter emits that list). Without a model the same-name narrowing stays, so a model-less
    journey keeps the loss.
  - (c) The reporter's second idea: a fixture answer declares which params its rows answer to, and
    only those narrow. A new key in the test fixture shape and in `FixtureIndex`.
  - (d) Drop the same-name narrowing in both fixture adapters everywhere (terminal and React
    `matchesParams`), leaving `by_params` and `q`/`search` as the only selectors. Moves what every
    document fixture answers and reverses the tested rule.
- JF-D1: (a), taken by the coordinator on 2026-11-25 (D120): rows a ui test itself declares answer
  as written, chosen by `by_params` only; the terminal target skips the same-name narrowing for a
  view the test replaced; a document's own fixtures keep it. Not (b) model-driven narrowing, not
  (c) a new key, not (d) dropping the narrowing everywhere.

## Delivery

Built as option (a) of JF-D1 in commit `ba4ba249ad` on the branch `issues/part-2-2026-11-24` (D120), merged with `main` 0.99.2 as `f93dcf8514`; pull request 830 is open. Rows a test declares (`fixtures.views`) answer as written, chosen by `by_params` only, on the terminal target as on the Playwright routes; a document's own fixtures keep the same-name narrowing. Test: ess-ui-test `fixture_answers` `declared_rows_with_their_own_id_answer_a_read_by_the_parents_id`, red before (0 rows where 2 are declared), green after.

## Not built

- The reporter's key idea, "a fixture row declares which params it answers to" (JF-D1 (c)): a new
  key in the test fixture shape and in `FixtureIndex` for a fact `by_params` already states, with
  every target obliged to read, diff and document it for as long as the format lives.
- The reporter's first idea, "filter only by the params the view's filter compares" (JF-D1 (b)): it
  needs a view filter the journey runner does not read, and leaves a journey without `--model`
  with the reported loss.
