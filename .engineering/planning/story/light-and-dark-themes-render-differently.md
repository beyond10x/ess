---
format: aep.planning-md/3
id: story:light-and-dark-themes-render-differently
kind: story
status: implemented
title: Light and dark themes render differently
summary: 'themes: with theme.chosen_by: shell.theme stores the state but nothing reads it; light and dark render alike (0.77.0)'
tags:
- ess-batch-5
- ess-next
- ess-ui
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:the-react-renderer-draws-the-declared-tokens
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-react/src/assets.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/shell.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/themes.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:01Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:03Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-13T04:54:18Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

A document's `themes:` and `theme:` change nothing in the generated React app. On ess 0.92.0 a
document with

```yaml
themes:
  light: {}
  dark:
    color: {surface: "#101010"}
theme: {default: light}
```

checks clean, and no generated file under `src/` mentions `data-theme` or a token value
(`repro/run.txt`, "B15": `grep -rn 'data-theme\|123456' … | wc -l` prints `0`). With
`theme: {default: light, chosen_by: shell.theme}` the shell stores the `theme` preference, and
nothing reads it, so light and dark render alike.

## Acceptance

As `docs/design/ui-style-tokens.md` § Renderers:

- `tokens.css` holds one `[data-theme="<name>"]` block per theme other than the default, with only
  what it overrides.
- The shell sets `data-theme` on the document element from the `chosen_by` state, through that
  state's own store module, and to `theme.default` before the state has a value or when there is no
  `chosen_by`.
- A test (`crates/ui/ess-ui-react/tests/themes.rs`) over a document with `light` and `dark` asserts
  the attribute for each state value and the override block's properties; a document without
  `themes:` generates no `data-theme`.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/shell.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/src/assets.rs`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-react/tests/themes.rs`
- confidence: high

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item B15 ("there is no `data-theme` switch") and
the 2026-10-29 report this story was first drafted from.

1. **Need.** A reader who picks dark sees dark. Reproduction above. The requester's words, theirs:
   "there is no `data-theme` switch".
2. **Class.** *Defect*: `themes`, `theme.default` and `theme.chosen_by` are accepted and documented
   (`schemas/ui/ess-ui.schema.yaml` `Theme`, `ThemeChoice`), and the design states the shell sets
   `data-theme` (`docs/design/ui-style-tokens.md` § Renderers).
3. **Already expressible?** No.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). Decided by
   the design page's own fit review (`ui-style-tokens.md` § "Decided in the fit review"); no new key.
5. **Second adopter.** Any app with a dark mode preference.
6. **Cost.** Generated output only; no source format change.
7. **Considered.** (a) Change nothing. (b) `prefers-color-scheme` only: out of scope by the design
   page (§ "Out of scope"). (c) Chosen: the design page's attribute switch.

## Decisions

**Accept as proposed.** As `docs/design/ui-style-tokens.md` decides. Depends on
`story:the-react-renderer-draws-the-declared-tokens`, which writes `tokens.css`. No new format
version.

## Source

`github:beyond10x/ess#723`
