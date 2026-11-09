---
format: aep.planning-md/3
id: story:a-declared-field-format-renders-or-is-refused
kind: story
status: draft
title: Every declared field format renders, or ess ui check refuses it
summary: '`as: metric` renders as text; durations lack minutes and hours; percent scale is undocumented (workbench report items 8, 41)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

Field formats the document accepts do not render as declared:

- A record field `as: metric` renders as plain text. `fields.tsx` has no `metric` case, and the
  check reports nothing (item 8, example `ui08`).
- `as: duration` takes `unit: ms` or `unit: s` only; `unit: m` is refused, and there is no hours
  unit (item 41).
- `format: percent` goes to `Intl` percent style and expects a fraction, so 0.5 shows 50%. Nothing
  documents that scale (item 41, example `ui41`; see #723).

Items 8 and 41 of the 2026-11-09 workbench report.

## Acceptance

- Every `as:` the schema admits for a record field renders as that format in React and the
  terminal, or `ess ui check` refuses it at that field.
- A duration may be stated in minutes and hours.
- The scale `format: percent` expects (fraction or hundredths) is documented in the reference and
  selectable, and a test pins it.

## Scope

- inferred: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` and `core.tsx.tmpl`,
  `crates/ui/ess-ui` (format and unit enums), `crates/ui/ess-ui-check`, `website/docs/reference/ess-ui.md`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
