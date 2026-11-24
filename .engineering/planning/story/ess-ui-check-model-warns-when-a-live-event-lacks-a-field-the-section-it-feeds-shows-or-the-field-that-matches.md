---
format: aep.planning-md/3
id: story:ess-ui-check-model-warns-when-a-live-event-lacks-a-field-the-section-it-feeds-shows-or-the-field-that-matches
kind: story
status: draft
title: ess ui check --model warns when a live event lacks a field the section it feeds shows, or the field that matches its row
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-per-value-channel-script-that-names-no-session-is-reported
revision: 1
---
## Need

A section shows fields of the view it reads, and a `live` block applies a channel's events to
those rows. An event of the model declares its own payload fields. When an event carries fewer
fields than the section shows, a row it inserts has empty cells for the missing ones. When it
lacks the field that identifies a row, `patch_row` and `remove_row` never find a row and
`insert_or_patch` inserts on every event. `ess ui check --model` says nothing about either.

Reported by a downstream application on 0.98.0 (third list, item 2): an event type carries three
fields fewer than the section's columns read, and the check passes.

What the checker holds today with `--model` (`crates/ui/ess-ui-check/src/lib.rs:176-215`;
reference table `website/docs/reference/ess-ui.md:2868-2884`):

- `event_in_model`: an event named in `channels.*.carries` or `live.on` resolves in the model
  (`crates/ui/ess-ui-check/src/model.rs:424-432`). It checks the name only.
- `field_in_model`: columns, record fields and `row.<field>` paths are fields of the view read
  (`crates/ui/ess-ui-check/src/names.rs:1-9`, `:171-232`).
- `row_fields`: `group_by`, `label_from`, a choice's `value`/`label`, `title_from.field` are row
  keys by wire name (`model.rs:897-960`).

No check relates an event's fields to a view's fields. The model has both sides:
`ResolvedEvent.fields` (`crates/specify/ess-compiler/src/ir.rs:2284-2299`) and the view's fields
and wire names (`model.rs:133`, `:302`). The checker keeps only event names today
(`model.rs:97`, `:320`).

What links an event to a row today: `live.match`, which is "row identity field; absent, the
read's `key`, else `id`" (`ess-ui.md:2316`; `crates/ui/ess-ui/src/model.rs:3341-3343`). Both
renderers merge the event's payload into the matched row field by field, or insert the payload
as the row (`crates/ui/ess-ui-tui/src/app.rs:4947-4988`;
`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:404-427`). So the payload's keys must be
the row's keys, by wire name.

Minimal reproduction (fresh, not run here): a model with a view `orders.Lines` (fields
`line_id`, `sku`, `qty`, `price`, `note`) and an event `orders.LineAdded {line_id, qty}`. A
section `lines` is a `collection` over `orders.Lines` with `columns: [line_id, sku, qty, price,
note]` and `live: {channel: lines, effect: insert_top}`, and the channel `lines` carries
`orders.LineAdded`. `ess ui check --model` reports `0 error(s)` for this. A played `LineAdded`
row shows `sku`, `price` and `note` empty in both renderers.

## Outcome

With `--model`, `ess ui check` reports a new warning, `live_event_fields`, at
`<section or node>/live`. It fires for each event the block takes (`live.on`, or every event the
channel carries) when:

- the effect inserts a row (`insert_top`, `insert_or_patch`, `replace`) and a field the section
  shows is missing from the event;
- the effect looks a row up (every effect except `refetch` and `replace`) and the event lacks the
  match field.

The message names the event, the fields it lacks, and what the reader sees: "a row it inserts
shows `sku`, `price`, `note` empty", or "it never finds a row: `orders.LineAdded` carries no
`line_id`". A channel that carries a view (`carries: {view}`) delivers whole rows of that view and
is not checked.

## Acceptance

- First: the reproduction, as a test in `crates/ui/ess-ui-check/tests/` with a model of its own,
  reports today `0 error(s), 0 warning(s)`. After the change it reports one
  `live_event_fields` warning at `pages/<p>/sections/lines/live` naming `sku`, `price`, `note`, and
  the exit status stays 0.
