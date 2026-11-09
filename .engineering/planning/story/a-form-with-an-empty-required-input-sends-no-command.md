---
format: aep.planning-md/3
id: story:a-form-with-an-empty-required-input-sends-no-command
kind: story
status: draft
title: A form with an empty required input sends no command
summary: Neither renderer enforces required command inputs, and ess-ui-test has no step asserting that no command was sent (0.77.0)
tags:
- defect
- ess-batch-4
- ess-next
- ess-ui
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/spec.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: inferred
  path: website/docs/reference/ess-ui-test.md
revision: 8
---
## Outcome

A person filling a form in either renderer cannot send a command while a required input is empty.
The form marks that input as required, and an `ess-ui-test/1` script can assert that nothing was
sent.

## Problem

Reported on 0.77.0 and reproduced on 0.92.0. Neither the terminal renderer nor the React target
enforces a command's required inputs in a form:

- Terminal: `submit` sends `form_input`, which drops every null field and checks nothing
  (`crates/ui/ess-ui-tui/src/app.rs`, `form_input` and `submit`).
- React: the form's `submit` passes the draft to `runCommand` unchecked
  (`crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`).
- Neither renderer receives the command's required inputs. A React command route carries only
  `body_required` (`templates/runtime/data.ts.tmpl`, `CommandRoute`), and `ess ui test --model`
  reads the model only for enum options.
- `ess-ui-test/1` has `expect_command` but no step asserting that no command was sent
  (`crates/ui/ess-ui-test/src/spec.rs`, `Step`).

Minimal reproduction. The model has one domain `probe.core` with `probe.core.CreateItem`, whose
inputs are `id: String` and `title: String`, both required. It creates `probe.core.Item`. The
document:

```yaml
format: ess-ui/1
app: probe
model: probe
placement_profile: fat
fixtures: {dir: fixtures, views: {probe.core.Items: items.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: items.new
  sections: [{name: all, pages: [items.new]}]
pages:
  items.new:
    kind: form_page
    title: New item
    sections:
      - name: form
        component: form
        does: probe.core.CreateItem
        fields:
          - {field: id, label: Id}
          - {field: title, label: Title}
```

A test types `i-1` into `id`, runs `act: pages/items.new/sections/form`, and expects
`probe.core.CreateItem` with `{id: i-1, title: ""}`, only so that the runner prints what was sent:

```text
$ ess ui test --path ui.yaml --model spec t-show.yaml
failed  probe shows the sent input (step 4)
        expect_command probe.core.CreateItem: command probe.core.CreateItem was sent with {id="i-1"}, not with {id="i-1", title=""}: title is absent, expected string ""
```

## Acceptance

- In both renderers, submitting a form whose command input is required while the field is empty
  sends no command, and the field at its path is marked as required. In the terminal, a `text`
  check at the form finds the mark; in React, the field's input carries `required` or
  `aria-invalid`.
- `ess-ui-test/1` gains a step asserting that no command was sent since a named point. The fit
  review decides whether this is an additive step in `ess-ui-test/1` or a new format version.
- A script over the document above submits the empty form, asserts that no command was sent,
  fills `title` and asserts `probe.core.CreateItem {id: i-1, title: <text>}`. It passes with
  `ess ui test` and in the spec written by `ess ui test --playwright`.

## Ambiguities

- `requires-stakeholder-input`: where "required" comes from when no `--model` is given (fixture
  mode). The options are a `required:` key on a form field, or only the model's command input,
  which leaves fixture runs unchecked. A format owner decides.
- `inferable`: an `Optional<…>` input is never required (the model's type says so).

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs`
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`
- cited: `crates/ui/ess-ui-test/src/spec.rs`
- inferred: `crates/ui/ess-ui-test/src/runner.rs`
- inferred: `crates/ui/ess-ui-test/src/playwright.rs`
- inferred: `crates/ui/ess-ui-check/src/model.rs`
- inferred: `website/docs/reference/ess-ui-test.md`

confidence: medium
