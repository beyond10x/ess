# Style tokens and themes (`tokens:`, `themes:`, `ess-ui/1`)

Story `ui-spec-style-tokens`, first step: the construct, its checks and what each renderer does
with it. This page is the design; no code lands with it.

## The gap

An `ess-ui/1` document says what a node means and never how it looks. It already names roles:
a badge or icon has a `tone` (`neutral`, `info`, `success`, `warning`, `danger`;
`crates/ui/ess-ui/src/model.rs:1822`, `schemas/ui/ess-ui.schema.yaml:1018`), a badge can pick its
tone from its value (`tone_by`, `model.rs:1855`), a text has a `style` (`body`, `caption`,
`heading`, `mono`; `model.rs:1782`) and a button an emphasis (`ButtonTone`, `model.rs:1891`). What
those roles look like is fixed by each renderer, and the document cannot change it:

- **React** writes one stylesheet, the same for every document
  (`crates/ui/ess-ui-react/src/assets.rs:124-127`). Its `:root` block declares ten colours as
  custom properties (`templates/runtime/styles.css.tmpl:1-15`); seven more distinct colours are
  literals in rules (`#d7f0df` and `#f7e7c6` at 41-42, `#f7e7c6` and `#d9e4fb` at 55-56, `#fff` at
  66-67 and 95, `#eef2fd` at 77, `#f6d5d1` at 92, `rgb(0 0 0 / 0.3)` at 135). Spacing, radii and
  type sizes are literals throughout. There is one theme.
- **The terminal** draws no colour at all. It has three styles, bold, reversed and dim
  (`crates/ui/ess-ui-tui/src/view.rs:35-44`). A badge is `[value]` in bold whatever its tone
  (`view.rs:1784-1787`); an icon's tone is ignored (`view.rs:1788-1795`).
- **The theme preference is inert.** The partner-portal example declares a `theme` state of
  class `preference`, typed `{enum: [light, dark]}`, in `local_storage`, pinned, default `light`,
  on its shell (`examples/partner-portal/ui.yaml:68`), and the reference names theme as shell
  state (`schema.yaml:258`) and as a preference (`schema.yaml:1326`). No renderer reads it.
- **Status colours are written by hand at every use.** The example's `status_badge` widget takes a
  value-to-tone map as an argument (`ui.yaml:35-42`), and each use writes the map out again
  (`ui.yaml:51`, `ui.yaml:322`). A second badge over the same state can colour `Failed` differently.

## The construct

```yaml
tokens:
  colour:
    surface: '#ffffff'
    text: '#1c1d21'
    danger: '#c0392b'
    danger_fill: '#f6d5d1'
  space:  {xs: 0.25rem, sm: 0.5rem, md: 1rem, lg: 1.5rem}
  radius: {sm: 4px, md: 6px, lg: 8px, pill: 999px}
  type:
    heading: {size: 1.05rem, weight: 600}
    mono:    {family: 'ui-monospace, monospace'}
  tone:
    danger: {text: danger, fill: danger_fill}
themes:
  light: {}
  dark:
    colour: {surface: '#1f2024', text: '#ecedf0', line: '#34363c', danger_fill: '#4a2420'}
theme: {default: light, chosen_by: shell.theme}
tone_maps:
  job_state:  {Done: success, Failed: danger, HumanEscalation: warning, Running: info}
  deal_stage: {lead: neutral, qualified: info, proposal: warning, won: success, lost: danger}
```

```yaml
- {name: state, primitive: badge, field: state, tone_by: {value: row.state, tones: job_state}}
```

### Tokens

`tokens:` has five groups, each a map of name to value. Every group is a map because no order is
meaningful in it, which is the schema's rule for maps (`schema.yaml:9-12`).

| group | value | names |
|---|---|---|
| `colour` | a colour literal: `#rgb`, `#rrggbb`, `#rrggbbaa` or `rgb(r g b / a)`, quoted | free |
| `space` | a length: `0`, or a decimal with `px`, `rem` or `em` | free |
| `radius` | a length, as `space` | free |
| `type` | `{family?: string, size?: length, weight?: 100..900 in steps of 100}` | the `TextStyle` variants only |
| `tone` | `{text: <colour name>, fill: <colour name>}` | the `Tone` variants only |

