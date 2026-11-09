---
format: aep.planning-md/3
id: story:the-react-renderer-draws-the-declared-tokens
kind: story
status: draft
title: The React renderer draws the declared tokens
summary: ess generate ui --target react ignores tokens:; styles.css is unchanged and hard-codes values that have tokens (0.77.0)
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
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/src/assets.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/project/main.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/primitives/badge.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/primitives/button.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/tokens.rs
revision: 7
---
## Need

`ess generate ui --target react` ignores a document's `tokens:`. On ess 0.92.0 a document that adds

```yaml
tokens:
  color: {surface: "#fafafa", accent: "#123456"}
```

to the triage's minimal document checks clean, and its generated `src/runtime/styles.css` is
byte-identical to the one generated without it; neither color appears anywhere in the generated
`src/` (`repro/run.txt`, "B15"). The stylesheet is one fixed template
(`crates/ui/ess-ui-react/src/assets.rs:167-168` writes `templates/runtime/styles.css.tmpl` as is),
with its colors, spacing, radii and type sizes as literals.

## Acceptance

As `docs/design/ui-style-tokens.md` § Renderers, step 2 of its implementation notes:

- The generator writes `src/runtime/tokens.css` from the document: one `:root` block with every
  token of the merged table (`--ui-color-<name>`, `--ui-space-<name>`, `--ui-radius-<name>`,
  `--ui-type-<style>-family|size|weight`, `--ui-tone-<tone>-text|fill`).
- `styles.css.tmpl` keeps no color, length or type literal; it reads `var(--…)` only. Button
  emphasis moves to its own class prefix (`ui-emphasis-*`), apart from `ui-tone-*`.
- A generator test (`crates/ui/ess-ui-react/tests/tokens.rs`) proves (a) changing one token changes
  exactly that custom property in `tokens.css`, and (b) a document without `tokens:` computes the
  same values as today's stylesheet.

## Scope

- cited: `crates/ui/ess-ui-react/src/assets.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-react/templates/project/main.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/primitives/badge.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/primitives/button.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/tests/tokens.rs`
- confidence: high

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item B15 (tokens; the theme switch is
`story:light-and-dark-themes-render-differently`) and the 2026-10-29 report this story was first
drafted from.

1. **Need.** An author who declares colors, spacing and type sees them in the generated app.
   Reproduction above. The requester's words, theirs: "the React target ignores `tokens` and
   `themes`; the runtime stylesheet hardcodes them".
2. **Class.** *Defect*: `tokens:` is accepted and documented (`schemas/ui/ess-ui.schema.yaml`
   `Tokens`), and the design states React writes them (`docs/design/ui-style-tokens.md` § Renderers).
3. **Already expressible?** No: the host can only overwrite the generated stylesheet.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). The design
   page decided the shape and was accepted in its own fit review (`ui-style-tokens.md` § "Decided in
   the fit review"); this story builds its React step, no new key.
5. **Second adopter.** Any adopter with a brand color.
6. **Cost.** Generated output changes (a new `tokens.css`, `var()` in `styles.css`, a class rename
   in generated markup only); no source format change.
7. **Considered.** (a) Change nothing. (b) Inline styles per node: a document-wide value repeated at
   every node. (c) Chosen: the design page's custom properties.

## Decisions

**Accept as proposed.** As `docs/design/ui-style-tokens.md` decides. No new format version.

## Source

`github:beyond10x/ess#723`
