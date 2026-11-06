---
format: aep.planning-md/3
id: story:a-journey-step-reaches-every-control-a-page-declares
kind: story
status: implemented
title: A journey step reaches every control a page declares
summary: ess ui test steps for section forms, toggles, multiple choices, row selection, filter windows, expand, sort, reorder, upload, URL page state and filter choices (beyond10x/ess#722 items 1-11)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T18:11:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T18:11:27Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-06T07:40:06Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Need

A mutation audit of `ess ui test` journeys against the terminal profile
(https://github.com/beyond10x/ess/issues/722, on 0.84.0, items 1 to 11) found controls a page declares
that no journey step can reach, so the commands behind them are never pinned. A downstream adopter's
journeys hit three more. The steps live in `crates/ui/ess-ui-test` (`spec.rs`, `runner.rs`) and the
terminal side in `crates/ui/ess-ui-tui` (`app.rs`, `keys.rs`, `view.rs`).

## Acceptance

Each bullet is a named journey test, red before the change, that drives the control and asserts the
command it sends:

- `a_section_form_submits` (#722 item 1): a form placed on a page section with `save: on_submit`.
- `an_on_change_section_form_sends_each_change` (item 2).
- `a_step_sets_a_form_toggle` (item 3): an untouched toggle is sent as its declared default, not
  absent.
- `choose_takes_a_multiple_choice_and_long_labels` (item 4).
- `a_step_selects_rows_for_a_bulk_action` (item 5), including its confirm.
- `a_step_sets_a_filter_window` (item 6).
- `a_step_expands_a_row` (item 7).
- `a_step_sorts_by_a_column_header` (item 8).
- `a_step_reorders_a_row` (item 9).
- `a_step_uploads_a_file` (item 10): the terminal profile takes a file path and sends the command.
- `open_takes_url_page_state` (item 11).
- `export_params_are_assertable`.
- `a_step_sets_a_filter_bar_choice`.
- `a_step_chooses_in_a_section_level_choice`.

## Out of scope

#722 items 12 to 32, which reach overlays, drawers and row items; the React target's own controls.

## Decisions

The coordinator answered the adversary pass's two questions on 2026-11-05:

- **Reorder payload.** The ESS specification's declared command input is the contract. When the
  specification declares the input of the command a reorder sends, both targets send that input.
  When it is silent, the terminal sends what the generated React app sends, `{id, delta}` with
  `delta` -1 or 1, because that is what the exported browser spec runs.
- **An untouched toggle.** The acceptance says "not absent", so both targets send a never-touched
  toggle as its declared default, `false` when none is declared. The React form template is fixed in
  this wave.

The same correction fixes the four introduced findings: a filter-bar or form action keyed `h` or `l`
is reached before the move keys, an uploaded file's leading byte order mark is dropped, and a file
that is not valid UTF-8 is decoded lossily as the browser does. A plain `open` after an `open` with
URL state shows no URL state, if the fix is small; otherwise it is a backlog draft.

## Review

Adversary pass 1 (2026-11-05): NEEDS-CHANGE, 4 introduced findings (actions keyed `h` or `l`, an upload's byte order mark, an upload that is not UTF-8) and 3 undecided (reorder payload, untouched toggle, URL state after a plain `open`), decided as recorded above. One correction fixed all seven; the pass's 12 cases are committed and green beside `journey_controls` (16 passed). Out of scope, filed: story:a-form-groups-own-on-change-saves-in-every-target and story:a-form-groups-own-action-runs-from-its-key.
