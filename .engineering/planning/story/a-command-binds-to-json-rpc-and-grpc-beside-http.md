---
format: aep.planning-md/3
id: story:a-command-binds-to-json-rpc-and-grpc-beside-http
kind: story
status: draft
title: A command or view binds to a JSON-RPC 2.0 method and a gRPC method, as ess-http/1 binds it to HTTP
summary: JSON-RPC 2.0 and gRPC bindings beside ess-http/1 (adopter item 8, beyond10x/a2a); blocked on the transport and IDL compile-targets ADR, not to be implemented before it is accepted.
tags:
- adopter-report
- feature-request
relations:
- serves: vision:O2
- depends_on: story:ess-projects-a-protobuf-idl-with-http-annotations
- informed_by: story:http-binding-carries-transport-framing
revision: 3
---
**Blocked on `architecture-decision-record:transport-and-idl-compile-targets`. Not to be implemented
before that record is accepted.**

## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 8 of 9, 2026-10-22). The
protocol serves the same operations over HTTP+JSON, JSON-RPC 2.0 and gRPC; ESS binds a command or
view to HTTP only (`ess-http/1`). No error text: there is no binding to author. Operator's words,
relayed: "technically ESS needs compile targets for all of these specs".

## Where it lives

- Sibling bindings: `docs/design/http-presentation-binding.md` (`ess-http/1`, decided via
  `decision-blocker:http-surface-scope`), `docs/design/event-transport-binding.md`
  (`ess-transport/1`).
- Draft `story:http-binding-carries-transport-framing` (streaming and framing on HTTP).

## 0.68.0

No.

## Cause

Not a defect: no RPC binding exists.

## Fit review

1. **Need.** Bind one command or view to a JSON-RPC 2.0 method and a gRPC method, with the result,
   error code and status per outcome stated and checked, as `ess-http/1` does for HTTP.
   Requester's wording: "JSON-RPC 2.0 and gRPC transport bindings beside ess-http/1".
2. **Class: gap.** The method names and error-code tables (A2A uses -32001 to -32009) live in
   adapter code nothing checks against the model.
3. **Already expressible?** No.
4. **Fit.** Same shape as `ess-http/1` and `ess-transport/1`: sibling documents `ess-jsonrpc/1` and
   `ess-grpc/1`, pinned to the source digest, compiled to their own `*-ir/1`, `ess/N` unchanged.
   gRPC messages and numbering come from the protobuf projection
   (`story:ess-projects-a-protobuf-idl-with-http-annotations`). Streaming needs a model-level
   streamed-answer notion shared with the HTTP SSE framing draft. Full outline:
   `docs/design/rpc-bindings.md`.
5. **Second adopter.** A language-server-style tool exposing commands over JSON-RPC 2.0; any
   service with gRPC and HTTP gateways.
6. **Cost.** Two new formats, generated servers and clients per target, a conformance target per
   binding, and possibly an `ess/N` bump for streaming. Large; hence the ADR.
7. **Alternatives.** (a) Change nothing. (b) Fold RPC into `ess-http/1`: a different protocol under
   an HTTP format. (c) A generic binding format: forbidden facet registry (`AGENTS.md`
   § Boundaries). (d) Chosen: two sibling formats, subject to the ADR.

## Decisions

**defer.** Blocked by `architecture-decision-record:transport-and-idl-compile-targets` (a `blocks`
edge from it). The story stays in draft until that record is accepted and
`docs/design/rpc-bindings.md` leaves draft. Depends on the protobuf projection story for gRPC.

## Acceptance (provisional, fixed after the ADR)

- An `ess-jsonrpc/1` binding for `examples/billing` compiles to `ess-jsonrpc-ir/1`; an unknown
  operation, duplicate method or unmapped error is refused by name.
- An `ess-grpc/1` binding compiles against the protobuf projection's numbering.
- Without either binding every artifact keeps its bytes.

## a2a wave 1

Relayed by the orchestrator on 2026-10-22 from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050). All four are parts of the JSON-RPC binding this story
outlines:

- **J1** (a2a wave-1 gaps list, line 22): no numeric `code` attribute on an error. In this story
  the code belongs to the binding's per-outcome error table (`ess-jsonrpc/1`), not to `ess/N`.
- **J2** (a2a wave-1 gaps list, line 23): no JSON-RPC method naming beside `naming.wire`.
- **J3** (a2a wave-1 gaps list, line 24): one endpoint for every method is refused by `ess-http/1`
  as ESS-HTTP-010 (`crates/specify/ess-http/src/lib.rs:5125`, "are one route on component"). JSON-RPC
  routes by method inside one HTTP endpoint, so it needs its own binding rather than more HTTP routes.
- **J7** (a2a wave-1 gaps list, line 28): no JSON-RPC or OpenRPC projection. Add an OpenRPC
  projection of `ess-jsonrpc/1` to this story's acceptance once the ADR is accepted.

Claims on these codes in authored scenarios, and the protocol-level refusals -32602 and -32601,
are `story:an-authored-scenario-claims-a-binding-error-code`, which depends on this one.

## Adopter report N28 2026-10-23

A JSON-RPC 2.0 envelope requested as `request: {envelope: ...}` in `ess-http/1` (ess#650 comment). It belongs here as `ess-jsonrpc/1`, behind ADR `transport-and-idl-compile-targets` (proposed).
