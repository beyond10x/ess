---
format: aep.planning-md/3
id: story:an-operation-may-have-additional-http-routes
kind: story
status: draft
title: An ess-http/1 operation may have additional routes beside its first, as protobuf additional_bindings
summary: ESS-HTTP-005 allows one route per operation; A2A serves each operation also under /{tenant}/ (a2a wave 2, H2, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 3
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H2**, a2a wave-2 HTTP binding gaps list, line 9. Every upstream
A2A operation has a second route under a tenant prefix (`/{tenant}/…`, protobuf
`additional_bindings`). `ess-http/1` allows one route per operation:

> ESS-HTTP-005: "command `a2a.tasks.SendMessage` is already routed by `routes[0]`; an operation has one route"

## Where it lives

- Refusal: `crates/specify/ess-http/src/lib.rs:3240-3244`.
- Design: `docs/design/http-presentation-binding.md:169` (`ESS-HTTP-005`, one operation routed twice
  in one document).
- Every consumer of the route table assumes one route per operation: refinement compares "the first
  route that names it" (`lib.rs:2793-2794`), and `ESS-HTTP-012` would have to compare route sets.
- The tenant variable is `Optional` in the adopter's model, which is H3
  (`story:an-optional-input-may-select-between-an-operations-routes`).

## Format

Changes what an `ess-http/1` document means (an operation's route becomes a set), and the IR route
table's identity (ordered by path, then method, then operation, `lib.rs:1837`) gains duplicates per
operation. Likely an additive key (`additional:` under a route) within `/1`, but the IR and the
refinement rule change shape; the fit review states whether that is `ess-http/2`. `ess/N` unchanged.
