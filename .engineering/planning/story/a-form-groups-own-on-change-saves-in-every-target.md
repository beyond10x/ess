---
format: aep.planning-md/3
id: story:a-form-groups-own-on-change-saves-in-every-target
kind: story
status: draft
title: A form group's own on_change saves in every target
summary: 'The terminal ignores a group''s save: on_change, and the exported browser spec clicks a submit an on_change form lacks (0.88.0 adversary pass, out of scope)'
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

Found by the 0.88.0 adversary pass on the journey-steps unit, outside that story's acceptance:

- A form group with its own `save: on_change` sends nothing in the terminal renderer: only the
  form's `save` is read (`crates/ui/ess-ui-tui/src/app.rs`, the on-change path).
- The exported browser spec's `act` on an `on_change` section form clicks a submit button the
  generated React app does not render.

## Acceptance

- `a_group_on_change_sends_each_change_in_the_terminal`: a group's own `save: on_change` sends each
  change, as a form-level `save: on_change` does.
- `the_browser_spec_does_not_click_a_submit_an_on_change_form_lacks`: the exported spec drives an
  `on_change` section form without a submit click.
