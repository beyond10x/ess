---
format: aep.planning-md/3
id: story:an-action-names-its-icon
kind: story
status: draft
title: An action names its icon and its emphasis, from closed vocabularies
summary: 'as: icon draws a generic glyph because an Action has no icon key'
tags:
- backlog
- external-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:zero-open-issues-2026-11
revision: 4
---
## Need

An icon is a free name nothing defines, and an action can name neither an icon nor an emphasis. On
ess 0.92.0:

- Every icon draws the same glyph: the icon primitive is `<span class="ui-icon ui-icon-<name>">`
  (`crates/ui/ess-ui-react/templates/runtime/primitives/icon.tsx.tmpl:5-8`) and the stylesheet's one
  rule is `.ui-icon::before { content: "●"; }` (`templates/runtime/styles.css.tmpl:117`). `icon:` on
  the primitive and on a navigation section takes any string, and `ess ui check` checks none.
- An action has no `icon` key: `{name: archive, as: icon, icon: archive, …}` is refused,
  `` unknown field `icon` ``. An `as: icon` action draws its label text or the generic glyph.
- An action has no emphasis: `{…, tone: danger, …}` is refused, `` unknown field `tone` ``; a primary
  action renders as a secondary (grey) button. The button primitive already has the emphasis
  `tone: primary | secondary | danger | ghost` (`schemas/ui/ess-ui.schema.yaml` `button`).

## Acceptance

- The schema declares a closed icon vocabulary (one list, each name with its meaning: at least
  add, edit, delete, archive, copy, download, upload, refresh, search, filter, settings, open,
  close, check, warning, info, arrow_up, arrow_down, arrow_left, arrow_right, home, user, play,
  pause). React draws each as its own inline glyph; the terminal draws each as its own one-cell
  glyph. `ess ui check` warns `icon_names` at the node's path for a name outside the list, which
  draws its label (so today's documents still load).
- An action takes `icon: <name>` from the same list: an `as: icon` action draws the glyph alone with
  its label as accessible name and title; any other `as` draws the glyph before the label
  (`crates/ui/ess-ui-react/tests/action_look.rs`).
- An action takes `tone:` with the button's words, as decided (G-7 answer a): React draws a
  `primary` action filled, `danger` destructive, `ghost` without border; default `secondary`, so
  today's output is unchanged. An unknown word is refused by path.
- The terminal shows an icon action's glyph before its key hint and its tone as the emphasis
  table of `docs/design/ui-style-tokens.md` § Renderers says.
- `website/docs/reference/ess-ui.md` is a fresh render, listing the icon vocabulary.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `schemas/ui/ess-ui.schema.yaml`
- cited: `crates/ui/ess-ui-react/templates/runtime/primitives/icon.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items G27/D11/B5, G28, and the glyph half of B9
(B9's choice of glyph by value is answered by an idiom, below); also the 2026-11-07 report this
story was first drafted from.

1. **Need.** A reader tells two icon-only actions apart, and sees which action of a group is the
   primary and which destroys. Reproductions above and in `repro/run.txt`. The requester's
   proposals, theirs: "icon names are a defined vocabulary", "an action carries a tone".
2. **Class.** Icon glyphs: *defect* (the schema names an icon, `schemas/ui/ess-ui.schema.yaml`
   `icon.fields.icon`, and every one draws alike). Action icon and emphasis: *gap*.
3. **Already expressible?** Partly: a `button` primitive with `tone:` and an `action:` draws an
   emphasised button, but not as a row action, a header action or a bulk action, which are
   `Action`s. An icon by value (B9) is expressible today: an `item:` icon primitive per value with
   `visible: "row.state == open"` checks and generates on 0.92.0, drawn per row (`repro/idioms/`).
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). `icon` on an
   action is the primitive's and the navigation section's key with the same values; `tone` on an
   action takes the button's emphasis words, not the status tones, because an action's look is a
   button's (`docs/design/ui-style-tokens.md` § Renderers, "Button emphasis is a renderer convention").
   Every sibling that names an icon (primitive, navigation section, action) is held to the one list.
5. **Second adopter.** A file manager whose row actions are download, rename and delete as icons,
   delete in the danger emphasis, upload the primary header action.
6. **Cost.** Additive in `ess-ui/1`: two optional `Action` keys and a vocabulary list; a new warning
   (`icon_names`), not an error, so no document that loads today stops loading.
7. **Considered.** (a) Change nothing: hosts style `ui-icon-<name>` themselves. (b) An open icon set
   mapped by a host table: renderers could not draw it and check could not hold it. (c) Chosen: a
   closed list both renderers draw, as tones are closed.

## Decisions

**Accept, redesigned.** Action `tone` as decided (`approval-record:ess-0870-wave`, G-7 answer a);
`icon` on an action reuses the primitive's key; the icon names become a closed list with a warning
outside it. The glyph chosen by a row value (B9) is declined with the idiom above. No new format
version; `docs/design/ui-style-tokens.md` gains the icon vocabulary before code.

## Source

`github:beyond10x/ess#723`
