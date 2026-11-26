---
format: aep.planning-md/3
id: story:a-bus-route-forwards-an-event-to-a-sink-when-its-condition-holds
kind: story
status: proposed
title: A bus route forwards an event to a sink when its condition holds
summary: 'Content-based router: three-valued condition over an event, a sink; F4 C, F5 B decided'
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- depends_on: story:an-event-bus-contract-states-publish-subscribe-and-fan-out-channels-over-outward-events
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T16:09:54Z", actor: "human:operator", revision: 3}
---
## Need

The owner, 2026-11-26 about 12:35 CET: "this eventbus could have a simple routing logic - if
(condition) forward -> sink_x". Inside one system a binding's `where:` already routes an event to
a command under a three-valued condition: "True invokes, False skips this binding occurrence and
nothing else, Unknown is an unmet obligation that invokes nothing"
(`crates/specify/ess-domain/src/binding/condition.rs:28-33`). Across systems nothing routes. A
public trigger-rule design elsewhere states "entities of kind K in state S where X holds start
protocol P" with a verdict `undetermined` when a field is unknown. Design:
`docs/design/outward-events-and-event-bus.md` § 3 (branch `design/outward-events-and-event-bus`).

## Outcome

A bus contract's `routes:` each name a channel, an event, a condition and a sink:

```yaml
routes:
  - name: slow-hosted-routes
    from: stats
    event: gateway.usage.RouteStats
    where: [event.backend == hosted, event.p95_latency_ms > 5000]
    to: {system: alerts, input: alerts.pages.Raise}
```

True forwards, False skips, Unknown forwards nothing and is counted as undetermined on the route.
A sink is a channel of the same contract or a declared input of a pinned system.

## Acceptance

- A contract with routes validates and compiles; refused by name: a condition reading a member the
  event does not declare, a comparison of mismatched types, a sink of another kind, an unknown
  channel or event.
- The content-based router's reference machine (`ess-protospec/1`) holds "an event whose condition
  is False or Unknown reaches no sink of that route".
- Synthesis writes, per route, one event that makes the condition True, one False and one Unknown,
  or names a refusal for the case it cannot witness.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- The bus format crate (story of the bus contract), `crates/specify/ess-primitives/src/predicate.rs`
  if F4 extends the fragment, `crates/verify/ess-conformance/src/synthesize/`, `docs/design/`.

## Decisions

F4 = C: the route condition is ESS's binding `where:` fragment extended by numeric comparison and
list membership, plus a tested translation into `canon-expr/1`. F5 = B: no sink kind of its own for
"start protocol P"; a trigger evaluator is a system with a declared input, and that input is the
sink. The owner, 2026-11-26, relayed by the operator session. Rejected: F4 A without the
translation, F4 B `canon-expr/1` unchanged; F5 A a sink kind of its own, F5 C outside the bus.
