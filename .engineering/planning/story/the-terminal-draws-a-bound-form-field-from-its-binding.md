---
format: aep.planning-md/3
id: story:the-terminal-draws-a-bound-form-field-from-its-binding
kind: story
status: draft
title: The terminal draws a bound form field from its binding, as React does
summary: Terminal reads draft.<field>, React reads binds; filter tokenizer whitespace differs (0.89.0 ride-along adversary; from-finding)
tags:
- backlog
- defect
- from-finding
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/form_toggle.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
revision: 5
---
## Need

Found by the ride-along adversary pass of the 0.89.0 wave. Pre-existing, outside that pass's
acceptance, and reproduced on 0.92.0. A form field that declares `binds:` is drawn from different
values by the two renderers:

- the terminal's `field_value` (`crates/ui/ess-ui-tui/src/app.rs`, near `:3174-3191`) reads the draft
  entry named by `field.field`, else that field of the first `form.loads` row, and never reads
  `field.binds`; the toggle is drawn from it;
- the React runtime reads `field.binds ?? draft.<field>`
  (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, near `:344`).

Reproduction (0.92.0): a `form_page` with page state
`show: {type: boolean, class: page_state, default: true}` and a section form
`{does: settings.Save, fields: [{field: show, as: toggle, binds: state.show, label: Show}]}`.

```
$ ess ui test --path ui.yaml t.yaml      # open: settings; expect: {at: pages/settings/sections/form/fields/show, checked: true}
failed  a bound toggle is drawn from its binding (step 2)
        expect at pages/settings/sections/form/fields/show: expected pages/settings/sections/form/fields/show to be on; it is off: Show [[ ]]
```

The same test written with `--playwright` checks the field's checkbox, which the React runtime draws
from `state.show` and so is checked.

Also pre-existing and of the same kind: the expression tokenizer skips whitespace with
`char::is_whitespace` in Rust (`crates/ui/ess-ui/src/filter.rs`, `tokenize`) and with `/\s/` in the
React runtime's filter mode (`crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl`, `tokenize`),
which disagree on U+0085 (whitespace in Rust only) and U+FEFF (whitespace in JavaScript only). The
`contains` word split was already aligned on Rust's set.

## Acceptance

- One `ess-ui-test` case: the reproduction above passes in the terminal, and its Playwright spec
  checks the checkbox as checked.
- One case each in `ess-ui` and `ess-ui-react` showing that the filter tokenizer treats U+0085 and
  U+FEFF identically.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (`field_value`)
- inferred: `crates/ui/ess-ui-tui/src/view.rs` (where form fields are drawn from `field_value`)
- cited: `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl` (`tokenize`)
- cited: `crates/ui/ess-ui/src/filter.rs` (`tokenize`)
- inferred: `crates/ui/ess-ui-test/tests/form_toggle.rs`, `crates/ui/ess-ui/tests/`, `crates/ui/ess-ui-react/tests/`
- inferred: `CHANGELOG.md`

confidence: high
