---
format: aep.planning-md/3
id: story:asyncapi-and-openapi-project-an-exchange
kind: story
status: draft
title: AsyncAPI and OpenAPI project an exchange
summary: One AsyncAPI channel per exchange; the OpenAPI 101 route links to it; size M
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- depends_on: story:an-exchange-is-declared-in-the-specification
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-http-route-upgrades-to-an-exchange
revision: 3
---
## Need

The generated documents must say which party sends which message of an exchange, whatever the
wire, and an OpenAPI reader must see that a route upgrades instead of answering (ADR
`messages-are-exchanged-independent-of-the-wire` § 3, lines 111-126). Today AsyncAPI has one
channel per event and nothing for a conversation, and OpenAPI has no way to show a `101`. With the
reproduction of `story:an-exchange-is-declared-in-the-specification` (`example.calls.Session`) and
the `upgrade:` route of `story:an-http-route-upgrades-to-an-exchange`, neither document mentions the
exchange.

## Acceptance

- `ess generate --path model --kind asyncapi` on the specification declaring `example.calls.Session`
  emits one channel for the exchange with `x-ess-exchange: example.calls.Session`, a `receive`
  operation for `example.calls.HangUp` and a `send` operation for `example.calls.CallEnded`, from
  the responder's side, and no protocol binding section.
- `ess generate --path model --kind openapi --http http-upgrade.yaml` emits `GET /calls` whose only
  success response is `101`, carrying `x-ess-exchange: example.calls.Session`; the messages are not
  described in OpenAPI.
- A specification without exchanges, and a binding without `upgrade:`, keep their AsyncAPI and
  OpenAPI bytes (existing byte tests pass unmodified).
- Tests in `crates/generate/ess-gen/tests/asyncapi_exchange.rs` (new) and one OpenAPI case beside
  the existing `ess-http` OpenAPI tests.

## Scope

- cited: crates/generate/ess-gen/src/asyncapi.rs (`documents`, line 228)
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-gen/tests/asyncapi_exchange.rs (new)
- inferred: crates/generate/ess-gen/tests/ (one OpenAPI upgrade case)
confidence: medium

## Fit review

1. **Need.** The projected documents state each message's direction and the upgrade, derived from
   the one exchange. No adopter syntax; the requester's "projected into the generated documents"
   (issue body, "Expected").
2. **Class: gap.** Neither projection has a channel or response for a conversation
   (`crates/generate/ess-gen/src/asyncapi.rs:228`, one document per component from events and
   bindings only).
3. **Already expressible?** No; projections derive from the model and the binding, which cannot
   state it before the two stories this depends on.
4. **Fit.** Not a grid cell (`docs/design/language-grids.md`); a projection. Reuses the existing
   `x-ess-*` extension naming (`asyncapi.rs:294`, `x-ess-periodic-bindings`). Protocol sections
   come from each binding's own story (`ws` in `story:a-websocket-binding-frames-an-exchange`,
   `nats` in `story:a-broker-binding-carries-an-exchange`), so this story owns the wire-free channel.
5. **Second adopter.** The price feed of the exchange story: its AsyncAPI shows `Subscribe` and
   `Unsubscribe` received and `QuoteChanged` sent on one channel.
6. **Cost.** No source or binding format; AsyncAPI and OpenAPI output change only for documents
   using the new constructs. No generated-code change.
7. **Alternatives.** (a) Change nothing: the exchange is invisible to readers of the documents.
   (b) Describe the socket's messages in OpenAPI: OpenAPI has no message model for it; refused by
   the ADR (line 122-123). (c) Chosen: AsyncAPI channel per exchange, OpenAPI `101` linking to it.

## Decisions

**accept as proposed** (as the accepted ADR `messages-are-exchanged-independent-of-the-wire` § 3
states it). Refined here: the OpenAPI half now depends on `story:an-http-route-upgrades-to-an-exchange`,
which the earlier body did not name, and protocol sections belong to each binding's story.

## Source

github:beyond10x/ess#650
