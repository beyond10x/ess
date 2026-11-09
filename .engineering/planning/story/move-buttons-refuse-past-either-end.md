---
format: aep.planning-md/3
id: story:move-buttons-refuse-past-either-end
kind: story
status: draft
title: Move buttons refuse a move past either end
summary: 'The React collection''s degraded move buttons (no_drag: move_buttons) still send a move past the first or last row (0.88.0 collection-controls correction, pre-existing)'
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/
revision: 5
---
## Need

Found by the 0.88.0 collection-controls correction, pre-existing: in the generated React app a
collection with `reorder` and `degrades: {no_drag: move_buttons}` draws move buttons that still send a
move past either end (up on the first row of a group, down on the last), while the drag handle and the
keyboard move refuse it through `canMove`.

Reproduction on 0.92.0: a `list_page` section `{component: collection, reads: {view: items.All},
columns: [name], reorder: {does: items.Move}, degrades: {no_drag: move_buttons}}` over two rows.
`ess generate ui --target react --path ui.yaml --out gen` writes
`gen/src/runtime/composites/collection.tsx` with

```
239:  const canMove = (key: string, delta: number): boolean => {
351:        <button type="button" className="ui-button" onClick={() => move(row, -1)}>
354:        <button type="button" className="ui-button" onClick={() => move(row, 1)}>
```

The buttons are never disabled and do not consult `canMove`, so the first row's up button sends
`items.Move {id, delta: -1}` and the last row's down button `{id, delta: 1}`. Under a client sort,
where `canMove` refuses every move, the buttons still send one.

## Acceptance

- `move_buttons_refuse_past_either_end`: in the generated project the up button of the first row of
  each group and the down button of the last row are `disabled`, and a click on them sends no command.
- Under a client sort (`canMove` false for every row) every move button is disabled.
- A move button between two rows still sends `{id, delta}` as before.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl` (the `moves`
  buttons near `:349-358`, `canMove` at `:239-242`)
- inferred: `crates/ui/ess-ui-react/tests/` (a case beside the 0.88.0 collection-controls tests)
- inferred: `CHANGELOG.md`

confidence: high
