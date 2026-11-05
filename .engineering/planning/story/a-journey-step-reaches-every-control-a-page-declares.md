---
format: aep.planning-md/3
id: story:a-journey-step-reaches-every-control-a-page-declares
kind: story
status: active
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
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T18:11:27Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T18:11:27Z", actor: "human:timo", revision: 3}
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
