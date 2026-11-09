---
format: aep.planning-md/3
id: story:a-websocket-binding-frames-an-exchange
kind: story
status: draft
title: A WebSocket binding frames an exchange
summary: ess-websocket/1, a new sibling binding and crate; size M
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
- depends_on: story:asyncapi-and-openapi-project-an-exchange
scope:
- confidence: inferred
  path: README.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/websocket.rs
- confidence: cited
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: inferred
  path: crates/specify/ess-websocket/Cargo.toml
- confidence: inferred
  path: crates/specify/ess-websocket/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-websocket/tests/compile.rs
- confidence: inferred
  path: docs/design/websocket-binding.md
revision: 5
---
## Need

Once a socket is open, nothing states how each message of the exchange travels on it: the
subprotocol, text or binary frames, the field that names the message in a frame and each message's
wire name, a ping interval. ADR `messages-are-exchanged-independent-of-the-wire` § 2, binding A,
puts this in a sibling document `ess-websocket/1`; F3 (a) gives it a `path` only when no
`ess-http` route carries the upgrade (ADR lines 101, 107-109, 166).

Minimal reproduction: 0.92.0 has no WebSocket binding at all (`ess specify --help` lists no
`websocket` area; `ess-transport/2` binds NATS only), and the socket route itself is refused
(`triage/650/repro/run.txt`, `ESS-HTTP-023`, unknown field `upgrade`).

## Acceptance

- `ess specify websocket validate --path model --binding ws.yaml` (exit 0) and `compile` (writes
  `ess-websocket-ir/1`, byte-identical on a second run) for:

  ```yaml
  type: ess-websocket/1
  specification: {system: example, version: v1, source_digest: sha256:<hex>}
  sockets:
    - exchange: example.calls.Session
      subprotocol: calls.v1
      frames: text
      discriminator: type
      messages:
        example.calls.HangUp: hang_up
        example.calls.CallEnded: call_ended
      path: /calls
  ```

- Refused by name, codes `ESS-WEBSOCKET-001` onward, each at its path: a digest that does not match;
  an unknown exchange; a message the exchange does not list; a listed message left unbound; two
  messages with one wire name; a `frames` value outside `{text, binary}`; with `--http <file>`, a
  `path` on an exchange an `upgrade:` route already opens, and a missing `path` where none does;
  without `--http`, a missing `path`.
- `ess generate --path model --kind asyncapi --websocket ws.yaml` adds the AsyncAPI `ws` binding
  section and the server to the exchange's channel; without `--websocket` the output keeps its
  bytes.
- `docs/design/websocket-binding.md` (new, status accepted) states the document, the IR and the
  codes before the crate lands.
- Tests in `crates/specify/ess-websocket/tests/` (new crate).

## Scope

- inferred: crates/specify/ess-websocket/src/lib.rs (new crate, shaped like crates/specify/ess-transport/src/lib.rs)
- inferred: crates/specify/ess-websocket/tests/compile.rs (new)
- inferred: crates/specify/ess-websocket/Cargo.toml (new), Cargo.toml (workspace member)
- inferred: crates/edge/ess-cli/src/websocket.rs (new, shaped like crates/edge/ess-cli/src/transport.rs)
- cited: crates/edge/ess-cli/src/main.rs (`SpecifyCommand` 147, `--transport` flag 284-287, `Beside` 3475-3565)
- cited: crates/generate/ess-gen/src/asyncapi.rs (`TransportedAsyncApi` is the pattern for a binding-fed document)
- inferred: docs/design/websocket-binding.md (new)
- inferred: README.md (crate tree list)
confidence: medium

## Fit review

1. **Need.** State the framing of an exchange on a WebSocket so it is validated against the
   specification and projected. Requester's words: "the messages each side sends after it"
   (issue body, "Expected").
2. **Class: gap.** No binding carries a socket (`ess-transport/2` is NATS-only,
   `docs/design/event-transport-binding.md`).
3. **Already expressible?** No (reproduction above).
4. **Fit.** Not a grid cell (`docs/design/language-grids.md`). A sibling binding pinned to the
   source digest with a closed vocabulary, the shape of `ess-http/1` and `ess-transport/1`
   (`AGENTS.md` § Boundaries forbids a generic binding with a free transport field; this is not
   one). It names messages the exchange lists, as `ess-transport` names events. Targets: AsyncAPI
   here; generated servers and a conformance target over the socket are not built and the design
   page says so; `ess verify diff` does not read bindings, as for `ess-http/1`.
5. **Second adopter.** The price feed served on `wss://…/quotes` with no HTTP surface: the binding
   alone, with its `path`.
6. **Cost.** A new format pair `ess-websocket/1` and `ess-websocket-ir/1`, a new crate, a CLI area
   and a generate flag. No `ess/N` change and no migration.
7. **Alternatives.** Options A, B, C of `decision-blocker:websocket-upgrade-binding-form`; the
   operator chose all three as bindings of one exchange
   (`approval-record:websocket-direction-is-a-transport-neutral-exchange`). (a) Change nothing.
   (b) WebSocket as an `ess-transport` broker protocol: broker vocabulary does not fit a
   point-to-point socket. (c) Chosen: `ess-websocket/1` binding the exchange.

## Decisions

**accept, redesigned.** The requester asked for the messages on the HTTP route; per the accepted
ADR `messages-are-exchanged-independent-of-the-wire` (F3 (a)) the messages are the exchange's and
their framing is this sibling binding. Changed from the earlier body: this story depends on
`story:an-http-route-upgrades-to-an-exchange` (the path check reads its `upgrade:` routes through
`--http`) and on `story:asyncapi-and-openapi-project-an-exchange` (the channel the `ws` section
attaches to), and a design page is written first.

## Source

github:beyond10x/ess#650
