---
format: aep.planning-md/3
id: story:a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end
kind: story
status: draft
title: A duration field shows the time between two instants of its row, the clock standing in for an absent end
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone
revision: 1
---
## Need

A support desk lists each ticket's wait, the time from `opened_at` to `answered_at`, which grows
with the clock while the ticket is unanswered. It also shows the handling time, from `answered_at` to
`closed_at`, which grows while the ticket is open. A field shows one value of its row. Nothing derives a
value from two fields and the clock, so each such column becomes a host slot.

- `Field` reads one `field` (`crates/ui/ess-ui/src/model.rs:2500-2546`). `as: duration` shows one
  number counted in `unit` (`website/docs/reference/ess-ui.md:1505-1506`).
- A model view field may compute `ended_at - started_at` (`expr:`, `ess/27`). `now` is refused as a
  value there, and in a view field from `ess/30` on (`website/docs/reference/spec-versions.md`, rows
  `ess/27` and `ess/30`). A view value is also fixed when the view is read, so it cannot grow.
- UI expressions have no arithmetic (`ess-ui.md:51-74`).
- Both renderers already keep a clock: React for refresh and live staleness
  (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:908-915`, `live.ts.tmpl:269-298`). Tests
  run both on one virtual clock (`crates/ui/ess-ui-test/src/clock.rs`; Playwright's page clock
  installed at zero, `crates/ui/ess-ui-test/src/playwright.rs:340`).

Minimal reproduction (fresh, not run here): rows `{opened_at: 1000, answered_at: null}` and
`{opened_at: 1000, answered_at: 4000}`, epoch milliseconds. No column can show `0:00:03` for the
second row and a growing time for the first.

The requester's proposal, restated: a field shows a duration between two fields, the clock
standing in for an unset one, written `{name: wait, between: [opened_at, answered_at], unset: now,
as: duration}`.

## Outcome

A column or record field `as: duration` may declare `since: <row field>`. It then shows the time
from the instant in `since` to the instant in `field`, both epoch milliseconds or ISO text, as
`h:mm:ss`. When `field` is absent, the field reads the clock and the shown value grows each second
while it is on screen. When `since` is absent, the cell is empty. Both renderers draw it, and an
`ess-ui-test/1` test pins it on the virtual clock.

## Acceptance

- Red first: `ess ui load` accepts `{name: wait, field: answered_at, since: opened_at, as:
  duration}`. Today it is refused as an unknown field. Case in `crates/ui/ess-ui/tests/`.
- Over the rows above, with the virtual clock at 61000 ms, the second row reads `0:00:03` and the
  first `0:01:00`. After the clock advances 5 s, the first reads `0:01:05`
  (`crates/ui/ess-ui-react/tests/value_formats.rs` with a fixed `Date.now`, and
  `crates/ui/ess-ui-tui/tests/value_formats.rs` through `App::advance`).
- An `ess-ui-test/1` case with a `play` step asserts the growing cell in both renderers through
  the existing `text` check.
- A row with no `since` value shows an empty cell. An instant that does not parse shows the raw
  value, as `as: timestamp` does.
- `since` on a field whose `as` is not `duration`, `since` beside `unit`, and `since` on a form
  field are refused at the field's path. With `--model`, `since` naming a field the view lacks is
  `row_fields`.
- The `ess-ui/1` reference lists `since` and says the clock stands in for an absent `field`.

## Fit review

Tree `bcfc51555`, read-only. Second report, G10.

1. **Need.** An elapsed time that is still running is a value about the row and the present. It
   must read the same in every renderer.
2. **Class.** *Gap*.
3. **Already expressible?** Half. Two present instants: the model view's `expr:` (`ess/27`). The
   running half: no. `now` is refused in a view field, and a view value does not grow.
4. **Fit.** No grid applies. It reuses `as: duration` and `field`. One key names the other instant.
   The model's `expr:` is the spelling of a derived value in the model. This key is narrower: the
   clock, which only a renderer has. So it does not repeat the model's construct. A general UI
   arithmetic (`answered_at - opened_at`) is not added: it would make the display expression a
   second expression language.
5. **Second adopter.** A parcel tracker shows the time in transit from `dispatched_at` to
   `delivered_at`, growing while the parcel travels.
6. **Cost.** One optional key in `ess-ui/1`. Without it a document keeps its meaning, so no new
   version. A per-second redraw while such a cell is shown: React re-renders only the cells that
   read the clock, and the terminal redraws on its existing tick.
7. **Considered.** (a) Change nothing: a host slot per column. (b) UI expression arithmetic with
   `now()`. (c) The requester's `between: [a, b], unset: now`: it names a list and a mode where one
   key and the existing `field` say the same. (d) Chosen: `since`.

## Decisions

**Accept, redesigned.** `field` is the end and `since` the start. Only an absent end reads the
clock.

- **SL-D6. How is a duration from two instants and the clock spelled?** (A) `since: <field>` on an
  `as: duration` field. An absent `field` reads the clock. (B) UI expression arithmetic with a
  `now()` form, in every `expr` position. (C) Decline: the application keeps a host slot per column.
  **Recommended: A.**

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`Field`), `crates/ui/ess-ui/src/path.rs` (beside `check_unit`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Field`)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` (`duration` case 185-189)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`cell_value` 1895-1898)
- inferred: `crates/ui/ess-ui-react/src/emit.rs`; a clock hook in `core.tsx.tmpl`;
  `crates/ui/ess-ui-check/src/model.rs` (`row_fields` for `since`); an `ess-ui-test` case;
  `website/docs/reference/ess-ui.md`
- Size: M, about 520 changed lines.

## Depends on

`sl06-time-zone` (the same `Field`, `path.rs` checks and the instant reader, which this story
reuses).

## Source

A downstream application, 2026-11-24, second report, G10.
