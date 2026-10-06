---
format: aep.planning-md/3
id: story:feature-request-390
kind: story
status: implemented
title: Events travel by a checked ess-transport/1 document, and AsyncAPI carries it
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#390
- provider: github
  reference: beyond10x/ess#392
relations:
- serves: vision:O2
- decomposes: epic:message-contract-clients
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:43:28Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:43:28Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:28Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

How the events of one exact ESS travel — broker, subject, the stream that captures it, envelope and delivery — is stated in a checked `ess-transport/1` document, and the AsyncAPI projection carries it.

## Acceptance

- `ess-transport/1` (`crates/specify/ess-transport`) parses strictly and compiles against the specification it pins (system, version, `sha256:` source digest) into canonical `ess-transport-ir/1`; every refusal in `docs/design/event-transport-binding.md` has its code `ESS-TRANSPORT-001`–`016` and the offending key's path (`crates/specify/ess-transport/tests/compile.rs`).
- A channel on a JetStream broker whose subject no stream captures is refused before anything runs (`ESS-TRANSPORT-015`), and so is one two streams capture (`ESS-TRANSPORT-016`).
- A channel's subject equals the event's `naming.wire` where the event declares one.
- `ess specify transport validate|compile --path <doc> --spec <model>` (`crates/edge/ess-cli/tests/transport.rs`).
- `ess generate --kind asyncapi --transport <doc>`: `servers` per used broker (fixed `host` or a `{host}` variable), a bound channel's address is its subject with `x-ess-address-source: transport`, `servers` and `x-ess-stream`; an `array` envelope makes the payload an array of the event payload with `x-ess-envelope: array`; the send operation carries `x-ess-delivery` and `x-ess-batch` (`crates/generate/ess-gen/tests/asyncapi_transport.rs`).
- Without `--transport`, every AsyncAPI byte is unchanged; `--transport` with another `--kind` is refused.

## Origin

beyond10x/ess#390 (no transport construct) and beyond10x/ess#392 (an event cannot travel in a JSON array envelope). Belongs to `epic:message-contract-clients`; the edge is added once that epic is on `main`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, 2026-10-03.

1. **Need.** A message contract is "publish to subject X; a persistent stream captures X for 90 days; one message is a JSON array of items; producers batch and do not retry; producers never create the stream". The AsyncAPI projection says "the specification declares no transport" and emits empty `servers` (`crates/generate/ess-gen/src/asyncapi.rs`, `describe`). Requester's proposal: a top-level `transports:` key in the model, and an envelope key on events.
2. **Class.** Gap.
3. **Already expressible?** No. `naming.wire` gives an address and nothing else; the event payload is always one object.
4. **Fit.** The model keeps no transport on purpose (`asyncapi.rs` header; `ess-domain` keeps a topic out of an event). A sibling document pinned by digest is how ESS already adds a concrete binding without changing `ess/N` (`crates/specify/ess-realization`, `ess-realization/1`). `delivery` reuses `ess_domain::binding::Delivery`. The envelope belongs to the transport, since the same fact may travel singly on another broker.
5. **Second adopter.** An order service publishing `OrderPlaced` to `orders.placed`, captured by a work-queue stream it owns (`owner: publisher`), one event per message.
6. **Cost.** A new authored format and crate, one `specify` verb, one `generate` flag, sixteen diagnostic codes; no `ess/N` bump, no diff kind; AsyncAPI bytes unchanged without the flag.
7. **Alternatives.** (a) Change nothing: transport in prose, no client can be generated. (b) `transports:` in the model (requester's proposal): an `ess/21` bump, and a broker inside the semantic model, which the model's own boundary refuses. (c) A sibling `ess-transport/1` (chosen).

## Decisions

- **accept, redesigned (2026-10-03):** a digest-pinned sibling document `ess-transport/1` instead of model keys; NATS is the only protocol; envelope and batch live on the channel; `owner: external | publisher` on streams. Address templates (beyond10x/ess#391) stay out.
