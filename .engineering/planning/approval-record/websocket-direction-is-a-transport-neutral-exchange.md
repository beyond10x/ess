---
format: aep.planning-md/3
id: approval-record:websocket-direction-is-a-transport-neutral-exchange
kind: approval-record
status: approved
title: 'The operator revises #650: a transport-neutral exchange, all three bindings possible (2026-10-28)'
summary: Supersedes the A-only answer; an ADR is proposed, nothing is built before acceptance
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- supersedes: approval-record:websocket-upgrade-is-a-sibling-binding
- decides: story:http-binding-carries-transport-framing
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:47:26Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

The operator, 2026-10-28, relayed verbatim by the orchestrator, superseding his earlier answer the
same day ("option A for the ESS WebSocket one."):

"Actually, I think we also kind of need B. Or I mean, we might even need A, B, and C. So it's just a
matter of, yeah, how we want to specify things, and most likely different ways should be possible. I
think still the most important is a generic transport one, yeah, because I mean this is about
messages being exchanged, and there should be a way to express that completely independent of
whether you are using a WebSocket or JSON RPC or whatever. So there need to be some interplay between
them, and we shouldn't restrict us to one or the other option."

The orchestrator's instruction with it: do not implement #650 in batch 1; record the direction;
draft an ADR, status proposed, for the operator's acceptance; no implementation before that.

## Effect

- `approval-record:websocket-upgrade-is-a-sibling-binding` is superseded.
- Amendment 2 of `architecture-decision-record:transport-and-idl-compile-targets` is marked
  withdrawn in a new revision; its text stays.
- `architecture-decision-record:messages-are-exchanged-independent-of-the-wire` is proposed: a
  transport-neutral exchange layer, with A, B and C (and `ess-jsonrpc/1`) as bindings of it.
- The WebSocket half of `story:http-binding-carries-transport-framing` waits on that record.

Decider: operator
