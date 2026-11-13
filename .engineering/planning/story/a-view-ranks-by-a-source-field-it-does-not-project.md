---
format: aep.planning-md/3
id: story:a-view-ranks-by-a-source-field-it-does-not-project
kind: story
status: active
title: A view may rank by a source field it does not project
summary: order_by naming an unprojected field is ESS-VIEW-001 on 0.93.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:28:04Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:28:06Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's list reads rank by a stored field the answer does not carry (a database
sequence, a creation instant). `order_by:` must name a projected field, so the view either
publishes a field the wire does not have or states no order.

Reproduced on 0.93.0 (`ess/37`; `view-paging.yaml` with `order_by: [type asc, job_id asc]` and
`type` removed from the view's `fields:`):

```text
  - error[ESS-VIEW-001]: `demo.jobs.JobList` ranks by `type`, which it does not project, so nothing reading this view can tell whether the order holds
  `ess-domain` refuses this as `undeclared_reference`
  help: projected here: `job_id`
```

## Acceptance

- At a new source format `order_by:` may name a field of the view's source that the view does not
  project, where the model lets conformance know the field's value for each arranged row: a
  creating or updating command writes it from an input, or from a minted value a declared event
  or response carries.
- Synthesis arranges two rows that differ on the key and expects their order in the view; a target
  that answers them in the other order fails. Where no command lets the suite know the key's
  value, the view is refused by name with that reason.
- The `order-flip` mutation covers such a key.
- A view that ranks by projected fields only keeps its IR bytes and its suite bytes.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
