---
format: aep.planning-md/3
id: story:a-form-groups-own-action-runs-from-its-key
kind: story
status: draft
title: A form group's own action runs from its key in the terminal
summary: The terminal shows a form group's actions in the hint line but form_key looks up only the form's actions (0.88.0 journey-steps correction, pre-existing)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

Found by the 0.88.0 journey-steps correction, pre-existing: in the terminal renderer a form group's
own actions are shown in the hint line, but the key lookup (`form_key` in
`crates/ui/ess-ui-tui/src/app.rs`) searches only the form's actions, so pressing a group action's key
does nothing and a journey cannot `act` on it.

## Acceptance

- `a_form_groups_own_action_runs_from_its_key`: a group action's key sends its command, and the
  journey step `act` reaches it.
