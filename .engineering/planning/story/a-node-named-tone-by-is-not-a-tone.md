---
format: aep.planning-md/3
id: story:a-node-named-tone-by-is-not-a-tone
kind: story
status: draft
title: A page, state or widget named tone_by is not read as a tone
summary: The loader reads any key spelled tone_by as the tone construct (UI-adopter wave 1 U1 fix-round note, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:w086-u1-adversary-pass-1
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui/src/expand.rs
- confidence: inferred
  path: crates/ui/ess-ui/tests/
revision: 5
---
## Need

The loader treats every mapping key spelled `tone_by` under `shells`, `pages` or `widgets` as the
tone construct (`crates/ui/ess-ui/src/expand.rs`, `Tones::value`). `"tone_by"` is also in
`OPAQUE`, so local shorthands and widget expansion skip anything under such a key. A page, page
state or widget named `tone_by` therefore fails to load ("has neither", from `Tones::tone_by`) or
loses its expansion. The value-labels unit fixed the same flaw for `labels` in fix round 1
(review-result:w086-u1-adversary-pass-1). This was first found by reading the base at 83391ebaeb.
It now reproduces on 0.92.0.

Minimal reproduction. `PAGE` and `STATE` are replaced per variant:

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: PAGE
  sections: [{name: all, pages: [PAGE]}]
pages:
  PAGE:
    kind: list_page
    title: A
    state:
      STATE: {type: string, class: page_state, default: x}
    sections:
      - {name: rec, component: record, reads: {view: probe.Items}, fields: [title]}
```

A widget variant declares `widgets: {WIDGET: {summary: A caption., body: [{name: caption,
primitive: text, text: Hello}]}}` and uses it as `{name: panel, component: WIDGET}`.

```text
PAGE=a,       STATE=picked   -> 0 error(s)
PAGE=tone_by, STATE=picked   -> error document_loads pages/tone_by: a `tone_by` has exactly one of `map` and `tones`; this one has neither
PAGE=a,       STATE=tone_by  -> error document_loads pages/a/state/tone_by: (same message)
WIDGET=caption_box           -> 0 error(s)
WIDGET=tone_by               -> error document_loads widgets/tone_by: (same message)
```

## Acceptance

- `ess ui check` over each `tone_by`-named variant above reports the same findings as its
  control, and `ess ui load` prints the same document apart from the name.
- A widget named `tone_by` used by a section expands as the control widget does: the same nodes
  appear at `pages/a/sections/panel/body/caption`.
- A real `tone_by` construct (with `map` or `tones`) on a badge or column inside those documents
  still resolves its `tones:` name to the `tone_maps` entry.

## Scope

- cited: `crates/ui/ess-ui/src/expand.rs`
- inferred: `crates/ui/ess-ui/tests/` (beside the `labels` name tests of fix round 1)

confidence: high

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
