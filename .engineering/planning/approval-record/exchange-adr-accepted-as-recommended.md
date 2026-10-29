---
format: aep.planning-md/3
id: approval-record:exchange-adr-accepted-as-recommended
kind: approval-record
status: approved
title: The exchange ADR is accepted with F1, F2 and F3 as recommended (operator, 2026-10-28)
summary: 'F1 a, F2 a, F3 a; #650 becomes its own format-moving batch'
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- decides: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:54:26Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

The operator, 2026-10-28, relayed verbatim by the orchestrator: "F1, F2, F3, AAA, all as you
suggested."

`architecture-decision-record:messages-are-exchanged-independent-of-the-wire` is accepted with its
three forks as recommended:

- F1 (a): the exchange lives in the specification, `exchanges:` at the next `ess/N`.
- F2 (a): order comes from the followed row's lifecycle and the commands' guards.
- F3 (a): an `ess-http` route carries the handshake and `ess-websocket/1` the framing; the
  WebSocket binding states a path only when no HTTP route opens the socket.

## Effect

#650 becomes its own batch, after the two batches that change no source format, because it moves
`ess/N`: `epic:messages-are-exchanged-independent-of-the-wire` and one story per layer.

Decider: operator
