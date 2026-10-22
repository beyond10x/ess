---
format: aep.planning-md/3
id: story:an-event-channel-may-be-bound-to-server-sent-events
kind: story
status: draft
title: A command's streamed answer may be bound to server-sent events
summary: SSE is refused as a broker protocol (a2a wave 1, G1); blocked on the transport and IDL compile-targets ADR.
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-response-stream-follows-one-row-until-a-terminal-state
revision: 2
---
**Blocked on `architecture-decision-record:transport-and-idl-compile-targets`. Not to be implemented
before that record is accepted.**

## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G1** (a2a wave-1 gaps list, line 10): server-sent events (SSE) as a channel binding for a
  command response. The transport binding refuses the protocol:

  ```text
  brokers[0].protocol: unknown variant sse, expected nats
  ```

  (the adopter's `components.yaml`). The protocol streams task updates to the caller over SSE on
  the same HTTP connection that carried the request.

## Where it lives

- `crates/specify/ess-transport/src/lib.rs:46-52`: `Protocol` has one member, `Nats`, documented as
  "A closed set: a second member is a format change."
- The streamed answer this channel would carry is
  `story:a-response-stream-follows-one-row-until-a-terminal-state`; this story depends on it.
- `story:http-binding-carries-transport-framing` names long-polling, CORS, JSONP and WebSocket
  framing on `ess-http/1`, not SSE; whether SSE belongs to `ess-transport/1` (a broker protocol) or
  to `ess-http/1` framing is the first design question, and part of the ADR's streamed-answer
  question.

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch.

## Acceptance (provisional, fixed after the ADR)

- A specification can bind a command's streamed answer to SSE; the binding validates and compiles,
  and the AsyncAPI or OpenAPI projection documents it.
- Without the binding every artifact keeps its bytes; an older format refuses the new protocol by
  name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1, not dispatchable before the ADR is accepted.
