---
format: aep.planning-md/3
id: story:a-sent-value-may-be-mapped-or-left-out
kind: story
status: draft
title: A sent param or input may map its value through a declared table, and a param may be left out when false
summary: no value map for what is sent (constant-name spellings), and a false param is always sent
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
## Need

A UI adopter reported two gaps on 2026-11-07, against 0.90.0. Both concern a value the UI sends,
not a value it shows.

1. **Another spelling.** A command takes a type by its constant name (`ZENDESK`, `CUSTOM`), but the
   row the user picked carries it in lower case. Only display text can be mapped (`labels`,
   `label_maps`). No expression form maps a value that is sent: no upper-casing and no value
   table. So the create command cannot be sent with what the backend binds.
2. **Left out when false.** A read param bound to a boolean state is sent as `false` when the state
   is false. The React project leaves out only `undefined`, `null` and `""`
   (`templates/runtime/data.ts.tmpl`), and the terminal only `null` (`ess-ui-tui/src/http.rs`).
   No grammar leaves a param out while its state is false, so an API that reads a present `false`
   as a filter answers differently.

## Acceptance

- A read param, a `bind:` input and an overlay param may map their value through a declared table,
  written once (as `label_maps` are) and named where it is used. A value the table does not name is
  refused at send time, never passed through.
- A read param may declare that it is left out when its value is `false`, as well as when it is
  unset.
- Both renderers send the same request for the same state. `ess ui check` refuses a table whose
  outputs do not fit the parameter's or input's type.
- The design decides whether the two are one construct. The adopter's wording is evidence of a
  need, not a design.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (expressions, Reads params, Action bind),
  `crates/ui/ess-ui-check`, `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` and
  `expr.ts.tmpl`, `crates/ui/ess-ui-tui/src/http.rs` and `expr.rs`.
