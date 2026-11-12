---
format: aep.planning-md/3
id: story:a-wide-table-scrolls-inside-its-column
kind: story
status: active
title: Page columns stack at narrow widths and a wide table scrolls inside its column
summary: 'A wide table overflows a layout.columns column: .ui-section has no overflow-x and nothing stacks below a breakpoint (0.77.0)'
tags:
- ess-batch-6
- ess-next
- ess-ui
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/narrow_layout.rs
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:54Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:56Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

At narrow widths a page laid out in `layout.columns` overflows. The generated stylesheet lays the
columns side by side with no wrap and no width rule: `.ui-layout-columns { display: flex; gap: 1rem;
align-items: flex-start; }`, and it holds no `@media` rule at all
(`crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`; on ess 0.92.0 `grep @media` over the
generated `styles.css` finds nothing). A two-column detail page on a phone-width window squeezes
both columns instead of stacking them.

What this story first asked for is partly shipped: since 0.88.0 a table sits in a frame that
scrolls sideways with a shadow on the side that has more (`.ui-table-frame { overflow-x: auto; … }`),
card rows wrap their text (`.ui-row-body { min-width: 0; overflow-wrap: anywhere; }`) and chip,
action and filter groups wrap (`flex-wrap: wrap`) (CHANGELOG 0.88.0, story
`a-collection-draws-the-controls-it-declares`).

## Acceptance

- Below a fixed renderer width (`48rem`), `layout.columns` and `layout.areas` stack their columns
  or areas into one column in declaration order, the order `degrades: {no_columns: stack}` already
  uses; above it they lay out as today. A generator test pins the rule in the generated
  `styles.css` (`crates/ui/ess-ui-react/tests/narrow_layout.rs`).
- A wide table inside a column scrolls inside that column and never widens the page (pinned by the
  same test, over a two-column page holding a ten-column table).
- No authored key is added; a document's generated page files are unchanged.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`
- inferred: `crates/ui/ess-ui-react/tests/narrow_layout.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: high

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item D13 (its table, card and chip halves shipped
in 0.88.0; the column half is this story) and the 2026-10-29 report this story was first drafted
from.

1. **Need.** A page read on a narrow window shows every section whole, one under another.
   Reproduction above (`repro/run.txt`). The earlier report's proposal, theirs: "columns stack below
   a declared breakpoint".
2. **Class.** *Defect*: `PageLayout` says "Renderers without columns or areas fall back to a stack"
   (`schemas/ui/ess-ui.schema.yaml` `PageLayout.doc`); a window too narrow for columns is that case,
   and the stylesheet never takes it.
3. **Already expressible?** `layout: stack` on the page stacks always, losing the wide layout; no.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). A
   breakpoint is a length, and no node has a length property by design
   (`docs/design/ui-style-tokens.md` § "Where a node names a token": "a region says what an area is
   for, not how it looks"); so the width is the renderer's, as the stack fallback already is.
5. **Second adopter.** Any detail page opened on a tablet in portrait.
6. **Cost.** Stylesheet only; no key, no format change.
7. **Considered.** (a) Change nothing. (b) The requested `breakpoint:` key on the layout: a length on
   a node, refused by the token design above. (c) Chosen: a renderer width rule.

## Decisions

**Accept, redesigned.** The breakpoint is the renderer's, not a declared key. No new format version.

**Delivered form (2026-11-12).** Acceptance 3 holds with one exception: a page laid out in `areas`
now emits its areas in section declaration order, with that order also passed as an `order` list.
So such a page's generated file changes.

The reason is that acceptance 1 and acceptance 3 cannot both hold for an areas page. Acceptance 1
asks for the order `degrades: {no_columns: stack}` uses, which the schema names
`section_declaration` and the terminal already follows. The React runtime receives `areas` as a
map from name to content, so it cannot know the section order; only the generator can pass it.

Acceptance 3 exists for "no key, no format change" (Fit review, item 6). A change to the order of
entries in a generated file is neither.

The order goes in as a list, not as object key order. `Object.entries` puts integer-like keys
first, and the name pattern admits an area named `1`. Columns keep their declared order.

## Source

`github:beyond10x/ess#723`
