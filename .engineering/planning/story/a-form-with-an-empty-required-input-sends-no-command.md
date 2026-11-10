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
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: cited
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/spec.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/required_input.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui/src/binding.rs
- confidence: inferred
  path: website/docs/reference/ess-ui-test.md
revision: 12
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
  `body_required` (`templates/runtime/data.ts.tmpl`, `CommandRoute`, built in
  `crates/ui/ess-ui-check/src/model.rs` from `crates/ui/ess-ui/src/binding.rs`), and
  `ess ui test --model` reads the model only for enum options.
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

All bullets run on the document above and its model, in a new test target
`crates/ui/ess-ui-test/tests/required_input.rs`, with
`cargo test -p ess-ui-test --locked --test required_input`.

- Terminal: a test that types `i-1` into `id`, runs `act: pages/items.new/sections/form` and then
  `expect_no_command: probe.core.CreateItem` passes under `ess ui test --path ui.yaml --model spec`,
  and an `expect: {at: pages/items.new/sections/form/fields/title, text: "Title *"}` step in the
  same test passes.
- Terminal: the same steps with `expect_command: probe.core.CreateItem` in place of
  `expect_no_command` fail at that step with `command probe.core.CreateItem was never sent; sent: []`.
- `ess-ui-test/1` accepts the new step `expect_no_command: <command>`: it holds when the named
  command has not been sent in the test so far, and fails with
  `command <command> was sent with <input>` when it has.
- The full script (type `i-1` into `id`, submit, `expect_no_command: probe.core.CreateItem`, type
  `t` into `title`, submit, `expect_command: {command: probe.core.CreateItem, input: {id: i-1,
  title: t}}`) passes under `ess ui test --path ui.yaml --model spec`.
- `ess ui test --playwright` over the same script writes
  `expect(commands.map((sent) => sent.command)).not.toContain("probe.core.CreateItem");` for the
  `expect_no_command` step, and the generated React project draws the `title` input with
  `aria-required="true"`; the test asserts both strings in the generated output.
- An `Optional<String>` input in place of `title` is not required: the empty submit sends
  `probe.core.CreateItem {id: i-1}` and the field carries no ` *` and no `aria-required`.

## Decisions

- `expect_no_command` is an additive step of `ess-ui-test/1`, not a new format version: the format
  has taken new steps and checks before without a version (`checked` at a toggle, beyond10x/ess#744
  in `CHANGELOG.md`), and an existing file never contains the new key.
- `expect_no_command` checks the whole test so far, not "since a named point": commands accumulate
  in the runner (`expect_command` in `crates/ui/ess-ui-test/src/runner.rs` reads every sent
  command), and the empty submit comes before any successful one in the script that needs it.
- React marks the input with `aria-required="true"`, not the native `required` attribute or
  `aria-invalid`: it is present before any submit, holds on every control kind the form draws, and
  does not turn on the browser's own validation.
- The terminal marks a required field by a trailing ` *` on its label, which a `text` check reads.
- "Required" comes only from the model's command input (a non-`Optional` input type); fixture runs
  without `--model` check nothing, because a separate `required:` key on a form field would restate
  the model's type and could disagree with it.

## Ambiguities

- `decided`: where "required" comes from when no `--model` is given (fixture mode): only the
  model's command input, so fixture runs stay unchecked (see Decisions).
- `inferable`: an `Optional<…>` input is never required (the model's type says so).

## Scope

This story moves only the `CHANGELOG.md` shared surface.

- cited: crates/ui/ess-ui-tui/src/app.rs
- cited: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- cited: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- cited: crates/ui/ess-ui-test/src/spec.rs
- cited: crates/ui/ess-ui-test/src/runner.rs
- cited: crates/ui/ess-ui-test/src/playwright.rs
- cited: crates/ui/ess-ui-check/src/model.rs
- cited: crates/ui/ess-ui/src/binding.rs
- inferred: crates/ui/ess-ui-test/tests/required_input.rs
- inferred: website/docs/reference/ess-ui-test.md

confidence: medium
