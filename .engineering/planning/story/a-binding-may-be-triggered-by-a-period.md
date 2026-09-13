---
format: aep.planning-md/1
id: story:a-binding-may-be-triggered-by-a-period
kind: story
status: archived
title: A binding may be triggered by a period, so scheduled work has a declared cause
summary: 'when: admits only an event, so a system that reconciles on a timer cannot say what causes the work, and synthesis builds no scenario for the path it spends most of its time in.'
relations:
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

A binding reacts to an event: `when: {event: …}` is the only trigger the format admits. A system that does
something **on a schedule** has no way to say so, and the work it does then has no declared cause.

Measured in an internal specification repository, 2026-09-11. The server re-reads a
campaign membership every two seconds, and the same read is also triggered by a push. The push-triggered
path has a home once a binding can express it; the timer-triggered path has none, so the specification
carries an `UNMAPPED:` naming this gap on the command that the poll invokes. The same shape recurs wherever
a system reconciles rather than reacts: a liveness sweep, a retry drain, a cache refresh.

The consequence is not only descriptive. `ess verify conform synthesize` builds a flow scenario from a
binding's `delivery:` and `on_failure:`; with no binding, the scheduled work appears in no scenario, so a
conformance suite never exercises the path a system spends most of its time in.

## What would settle it

A second trigger beside `event:`, carrying a period rather than a schedule expression:

```yaml
bindings:
  - name: refresh-campaign-status
    when: {every: PT2S}          # the spelling is the design's to choose
    invoke: {command: outbound.CampaignStatusRefresh}
    delivery: at_most_once
    on_failure: drop
```

Questions the design has to answer rather than this story: whether `mapping:` means anything without an event
to read from (in the measured case the command's inputs come from a read the handler performs, so the answer
may be "no mapping, and the command's inputs must all be `Optional` or supplied by the handler"); what a
conformance scenario for a periodic binding asserts, given it cannot wait two seconds; and whether the period
is a claim about the implementation or documentation only.

`story:elapsed-time-claims` is the neighbour to read first: it gave scenarios a way to claim a length of
time, which is what a scenario for a periodic binding would need.

## Acceptance

- [ ] A binding may be triggered by a period, refused when the period is not a valid duration.
- [ ] The projections (docs, AsyncAPI, graph) print the trigger; the graph draws the edge.
- [ ] Synthesis produces a scenario for it, or records a named refusal saying why a period cannot be witnessed.
- [ ] the internal specification repository loses the `UNMAPPED:` on its polled campaign read.

## Out of scope

Cron expressions, calendars, jitter and backoff policy. One period.

## Provenance

Filed 2026-09-11 from an internal application repository wave 3; the poll is cited in that repository's
`story:campaign-steady-state-has-no-reachable-outcome`.
