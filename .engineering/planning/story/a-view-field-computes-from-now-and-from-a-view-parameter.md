---
format: aep.planning-md/3
id: story:a-view-field-computes-from-now-and-from-a-view-parameter
kind: story
status: active
title: A computed view field reads now and the view's parameters
summary: '`due - now` and `param.k` are refused in computed view fields (workbench report item 12)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:a-suite-fixes-the-targets-clock
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:48:07Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:48:08Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

A view cannot publish a waiting time or a value relative to its parameters:

- `due - now`: `ESS-VIEW-009 \`due - now\` reads \`now\` as a value; a value read at the current time
  has no clock a suite can fix yet` (example `12a`).
- `param.k` in a computed field: `ESS-VIEW-001 \`param.k\` is no field of \`gap.demo.Item\`` and
  `ESS-VIEW-003` (example `12b`).

Item 12 of the 2026-11-09 workbench report. Suites can already fix the target's `now` for a step
(story:a-suite-fixes-the-targets-clock), so the clock reason may no longer hold.

## Acceptance

- A computed view field may read `now`. Synthesis fixes the clock, so the expected value is
  deterministic.
- A computed view field may read a declared view parameter.
- Generated queries compute both, or keep the view an obligation naming the read.

## Scope

- inferred: `crates/specify/ess-domain/src/view.rs`, the expression typing, synthesis view
  expectations, `crates/generate/ess-synth/src/view_query.rs`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 34 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Question carried into the fit review, not a condition of the acceptance: what a target that
cannot hold `now` for a step reports for a view that reads it.

Not folded into this story: a second request for a clock read at query time, held in a later
batch of external requests. The review read 12 of its notes: 3 ask for computed fields, which
this story covers, and 9 ask for default time windows and end-of-today bounds, which its
acceptance does not cover. Those 9 stay open with that batch.
