---
format: aep.planning-md/3
id: story:a-form-groups-own-action-runs-from-its-key
kind: story
status: active
title: A form group's own action runs from its key in the terminal
summary: The terminal shows a form group's actions in the hint line but form_key looks up only the form's actions (0.88.0 journey-steps correction, pre-existing)
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/journey_controls.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:33:53Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:33:55Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Found by the 0.88.0 journey-steps correction; the defect existed before it and still reproduces on
0.92.0. The terminal renderer shows a form group's own actions in the hint line
(`crates/ui/ess-ui-tui/src/view.rs`, the form's lines: `action_hint(&group.actions, …)`). The key
lookup (`form_key` in `crates/ui/ess-ui-tui/src/app.rs`) searches only `form.actions`, so pressing
a group action's key does nothing, and an `ess-ui-test/1` `act` step on it sends no command. The
group's keys come from a separate `action_keys_besides` call over the group's actions alone, so a
group action can be shown with the same letter as a form action. The React target runs group
actions (`form.tsx.tmpl`, `group.actions`).

Minimal reproduction (`fixtures/current.yaml` holds one `probe.Current` row
`{id: s-1, owner: Ada, level: low}`):

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {dir: fixtures, views: {probe.Current: current.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: settings.edit
  sections: [{name: all, pages: [settings.edit]}]
pages:
  settings.edit:
    kind: form_page
    title: Settings
    sections:
      - name: form
        component: form
        loads: {view: probe.Current}
        does: probe.Save
        fields:
          - {field: owner, label: Owner}
        actions:
          - {name: refresh, label: Refresh, does: probe.Refresh}
        groups:
          - name: audit
            label: Audit
            fields: [{field: level, label: Level}]
            actions:
              - {name: purge, label: Purge, does: probe.Purge}
```

```yaml
format: ess-ui-test/1
document: ui.yaml
tests:
  - name: a form action runs (control)
    steps:
      - open: settings.edit
      - act: pages/settings.edit/sections/form/actions/refresh
      - expect_command: {command: probe.Refresh, input: {}}
  - name: a group action runs
    steps:
      - open: settings.edit
      - act: pages/settings.edit/sections/form/groups/audit/actions/purge
      - expect_command: {command: probe.Purge, input: {}}
```

```text
$ ess ui test --path ui.yaml t.yaml
passed  a form action runs (control)
failed  a group action runs (step 3)
        expect_command probe.Purge: command probe.Purge was never sent; sent: []
1 passed, 1 failed                                       # exit 1
```

## Acceptance

- `a_form_groups_own_action_runs_from_its_key`: over the document above, `ess ui test` passes
  both tests: the `act` step at `…/groups/audit/actions/purge` sends `probe.Purge`.
- Pressing the letter the hint line shows beside a group action sends that action's command.
- No two actions of one form, its own or any group's, are shown with the same key.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs`
- cited: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `crates/ui/ess-ui-test/tests/journey_controls.rs`

confidence: high
