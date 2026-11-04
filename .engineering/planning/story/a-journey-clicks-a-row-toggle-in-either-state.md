---
format: aep.planning-md/3
id: story:a-journey-clicks-a-row-toggle-in-either-state
kind: story
status: active
title: A journey clicks a row toggle in either state
summary: The ui journey runner matches a row toggle whose hint carries the 0.86.0 state mark ([x] or [ ]); 18 of an adopter's 573 journeys broke on 0.86.0
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T09:03:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-04T09:03:47Z", actor: "human:timo", revision: 3}
---
## Outcome

A ui journey step that clicks a row toggle finds it in the terminal again, in either state, as it did on 0.85.0.

## Context

Since 0.86.0 (b8b53b81da, "a toggle shows the row state it flips"), the terminal renderer writes a row toggle's hint
with its state mark: `k [x] Enabled` or `k [ ] Enabled` (`switch_mark`, crates/ui/ess-ui-tui/src/view.rs). The
journey runner looks an action up by its plain label (`hint_key`, crates/ui/ess-ui-test/src/runner.rs), so it no
longer finds a toggle and fails the step with "the row does not offer <label>".

An adopter reported it against 0.86.0: 18 of its 573 journeys fail this way, and all of them were green on 0.85.0.

## Acceptance

Named tests in `ess-ui-test`; each runs and passes, and each fails on the 0.86.0 runner first.

- `a_journey_clicks_a_row_toggle_that_is_off` - a row whose toggle hint reads `[ ] <label>` is clicked by its
  label and the step passes.
- `a_journey_clicks_a_row_toggle_that_is_on` - the same with `[x] <label>`.
- `a_row_action_whose_label_starts_with_brackets_still_matches_exactly` - an action that is not a toggle is
  matched by its whole label as before, and a toggle's mark never makes a different action match.
- Every existing `ess-ui-test` test passes unchanged.

## Out of scope

- Changing the rendered hint: the state mark stays, as 0.86.0 decided.

## Schedule

Ride-along of the 0.87.0 wave (approval-record:ess-0870-wave), on the orchestrator's word of 2026-11-04. The
CHANGELOG names it as a fix of the 0.86.0 regression.
