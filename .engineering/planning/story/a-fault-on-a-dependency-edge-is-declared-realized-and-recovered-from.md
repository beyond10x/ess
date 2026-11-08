---
format: aep.planning-md/3
id: story:a-fault-on-a-dependency-edge-is-declared-realized-and-recovered-from
kind: story
status: draft
title: A fault on a deployment's dependency edge is declared, realized by a projection, and recovery from it is a checked outcome
relations:
- serves: vision:O2
- informed_by: story:declared-fault-injection
- informed_by: epic:concurrent-history-conformance
revision: 1
---
## Need

An adopter on 0.90.0 specified a WebSocket push service (several language implementations, replicas
fanned out over a message broker, sessions in memory or in a replicated key-value store). Every
implementation passed its full synthesized conformance suite, yet one delivered messages late after a
broker reconnect: the client library buffered them, where the specification says a message older than
one second is dropped and counted.

The suite could not see it, for three reasons:

1. The specification declares the broker's condition as an input (`answering | unavailable`), and
   the adopter realizes `unavailable` in hand-written target code by pausing the broker process. A
   paused peer keeps its TCP connection open, so the client never observes a disconnect. Which kind
   of outage a scenario means is not stated anywhere ESS reads.
2. The fault is held until the scenario ends; no scenario observes what is delivered after recovery.
3. The service's dropped-message counter (with a reason) is not mapped to anything a scenario can
   read.

ESS already injects the faults a specification declares inside its own model
(story:declared-fault-injection: a duplicated `at_least_once` delivery, a client retry, a delayed
or unanswered `external:` branch). It has no vocabulary for a fault on a dependency edge of the
deployment (service to broker, service to store), no recovery outcome, and no realization of such a
fault that does not live in hand-written target code.

## Requester's proposal (theirs, evidence of the need, not a design)

- A document kind such as `ess-faults/1` naming the dependency edges an `ess-stack/1` declares and
  the fault kinds allowed on each: pause (silent peer), reset or kill, blackhole or partition,
  latency or loss, error responses at a proxy, restart with state loss, clock skew, resource
  exhaustion.
- Scenarios take a fault as a typed input with a duration and a phase (before, during, after the
  step); recovery expectations as outcomes: ready again within N seconds, nothing delivered after
  recovery within a window, a named metric incremented with a reason, invariants that hold
  throughout.
- Projections realize the faults (a container sidecar proxy, Kubernetes chaos resources, proxy
  fault-filter values in a chart), behind one fault-control interface in the generated runner.
- Optionally a seeded, randomized fault schedule that checks the invariants and shrinks a failing
  schedule.
- Metrics as observable outputs a scenario can read.

## Fit review (preliminary; to be completed before the story leaves draft)

1. **Need.** A specification cannot state which observable kind of outage an unavailable dependency
   is, nor what must hold after it recovers, so a suite passes an implementation that violates the
   specification's own timing rule after a reconnect. Gap, not convenience: no construct reaches it
   today (`external:` branches model a dependency's answer, not the transport's state).
2. **Class.** Gap.
3. **Already expressible?** Partly: the condition as an input enum, realized by target code. That
   moves the checked figure (which outage, what after recovery) out of the specification, so it is
   no idiom under `docs/design/generic-language-features.md` § Policy, policy 1.
4. **Fit.** It extends story:declared-fault-injection and epic:concurrent-history-conformance (one
   checker, one model, faults only where declared) from model-level faults to deployment edges. A
   fault realization is a projection of declared intent; ESS still applies nothing (AGENTS.md
   § Boundaries): the generated runner drives a fault-control port the adopter's environment
   provides, as the target port does today.
5. **Second adopter.** Any service with a database: a primary fails over, and the specification
   says writes during the failover are refused, never lost; the suite should cut the edge, then
   check refusals during and every acknowledged write after.
6. **Cost.** At least one new format (or keys in `ess-stack/2`), new scenario inputs and outcome
   kinds in the suite format, a fault-control port in each runner (Rust, Go, TypeScript), and
   projections per platform. Size L; to be split into stories once the open questions are decided.
7. **Alternatives.** (a) Change nothing: adopters keep hand-written fault code and the gap stays
   invisible. (b) Recovery outcomes and metric reads only, faults still realized by the target:
   closes reasons 2 and 3, not reason 1. (c) The full shape above, staged: (b) first, then declared
   fault kinds on stack edges, then projections, then randomized schedules.

## Open questions (architecture, to be decided before decomposition)

- Where fault kinds are declared: a separate document beside the stack, or on the stack's edges in
  `ess-stack/2` (decided for optional systems on 2026-11-08).
- Whether a metric is a new observable kind in the specification (a counter with labels) or a view
  over an event the implementation publishes.
- Which realizations ship first, and whether the fault-control port is one interface across
  platforms.

## Decisions

Deferred: the need is accepted as a gap; the shape waits on the open questions above. Staging
recommendation: alternative (c), starting with recovery outcomes and metric reads.

## Acceptance (target, for the decomposed stories)

- A specification states the fault kind on a dependency edge; a scenario takes it with a duration
  and phase, and states a recovery outcome.
- A generated realization replaces hand-written fault code for at least the pause and the
  connection-reset kinds.
- On a minimal neutral example of the case above, a disconnect-type fault fails the suite against a
  build that delivers buffered messages late, and passes against a build that drops and counts them;
  the counter's increment, with its reason, is read by the scenario.
- A specification that declares no faults produces the same suite bytes as before.
