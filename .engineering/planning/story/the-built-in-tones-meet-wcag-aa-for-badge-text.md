---
format: aep.planning-md/3
id: story:the-built-in-tones-meet-wcag-aa-for-badge-text
kind: story
status: draft
title: The built-in tones meet WCAG AA for badge text
summary: 'Badge text in the built-in tone table is below WCAG AA: warning 2.68, info 3.04, success 3.18, danger 3.98 to 1 (0.77.0)'
tags:
- defect
- ess-batch-6
- ess-next
- ess-ui
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/
- confidence: cited
  path: crates/ui/ess-ui/src/style.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 7
---
## Outcome

A reader of a generated React app can read every badge drawn with a built-in tone: its text and
background reach WCAG AA (4.5:1) without a document overriding `tokens:`.

## Problem

Reported on 0.77.0 by a downstream adopter and reproduced on 0.92.0. The built-in token table
(`schemas/ui/ess-ui.schema.yaml`, `constructs.Tokens.builtins`) and the React stylesheet draw a badge
on `--ui-line` (#dcdde1) with the tone colour as its text, and a danger badge as #c0392b on
`danger_fill` #f6d5d1.

Reproduction: any document, `ess generate ui --target react --path ui.yaml --out gen`, then the
WCAG 2 contrast of each pair in `gen/src/runtime/styles.css`:

```
.ui-badge { ... background: var(--ui-line); }      --ui-line: #dcdde1
.ui-tone-info    { color: var(--ui-info); }        #2f7fd3 on #dcdde1: 3.04:1
.ui-tone-success { color: var(--ui-success); }     #2e8a4f on #dcdde1: 3.18:1
.ui-tone-warning { color: var(--ui-warning); }     #b7791f on #dcdde1: 2.68:1
.ui-badge.ui-tone-danger { background: #f6d5d1; color: var(--ui-danger); }   #c0392b: 3.98:1
```

Every pair is below 4.5:1.

## Acceptance

- A test computes the WCAG 2 contrast ratio of each built-in tone's badge text over its badge
  background (info, success, warning, danger) from the built-in token table and the generated
  stylesheet, and each is at least 4.5:1.
- The same test covers every theme the schema ships built in, if any; a document's own `tokens:` or
  theme is not checked.
- A document without `tokens:` still generates a stylesheet whose only changes are the tone colours
  (or badge backgrounds) the fix moves.

## Out of Scope

- Contrast of document-supplied tokens and themes.
- The terminal renderer, which draws tones with terminal colours.
- Button tones (`.ui-tone-primary`, `.ui-tone-danger` buttons with white text).

## Scope

- cited: `schemas/ui/ess-ui.schema.yaml` (`constructs.Tokens.builtins`, the colour entries near `:1604-1610`)
- cited: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl` (`:root` tokens `:2-11`, badge and tone rules `:102-106`)
- cited: `crates/ui/ess-ui/src/style.rs` (`Tokens::builtin`)
- inferred: `website/docs/reference/ess-ui.md` (the rendered built-in table)
- inferred: `crates/ui/ess-ui-react/tests/` (the contrast test)
- inferred: `CHANGELOG.md`

confidence: high
