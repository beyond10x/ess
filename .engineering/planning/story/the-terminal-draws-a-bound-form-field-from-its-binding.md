---
format: aep.planning-md/3
id: story:the-terminal-draws-a-bound-form-field-from-its-binding
kind: story
status: draft
title: The terminal draws a bound form field from its binding, as React does
summary: Terminal reads draft.<field>, React reads binds; filter tokenizer whitespace differs (0.89.0 ride-along adversary; from-finding)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Found by the ride-along adversary pass of wave-2026-11-06-ess (ESS 0.89.0). Pre-existing, outside
that pass's acceptance. A form field that declares `binds:` is drawn from different values by the two
renderers:

- the terminal's `field_value` (`crates/ui/ess-ui-tui/src/app.rs:3137-3153`) reads the draft entry
  named by `field.field`, else that field of the first `form.loads` row, and never reads
  `field.binds`; the toggle is drawn from it (`view.rs:1678`);
- the React runtime reads `field.binds ?? draft.<field>` (`fields.tsx.tmpl:327-328`).

Reproduction (from the code, not run): a section form with `{field: show, as: toggle, binds:
state.show}` and page state `show` defaulting to `true`. `expect: {at: <form>/fields/show, checked:
true}` passes in the browser and fails in the terminal, which draws `[ ]`. The `checked` step that
0.89.0 adds makes the difference testable.

Also pre-existing and of the same kind: the filter tokenizer splits on `char::is_whitespace` in Rust
and on `/\s/` in the React runtime, which disagree on U+0085 and U+FEFF.

## Acceptance

- One `ess-ui-test` case: the reproduction above passes in the terminal and in the Playwright spec.
- One case each in `ess-ui` and `ess-ui-react` showing that the filter tokenizer splits identically
  on U+0085 and U+FEFF.

## Scope

- inferred: `crates/ui/ess-ui-tui/src/app.rs`, `crates/ui/ess-ui-tui/src/view.rs`,
  `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl`, `crates/ui/ess-ui/src/filter.rs`.
