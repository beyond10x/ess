---
format: aep.planning-md/3
id: story:act-on-a-row-overlay-form-that-saves-on-change-is-refused
kind: story
status: draft
title: Act on a row overlay form that saves on change is refused
summary: 'Follow-up of story:a-form-groups-own-on-change-saves-in-every-target: act at an overlay opened from a row action, whose form saves on change, is not refused (adversary pass 2, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:a-form-groups-own-on-change-saves-in-every-target` makes `ess ui test` refuse `act` on a
form whose own `save` is `on_change`, in a section and in an overlay, because the generated form
draws no submit. `parity::submit` in `crates/ui/ess-ui-test/src/parity.rs` returns no refusal
when the target is under a row (`target.row.is_some()`), so `act` at the overlay of a row action
(`<collection>/rows/<key>/...`) whose form saves on change is not refused: the terminal acts and
the Playwright spec clicks a primary button the generated form does not draw.

Found by the second adversary pass over the form-groups story. No test was written; the case is
outside that story's acceptance.

## Acceptance

1. A journey test in `crates/ui/ess-ui-test/tests/journey_controls.rs` with `act` at a row
   action's overlay whose form has `save: on_change` fails the step in the terminal and holds a
   `test.fixme` in the Playwright spec, both with the reason `parity::submit` gives for a section
   form.
2. `act` at a row action's overlay whose form saves on submit still passes in both renderers.
