---
format: aep.planning-md/3
id: epic:message-contract-clients
kind: epic
status: draft
title: A message contract in ESS projects a complete AsyncAPI and generates typed Go and Rust publishers
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A message contract written in ESS — a producer publishes events to a broker subject that a persistent stream captures — projects to a JSON Schema and an AsyncAPI document that say everything a hand-written contract says, and generates typed Go and Rust publisher clients from the same model.

## Scope

- Integer bounds and constants an invariant states reach JSON Schema keywords, and bounded integers realize at native widths (beyond10x/ess#394).
- An event's payload is selectable as a generated type (beyond10x/ess#393).
- A transport document binds events to a broker, subject, stream and envelope, and the AsyncAPI projection carries it (beyond10x/ess#390, beyond10x/ess#392).
- `ess generate client` emits a typed publisher over that transport for Go and Rust (beyond10x/ess#395).

Out of scope: channel address templates bound to payload fields (beyond10x/ess#391), consumer clients.

## Origin

Six gaps filed 2026-10-02 (beyond10x/ess#390–#395) by an adopter specifying a metering contract whose only substance is "publish these items to this subject, captured by this stream". The adopter's hand-written AsyncAPI, JSON Schema and two hand-written clients are the parity bar.