**The schema carries a built-in table**, and a document's `tokens:` is merged over it group by group
and name by name. The built-in values are the React stylesheet's values today: the ten `:root`
colours under their current names without the `--ui-` prefix (`bg`, `surface`, `text`, `muted`,
`line`, `accent`, `info`, `success`, `warning`, `danger`), the literals as `danger_fill`,
`success_fill`, `warning_fill`, `info_fill`, `focus`, `on_accent` and `backdrop`, and a tone table
that reproduces today's badges (`styles.css.tmpl:88-92`): each tone's `text` is its own colour
(`neutral` uses `text`) and its `fill` is `line`, except `danger`, whose fill is `danger_fill`. So
a document without `tokens:` looks exactly as it does now, and a document that names three colours
changes three. A `type` entry falls back field by field to `body`, then to the built-in `body`.

### Themes and the theme preference

`themes:` maps a theme name to overrides: the same group-and-name shape as `tokens:`, holding only
the values that differ. A theme with no overrides (`light: {}` above) is the base values. A theme is
the merge of the built-in table, `tokens:` and its own overrides, so **every theme defines every
token by construction**. The one error left is an override naming a token nothing declares, which
`theme_tokens` refuses (below).

`theme:` says which theme is shown. `default` names a theme. `chosen_by` is optional and names
shell state in the expression form `shell.<name>` (`schema.yaml:64`): a state of class `preference`
whose type is an enum, each variant naming a theme. A renderer shows the theme that state holds,
and `default` where the shell declares no such state or it holds no value. The user changes it like
any state: an action `sets: {shell.theme: dark}` (`Action.sets`, `schema.yaml:1165`) on a button
or an account-menu entry. Where it is stored, how long it lives and who shares it is the state's
own `store`, `scope` and `pinned`, resolved by the placement profile as for every other state.
The example's existing state at `ui.yaml:68` fits as it stands; for the choice, the example gains
only the `theme:` line.

### Tones and status colours

`tone_maps:` names a value-to-tone map once. `tone_by` gains `tones: <name>` beside `map:`, exactly
one of the two. The loader resolves `tones:` to the named map during expansion, after widget
arguments are substituted (step 3 of `crates/ui/ess-ui/src/lib.rs:4-11`), so after loading
`ToneBy.map` is always a map (`model.rs:1859`) and no renderer reads `tones:`. A value the map
does not name takes the badge's `tone`, default `neutral`, which is what React does today
(`templates/runtime/primitives/badge.tsx.tmpl:17-23`). A widget parameter that carries a map name
is typed `{ref: tone_map}`.

The chain is then value → tone (`tone_maps`) → `{text, fill}` (`tokens.tone`) → colour values
(`tokens.colour`, per theme). An author colours a state by what it means, once per document.

## Where a node names a token

In this step a node names a token only through a role it already has: `tone` and `tone_by` on
badge and icon (through `tokens.tone`), and `style` on text (through `tokens.type`). Space and
radius are named by the renderer's own stylesheet, not by nodes: no composite, primitive or region
has a length or colour property today, and a region says what an area is for, not how it looks
(`schema.yaml:278`). Inside `tokens.tone` and `themes`, a reference is a bare name, read in the
group its position implies (`tone.*.text` and `tone.*.fill` name colours). The story's
`{token: <group>.<name>}` form on a node is not introduced here (question 3).

## Rules

New entries of `CHECKS` (`crates/ui/ess-ui-check/src/lib.rs:107-141`) and of the schema's
`checks.list` (`schema.yaml:1475`), each finding at the node path of the value at fault:

| check | rule | severity |
|---|---|---|
| `token_values` | a value is not its group's type: a colour that is not one of the four literal forms, a length without a unit or with another one, a weight outside 100..900 or not a multiple of 100 | error |
| `token_names` | a `type` key that is not a `TextStyle` variant, or a `tone` key that is not a `Tone` variant | error |
| `token_refs` | a `tokens.tone` entry names no colour of the merged table | error |
| `theme_tokens` | a theme overrides a group or a name the merged table does not declare, or gives a value of the wrong type | error |
| `theme_choice` | `theme.default` names no theme; `theme:` without `themes:`; `chosen_by` is not `shell.<name>`, or names a state no shell declares, or one that is not class `preference`, not an enum, or has a variant or `default` that names no theme | error |
| `tone_map_refs` | `tone_by.tones` names no entry of `tone_maps`: a loader refusal, which needs its own branch in `classify` (`crates/ui/ess-ui-check/src/classify.rs:51-72`) or it is filed as `document_loads` | error |
| `tone_map_unused` | a `tone_maps` entry no `tone_by` names | warning |

