---
title: "0.86 — UI documents label their values, show a toggle's state and page by size"
description: >
  0.86.0 closes four gaps a downstream adopter of ess-ui/1 reported: a field or badge shows a
  declared label for each value, a row toggle shows the state it flips, a server-paged collection
  gains a page-size control and a pager footer, and a refused document is reported at the refused
  node's own path.
slug: ui-labels-toggles-pager-and-refusal-paths
tags: [release, ess]
date: 2026-11-03T22:00:00+01:00
release_tag: "0.86.0"
---

0.86.0 changes the `ess-ui/1` grammar, its checks, the generated React runtime and the terminal
renderer. No source format moves, and a UI document that uses none of the new keys renders as
before. A downstream adopter reported the gaps
([#723](https://github.com/beyond10x/ess/issues/723)). The
[changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change; this note
covers each of them.

{/* truncate */}

## A value shows its declared label

- `labels:` on a field or a `badge` primitive gives the text shown for each value. It is either a
  map written out or the name of an entry of the new document-level `label_maps`. Booleans are
  keyed `"true"` and `"false"`.
- A value the map does not name shows its raw value. A badge keeps the raw value in its class and
  its tone.
- A choice column with fixed `options` shows each option's label.
- `ess ui check` reports a name that resolves to no map (`label_map_refs`) and a map nothing names
  (`label_map_unused`).
- Labels are declared, never derived from a field name.

## A toggle shows the state it flips

- A row toggle that runs a command used to render off on every row. It now takes its state from an
  optional `checked:` expression, or else from its one bind entry written `not <expr>`.
- A `does:` toggle with neither warns `toggle_state_unknown`.
- `checked:` without `as: toggle` is refused (`checked_needs_toggle`).
- React and the terminal read the same expression.
- A column `as: toggle` renders a switch, not the text `true` or `false`.

## A server-paged collection has a page-size control

- A read with `paging: server` takes `pager: {page, size, sizes}`, naming the read params that
  carry the page and the page size. `sizes` defaults to `[25, 50, 100]`.
- The footer shows Previous, the page (and of how many when the total is known) and Next.
- With a `size`, the footer adds a per-page select that sets the size and returns to page 1.
- The footer writes the state paths the params bind, never a fixed `state.page`.
- `ess ui check` refuses a pager it cannot honour: one without server paging, a param that is not
  bound to a state path, page and size on overlapping paths, an empty size list, or a pager on any
  read but a collection's.

## A refusal is reported where it is

- A refused node is reported at its own path. Before, a list entry whose name or `field` matched a
  key the schema holds an action under was read as that action and reported first.
- A refused key in a section's `states` is reported at that state.
- An export's `params` may be a map beside the single expression. `selection.<field>` is allowed
  only on a bulk action (`export_selection`).
- An empty state may carry a `title`.
