---
format: aep.planning-md/3
id: story:a-command-binds-to-json-rpc-and-grpc-beside-http
kind: story
status: draft
title: A command or view binds to a JSON-RPC 2.0 method beside HTTP
summary: JSON-RPC 2.0 and gRPC bindings beside ess-http/1 (adopter item 8, beyond10x/a2a); blocked on the transport and IDL compile-targets ADR, not to be implemented before it is accepted.
tags:
- adopter-report
- feature-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- depends_on: story:ess-projects-a-protobuf-idl-with-http-annotations
- informed_by: story:http-binding-carries-transport-framing
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/edge/ess-cli/src/jsonrpc.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/specify/ess-jsonrpc/Cargo.toml
- confidence: inferred
  path: crates/specify/ess-jsonrpc/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-jsonrpc/tests/compile.rs
- confidence: cited
  path: docs/design/rpc-bindings.md
revision: 8
---
## Need

A protocol that serves its operations as JSON-RPC 2.0 methods (one endpoint, the method named in
the envelope, an error code per failure) cannot bind them: `ess-http/1` binds a route per operation
and has no envelope. The method names and error-code tables then live in adapter code nothing checks
against the model. This body covers JSON-RPC only; gRPC is split to
`story:a-command-or-view-binds-to-a-grpc-method-beside-http` and the OpenRPC projection to
`story:ess-generates-an-openrpc-document-from-a-json-rpc-binding`. Suggested title: "A command or
view binds to a JSON-RPC 2.0 method, as ess-http/1 binds it to HTTP".

Minimal reproduction on 0.92.0 (the requester's envelope key on an `ess-http/1` route):

```text
$ ess specify http validate --path model --binding http-envelope.yaml
error: reading http-envelope.yaml as HTTP binding YAML: routes[0].request: unknown field `envelope`, expected one of `encoding`, `unknown`, `body` at line 10 column 15
exit 1
```

Three findings of one downstream report are parts of it: no numeric code on an error (J1), no
method naming (J2), and one endpoint for every method refused by `ess-http/1` as `ESS-HTTP-010`
(J3, `crates/specify/ess-http/src/lib.rs:3088`).

## Acceptance

- `docs/design/rpc-bindings.md` leaves draft for its JSON-RPC sections: status accepted, open
  question 1 answered (batch requests refused by name in `/1`), open question 3 answered by the
  exchange ADR (a streamed answer is an exchange), open question 4 answered (the error table is
  per binding).
- `ess specify jsonrpc validate --path model --binding rpc.yaml` (exit 0) and `compile` (writes
  `ess-jsonrpc-ir/1`, identical bytes on a second run) for a document pinning the source digest with
  `methods: [{command: example.orders.PlaceOrder, method: orders/place, params: by-name,
  errors: {example.orders.OutOfStock: -32001}}]` and one for a view.
- Refused by name, codes `ESS-JSONRPC-001` onward: an unknown command, view or error; a method
  name given twice; `params: by-position`; an error code given twice; a code in the reserved range
  -32768 to -32000 outside -32099 to -32000; an error the bound operation can answer left without
  a code; a batch key; a digest that does not match.
- Without a JSON-RPC binding every artifact keeps its bytes.
- Tests in `crates/specify/ess-jsonrpc/tests/` (new crate).

## Scope

- cited: docs/design/rpc-bindings.md
- inferred: crates/specify/ess-jsonrpc/src/lib.rs (new crate, shaped like crates/specify/ess-http/src/lib.rs)
- inferred: crates/specify/ess-jsonrpc/tests/compile.rs (new)
- inferred: crates/specify/ess-jsonrpc/Cargo.toml (new), Cargo.toml (workspace member)
- inferred: crates/edge/ess-cli/src/jsonrpc.rs (new, shaped like crates/edge/ess-cli/src/http.rs)
- cited: crates/edge/ess-cli/src/main.rs (`SpecifyCommand`, line 147)
- inferred: README.md (crate tree list)
confidence: medium

## Fit review

1. **Need.** Bind a command or view to a JSON-RPC 2.0 method with its params, result and an error
   code per ESS error, checked against the model. Requester's syntax:
   `request: {envelope: {jsonrpc: "2.0", params: input}}` inside `ess-http/1` (issue comment).
2. **Class: gap.** No binding states a method or an error code (reproduction above).
3. **Already expressible?** No: `ess-http/1` refuses one endpoint for several operations
   (`ESS-HTTP-010`), and a POST route per method misstates the wire.
4. **Fit.** Not a grid cell (`docs/design/language-grids.md`). Same shape as `ess-http/1` and
   `ess-transport/1`: a sibling document pinned to the digest with a closed vocabulary
   (ADR `transport-and-idl-compile-targets` § Decision 2, accepted 2026-10-23). Its methods are
   bindings of implicit one-message exchanges (ADR `messages-are-exchanged-independent-of-the-wire`
   § 2, § 5), so `story:json-rpc-binds-an-exchange` extends it later without changing it. Error
   codes per binding, as `ess-http/1` keeps statuses per binding. Generated servers and a JSON-RPC
   conformance target are later stories; the design page names them as not built.
5. **Second adopter.** A language-server-style tool exposing commands over JSON-RPC 2.0.
6. **Cost.** A new format pair `ess-jsonrpc/1`/`ess-jsonrpc-ir/1`, a crate and a CLI area. No
   `ess/N` change (ADR `transport-and-idl-compile-targets` § Decision 3), no migration.
7. **Alternatives.** (a) Change nothing. (b) The requester's `envelope` key inside `ess-http/1`: a
   different protocol under an HTTP format, and still one route per method; refused. (c) A generic
   binding with a transport field: forbidden (`AGENTS.md` § Boundaries). (d) Chosen: the sibling
   `ess-jsonrpc/1`.

## Decisions

**accept, redesigned.** The earlier body deferred on ADR `transport-and-idl-compile-targets`, which
the operator accepted on 2026-10-23, so the defer is lifted. The requester's `envelope` key is
refused; the envelope is the sibling binding `ess-jsonrpc/1`. Narrowed to JSON-RPC (one crate);
gRPC and OpenRPC are split out. The `depends_on` on the protobuf projection story moves to the gRPC
story.

## Source

github:beyond10x/ess#650
