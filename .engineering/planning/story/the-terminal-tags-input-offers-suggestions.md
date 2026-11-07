---
format: aep.planning-md/3
id: story:the-terminal-tags-input-offers-suggestions
kind: story
status: draft
title: The terminal tags input offers the suggestions of its read, as React does
summary: suggests is drawn by React from 0.89.0 and ignored by the terminal
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

A field `as: tags` with `suggests: {reads, value}` suggests the rows of a read and keeps typed
values, so no create command is needed. React does this from 0.89.0
(`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `tests/tags_suggest.rs`). The
terminal ignores `suggests`, and its input is drawn as before (`crates/ui/ess-ui-tui/src/app.rs`,
the 0.89.0 CHANGELOG entry). The two renderers disagree on one document.

## Acceptance

- The terminal reads the `suggests` view and offers each value the input does not already hold.
- A picked suggestion is added as a chip. A typed value is kept whether or not a row holds it.
- A field without `suggests` is unchanged.

## Scope

- inferred: `crates/ui/ess-ui-tui/src/app.rs` (tags input), terminal tests beside the React
  `tags_suggest` cases.
