---
format: aep.planning-md/3
id: story:the-terminal-draws-an-object-tag-by-its-label
kind: story
status: draft
title: The terminal draws an object tag by its label, as React does
summary: 'Terminal draws an object tag as key: value, React draws its label (0.90.0 U3 implementor; from-finding)'
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Found by the U3 implementor of wave-2026-11-07-ess (ESS 0.90.0). Pre-existing, outside that
unit's acceptance.

A field `as: tags` whose entries are objects draws their `label` in the React project (`tagText`
in `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`). The terminal draws such an entry
as `key: value` text instead. The two renderers disagree on one document.

## Acceptance

- A record or column field `as: tags` over `[{label: "Alpha", id: 1}, {label: "Beta", id: 2}]`
  draws the chips `Alpha` and `Beta` in the terminal, as React does.
- An object entry without `label` draws the same text in both renderers.
- Plain string entries are unchanged.

## Scope

- inferred: `crates/ui/ess-ui-tui/src/` (tags display), the terminal value-format tests.
