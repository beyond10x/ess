---
format: aep.planning-md/3
id: story:a-time-date-or-timestamp-field-declares-whether-it-is-drawn-in-utc-or-the-reader-s-local-zone
kind: story
status: draft
title: A time, date or timestamp field declares whether it is drawn in UTC or the reader's local zone
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
- depends_on: story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer
revision: 1
---
## Need

The reference defines `time` and `timestamp` in UTC: "`time` shows the time of day and
`timestamp` the date and the time, both in UTC (the locale's order in React, ISO order in the
terminal)", and "a form field `timestamp` edits an instant in UTC"
(`website/docs/reference/ess-ui.md:1505`). A back-office screen used at one site, whose other screens
show local wall-clock time, cannot say "local". An application that draws local time anyway departs
from the format's meaning.

- React fixes the zone to UTC (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:784-803`,
  `timeZone: "UTC"` at 794 and 797).
- The terminal computes the instant in UTC (`crates/ui/ess-ui-tui/src/view.rs:1865-1885`, `instant`
  at 1965-1980). The terminal crate has no time zone dependency (`crates/ui/ess-ui-tui/Cargo.toml`).
- `Field` has no zone key (`crates/ui/ess-ui/src/model.rs:2500-2546`).

Confirmed as reported. It is not a defect: the reference promises UTC, and both renderers keep it.

Minimal reproduction (fresh, not run here): `{field: ended_at, as: timestamp}` over
`2026-01-02T23:30:00Z` reads `2026-01-02 23:30` for a reader in UTC+1, whose wall clock read
`2026-01-03 00:30`.

The requester's proposal, restated: a document or a field chooses the zone, `zone: local`.

## Outcome

A field `as: time`, `as: date` or `as: timestamp` may declare `zone: utc` (the default, today's
meaning) or `zone: local`, the zone of the machine the renderer runs on. A local value shows and
edits in that zone, and a form still sends the instant. The reference states both. The same
document draws the same text in both renderers when they run under one zone, and a test pins it
under a fixed zone.

## Acceptance

- Red first: `ess ui load` accepts `{field: ended_at, as: timestamp, zone: local}`. Today it is
  refused as an unknown field of `Field`. Case in `crates/ui/ess-ui/tests/`.
- Under the zone `Europe/Berlin` set for the test process (`TZ` for node and for the terminal
  test), `2026-01-02T23:30:00Z` with `zone: local` renders a date of `2026-01-03` and a time of
  `00:30` in React (`crates/ui/ess-ui-react/tests/value_formats.rs`) and `2026-01-03 00:30` in the
  terminal (`crates/ui/ess-ui-tui/tests/value_formats.rs`). Without `zone`, both render 23:30 UTC,
  which is unchanged.
- A date across a daylight-saving change renders the zone's offset of that date, not today's.
- A form field `as: timestamp, zone: local` edits local wall-clock time and sends the instant in
  UTC (`Z`).
- `zone` on a field whose `as` is not `time`, `date` or `timestamp` is refused at its path.
- The `ess-ui/1` reference lists `zone`, and the `as` note says "in UTC unless the field declares
  `zone: local`".

## Fit review

Tree `bcfc51555`, read-only. Second report, G6.

1. **Need.** Which zone an instant is drawn in is part of what the value means to its reader. A
   site's back office reads local time. Reproduction above.
2. **Class.** *Gap*. The reference fixes UTC.
3. **Already expressible?** No. A model view cannot shift an instant into a reader's zone: the
   reader's zone is not the server's.
4. **Fit.** No grid applies. One optional key beside `unit` and `scale`, held by the same "only on
   such a field" check (`crates/ui/ess-ui/src/path.rs:499-518`). A named zone (`Europe/Berlin`) is
   not taken: the need is "the reader's zone", and a fixed named zone is a third meaning with its
   own database in both renderers.
5. **Second adopter.** A shop's back office shows order times in the shop's local time on the
   counter terminal.
6. **Cost.** One optional key in `ess-ui/1`. Without it a document keeps its meaning and bytes, so
   no new version. The terminal needs the local offset of an instant. That is a new dependency, a
   zone database reader such as `jiff`, or reading the system's zone file. Either way it is pinned
   in `Cargo.toml` and the lock file. A renderer is not a generator, so the generator timing rule
   does not apply.
7. **Considered.** (a) Change nothing: the application departs from the format. (b) A document-wide
   default zone (SL-D5 B). (c) IANA zone names. (d) Chosen: `zone: utc | local` per field.

## Decisions

**Accept, redesigned.** Per field, two values, as requested. The document-wide form is put to the
operator.

- **SL-D5. Where is the zone chosen?** (A) Per field only, with a widget or page kind carrying it
  where it repeats. (B) Per field and as a document default. (C) Also IANA zone names.
  **Recommended: A.** It is the smallest form, and B can be added later without changing A.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs` (`Field`), `crates/ui/ess-ui/src/path.rs` (beside `check_unit`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Field`, the `as` note)
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (784-803), `fields.tsx.tmpl` (the timestamp input)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`cell_value` 1865-1885, `instant` 1965-1980), `crates/ui/ess-ui-tui/Cargo.toml`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`, `crates/ui/ess-ui-tui/src/app.rs` (the form's timestamp edit),
  tests named above, `website/docs/reference/ess-ui.md`
- Size: M, about 420 changed lines.

## Depends on

`sl03-number-as-written` and `sl04-list-separator` (the same `formatValue` and `Field`).

## Source

A downstream application, 2026-11-24, second report, G6.
