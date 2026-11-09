---
format: aep.planning-md/3
id: story:a-form-groups-own-on-change-saves-in-every-target
kind: story
status: draft
title: A form group's own on_change saves in every target
summary: 'The terminal ignores a group''s save: on_change, and the exported browser spec clicks a submit an on_change form lacks (0.88.0 adversary pass, out of scope)'
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
revision: 3
---
## Need

Found by the 0.88.0 adversary pass on the journey-steps unit, outside that story's acceptance.
Both parts still reproduce on 0.92.0:

- The terminal renderer sends nothing for a form group with its own `save: on_change`.
  `save_on_change` in `crates/ui/ess-ui-tui/src/app.rs` reads only the form's `save` and sends the
  form's `does`; it never reads the group's `save` or `does`. The React target does save the group
  on change (`form.tsx.tmpl`, `savesOnChange(group.does ?? props.does)`).
- The exported browser spec writes `act` on any section form as a click on
  `button[type="submit"]` (`crates/ui/ess-ui-test/src/playwright.rs`, `Step::Act`, `Kind::Form`).
  The generated React form draws no submit button when it saves on change (`form.tsx.tmpl`: "a
  form that saves on change has no submit"), so the spec times out against a correct app. The same
  step passes in the terminal.

Minimal reproduction (`fixtures/current.yaml` holds one `probe.Current` row
`{id: s-1, owner: Ada, audit_on: false}`):

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
  sections: [{name: all, pages: [settings.edit, prefs]}]
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
        groups:
          - name: audit
            label: Audit
            save: on_change
            does: probe.SaveAudit
            fields: [{field: audit_on, as: toggle, label: Audit on}]
  prefs:
    kind: settings_page
    title: Preferences
    sections:
      - name: settings
        component: form
        does: probe.SetPrefs
        save: on_change
        fields: [{field: dark, as: toggle, label: Dark}]
```

`t.yaml` has two tests. The first opens `prefs`, chooses `'true'` for `dark` and expects
`probe.SetPrefs {dark: true}`. The second opens `settings.edit`, chooses `'true'` at
`…/groups/audit/fields/audit_on` and expects `probe.SaveAudit {audit_on: true}`. `t-act.yaml`
chooses `dark`, then runs `act: pages/prefs/sections/settings`.

```text
$ ess ui test --path ui.yaml t.yaml
passed  a form-level on_change sends each change (control)
failed  a group on_change sends each change (step 3)
        expect_command probe.SaveAudit: command probe.SaveAudit was never sent; sent: []
$ ess ui test --path ui.yaml t-act.yaml --playwright act.spec.ts
$ grep -n submit act.spec.ts
18:    await page.locator('[data-ui-path="pages/prefs/sections/settings"] button[type="submit"]').click();
```

## Acceptance

- `a_group_on_change_sends_each_change_in_the_terminal`: over the document above, `ess ui test`
  passes both tests in `t.yaml`. Choosing `audit_on` sends `probe.SaveAudit {audit_on: true}`,
  the group's own `does`, as a form-level `save: on_change` sends the form's command. A group with
  `save: on_change` and no `does` sends the form's `does`.
- `the_browser_spec_does_not_click_a_submit_an_on_change_form_lacks`: the spec written for
  `t-act.yaml` has no `button[type="submit"]` click for the `on_change` form. Either the step is
  written without a click, or `ess ui test --playwright` refuses `act` on an `on_change` form,
  naming the path, and the terminal runner refuses it the same way.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs`
- cited: `crates/ui/ess-ui-test/src/playwright.rs`
- inferred: `crates/ui/ess-ui-test/src/runner.rs`
- inferred: `crates/ui/ess-ui-test/tests/journey_controls.rs`

confidence: high
