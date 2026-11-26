---
title: Refusal order and caps
sidebar_position: 4
description: A refusal the request alone decides answers before the input refusals, and a row-set cap holds only for the requests an input guard selects.
---

# Refusal order and caps

Two guard forms beside [Guards and outcome selection](./guards-and-predicates.md), each with a
model that validates and synthesizes.

## Answer before the input is checked

A refusal the request alone decides, such as a missing session, can answer before every input
refusal. Declare it as an `external:` refusal marked `before_row: true`, above the input refusals:
step 1 holds both, and the first declared answers.

```yaml
format: ess/37
system: demo
version: v1
domain: demo.desk
summary: A seat opened with a label; a request without a valid session is refused first.
types:
  - {name: demo.desk.SeatId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.Seat
    identity: {name: seat_id, type: demo.desk.SeatId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.OpenSeat]
errors:
  - {name: demo.desk.Unauthenticated, fields: []}
  - {name: demo.desk.LabelTooLong, fields: []}
events:
  - name: demo.desk.SeatOpened
    fields: [{name: seat_id, type: demo.desk.SeatId}]
commands:
  - name: demo.desk.OpenSeat
    input:
      - {name: label, type: String}
    outcomes:
      - name: unauthenticated
        external: the request carries no valid session
        before_row: true
        error: demo.desk.Unauthenticated
      - name: label-too-long
        when: input.label.count > 8
        error: demo.desk.LabelTooLong
      - name: opened
        creates: demo.desk.Seat
        instance: seat_id
        sets: {label: input.label}
        emits: [demo.desk.SeatOpened]
        payload: {demo.desk.SeatOpened: {seat_id: {generated: true}}}
components:
  - component: desk
    owns: {domains: [demo.desk]}
    accepts: {commands: [demo.desk.OpenSeat]}
    reached_by: network
```

The synthesized suite forces the external cause twice: once with a short label, and once with a
label `label-too-long` also refuses, requiring `unauthenticated` both times. A target that checks
the label first fails that second send. Declared below an input refusal, the external refusal
answers after it: the input refusal answers first. An `input_absent:` branch still answers
before both, at step 0.
[The precedence order](../../reference/predicates.md#a-guard-over-another-entitys-row) lists
every step.

## A cap that holds for some requests only

"A trial tenant holds at most two seats; a paid one has no cap" is one refusal with two conditions:
a row-set `when_related:` with `count:` and, beside it, an input `when:` that narrows it to the
requests it holds for. Both hold together.

```yaml
format: ess/37
system: demo
version: v1
domain: demo.cap
summary: Seats added per tenant; a trial tenant holds at most two.
types:
  - {name: demo.cap.SeatId, kind: newtype, of: Uuid}
  - {name: demo.cap.Plan, kind: enum, variants: [trial, paid]}
entities:
  - name: demo.cap.Seat
    identity: {name: seat_id, type: demo.cap.SeatId}
    fields:
      - {name: tenant, type: String}
      - {name: plan, type: demo.cap.Plan}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - name: demo.cap.Admin
    may: [demo.cap.AddSeat]
errors:
  - {name: demo.cap.TrialFull, fields: []}
events:
  - name: demo.cap.SeatAdded
    fields: [{name: seat_id, type: demo.cap.SeatId}]
commands:
  - name: demo.cap.AddSeat
    input:
      - {name: tenant, type: String}
      - {name: plan, type: demo.cap.Plan}
    outcomes:
      - name: trial-full
        when: input.plan == trial
        when_related:
          entity: demo.cap.Seat
          where: tenant == input.tenant
          count: {gte: 2}
        error: demo.cap.TrialFull
      - name: added
        creates: demo.cap.Seat
        instance: seat_id
        sets: {tenant: input.tenant, plan: input.plan}
        emits: [demo.cap.SeatAdded]
        payload: {demo.cap.SeatAdded: {seat_id: {generated: true}}}
components:
  - component: seats
    owns: {domains: [demo.cap]}
    accepts: {commands: [demo.cap.AddSeat]}
    reached_by: network
```

The synthesized suite arranges two seats of one tenant and sends `plan: trial` for it, requiring
`trial-full`. A count written inside `when_related: all:` is refused, since `all:` lists rows an
input identity names; write the cap in this form instead. The row-set form and its `count:` are in
[the predicates reference](../../reference/predicates.md#a-guard-over-another-entitys-row).
