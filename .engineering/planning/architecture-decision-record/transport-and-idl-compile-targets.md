---
format: aep.planning-md/3
id: architecture-decision-record:transport-and-idl-compile-targets
kind: architecture-decision-record
status: proposed
title: 'Transport and IDL compile targets beyond ess-http/1: protobuf IDL projection, JSON-RPC 2.0 and gRPC bindings'
summary: Reverses the 2026-09-15 deferral of protobuf for the projection direction and admits ess-jsonrpc/1 and ess-grpc/1 sibling bindings; import of .proto stays deferred.
relations:
- serves: vision:O2
- informed_by: task:deferred-protocol-ui-bindings
- informed_by: approval-record:ess-evolution-20260915
- informed_by: architecture-decision-record:ess-evolution-02-semantics-before-bindings
- blocks: story:ess-projects-a-protobuf-idl-with-http-annotations
- blocks: story:a-command-binds-to-json-rpc-and-grpc-beside-http
- blocks: story:an-event-channel-may-be-bound-to-server-sent-events
- blocks: story:a-response-stream-follows-one-row-until-a-terminal-state
- blocks: story:an-authored-scenario-claims-a-binding-error-code
revision: 1
---
Status: proposed (draft). Not accepted; nothing under it is dispatched before acceptance.

## Context

ESS projects one transport today. A command or view binds to HTTP through `ess-http/1`
(`docs/design/http-presentation-binding.md`, scope decided on `decision-blocker:http-surface-scope`),
an event to a broker through `ess-transport/1` (`docs/design/event-transport-binding.md`), and
`ess generate` projects docs, site, docs-ir, schema, openapi and asyncapi
(`crates/edge/ess-cli/src/main.rs:550-562`). There is no IDL projection and no RPC binding.

On 2026-09-15 the operator deferred generic protobuf, with UI and Flutter, out of the ESS evolution
initiative: "Track generic protobuf, UI and Flutter as separate deferred follow-up"
(`approval-record:ess-evolution-20260915`), preserved as `task:deferred-protocol-ui-bindings` and
`docs/design/ess-evolution/feature-preservation.md:41`.

On 2026-10-22 an adopter specifying A2A (beyond10x/a2a) needs the same operations served over
HTTP+JSON, JSON-RPC 2.0 and gRPC, with a protobuf IDL as the normative message source. The
operator's words, relayed by the orchestrator: "technically ESS needs compile targets for all of
these specs".

## Decision

Reverse the 2026-09-15 deferral of protobuf for the projection direction, and admit RPC bindings:

1. **IDL projection.** `ess generate --kind proto` projects a protobuf IDL (`.proto`) from the
   compiled model, with `google.api.http` annotations from `ess-http/1` when present. Field numbers
   are wire identity and live in a sibling binding pinned to the source digest, not in `ess/N`.
   Design: `docs/design/protobuf-projection.md`.
2. **RPC bindings.** Sibling documents `ess-jsonrpc/1` and `ess-grpc/1`, shaped like `ess-http/1`:
   authored separately, pinned to the specification's source digest, compiled against the resolved
   IR into their own `*-ir/1`. Design: `docs/design/rpc-bindings.md`.
3. **The model does not change.** `ess/N` stays transport-free. The one model-level question these
   raise, a streamed answer, is decided in the design pages and, if it needs `ess/N`, takes the
   next format under its own decision.
4. **Still deferred.** Importing an existing `.proto` into ESS, and the UI and Flutter parts of
   `task:deferred-protocol-ui-bindings`.

## Rejected alternatives

- **Keep the deferral.** Adopters of multi-binding protocols keep a hand-written IDL and RPC
  adapters that nothing checks against the model.
- **Transport keys in `ess/N`.** Puts wire choices into the domain model, against ADR 02
  (`architecture-decision-record:ess-evolution-02-semantics-before-bindings`) and the precedent of
  `ess-http/1` and `ess-transport/1`.
- **One generic "binding" format with a transport field.** A generic facet registry, which
  `AGENTS.md` § Boundaries forbids.
- **Field numbers derived from declaration order.** Silent renumbering on reorder breaks every
  deployed client.

## Compatibility and migration

Opt-in only. Without a protobuf, JSON-RPC or gRPC binding every artifact keeps its bytes. Each new
format (`ess-proto/1`, `ess-jsonrpc/1`, `ess-grpc/1`) starts at 1 and refuses unknown keys by name.
Union `oneof` mapping depends on `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`, after the two design
pages are reviewed. Until then `story:ess-projects-a-protobuf-idl-with-http-annotations` and
`story:a-command-binds-to-json-rpc-and-grpc-beside-http` stay in draft and are not implemented.
