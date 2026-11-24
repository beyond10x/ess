---
format: aep.planning-md/3
id: story:a-page-header-s-badge-takes-its-tone-and-words-from-its-value-as-a-badge-does
kind: story
status: draft
title: A page header's badge takes its tone and words from its value, as a badge does
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A detail page shows the record's state in its header badge, `header.badge.text:
section.summary.row.state`. The badge has one fixed tone for every value, so a failed record and a
finished one share a colour role. The state column of the list page tones the same value by a tone
map. The header badge is the one place a value is shown that cannot be toned by it.

- `HeaderBadge` is `{text, tone}` (`crates/ui/ess-ui/src/model.rs:1826-1834`; schema
  `schemas/ui/ess-ui.schema.yaml:735`; reference `website/docs/reference/ess-ui.md` § header, the
  `badge` row).
- The badge primitive has `text`, `field`, `tone`, `tone_by: {value, map | tones}` and `labels`
  (`model.rs:2365-2393`; reference § badge, `ess-ui.md:1776`, `:1785`). A column has `tone`,
  `tone_by: <tone map name>` and `labels` (`model.rs:2529-2536`).
- React draws the header badge with `props.badge.tone ?? "neutral"`
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:717-719`). The terminal does the same
  (`crates/ui/ess-ui-tui/src/view.rs:489-493`).

Minimal reproduction (fresh, not run here):

```yaml
header:
  title: Order
  badge: {text: section.summary.row.state, tone_by: {value: section.summary.row.state, tones: order_state}}
```

`ess ui load` refuses `tone_by` as an unknown field of the header badge.

The requester's proposal, restated: `header.badge.tone_by`, written
`badge: {text: section.summary.row.state, tone_by: order_state}`.

Only the tone half of the second report's G11 is here. A colour of its own per value, outside the
five tones, is styling and is settled in SL-D10 (declined under the owner's word).

## Outcome

`header.badge` takes the badge primitive's `tone_by` and `labels`, with their meaning unchanged.
`tone_by.value` reads the same expressions `text` reads. A value the map does not name takes
`tone`, else `neutral`. `labels` gives the words shown, and the tone is still chosen by the raw
value. Both renderers draw it.

## Acceptance

- Red first, in `crates/ui/ess-ui/tests/`: the header above loads. It fails before the change.
- In `crates/ui/ess-ui-react/tests/page_header.rs`, with `tone_maps: {order_state: {failed:
  danger, done: success}}` and a first row in state `failed`, the badge carries `ui-tone-danger`.
  In state `done` it carries `ui-tone-success`. In a state the map does not name, it carries the
  badge's `tone`, else `ui-tone-neutral`.
- The terminal draws the tone's emphasis for the same rows (`crates/ui/ess-ui-tui/tests/page_header.rs`).
- `labels: {failed: Failed}` shows `Failed` and keeps `ui-tone-danger`.
- `tone_by` naming no `tone_maps` entry is refused at its path, as on a badge. `tone_by` with both
  `map` and `tones` is refused, as on a badge.
- The reference's header `badge` row lists `tone_by` and `labels`.

## Fit review

Tree `bcfc51555`, read-only. Second report, G11 (tone half).

1. **Need.** A value carries the same semantic tone (one of the five) wherever it is shown, the
   header included.
2. **Class.** *Gap*: a sibling lacks what the badge primitive and the column already have.
3. **Already expressible?** Only with a host slot in place of the header badge, or by dropping
   the badge for a section with a badge primitive.
4. **Fit.** No grid applies. The siblings rule of the skill: a capability already on the badge
   primitive and the column is extended to the header badge, in the badge primitive's spelling
   (`{value, map | tones}`). The requester's bare `tone_by: <map>` is the column's spelling. It
   would read the value from `text`, but `text` may be literal. The badge's explicit `value` is
   taken instead.
5. **Second adopter.** A ticket page shows its priority in the header, toned by the same map as
   the priority column.
6. **Cost.** Two optional keys in `ess-ui/1`. Without them a document draws the same thing. No
   version (`ess-ui.md` precedent: the header badge itself was added within `ess-ui/1` by
   `story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page`).
7. **Considered.** (a) Change nothing. (b) The column's short spelling. (c) Chosen: the badge's
   record.

## Decisions

**Accept, redesigned.** The requester's spelling is changed to the badge primitive's record.
Colour per value is declined (SL-D10).

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`HeaderBadge` 1826-1834; reuses `ToneBy`, `Labels`)
- cited: `schemas/ui/ess-ui.schema.yaml` (735)
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (717-719)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (489-493)
- inferred: `crates/ui/ess-ui/src/expand.rs` (resolves `tones` to `map`, as for a badge),
  `crates/ui/ess-ui-react/src/emit.rs`, tests named above, `website/docs/reference/ess-ui.md`
- Size: S, about 200 changed lines.

## Depends on

Nothing. It edits `core.tsx.tmpl` and `view.rs` in the header functions, away from `formatValue`
and `cell_value`, so it can be built beside the value-format stories, merged in order.

## Source

A downstream application, 2026-11-24, second report, G11 (tone half).
