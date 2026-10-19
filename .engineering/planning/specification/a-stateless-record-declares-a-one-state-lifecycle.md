---
format: aep.planning-md/3
id: specification:a-stateless-record-declares-a-one-state-lifecycle
kind: specification
status: draft
title: A record with no meaningful states declares a one-state lifecycle
summary: Decline of downstream finding 15 with the idiom, run on the integration build
tags:
- adopter-report
relations:
- informed_by: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Statement

An entity always declares a lifecycle. A record with no meaningful states declares a one-state
lifecycle — `states: [S]`, `terminal: [S]`, `transitions: []` — which validates, synthesizes and is
read as "this record never changes state". A value with no identity of its own is a `struct` type
on a field of the entity that holds it, not an entity. A non-terminal state with no way out is
refused as `dead_end_state` on purpose: a record that reaches it is stuck.

Answers finding 15 of a downstream specification report (2026-10-19): "every entity must declare a
lifecycle; a value-like entity needs a dummy one, and removing its terminal flag is refused as
dead_end_state". Decision: **decline, with this idiom** (convenience; no new surface). Design
precedent: "A single-state lifecycle has one honest group and needs no fake state"
(`docs/design/aggregate-group-selection.md:147-148`).

## Run

Neutral models under `.engineering/repro/value-entity-lifecycle/`, integration build (0.62.0 plus
the 2026-10-13 port), 2026-10-19:

```console
$ ess specify validate --path no-lifecycle.yaml
no-lifecycle.yaml was refused:
  - no-lifecycle.yaml: missing field `lifecycle`
$ ess specify validate --path no-terminal.yaml
  - [dead_end_state] entity relstate.desk.Ticket.states.Waiting: `Waiting` has no outgoing
    transition and is not declared terminal, so an entity that reaches it is stuck
$ ess verify conform synthesize --path idiom-one-state.yaml --out <file>
8 scenario(s) (0 authored), 0 refusal(s)
```

`idiom-one-state.yaml` declares the one-state lifecycle:

```yaml
    lifecycle:
      initial: Waiting
      states: [Waiting]
      terminal: [Waiting]
      transitions: []
```

## Left open

The refusal for a missing `lifecycle:` is a bare parse message (`missing field 'lifecycle'`) with
no validation code and no hint. A hint naming the one-state form would have answered this finding
at the point of failure; it is a diagnostic change, not a language one, and is not scheduled here.
