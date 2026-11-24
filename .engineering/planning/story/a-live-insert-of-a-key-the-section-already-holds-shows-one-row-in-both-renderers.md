---
format: aep.planning-md/3
id: story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers
kind: story
status: draft
title: A live insert of a key the section already holds shows one row, in both renderers
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A section with `live: {effect: insert_top}` that receives an event for a row whose key it
already holds inserts the row a second time. Two rows then share one key, and so one row path:
`ess-ui-test/1` counts and finds rows by their keys (`website/docs/reference/ess-ui-test.md:410`),
so the test sees the same key twice. With a fixture script that loops, every cycle replays the
same payloads and the section grows by the script's inserts each time.

Reported by a downstream application on 0.98.0 (third list, item 1), with a reproduction: a
page opened, `advance: 30s`, `expect: {at: pages/<page>/sections/<section>, rows: 6}`;
`ess ui test` fails with "it shows 9 [5, 4, 6, 5, 4, 6, 1, 2, 3]" (three inserted keys twice,
above the three fixture rows). The React runtime behaves the same.

The shipped example has the same shape: `examples/partner-portal/fixtures/scripts/activity.yaml`
loops and feeds two `insert_top` sections (`examples/partner-portal/ui.yaml:219`, `:257`).

What the reference says:

- `effect` "decides what an event does: patch a row, insert or patch, insert at the top of a
  feed, remove, replace, or re-read" (`website/docs/reference/ess-ui.md:2307`); the table names
  the six values (`:2315`); `max_rows` is the "cap for insert_top feeds" (`:2320`). The schema says
  the same (`schemas/ui/ess-ui.schema.yaml:1418`, `:1423`), and the model's doc comments are
  "Insert or patch." and "Insert at the top of a feed." (`crates/ui/ess-ui/src/model.rs:3360-3363`).
- Nothing says what `insert_top` does with a key already held. `insert_or_patch` patches a held
  row in place and inserts a new key at the top.

Where the insert is:

- Terminal: `apply` (`crates/ui/ess-ui-tui/src/app.rs:4957-4988`). `InsertOrPatch` looks the key
  up and merges or inserts (`:4970-4973`); `InsertTop` inserts at 0 without looking, then caps by
  `max_rows` (`:4974-4979`). Before `apply`, the section and nested-node paths judge `only_if`
  and the read filter only for a new key (`inserts` at `:2497` and `:2286`).
- React: `apply` (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:404-427`). `insert_top`
  prepends without looking (`:410-413`); `insert_or_patch` looks up `index` and merges or prepends
  (`:414-418`). The filtered path judges a new key at `:482`.
- The test runner drives the terminal `App` (`crates/ui/ess-ui-test/src/runner.rs:176-180`), so it
  inherits the terminal's behaviour.

Hotfix 0.98.1 fixes this on the released tag (brief `hotfix-0981`). This story records the
defect and its acceptance in the plan store; it adds nothing beyond the hotfix.

## Outcome

A section never shows two rows with one key. An `insert_top` of a held key merges the event into
the held row and moves it to the top, then `max_rows` caps the rows. A new key is inserted at the
top as before. The terminal, the test runner and the React runtime give the same rows and order
for the same script. The reference and the changelog state the rule.

## Acceptance

- First, red on 0.98.0 and green after: the hotfix's `ess-ui-test` target
  `crates/ui/ess-ui-test/tests/live_insert.rs` (tests as written in the hotfix tree on 2026-11-24:
  `a_looping_script_that_inserts_a_held_key_again_moves_its_row_to_the_top`,
  `an_insert_of_a_fixture_row_s_key_moves_that_row_to_the_top_with_the_event_s_values`,
  `the_cap_holds_after_a_held_key_moves`). A looping script re-inserts held keys within one
  `advance`; the test expects the row count and the exact key order, and on 0.98.0 it showed the
  duplicated keys.
- React: `an_insert_top_of_a_held_key_moves_that_row_to_the_top_once` in
  `crates/ui/ess-ui-react/tests/read_filter.rs` holds the runtime's `apply` to the same rule.
- `insert_or_patch` is unchanged: a held key is patched in place.
- The schema's `Live` text, which `website/docs/reference/ess-ui.md` is generated from, says what
  `insert_top` does with a held key, and `CHANGELOG.md` 0.98.1 has a `### Fixed` entry.

## Fit review

Verdict: **adopt (defect)**. Tree `bcfc51555` (`integration/wave-2026-11-24`), read-only; the
insert code is the same as on `0.98.0` (no diff in these files).

1. Need: a row path names one row. Reproduction above, as the reporter's run shows; the hotfix
   adds a fresh generic one.
2. Class: defect. `ess-ui-test` counts and finds rows by key (`ess-ui-test.md:410`), and an
   effect that inserts a duplicate key breaks that.
3. Already expressible: `insert_or_patch` avoids the duplicate but puts a held row where it is,
   not on top. That is the workaround, and it is not what a feed means.
4. Fit: no new key, no format change; one behaviour in two renderers, the same as
   `insert_or_patch` for a held key except where the row goes.
5. Second adopter: the shipped `partner-portal` activity feed (`ui.yaml:219`, `:257`) with its
   looping script.
6. Cost: a documented behaviour change for a held key only. No `ess-ui/1` version.
7. Considered: "stays in place" makes `insert_top` equal to `insert_or_patch`; "refuse the event"
   loses the event's values. The hotfix chose move-to-top because the reference is silent
   (hotfix `STATE.md`).

Covers report item 1 of the third list. The generic feature is "a live effect keeps one row per
key".

## Decisions

- **SP-D1. Should a fixture script that loops and re-inserts held keys be reported?**
  (A) No report: after 0.98.1 a held key's re-insert moves the row and keeps one row per key,
  which is defined and harmless. The `loop` sentence of `FixtureIndex` says that every cycle
  replays the same payloads. (B) `ess ui check` warns, under a new id `script_reinserts_key`,
  about a looping script that feeds an `insert_top` section. By construction this fires on every
  such script, the shipped example included. (C) `ess ui test` prints a note line (text report
  only, `ess-ui-test-report/1` unchanged) when a looping script first replays an insert of a held
  key. **Recommended: (A).**

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (`apply`), `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (`apply`)
- cited: `schemas/ui/ess-ui.schema.yaml` (`Live` doc), `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`, `release-notes/0.98.1.md`
- cited: `crates/ui/ess-ui-test/tests/live_insert.rs` (new), `crates/ui/ess-ui-react/tests/read_filter.rs`
- Size: S (about 150 changed lines with tests). Hotfix-sized: yes, done as 0.98.1.

## Depends on

- None. `sp4-event-before-read` depends on this story.