- The same section with `effect: patch_row` and an event that carries `line_id` gives no finding:
  a patch sets what it carries.
- With `effect: patch_row` and an event without `line_id`, there is one finding naming the match
  field. With `match: sku` and an event that carries `sku`, there is none.
- A nested composite's `live` (`model.rs:1240-1244`) is checked the same way at its own path.
- A field renamed on the wire is compared by its wire name on both sides, and a test pins it.
- With `refetch`, or a channel that carries a view, there is no finding.
- `schemas/ui/ess-ui.schema.yaml` lists the check (warning) beside `event_in_model`. The generated
  `ess-ui.md` tables for `Live` and the check list show it, and `CHANGELOG.md` names it.
- The repository's examples check with no new finding, or each new finding is named in the
  change. `examples/partner-portal` has no model, so it is not reached.

## Fit review

Verdict: **adopt, as a check (gap)**. Tree `bcfc51555`, read-only.

1. Need: a live event fills the row it inserts, and finds the row it patches. Reproduction above.
   The requester asked only for a warning.
2. Class: gap. Nothing checks the event against the view, and both sides are in the model.
3. Already expressible: no. `event_in_model` resolves the name only.
4. Fit: a check over names the document already writes, no new key. It belongs with
   `field_in_model` and `row_fields` (names.rs, model.rs) and compares by wire name, as
   `row_fields` does. The sibling of this check is `script_session` from
   `story:a-per-value-channel-script-that-names-no-session-is-reported` (P1 item 7,
   its fit review). Both are "fixture and live declarations checked
   against what the section reads": that story checks a fixture script's session against the read's
   params, and this checks a model event's fields against the read's view. Both add an id to
   `CHECKS` (`lib.rs`) and to the schema's check list, so they land one after the other.
5. Second adopter: any document whose events are narrower than its views. ESS events are
   commonly deltas, and views are projections.
6. Cost: one warning id in `ess-ui-check/1` findings, with no format version (findings'
   `check` values grow, as that story records). A document that passes today keeps exit 0, because
   `ess ui check` fails on errors only (`lib.rs:375`).
7. Considered: (a) change nothing; the empty cells then show only at run time. (b) An error: it
   would fail documents that pass today, and an `insert_or_patch` fed by patch-only events is
   legitimate. (c) A mapping from event fields to row fields: that is the draft
   `story:a-live-block-applies-several-effects-and-names-the-row-fields-an-event-sets`
   (second list, item 3, of an earlier consumer report). When that mapping lands, this check
   reads the mapped names and does not change shape. Chosen: the warning.

Covers report item 2 of the third list. The generic feature is "a live block is checked against
the view it feeds".

## Decisions

- **SP-D2. Severity and reach of the new check.** (A) A warning with `--model`: the shown fields
  for inserting effects, and the match field for effects that look a row up. (B) An error: it
  fails documents that pass today and needs the next `ess-ui` decision. (C) (A), plus the same
  rule over fixture-script payloads without `--model`, under the same id. That reaches
  placeholder documents but warns about partial demo payloads. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-check/src/model.rs` (keep event fields beside `events`, `:97`, `:320`)
- cited: `crates/ui/ess-ui-check/src/names.rs` (the check, beside `composite_fields`, `:230`)
- cited: `crates/ui/ess-ui-check/src/lib.rs` (`CHECKS`, beside `event_in_model`, `:181`)
- cited: `schemas/ui/ess-ui.schema.yaml` (checks list, `:1813`), `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- inferred: `crates/ui/ess-ui-check/tests/live_event_fields.rs` (new) with a small model
- Size: M (about 300 changed lines). Hotfix-sized: no (a new check).

## Depends on

- Land after or beside `story:a-per-value-channel-script-that-names-no-session-is-reported`
  (same `CHECKS` list and schema check list). Not a hard dependency.
