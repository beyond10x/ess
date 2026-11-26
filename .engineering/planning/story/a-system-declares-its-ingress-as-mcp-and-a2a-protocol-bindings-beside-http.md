---
format: aep.planning-md/3
id: story:a-system-declares-its-ingress-as-mcp-and-a2a-protocol-bindings-beside-http
kind: story
status: draft
title: A system declares its ingress as MCP and A2A protocol bindings beside HTTP
summary: Semantics, protocol port, binding and transport apart, a port pinned by digest; F7 open
tags:
- design
- event-bus
- issues-2026-11-26
relations:
- serves: vision:O2
- decomposes: epic:outward-events-and-an-event-bus-contract
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- informed_by: story:a-command-binds-to-json-rpc-and-grpc-beside-http
revision: 1
---
## Need

The owner, 2026-11-26 about 12:47 CET: "rfc based or conformance based "inputs" ingresses? could
also include mcp,a2a,etec". A system's commands and views bind to HTTP through `ess-http/1`; the
JSON-RPC and gRPC bindings are a draft (`docs/design/rpc-bindings.md`); MCP and A2A have no binding,
and no RFC-derived or published-suite protocol port is pinned by a binding. Design:
`docs/design/outward-events-and-event-bus.md` § 5 (branch `design/outward-events-and-event-bus`).

## Outcome

A system declares its ingress as protocol bindings, four layers apart: the semantics (commands,
views, an exchange), the protocol port (an `ess-protospec/1` document written from an RFC or from a
published specification with its own suite), the binding (which operation carries which command or
view), and the transport (standard streams, HTTP with server-sent events, a unix socket). Under F7
A:

- an MCP binding maps a command to a tool (name, input schema from the command's input) and a view
  to a resource;
- an A2A binding maps a command to a skill and a task's updates to outward events of that exchange;
- each pins its protocol port by digest, as the broker-subject story pins `protocol:`.

## Acceptance

- An MCP binding of two commands and one view validates and compiles; refused by name: an unknown
  command or view, a tool named twice, an input schema the command's input does not give, a port
  digest that differs.
- The same for an A2A binding of one command.
- The specification's scenarios run through a target adapter per binding against a generated or
  reference server; a faulty server (a tool answering another command's outcome) fails naming it.
- Each test runs within 60 s, each test step within 120 s.

## Scope

- New crates under `crates/specify/` per binding format, `crates/verify/ess-conformance/src/` for
  the adapters, `docs/design/rpc-bindings.md` (the JSON-RPC layer both protocols use),
  `website/docs/reference/formats.md`.

## Decisions

Open, for the owner: F7 (A one sibling format per protocol, B one `ess-ingress/1` with a closed
`protocol:` set, C MCP and A2A as methods of the draft `ess-jsonrpc/1`). The design page recommends
A; no default is taken.
