---
format: aep.planning-md/3
id: story:ess-projects-a-protobuf-idl-with-http-annotations
kind: story
status: draft
title: ess generate --kind proto projects a protobuf IDL with google.api.http annotations
summary: Protobuf IDL projection (adopter item 7, beyond10x/a2a); blocked on the transport and IDL compile-targets ADR, not to be implemented before it is accepted.
tags:
- adopter-report
- feature-request
relations:
- serves: vision:O2
- depends_on: story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant
- informed_by: task:deferred-protocol-ui-bindings
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
revision: 1
---
**Blocked on `architecture-decision-record:transport-and-idl-compile-targets`. Not to be implemented
before that record is accepted.**

## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 7 of 9, 2026-10-22). The
protocol's normative message definitions are a protobuf IDL with `google.api.http` annotations; ESS
cannot project one. No error text: `ess generate` has no such kind (`docs, site, docs-ir, schema,
openapi, asyncapi`). Operator's words, relayed: "technically ESS needs compile targets for all of
these specs".

## Where it lives

- Projection enum `crates/edge/ess-cli/src/main.rs:550-562`; projections in
  `crates/generate/ess-gen/` (`openapi.rs`, `asyncapi.rs`).
- Protobuf deferred on 2026-09-15 in `task:deferred-protocol-ui-bindings` (draft) and
  `docs/design/ess-evolution/feature-preservation.md:41`; that deferral is what the ADR reverses.

## 0.68.0

No.

## Cause

Not a defect: a projection that was deliberately deferred.

## Fit review

1. **Need.** Project a protobuf IDL from the model, so the IDL is derived and drift-checked rather
   than hand-maintained beside the specification. Requester's wording: "--kind proto projection
   (protobuf IDL + google.api.http)".
2. **Class: gap.** The model's messages cannot be stated as an IDL; protocols whose normative
   source is `.proto` have an unchecked second source.
3. **Already expressible?** No. The OpenAPI projection covers HTTP+JSON only.
4. **Fit.** Placed beside openapi and asyncapi in ess-gen; `ess/N` unchanged; field numbers in a
   sibling binding pinned to the source digest (`ess-proto/1`), as `ess-http/1` pins routes. Unions
   map to `oneof` via `story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`. Every
   construct `.proto` cannot express is refused by name with coverage reported. Full outline:
   `docs/design/protobuf-projection.md`.
5. **Second adopter.** Any service published to gRPC clients, e.g. an internal inventory service
   whose consumers generate stubs from `.proto`.
6. **Cost.** A new projection kind, a new sibling format (`ess-proto/1`), new diff classifications
   for numbers (`reserved`), a descriptor-level compatibility check. Large; hence the ADR.
7. **Alternatives.** (a) Keep the deferral. (b) Numbers from declaration order: unstable on
   reorder. (c) Numbers in `ess/N`: wire identity in the model. (d) Chosen: sibling numbering
   binding, outline above, subject to the ADR.

## Decisions

**defer.** Blocked by `architecture-decision-record:transport-and-idl-compile-targets` (a `blocks`
edge from it). The story stays in draft until that record is accepted and
`docs/design/protobuf-projection.md` leaves draft.

## Acceptance (provisional, fixed after the ADR)

- `ess generate --kind proto` on `examples/billing` with an `ess-proto/1` binding writes a `.proto`
  that `protoc` accepts, with `google.api.http` annotations matching the HTTP projection.
- Without a numbering binding the projection refuses, or marks numbers unstable, by name.
- A construct `.proto` cannot express is refused by name and counted in coverage.
