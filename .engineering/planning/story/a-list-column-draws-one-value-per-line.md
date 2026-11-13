---
format: aep.planning-md/3
id: story:a-list-column-draws-one-value-per-line
kind: story
status: active
title: 'A column or record field as: list draws one value per line'
summary: 'list values are comma-joined in both renderers; as: list on a column has no meaning yet'
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:47:58Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:47:59Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

On an input, `as: list` is a list editor (0.87.0). On a column or a record field, a list value is
joined with ", " in both renderers (`core.tsx.tmpl` FieldValue, `ess-ui-tui/src/expr.rs`). The
adopter's source application draws one value per line, and the document cannot ask for that.

## Acceptance

- A column or record field `as: list` draws each element on its own line in React and in the
  terminal, without the separator.
- An empty list draws nothing. Any other `as` keeps today's joined text.
- The reference page documents what `as: list` means on a column.

## Scope

- inferred: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`, `fields.tsx.tmpl`,
  `styles.css.tmpl`, and the terminal cell display in `crates/ui/ess-ui-tui/src/view.rs`.

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 31 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

The story named `epic:downstream-reported-gaps`, archived on 2026-11-02. It now decomposes
`epic:ess-ui-renderer-neutral-ui`; the older `informed_by` edge stays as history.
