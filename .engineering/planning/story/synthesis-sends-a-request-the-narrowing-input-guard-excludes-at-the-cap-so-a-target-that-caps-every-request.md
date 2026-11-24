---
format: aep.planning-md/3
id: story:synthesis-sends-a-request-the-narrowing-input-guard-excludes-at-the-cap-so-a-target-that-caps-every-request
kind: story
status: draft
title: Synthesis sends a request the narrowing input guard excludes at the cap, so a target that caps every request fails
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
revision: 1
---
## Outcome

When a refusal is narrowed by an input guard (`when:` beside a row-set `when_related:` with
`count:`), synthesis sends the request the guard excludes with the row set at the count, and
requires that request to be admitted by the accepting branch. Today no scenario sends it, so a
target that applies the cap to every request, whatever the narrowing input says, passes the suite.

Found while writing `story:the-guards-page-shows-a-cap-that-holds-for-some-requests-only-a-row-set-count-refusal-narrowed-by-an-input-guard`
(beyond10x/ess#816 item 7), whose choice W5-D1 was decided A: that story stays documentation only
and this defect is its own story.

## Reproduction

Reproduced on 0.98.0. Model (`format: ess/36`, generic names): `cap.a.AddSeat` refuses `trial-full`
for `plan == trial` when the tenant already holds two seats, and otherwise adds a seat.

```yaml
format: ess/36
system: cap
version: v1
domain: cap.a
summary: A cap that applies only to the trial plan.
types:
  - {name: cap.a.SeatId, kind: newtype, of: Uuid}
  - {name: cap.a.Plan, kind: enum, variants: [trial, paid]}
entities:
  - name: cap.a.Seat
    identity: {name: seat_id, type: cap.a.SeatId}
    fields:
      - {name: tenant, type: String}
      - {name: plan, type: cap.a.Plan}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - name: cap.a.Admin
    may: [cap.a.AddSeat]
errors:
  - {name: cap.a.TrialFull, summary: A trial tenant holds at most two seats.}
events:
  - name: cap.a.SeatAdded
    fields: [{name: seat_id, type: cap.a.SeatId}]
commands:
  - name: cap.a.AddSeat
    input:
      - {name: tenant, type: String}
      - {name: plan, type: cap.a.Plan}
    outcomes:
      - name: trial-full
        when: input.plan == trial
        when_related:
          entity: cap.a.Seat
          where: tenant == input.tenant
          count: {gte: 2}
        error: cap.a.TrialFull
      - name: added
        creates: cap.a.Seat
        instance: seat_id
        sets: {tenant: input.tenant, plan: input.plan}
        emits: [cap.a.SeatAdded]
        payload:
          cap.a.SeatAdded: {seat_id: {generated: true}}
views:
  - name: cap.a.Seats
    source: cap.a.Seat
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: cap.a.SeatId}
      - {name: tenant, type: String}
```

```console
$ ess specify validate --path cap
cap v1 — 1 file(s), valid
$ ess verify conform synthesize --path cap --out cap-suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to cap-suite.json
$ ess verify conform run --path cap --suite cap-suite.json --target interpreted --report-format 2
  2 scenarios: 2 passed, 0 failed, 0 error, 0 unsupported
$ ess verify conform mutate --path cap --target interpreted
mutation audit of cap v1 against interpreted: 4 mutant(s), 3 killed, 0 survived, 0 inconclusive, 1 stillborn, ...
```

`trial-full` arranges two seats of one tenant and sends `plan: trial` for it; the mutants
`count: {gt: 2}` and `when: not (plan == trial)` are killed. No scenario sends `plan: paid` for a
tenant that already holds two seats, so a target that caps every plan passes the suite, and the
audit has no mutant that drops the input conjunct.

## Open

Acceptance, scope and gate are written when the story is taken up. It is a synthesis defect: a
change to synthesis moves the synthesis-sources stamp and the repository-wide byte pins.
