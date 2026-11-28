---
format: aep.planning-md/3
id: story:a-periodic-aggregate-event-publishes-an-aggregate-view-s-answer-every-period
kind: story
status: proposed
title: A periodic aggregate event publishes an aggregate view's answer every period
summary: Statistics once per period, checked against a view; F3 A decided
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- depends_on: story:a-system-declares-which-of-its-published-events-are-outward
- informed_by: story:a-component-declares-level-and-time-steps-that-validate-checks-and-the-interpreter-runs
- informed_by: story:periodic-binding-trigger-contract
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T16:09:54Z", actor: "human:operator", revision: 3}
---
## Need

A producer publishes statistics once per period (requests, tokens, latency and cost per route),
never one event per occurrence. Today a periodic binding runs a command every period
(`docs/design/periodic-binding-triggers.md:5-9`) and an aggregate view answers counts, sums and
extremes (`docs/design/aggregate-views.md`), but nothing publishes a view's answer as an event on a
period, so the statistics payload is checked against nothing. Design:
`docs/design/outward-events-and-event-bus.md` § 2, "Periodic aggregate" (branch
`design/outward-events-and-event-bus`).

## Outcome

Under F3 A an outward event may be declared as the answer of an aggregate view, sampled every
period:

```yaml
outward:
  - event: gateway.usage.RouteStats
    every: PT60S
    from_view: gateway.usage.RouteTotalsSinceLast
```

The payload type is the view's row type as a list; a period with no rows publishes an empty list.
The view's window is the time since the previous emission.

## Acceptance

- Validation refuses a `from_view` that is not an aggregate view, an event whose payload type is
  not the view's row list, and an `every` that is not a positive duration, each by name.
- The interpreter, on logical time, publishes one event per period whose payload equals the view's
  answer for that window, including an empty one; synthesis writes a scenario with rows and one
  without.
- `ess verify diff` reports a changed period or view as breaking.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- `crates/specify/ess-domain/src/component.rs`, `crates/specify/ess-compiler/src/`,
  `crates/verify/ess-conformance/src/interpret/`, `crates/verify/ess-conformance/src/synthesize/`,
  `crates/verify/ess-diff/src/change.rs`, `docs/design/`.

## Decisions

F3 = A: an outward event sampled from an aggregate view every period (`every`, `from_view`); F1 = A
puts `outward:` in the specification. The owner, 2026-11-26, relayed by the operator session.
Rejected: B, an ordinary event a time step emits (its payload is checked against no view); C,
aggregated by the bus (per-occurrence traffic the owner excluded).
