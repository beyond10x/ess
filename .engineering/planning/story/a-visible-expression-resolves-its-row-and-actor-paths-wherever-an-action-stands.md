---
format: aep.planning-md/3
id: story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands
kind: story
status: active
title: A visible expression resolves its row and actor paths wherever an action stands
summary: ess ui check --model refuses a misspelled row path in visible only on a collection's row_actions; record and header actions and every actor path pass unchecked on 0.94.0 (adopter report)
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:19Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:21Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

`ess ui check --model` resolves a `row.<field>` path inside an action's `visible` expression only
for a collection's `row_actions`. The same misspelled path on a record's action, or on a page
header action, is accepted with no finding, and an `actor.<field>` path outside the reference's
closed list is accepted everywhere. A typo in a visibility condition then hides or shows a button
for every user, and only a test that drives the rendered page finds it.

Reported by an adopter on 0.90.0 (2026-11-11); reproduced on 0.94.0 (main `9f8141cb8b`; the build
used does not differ from it under `crates/ui/ess-ui-check/src`).

Document, checked against the model in `crates/ui/ess-ui-check/tests/fixtures/model` (view
`stock.Items` has the fields `item_id` and `label`):

```yaml
format: ess-ui/1
app: t
model: shop
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages: {p: {kind: detail_page, title: P, sections: [
  {name: summary, component: collection, reads: stock.Items, columns: [label, item_id],
   row_actions: [{name: peek, opens: nowhere_needed, visible: "row.label == x"}]},
  {name: one, component: record, reads: stock.Items, fields: [label],
   actions: [{name: act, opens: nowhere_needed, visible: "row.label == x"}]}]}}
```

One `visible` changed per run, `ess ui check --path <doc> --model <model>`:

| changed `visible` | where | finding about the path |
|---|---|---|
| `row.labelX == x` | collection `row_actions` | error `field_in_model` at `pages/p/sections/summary/row_actions/peek`, naming `row.labelX` and the fields `item_id`, `label` |
| `row.labelX == x` | record `actions` | none |
| `row.labelX == x` | page `header.actions` | none |
| `row.label == actor.user_idx` | collection `row_actions` | none |
| `row.label == actor.user_idx` | record `actions` | none |

Cause, read from the source: `composite_fields` in `crates/ui/ess-ui-check/src/names.rs` calls
`Known::row_paths` for `Composite::Collection` only (`row_actions`); the `Composite::Record` arm
checks `fields` and not `actions`. No check reads an `actor.` path. The reference
(`website/docs/reference/ess-ui.md`, expression roots) gives `row.<field>` as "the current row of a
collection, record or board" and `actor.<field>` as the closed list `user_id`, `account_id`,
`roles`, `signed_in`, `may(<command>)`.

## Second point, smaller

A page header action has no row by that sentence of the reference, so a condition on the page's
own record (for example: its owner is the signed-in actor) cannot be stated on a header action; the
adopter moved the actions into the record composite. The check does not say so: `row.<field>` in a
header action's `visible` is accepted (third row of the table). The reference does not say whether
this is intended.

## Acceptance

1. With `--model`, a `row.<field>` path in the `visible` of a record's action that names no field
   of the view the record reads is an error `field_in_model` at the action's node path, with the
   same message form as for `row_actions`.
2. The same holds for every other composite the reference gives a row to (board), for each action
   list of that composite that has a row.
3. An `actor.<name>` path in any `visible` whose `<name>` is outside the reference's list is an
   error naming the path and the list; it needs no `--model`.
4. A `row.` path in the `visible` of an action that has no row (a page header action, a
   collection's `actions` and `bulk_actions` if they have none) is an error saying the action has
   no row and naming where one is available.
5. The reference states, at the header's `actions`, that a header action has no row and how a page
   states a condition on its own record, or the story's fit review decides that a header action
   reads the page's record and the reference and both renderers say so.
6. `examples/partner-portal/ui.yaml` still passes with its one placeholder warning.

## Fit review

Not done. This is a draft from an external report
(`.agents/skills/assessing-external-requests/SKILL.md`): points 1 to 4 are a check that is
missing where the reference already defines the names, so no new syntax is asked for; point 5 is a
design question (whether a header action reads the page's record) and needs the review's seven
questions before it is built. Other expression sites (`when`, `bind`, a section's or a field's
`visible`) were not probed; the review should list which of them resolve `row.` and `actor.` paths.

## Evidence

Reproduction script and its output: session scratch `ess/adopter6/repro.sh`, `repro.txt`
(not in the repository). Existing case for the one checked site:
`crates/ui/ess-ui-check/tests/checks.rs`, test `field_in_model`.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Scope of the acceptance: acceptance points 1 to 4 (the missing check). Point 5, whether a header
action reads the page's record, is a design question and is not accepted with them; the fit
review states it as a question for the operator.
