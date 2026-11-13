---
format: aep.planning-md/3
id: story:a-group-key-buckets-a-timestamp-by-hour-day-or-week
kind: story
status: active
title: A group key buckets a Timestamp by hour, day or week
summary: There is no time bucket for an aggregate group key (workbench report item 15)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:aggregate-group-keys-are-expressions
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:37Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:39Z", actor: "human:timo", revision: 5, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

An aggregate view cannot group by a time bucket of a Timestamp, such as items per week. A guess
`expr: week(due)` gives `ESS-VIEW-009 ... \`week\` computes from one row; arithmetic over measures is
not admitted yet`. Item 15 of the 2026-11-09 workbench report; example `15`.
story:aggregate-group-keys-are-expressions makes a group key an expression, and a bucket function
fits there.

## Acceptance

- A group key may be a Timestamp truncated to an hour, a day or an ISO week, in UTC unless a zone
  is declared.
- The interpreted target, synthesis and generated queries bucket alike, and synthesis puts rows in
  two buckets and at a bucket boundary.

## Scope

- inferred: `crates/specify/ess-domain/src/view.rs` (group keys), the expression functions,
  `crates/verify/ess-conformance/src/interpret/views.rs`, `crates/generate/ess-synth/src/view_query.rs`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.

Second report, 2026-11-12, from an adopter on 0.90.0, reproduced on the 0.94.0 release binary: a
view that counts rows per hour or per day of a Timestamp. `{expr: "hour(input.at)"}` is refused
with `ESS-SPEC-012` ("`hour` is no function of the expression language; the functions are `round`,
`abs`, `min`, `max`, `coalesce`, `sha256`, `base64url`, `ascii`"), and the reporter got
`ESS-VIEW-009` ("grouping by a timestamp is time bucketing, which is not in this cut") for a group
key of type Timestamp. The adopter removed its per-hour chart and the view behind it. Hour is the
bucket this report adds to the acceptance's list as its first use.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).
