---
format: aep.planning-md/3
id: story:both-renderers-tokenize-whitespace-alike
kind: story
status: draft
title: Both renderers tokenize whitespace alike
summary: React splits expressions on ASCII whitespace only while the filter grammar accepts a no-break space (UI-adopter wave 1 U2 pass-4 note, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:w086-u2-adversary-pass-4
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/toggle_state.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/tests/
- confidence: cited
  path: crates/ui/ess-ui/src/filter.rs
revision: 5
---
## Need

The React runtime's filter tokenizer and the filter grammar the terminal and `ess ui check` use
split on different sets of whitespace. The U2 adversary pass 4 of UI-adopter wave 1 found it
(review-result:w086-u2-adversary-pass-4, INFEASIBLE note, introduced). The red case is a hand-built
expression; no input from a downstream adopter is known to reach it.

The no-break space (U+00A0) case from pass 4 is fixed: since 0.89.0 React reads a toggle's
`checked` with `holds` (`crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:266`), which
tokenizes in filter mode, and JavaScript's `\s` includes U+00A0. Both renderers now draw
`not<U+00A0>row.off` off, on. The two sets still differ, though:

- React, filter mode: `/\s/.test(char)` (`expr.ts.tmpl:48`). This is the ECMAScript set. It includes
  U+FEFF and leaves out U+0085.
- Rust: `c.is_whitespace()` (`crates/ui/ess-ui/src/filter.rs:239`). This is Unicode White_Space. It
  includes U+0085 and leaves out U+FEFF.
- The `contains` word split already uses the Rust set on purpose (`expr.ts.tmpl:321-322`, "`\s` would
  add U+FEFF and miss U+0085"). The tokenizer does not.

Reproduction on 0.92.0. Rows `[{id: a, off: true}, {id: b, off: false}]`, row actions on a table:

```yaml
- {name: tn, as: toggle, label: November, does: blog.posts.SetT, bind: {id: row.id}, checked: "not\u0085row.off"}
- {name: tf, as: toggle, label: Foxtrot, does: blog.posts.SetT, bind: {id: row.id}, checked: "not﻿row.off"}
```

| toggle | `ess ui check` | terminal (a, b) | React `holds` (a, b) |
|---|---|---|---|
| `tn` (U+0085) | silent | off, on | on, on |
| `tf` (U+FEFF) | `checked_unreadable` warning | on, on | off, on |

The terminal reads `tn` as `not row.off`. React's filter tokenizer throws on U+0085, so `holds`
falls back to `evaluate`, which returns the text, and the text is truthy. `checked_unreadable` is
silent because the filter grammar parses the text with admitted roots. `tf` already gets a warning,
which the acceptance below allows.

## Acceptance

- React's filter-mode tokenizer (`expr.ts.tmpl:48`) skips exactly the characters Rust's
  `char::is_whitespace` skips: the class the `contains` split uses at `expr.ts.tmpl:322`.
- With that change, `tn` above draws off, on in both renderers, and `tf` draws on, on in both and
  keeps its `checked_unreadable` warning.
- A React test pins `holds("not\u0085row.off", …)` and `holds("not row.off", …)` against the
  terminal's marks on the same rows.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl` (`tokenize` :43-48, `holds` :266, the `contains` split :322)
- cited: `crates/ui/ess-ui/src/filter.rs` (`tokenize` :234-239, the reference set)
- inferred: `crates/ui/ess-ui-react/tests/toggle_state.rs` (a new case beside the toggle-state tests)
- inferred: `crates/ui/ess-ui-tui/tests/` (the terminal twin of the case)

confidence: high

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
