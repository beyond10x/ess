---
format: aep.planning-md/3
id: story:terminal-header-toggles-show-their-state
kind: story
status: active
title: Terminal header toggles show their state, and toggle row paths are checked
summary: The terminal draws header toggles without a mark, and checked:/bind row paths are not checked against the view (UI-adopter wave 1 U2 review note, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:w086-u2-adversary-pass-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/names.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:58Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:00Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Two gaps around toggles, pre-existing at 83391ebaeb and still present on 0.92.0, found by the
UI-adopter wave 1 adversary pass on the toggle unit (review-result:w086-u2-adversary-pass-1, note):

- The terminal draws a header toggle with no on/off mark (`crates/ui/ess-ui-tui/src/view.rs`,
  `header_lines`, the header actions near `:440-467`, which format every action as `[<label>]`),
  while a row toggle shows `[x]`/`[ ]` through `switch_mark`.
- A toggle's `checked:` expression and its bind entries may name a `row.<field>` the view does not
  return; `ess ui check --model` compares only a row action's `visible` with the view's fields
  (`crates/ui/ess-ui-check/src/names.rs`, `Known::row_paths`).

Reproduction of the first gap (0.92.0): a `list_page` with page state
`live: {type: boolean, class: page_state, default: true}`, a header action
`{name: live, does: items.SetLive, bind: {on: not state.live}, checked: state.live, as: toggle, label: Live}`
and a collection whose row action is
`{name: on, does: items.SetOn, bind: {id: row.id, on: not row.nope}, checked: row.nope, as: toggle, label: On}`.

```
$ ess ui test --path ui.yaml t.yaml
passed  the row toggle shows its mark                      # expect text "[ ] On" at the section
failed  the header toggle shows its mark (step 2)          # expect text "[x] Live" at pages/items/header
        expected "[x] Live" at pages/items/header; it shows:
        Items (1)
        [Live] (: to run)
```

## Acceptance

- The terminal shows a header toggle's state the way it shows a row toggle's: the reproduction's
  header draws `[x] Live` while `state.live` is true and `[ ] Live` after the toggle is acted on, and
  `act: pages/items/header/actions/live` still sends `items.SetLive`.
- `ess ui check --model` reports a `checked:` or bind `row.<field>` path the read view does not
  return as `field_in_model`, at the action's path; a path the view returns is not reported.

## Scope

- cited: `crates/ui/ess-ui-tui/src/view.rs` (`header_lines`, header actions; `switch_mark`)
- cited: `crates/ui/ess-ui-check/src/names.rs` (`Known::row_paths`, extended to `checked` and `bind` values)
- inferred: `crates/ui/ess-ui-test/src/runner.rs` (only if a header hint with a mark changes how `act` finds it; today it runs `:do: <label>`)
- inferred: `crates/ui/ess-ui-test/tests/`, `crates/ui/ess-ui-check/tests/checks.rs`
- inferred: `CHANGELOG.md`

confidence: high

## Schedule

Waved under epic:zero-open-issues-2026-11 (accepted 2026-11-10).

