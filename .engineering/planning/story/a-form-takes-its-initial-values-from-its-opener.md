---
format: aep.planning-md/3
id: story:a-form-takes-its-initial-values-from-its-opener
kind: story
status: active
title: A form takes its initial values from its opener's params and row, without a read
summary: an opens form is prefilled only by loads:, so a drawer filled from its row needs an extra read
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:47:52Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:47:53Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

An edit form opened from a list row (`opens:` from a row action) is prefilled only by `loads:`, a
read of the record by id (`form.tsx.tmpl`, which prefills from `loaded.rows[0]`). An overlay's
`params` are passed to the command that is sent, but no input shows them. A field bound to a row
value therefore starts empty. An application that fills its edit drawer from the row it already
holds has to issue an extra read, by an endpoint that may not exist.

## Acceptance

- A form may state the initial value of its inputs from expressions it can read: `params.<x>` of
  its opener, and through them `row.<x>`. The inputs show those values and may be edited.
- `loads:` still wins where both are declared, or the checker refuses the combination by name.
  Which of the two applies is decided in the fit review.
- Both renderers prefill the same values. `ess ui check` refuses an expression the form cannot
  read.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (Form, Field), `crates/ui/ess-ui-check`,
  `crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`, the terminal form.

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 29 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

The story named `epic:downstream-reported-gaps`, archived on 2026-11-02. It now decomposes
`epic:ess-ui-renderer-neutral-ui`; the older `informed_by` edge stays as history.
