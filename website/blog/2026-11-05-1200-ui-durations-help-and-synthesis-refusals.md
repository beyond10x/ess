---
title: "0.87 — UI values render in their declared form, and synthesis refuses a misplaced not-found once"
description: >
  0.87.0 renders each ess-ui/1 value in the form its field declares, durations and byte counts
  included, shows a choice's placeholder and a form's help, refuses a not-found branch declared
  after a guarded move once with its repair, and lets a journey find a row toggle again.
slug: ui-durations-help-and-synthesis-refusals
tags: [release, ess]
date: 2026-11-05T12:00:00+01:00
release_tag: "0.87.0"
---

0.87.0 changes the `ess-ui/1` grammar, the generated React runtime, the terminal renderer, the
`ess ui test` journey runner, conformance synthesis and the OpenAPI projection's wording. No source
format moves, and a UI document that uses none of the new keys renders as before. A downstream
adopter reported most of the UI gaps ([#723](https://github.com/beyond10x/ess/issues/723)). The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change; this note
covers each of them.

{/* truncate */}

## A value renders in the form its field declares

- A field `as: duration` takes `unit: ms | s`, default `ms`, and shows `h:mm:ss`. Hours are never
  folded into days, and part of a second is dropped. A form input keeps the number.
- A field `as: bytes` and a metric `format: bytes` show B, KB, MB or GB in 1024 steps.
- In React, a `date` field shows an epoch-milliseconds value as a date, `audio` renders a player,
  `link` an anchor, a tag holding an object its `label`, a `json` input sends parsed JSON and a
  `list` input edits a list.
- A placeholder read's `fixture:` resolves inside the document's `fixtures.dir`.

## Choices, filter bars and forms show what they declare

- A choice takes `placeholder:`, shown while nothing is chosen and as a dropdown's empty option.
  It is named by the declared label of the field or action it picks for.
- A filter bar's `reset` is `true` or `{label}`. Its search box is drawn only when a read on the
  page takes the search's state.
- A form group and a field take `help: {text, link}`; a field's help is shown with its input and
  on its column header.
- A form with `save: on_change` saves each change and shows no submit button.
- Help links and the `link` primitive load only `http`, `https`, `mailto` or relative addresses;
  any other address is shown as text.

## Tables and journeys

- `ess ui run --tui` no longer cuts a table column at 28 cells while the table has room.
- `ess ui test` finds a row toggle again in either state. Since 0.86.0 the terminal writes a
  toggle's hint with its state mark, and every journey step on a toggle failed.

## Synthesis

- An `external:` branch declared after an accepting `when:` branch that claims every input is
  refused once, as `ESS-SYNTH-003`, with its repair: declare it first with `before_row: true`, or
  narrow the guard. It was also refused as `ESS-SYNTH-019`, which called it a defect in ess.
- A value type with invariants that nothing reaches is named in a note instead of being refused
  as `ESS-SYNTH-013`.

## OpenAPI

- `ess generate openapi` calls an event published to consumers only when the component lists it
  under `publishes:`.
