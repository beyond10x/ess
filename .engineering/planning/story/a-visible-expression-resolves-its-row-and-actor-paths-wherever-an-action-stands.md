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
- depends_on: story:ui-check-resolves-the-root-and-section-of-every-binding-path
- depends_on: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
revision: 8
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

Redesigned by the fit review (2026-11-12).

1. With `--model`, a `row.<field>` path that names no field of the view the composite reads is an
   error `field_in_model` at the node path of the expression's owner, with the message form used
   for `row_actions` today, in every expression of a record's actions: `visible`, a navigate's
   `params`, and each other field the document model types as an expression.
2. The same holds for a collection's `row_actions` beyond `visible`, and for every other
   composite the reference gives a row to (board), for each action list that has a row.
3. An `actor.<name>` path whose `<name>` is outside the reference's list is an error naming the
   path and the list, at every expression site; it needs no `--model`. The list is data in the
   schema's `expressions.forms`, and the reference's table is generated from it.
4. A `row.` path in an expression where no row is in scope (a page header action; a collection's
   `actions` and `bulk_actions` if they have none) is an error. It says the action has no row and
   names where one is: the composite's row actions, and `section.<name>.row` for the record a
   section shows.
5. The reference says, at the header's `actions`, that a header action has no row and that the
   record a section shows is read as `section.<name>.row`.
6. `examples/partner-portal/ui.yaml` passes with its one placeholder warning, and every document
   under `examples/` keeps its findings.
7. Tests that fail before the change: rows 2 to 5 of the table above, and `row.nosuch` and
   `actor.nosuch` in a navigate parameter of a record's action.

## Fit review

Source read at main `582524ae95`; probes on the 0.94.0 release binary, 2026-11-12.

1. **Need.** A misspelt field in `row.<field>` or `actor.<field>` inside a condition hides or
   shows a button for every user, and `ess ui check` is silent. The requester proposed no syntax.
   Minimal reproduction: the document and table in the Defect section.
2. **Class.** Defect. The reference says "a validator resolves every path"
   (`website/docs/reference/ess-ui.md:55`), gives `row.<field>` as the current row of a
   collection, record or board (`:62`) and `actor.<field>` as a closed list (`:69`). The check
   resolves a row path only in the `visible` of a collection's `row_actions`
   (`crates/ui/ess-ui-check/src/names.rs:208-213`, `:244`); the record arm (`:246`) checks
   `fields`. No check reads an `actor.` path.
3. **Already expressible?** Points 1 to 4 ask for no construct. The second point (a condition on
   the page's own record in a header action) is not expressible on 0.94.0: a header action has no
   row. `story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither`
   chose form A in its fit review: `section.<name>.row.<field>` reads the record a section shows,
   at every expression site. A header action then states the condition as
   `section.<name>.row.owner_id == actor.user_id`.
4. **Fit.** The language grids (`docs/design/language-grids.md`) have no row for the binding
   expressions of a UI document; the triage is on the reference's table of forms.
   - Vocabulary: `field_in_model` with its message form, at more sites. No new key.
   - One source: the closed list of `actor.<field>` is prose in the schema's form note
     (`schemas/ui/ess-ui.schema.yaml:67`). It becomes data there, so the reference and the check
     read one list, as `story:ui-check-resolves-the-root-and-section-of-every-binding-path` does for
     the members of `section.<name>`.
   - Siblings: the draft names `visible` only. A probe on 0.94.0 with `--model`, one navigate
     parameter of a record's action per run: `row.nosuch` and `actor.nosuch` add no finding. The
     row is the same row in every expression of that action, so the check holds at every
     expression site that has a row in scope, and `actor.` at every expression site. The walk
     over the sites is the one `story:ui-check-resolves-the-root-and-section-of-every-binding-path`
     builds; this story depends on it.
   - Targets: the check and one sentence of the reference. No renderer changes.
5. **Second adopter.** A ticket page whose record has an action "close" with
   `visible: row.asignee_id == actor.user_id`: the button shows for nobody, and the check passes.
6. **Cost.** No format change. `field_in_model` reports at more sites; one new error for an
   `actor.` member outside the list; one new error for a `row.` path where no row is in scope. A
   document with such a path now fails the check; its condition was false or empty on screen. The
   changelog names the new errors.
7. **Alternatives.** (a) Change nothing. (b) The draft, `visible` only: leaves the navigate
   parameters of the same action unresolved (probe in point 4). (c) Chosen: every expression site
   with a row in scope. (d) A header action reads the page's record without naming a section (the
   draft's point 5, second half): a second way to reach what `section.<name>.row` reaches, and
   "the page's record" is not defined on a page with two reading sections; refused.

## Evidence

Reproduction script and its output: session scratch `ess/adopter6/repro.sh`, `repro.txt`
(not in the repository). Existing case for the one checked site:
`crates/ui/ess-ui-check/tests/checks.rs`, test `field_in_model`.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`. The acceptance covered the draft's points 1 to 4 and left point 5 as a question.

**Fit review (2026-11-12): accept, redesigned.** What changed from the draft and why:

- The check holds at every expression site with a row in scope, not in `visible` only. The probe
  in the fit review shows the navigate parameters of the same action unresolved.
- The list of `actor.<field>` members becomes data in the schema.
- The draft's point 5 is closed without a new design. The sibling story's fit review chose form
  A, so a header action reads a section's record as `section.<name>.row.<field>`. What is left of
  point 5 is one sentence of the reference (acceptance point 5). "A header action reads the page's
  record" is refused (fit review, point 7 d). No question remains for the operator from this
  story.
- This story depends on the two sibling stories: the walk over expression sites and the member
  `row` of `section.<name>`.

The requester is told in one message before the work starts.

## Reported cases (2026-11-13)

From a consumer's UI document, second collected report, reproduced on ess 0.94.0 with a document of
our own. Recorded here because the case is this story's class; the acceptance above or below is
not changed by this section, and the proposed line is taken up when the story is scoped.

Item 5, the signed-in actor's display name. On 0.94.0 `actor.<field>` reads every field of the
first row of the first view the home page's shell preloads: with that row holding
`display_name: Ada Example`, a text `actor.display_name` shows `Ada Example` in the terminal. The
reference lists `user_id`, `account_id`, `roles`, `signed_in` and `may(<command>)` only.

Acceptance 3 of this story makes a name outside the reference's list an error. As written it also
refuses `actor.display_name`, the one way a document shows its user's name today.
`story:an-actor-expression-reads-the-fields-of-the-session-view` (draft) asks what the actor's
fields are. Whoever implements acceptance 3 first reads that draft: either the list the check
refuses against comes from the session view, or the reference's list grows before the check closes
it.
