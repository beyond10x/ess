---
format: aep.planning-md/1
id: story:a-timestamp-names-the-clock-it-was-read-from
kind: story
status: archived
title: A timestamp names the clock it was read from, and crossing clocks is declared
summary: Timestamp is one universal line; in a distributed system it is a reading from a named clock in a named representation. Where they differ the model is silent, and the assumption lives in a comment.
relations:
- informed_by: story:elapsed-time-claims
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

The model has a `Timestamp` and, since `story:elapsed-time-claims`, a scenario can claim a length of time.
Neither says anything about **which clock a timestamp came from, what it is expressed in, or what a system
may assume about it**. So a specification cannot state the facts that decide whether two timestamps may be
compared at all.

Measured in an internal application repository, 2026-09-11, where every one of these is live and none is typable:

| the fact | where it bites |
|---|---|
| the wire carries a local-time string with a literal `Z`, stamped on the **JVM default timezone** of the producing process | the consumer parses it as UTC; correct only because the producer pins its default timezone at boot, which the consumer cannot see and does not check |
| the producing process's timezone **before** that pin is applied is unknown | the specification carries an `UNMAPPED:` for exactly this |
| a timestamp minted by the consumer's own clock and one read off the wire are compared | nothing declares them comparable, and nothing would refuse it if they were not |
| a duration is derived by subtracting two timestamps from **different** producers | same |

The general shape: a `Timestamp` is treated as a point on one universal line, and in a distributed system it
is a reading from a named clock with a named representation. Where those differ, the model is silent and the
code is where the assumption lives — in a comment, if anywhere.

## What would settle it

A clock as a declared thing, and a timestamp that names one:

```yaml
clocks:
  - name: billing.Ledger
    representation: utc-instant          # or: local-with-offset, local-no-offset
    monotonic: false
types:
  - name: billing.invoice.IssuedAt
    kind: newtype
    of: Timestamp
    clock: billing.Ledger
```

What it buys, and what the design must decide rather than this story: comparing or subtracting timestamps of
two different clocks is refused unless a conversion is declared (the same shape `conversions:` already has
for types, and for the same reason — a crossing nobody explained is a silent widening); a representation that
carries no offset is refused as a source for an instant, because it is not one; and a conformance target may
be asked which clock it answered with, so "the two sides agree about time" becomes checkable rather than
assumed.

The neighbour to read first is `story:elapsed-time-claims`, which is about how long something took. This is
about what a point in time **is**, and the two compose: a duration between readings of two clocks is exactly
the claim that needs both.

## Acceptance

- [ ] A specification can declare a clock and bind a timestamp type to it.
- [ ] Comparing, ordering or subtracting timestamps of different clocks is refused without a declared crossing.
- [ ] A representation without an offset cannot stand as an instant.
- [ ] the internal specification repository can state that its push timestamps are local-time strings from a named producer's
      clock, and its own `UNMAPPED:` about that producer's pre-bootstrap timezone becomes a typed unknown
      rather than a comment.

## Out of scope

Leap seconds, calendars and civil-time arithmetic. Clock synchronisation as a runtime concern; this is about
what the model can say, not about keeping clocks in step.

## Provenance

Filed 2026-09-11 at the operator's request, from an internal application repository wave 3 — the wire-timestamp
finding is in that repository's `story:backend-dead-reads-and-stale-comments`.
