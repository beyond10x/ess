---
format: aep.planning-md/3
id: story:an-action-names-its-icon
kind: story
status: draft
title: An action names a semantic icon
summary: 'as: icon draws a generic glyph because an Action has no icon key'
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

An action `as: icon` is drawn icon-only in a page header, with `aria-label` and `title`
(`actions.tsx.tmpl`). An Action has no `icon` key, however. The glyph is the class
`ui-icon-<action name>` with a generic "●" (`styles.css.tmpl`), so two header actions look the
same unless the host styles each action's name.

## Acceptance

- An action may name a semantic icon (`icon: <name>`), mapped by the renderer as the `icon`
  primitive's name is. It applies to `as: icon` and beside a label.
- In the terminal, an icon-only action shows a glyph or its label, as the `icon` primitive does.
- An action without `icon` is unchanged.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (Action), `crates/ui/ess-ui-react/src/emit.rs`,
  `templates/runtime/actions.tsx.tmpl`, the terminal action display, the reference page.
