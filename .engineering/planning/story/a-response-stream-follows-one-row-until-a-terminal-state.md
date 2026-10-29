---
format: aep.planning-md/3
id: story:a-response-stream-follows-one-row-until-a-terminal-state
kind: story
status: draft
title: A response stream follows one row's events and closes at a terminal state
summary: No construct for a row-keyed, broadcast response stream (a2a wave 1, G5); blocked on the transport and IDL compile-targets ADR.
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 2
---
**Blocked on `architecture-decision-record:transport-and-idl-compile-targets`. Not to be implemented
before that record is accepted.** The ADR names a streamed answer as the one model-level question
its transports raise.

## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G5** (a2a wave-1 gaps list, line 14): a response stream keyed by one row. After a command or
  subscription, the caller receives every event about that row (broadcast to every subscriber of
  the row) and the stream closes when the row reaches a terminal state. No construct.

## Where it lives

No construct exists. `story:an-event-channel-may-be-bound-to-server-sent-events` carries such
a stream over SSE and depends on this story; `story:a-command-binds-to-json-rpc-and-grpc-beside-http`
names the same streamed-answer notion for gRPC streaming.

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch.

## Acceptance (provisional, fixed after the ADR)

- A command or view can declare a streamed answer keyed by its subject row, closed on a declared
  terminal state; synthesis checks that the stream carries the row's events in order and closes at
  the terminal state.
- Without the declaration every artifact keeps its bytes; an older format refuses it by name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1, not dispatchable before the ADR is accepted.
Likely a new format version.
