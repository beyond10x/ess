---
format: aep.planning-md/3
id: story:the-terminal-tags-input-offers-suggestions
kind: story
status: draft
title: The terminal tags input offers the suggestions of its read, as React does
summary: suggests is drawn by React from 0.89.0 and ignored by the terminal
tags:
- backlog
- defect
- external-request
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0; reproduced on 0.92.0.

A field `as: tags` with `suggests: {reads, value}` suggests the rows of a read and keeps typed
values, so no create command is needed. React does this from 0.89.0
(`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `crates/ui/ess-ui-react/tests/tags_suggest.rs`).
The terminal ignores `suggests`, and its input is drawn as before (`crates/ui/ess-ui-tui/src/app.rs`
mentions `suggests` only as `suggests: None` in field literals; the 0.89.0 CHANGELOG entry says
"The terminal renderer draws the input as before"). The two renderers disagree on one document.

Reproduction (0.92.0): a `form_page` section form `{does: items.Save, fields: [{field: tags, label:
Tags, as: tags, suggests: {reads: {view: items.Tags}, value: name}}]}` with a fixture read
`items.Tags` answering rows named `urgent` and `later`.

```
$ ess ui check --path ui.yaml
ui.yaml: 0 error(s), 0 warning(s)
$ ess ui test --path ui.yaml t.yaml      # open: edit; expect: {at: pages/edit/sections/form, text: urgent}
failed  the tags input offers the values of its read (step 2)
        expected "urgent" at pages/edit/sections/form; it shows:
        Tags             []
        [ Submit ] ctrl-s · enter edits · tab next field
```

## Fit review

1. **Need, not syntax.** No new syntax: `suggests` is a documented key of a tags field
   (CHANGELOG 0.89.0, beyond10x/ess#746), and one target silently ignores it. The requester proposed
   nothing beyond "as React does".
2. **Class.** Defect: a target silently ignores a construct the document declares, which
   `.agents/skills/assessing-external-requests/SKILL.md` lists as a red flag ("a target would silently
   ignore the new construct"); every target handles a construct or refuses it by name.
3. **Already expressible?** No. In the terminal the only way to add a value is to type it.
4. **Fit.** Reuses `suggests` as declared, its read, its `value` field and its client row `filter`,
   as React does; no new key.
5. **Second adopter.** Any document with a tags field over a lookup read (labels on a ticket,
   categories on an item) run with `ess ui run --tui` or tested with `ess ui test`.
6. **Cost.** Terminal renderer and its tests; no format change, no new diagnostic. Size S-M.

## Decisions

Accept as proposed: the terminal draws the suggestions of `suggests`, with no format change.

## Acceptance

- On the reproduction, the terminal reads the `suggests` view and shows each value the input does
  not already hold (`urgent`, `later`) at the field; `ess ui test` `expect: {at:
  pages/edit/sections/form/fields/tags, text: urgent}` passes.
- A picked suggestion is added as a chip and is no longer offered; the submission then sends
  `tags: [urgent]`. A typed value is kept whether or not a row holds it.
- A client row `filter` on the `suggests` read narrows the offered values in the terminal as in React.
- A field without `suggests` is drawn and edited as before.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (tags input editing, the read of `suggests`)
- inferred: `crates/ui/ess-ui-tui/src/view.rs` (drawing the offered values at the field)
- inferred: `crates/ui/ess-ui-test/src/runner.rs` (only if picking a suggestion needs a step mapping, e.g. `choose`)
- inferred: `crates/ui/ess-ui-test/tests/` (cases beside the React `tags_suggest` cases)
- inferred: `CHANGELOG.md`

confidence: medium
