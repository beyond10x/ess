---
format: aep.planning-md/3
id: story:a-badge-field-takes-a-tone
kind: story
status: active
title: A field takes a tone, a widget passes its fallback tone, and a tone map may band numbers
summary: 'tone and tone_by on a Field with as: badge are refused as an unknown field, so tone maps cannot reach status columns (0.77.0)'
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
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/enums.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/style.rs
- confidence: cited
  path: crates/ui/ess-ui-check/tests/field_tone.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/primitives/badge.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/tests/field_tone.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/expand.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui/tests/tokens.rs
- confidence: cited
  path: docs/design/ui-style-tokens.md
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:37:38Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:37:39Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A value shown in a collection column or a record field cannot be colored by what it means, and a
toned badge cannot say what a value no map names, or a number in a range, looks like. On ess 0.92.0
(minimal document in the triage's `repro/`):

- A column cannot take a tone: `{field: state, as: badge, labels: order_state, tone_by: order_state}`
  is refused, `` unknown field `tone_by`, expected one of `field`, `name`, `label`, `as`, … ``. The
  document's `tone_maps.order_state` is then reported `tone_map_unused`, because only a badge or
  icon primitive can name it (`docs/design/ui-style-tokens.md` § "Tones and status colors").
- A widget cannot hand its badge the tone of an unmapped value: a body badge
  `tone: args.otherwise`, the param typed `{enum: [neutral, info, success, warning, danger]}`, is
  refused, `data did not match any variant of untagged enum Tone`.
- A tone map keys exact values only: `load: {"0..50": success, "50..": danger}` loads, and `0..50`
  is a literal key no number ever equals, so a numeric column cannot be colored by range.

## Acceptance

- A field (collection column, record field) takes `tone` and `tone_by`, as decided (G-8 answer a):
  `tone_by: <tone map name>` reads the field's own value, `tone` is the tone of a value the map
  does not name (default `neutral`). React draws the value as a badge in that tone when `as: badge`
  and as toned text otherwise; the terminal applies its emphasis table
  (`docs/design/ui-style-tokens.md` § Renderers). `tone_by` naming no map is `tone_map_refs` at the
  field's path, and a map a field names is no longer `tone_map_unused`
  (`crates/ui/ess-ui-check/tests/field_tone.rs`, `crates/ui/ess-ui-react/tests/field_tone.rs`).
- Inside a widget body, `tone: args.<param>` is substituted at expansion when the param is typed as
  the tone enum; any other param type is refused by path.
- A tone map may be written as bands, `{bands: [{from: 0, tone: success}, {from: 50, tone: warning}, {from: 80, tone: danger}]}`:
  a number takes the tone of the last band whose `from` it reaches; below the first band, or a
  value that is no number, takes the node's `tone`. `ess ui check` refuses bands whose `from` is
  not strictly ascending, and a band map named where values are not numbers is not checked
  (no model types; the reference says so).
- `website/docs/reference/ess-ui.md` is a fresh render of the schema; a document without the keys
  generates the same bytes.

## Scope

Re-derived 2026-11-10 by `story-scoper` on the integration tree. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- cited: crates/ui/ess-ui/src/model.rs (`Field` has no `tone`/`tone_by` and is `deny_unknown_fields` at 2370-2403; `ToneMap` is an exact-key `BTreeMap<String, Tone>` at 3316)
- cited: crates/ui/ess-ui/src/expand.rs (`Tones::tone_by` skips a non-mapping `tone_by` at 1532-1535; `declaration_view` reads a widget body with `args.<param>` left in typed positions at 879-905)
- cited: schemas/ui/ess-ui.schema.yaml (`Field` at 927-958, `ToneMap` at 1673-1686, checks `tone_map_refs`/`tone_map_unused` at 1737-1738)
- cited: crates/ui/ess-ui-check/src/style.rs (`tone_map_unused` collects only badge and icon `tone_by` at 363-371; the bands-ascending check belongs here)
- cited: crates/ui/ess-ui-check/src/enums.rs (its unit test fails on a schema enum named in neither `CHECKED` nor `TYPED`, 5-6, 16-31, `TYPED` from 172; `badge.tone_by.map.*` iterates exact keys only at 120-141)
- cited: crates/ui/ess-ui-react/src/emit.rs (the field spec emits no tone at 905-938)
- cited: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl (`FieldSpec` at 10-33; `FieldValue` badge has no tone class at 140-143)
- cited: crates/ui/ess-ui-react/templates/runtime/primitives/badge.tsx.tmpl (exact-key tone lookup that bands must extend at 19-24)
- cited: crates/ui/ess-ui-tui/src/view.rs (`labelled_cell`/`cell_value` return unstyled text at 1574-1592; a badge is always bold at 2265-2271)
- cited: docs/design/ui-style-tokens.md (Fit review item 6: the design gains the field tone and the band form before code)
- cited: website/docs/reference/ess-ui.md (acceptance: a fresh render of the schema)
- cited: crates/ui/ess-ui-check/tests/field_tone.rs (new, named in Acceptance)
- cited: crates/ui/ess-ui-react/tests/field_tone.rs (new, named in Acceptance)
- inferred: crates/ui/ess-ui-check/src/lib.rs (`CHECKS` at 147-150, only if the bands-ascending refusal gets its own check id)
- inferred: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl (`.ui-tone-danger` is the button fill at 68 and no tone has a text-only rule at 103-106, so toned text may need CSS)
- inferred: crates/ui/ess-ui/tests/tokens.rs (loader tests beside the existing `tone_by` tests at 411-470)

confidence: high — every non-test path was read at the cited line; the open points are the three inferred lines and the terminal emphasis table (`ui-style-tokens.md:219-232` is not built in `crates/ui/ess-ui-tui/src`, so `profile.rs` may join)

Corrections to the earlier pass: removed `crates/ui/ess-ui-check/src/rules.rs` (no tone code; the tone-map checks live in `style.rs`) and `crates/ui/ess-ui-check/src/classify.rs` (files `tone_map_refs` by message at `:56`, no edit when the refusal reuses the message at `expand.rs:1570`); added `style.rs`, `enums.rs`, `emit.rs`, the two test files, `lib.rs`, `styles.css.tmpl` and `tests/tokens.rs`; `badge.tsx.tmpl`, `view.rs`, `ui-style-tokens.md` and `ess-ui.md` moved from inferred to cited.

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`.

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items C18, "a widget has no fallback tone" and
"a tone map cannot cover a value range"; also the 2026-10-29 report this story was first drafted
from (a badge field refused `tone_by`).

1. **Need.** A status column, a level column or a widget's badge shows the tone of what its value
   means, from one map per meaning, including values the map does not list and numbers by range.
   Reproductions above (`repro/run.txt`, `repro/widget.txt`). The requester's proposals, theirs:
   "columns take `tone_by`", "a fallback tone", "a tone map over a value range".
2. **Class.** *Gap*, all three: a field has no tone (`schemas/ui/ess-ui.schema.yaml` `Field`), a
   widget param cannot reach `tone`, and `ToneMap` keys are exact strings.
3. **Already expressible?** A column: no; the idiom of an `item:` node holding a toned badge
   primitive draws a second cell beside the column, not the column, and does not sort with it.
   A range: no; the expression grammar has no ordering operator (`schemas/ui/ess-ui.schema.yaml`
   `expressions.forms`: `==, !=, in, not, and, or`). The idiom "the view publishes a band enum" moves
   a presentation choice into the model and is not taken.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). The
   sibling set is every node that shows a value with a tone: badge, icon, and now field; each takes
   `tone` and `tone_by` with the same meaning (`docs/design/ui-style-tokens.md` § "Tones and status
   colors": "a value the map does not name takes the node's `tone`"). `label_maps` do not take
   bands: the words for a band are domain vocabulary a view should publish, while a tone is
   presentation of a number the model already publishes.
5. **Second adopter.** A capacity page: a disk-usage column green below 70%, amber to 90%, red
   above; a job list whose state column follows the job-state tone map.
6. **Cost.** Additive in `ess-ui/1`: two optional keys on `Field`, a second `ToneMap` form,
   `args` substitution into `tone`; no migration. `docs/design/ui-style-tokens.md` gains the field
   tone and the band form before code.
7. **Considered.** (a) Change nothing. (b) Per-value conditions (`tone_if: [{when, tone}]`): needs
   ordering operators in the expression grammar, a larger change across both renderers. (c) Chosen:
   the decided field tone, and bands as a second form of the existing map.

## Decisions

**Accept, redesigned.** Field `tone`/`tone_by` as decided (`approval-record:ess-0870-wave`, G-8
answer a); the fallback tone is the node's existing `tone`, reachable from a widget param; ranges are
a band form of `tone_maps` rather than range-shaped keys. No new format version.

## Source

`github:beyond10x/ess#723`