An unknown key in `tokens`, a theme, `theme` or a `type` or `tone` entry is refused by the reader
(`deny_unknown_fields`), as everywhere in the document.

## Format and version

**Additive within `ess-ui/1`.** The repository's rule (`AGENTS.md`, "Determinism and formats") asks
for a new version when meaning, identity, references, canonicalization, names or the envelope
change. Here every document that loads today loads to the same `Document` and renders the same,
because the built-in table is today's values. The new keys are optional. What changes is what a
newer reader accepts.

An older reader, any `ess` release to date (0.50.0 is the newest, `CHANGELOG.md:29`), refuses a
document that uses them and never renders it with its tokens dropped: `Document` refuses keys it does not declare (`model.rs:183`),
so `tokens:`, `themes:`, `theme:` and `tone_maps:` are each a load error at the root, and
`tone_by.tones` one at the badge (`ToneBy` is `deny_unknown_fields` too, `model.rs:1853`). Both
renderers load through `ess_ui` (`crates/ui/ess-ui-react/src/lib.rs:127`,
`crates/ui/ess-ui-tui/src/app.rs:236`), and `ess ui check` files the refusal under
`document_loads` (`classify.rs:51-72`). The refusal says "unknown field" rather than "needs a
newer format"; question 1 is whether that is enough.

## Renderers

**React.** The generator writes `src/runtime/tokens.css` from the document beside `styles.css`
(`assets.rs:124-127`): one `:root` block with every token of the default theme, and one
`[data-theme="<name>"]` block per other theme holding only what it overrides. Names are
`--ui-colour-<name>`, `--ui-space-<name>`, `--ui-radius-<name>`, `--ui-type-<style>-family`,
`-size` and `-weight`, and `--ui-tone-<tone>-text` and `-fill`. `styles.css.tmpl` loses its
`:root` values and its literals and uses `var(--…)` only; the badge and icon rules read the tone
properties, which also gives `neutral` a rule (it has none today). Button emphasis moves to its own
class prefix, because `.ui-tone-danger` serves both a `ButtonTone` and a `Tone` today
(`styles.css.tmpl:67` and `92`). The shell sets `data-theme` on the document element from the
`chosen_by` state, through the state's own store module, and from `theme.default` before that
state has a value.

**Terminal.** The terminal lacks two new capabilities, which join the schema's capability table
(`schema.yaml:989-1001`) and the terminal profile (`crates/ui/ess-ui-tui/src/profile.rs:58-111`):

| capability | fallbacks | applies to | the terminal |
|---|---|---|---|
| `no_colour` | `emphasis` | `Tokens`, badge, icon | `Lacks(["emphasis"])`: a tone is drawn as emphasis |
| `no_metrics` | `ignore` | `Tokens` | `Lacks(["ignore"])`: space, type and radius are not read |

`emphasis` maps tones to modifiers and adds no text, so screen text and every text assertion of
`ess-ui-test` stay as they are: `neutral` and `success` bold (a badge is bold today), `info`
italic, `warning` underlined, `danger` bold and underlined. It never uses reversed, because
reversed is the cursor: `ess-ui-test` finds focus by reversed cells
(`crates/ui/ess-ui-test/src/screen.rs:75`, `120-138`). Neither capability has `refuse` among its
fallbacks, so `degrades_cover` (`crates/ui/ess-ui-check/src/rules.rs:1137`) never fires for them,
and a document never has to declare `degrades` for tokens. A badge or icon may still write
`degrades: {no_colour: emphasis}`, the one fallback there is; `Tokens` is not a node, so its uses
resolve from the table's first fallback. The terminal never reads `theme:`. React lacks neither
capability.

A renderer that one day has colour drops `no_colour` from its profile and maps each colour token to
its palette.

## Departures from the story

The story's acceptance is the need; these are the places this design reads it differently.

