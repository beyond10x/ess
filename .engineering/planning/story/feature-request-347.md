---
format: aep.planning-md/3
id: story:feature-request-347
kind: story
status: draft
title: Reconcile grant refusal log observation with consumer runners
refs:
- provider: github
  reference: beyond10x/ess#347
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

Reconcile the consumer report that expect_not_granted cannot observe unpublished events against the shipped target protocol and runner behavior; identify any missing integration guidance without inventing a redundant suite step.

## Fit review

1. Need: a custom consumer runner must detect a command that commits/publishes before returning not-granted, including events absent from its direct answer. Issue347 reports91 grant cases passing an answer-only check.
2. Class: source evidence points to a consumer protocol/integration gap, not absence of this capability in ESS runtimes. runner.rs:432-445 counts before invocation; :1417-1477 observes/counts after refusal. ConformanceTarget::observe_events at target.rs:212 exposes the independent event log.
3. Existing expression: expect_not_granted.unpublished already requires whole-log observation (scenario.rs:2067-2074). author-scenarios.md:147-151 states this requirement; runners.md names ObserveEvents/observeEvents. Command direct events alone are insufficient by design.
4. Fit: retain the current expectation and event-observation port. Consumers implementing their own suite runner must perform its pre-send and post-refusal log observations. A new explicit step would duplicate the current semantics and require a format decision; it is not necessary merely to expose the existing port.
5. Second adopter: a denied invoice command publishes a duplicate of an earlier event. Counting occurrences rather than comparing sets must fail it.
6. Cost: no authored/suite format or runtime change demonstrated. Any documentation clarification must explain the port, correlation and ordering; an adapter that cannot observe the log must not report pass for an unverified obligation.
7. Alternatives: add a redundant standalone observation step; use before/after views that miss event-only changes; selected existing log-observation expectation plus precise integration evidence.

## Current evidence

At exact published55061600, executed cargo test -p ess-conformance --locked --test adversary_265_pass2 -- --nocapture:7passed,0failed,0ignored. Node and tsc were available; neither generated runtime test skipped. Cases include a late-refusing gatepass surface, repeated earlier occurrence, generated Go refusal carrying published events, and generated TypeScript target publishing into its log before refusal. Raw evidence: managed tree batch-0-51, target/ui-consolidation/grant-log-audit.log. This is actual existing-behavior verification, not a new regression fix or reproduction of the private consumer runner.

## Decisions

Existing ESS expectation/port satisfies the requested independent observation capability. Do not claim the private consumer integration is corrected; its runner must implement ObserveEvents with the actual event log and perform both observations. Keep the reported downstream failure distinguished from ESS runtime behavior. Further work is a bounded integration-guidance clarification if the current docs leave the before-send requirement unclear.

## Acceptance

Retain exact runtime evidence and identify the documented target method, correlation scope, before/after ordering and duplicate-occurrence semantics. No answer-only or view-only substitute can discharge unpublished event checks. Private consumer correctness remains unverified without its adapter evidence.