| story | this design | why |
|---|---|---|
| groups include `elevation` | four groups and `tone` | the stylesheet has no shadow (`styles.css.tmpl`, no `box-shadow`); nothing would read it |
| a check refuses "a theme that leaves a token undefined" | impossible by construction; `theme_tokens` refuses the other side, an override of an undeclared token | a theme is the base plus overrides, and the base always has a value |
| nodes name tokens (`padding: {token: space.md}`), with a `raw:` escape | nodes name roles (`tone`, `style`); no `{token: …}` on nodes and no `raw:` yet | no node has a length or colour property to put it on, and adding them makes the document a stylesheet |
| `preferences:` lists theme, density and language with values, default and `store:`, replacing the theme state | no `preferences:`; `theme.chosen_by` names the existing `preference` state | `State` already declares the values (enum type), the default and the placement; a second declaration would have to be kept equal to it. Density and language have no construct that would read them |
| capability `fixed_density` | `no_metrics` | density is not in this step; every existing capability is named `no_<what is missing>` |

## Out of scope

Density and language; per-node token references and `raw:`; a built-in `dark` theme; following the
operating system's colour scheme; mapping colour tokens to a terminal palette; enum coverage of a
tone map against the ESS model, which `ess ui check --model` cannot do yet because its `Model` holds
views, commands and events but no types (`crates/ui/ess-ui-check/src/model.rs:49-55`).

## Implementation notes

In order, each its own pull request:

1. **Model, schema, loader, check.** `Document` gains four optional fields; `ToneBy` gains `tones`;
   expansion resolves it; seven checks; the schema gains the constructs with summary, doc, notes,
   example, group and order (`crates/ui/ess-ui/tests/schema.rs:39` holds every construct to that),
   probably in a new `style` group (`schema.yaml:125-132`); the reference is regenerated with
   `ess ui docs`; the partner-portal example gains `tone_maps`, a `dark` theme and the `theme:`
   line, and its two `status_badge` uses name a map. Files:
   `crates/ui/ess-ui/src/{model,expand}.rs`, `crates/ui/ess-ui-check/src/*`,
   `schemas/ui/ess-ui.schema.yaml`. None of them is in open work.
2. **React.** `assets.rs` (the new file), `emit.rs` (tokens out of the document), `styles.css.tmpl`,
   the badge, icon, button and text templates, `shell.tsx.tmpl` (`data-theme`), and
   `project/main.tsx.tmpl:4` (the import). **This collides with uilab's in-flight
   `ui-react-live-binding`**, which touches `ess-ui-react/src/{lib,assets,emit}.rs`, the runtime
   templates `data.ts`, `actions.tsx`, `answer.ts`, `composites/form.tsx`, `composites/confirm.tsx`
   and `project/{main.tsx,index.html,README.md}`. The overlap is `assets.rs`, `emit.rs` and
   `main.tsx`; start this step after that unit lands, or rebase onto it.
3. **Terminal.** `profile.rs` and `view.rs`. uilab's next unit, `ui-tui-live-binding`, touches
   `ess-ui-tui` `data`, `app`, `lib` and `http`; this step stays out of those files.

## Open questions

1. **Format.** Is an "unknown field `tokens`" refusal from an older reader enough, or should the
   loader name the format, so that ess-ui gets a version marker per construct the way `ess/N` has
   `unsupported_format_version`? This page assumes additive `ess-ui/1`; the maintainer decides.
2. **Colour literal grammar.** Hex and `rgb()` only, or also `hsl()` and named colours? A closed
   grammar keeps a later terminal palette mapping total.
3. **`{token: …}` on nodes.** Is there a node property the editor needs to set directly (a section
   padding, a card radius)? If so, which property, and does `raw:` come with it?
4. **Button emphasis.** Should `ButtonTone` map through the token table too (`primary` to
   `accent`/`on_accent`), or stay a renderer convention over colour tokens?
5. **Several shells.** `chosen_by` reads one shell state name; a shell that does not declare it
   (a sign-in shell) shows `theme.default`. Is a theme that changes when the user signs in
   acceptable?
6. **Emphasis mapping.** The tone-to-modifier table above is a proposal and is not grounded in an
   existing terminal convention.
7. **`preferences:`.** Is a settings page that lists the user's options the reason for the story's
   block? If so it can come later as a list of `shell.<name>` references with labels, over the same
   state.
