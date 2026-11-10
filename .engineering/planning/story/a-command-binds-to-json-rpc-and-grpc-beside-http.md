---
format: aep.planning-md/3
id: story:a-command-binds-to-json-rpc-and-grpc-beside-http
kind: story
status: draft
title: A command or view binds to a JSON-RPC 2.0 method beside HTTP
summary: A JSON-RPC 2.0 binding beside ess-http/1 (#650); gRPC and OpenRPC are split out and not part of this story.
tags:
- adopter-report
- feature-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#650
relations:
- serves: vision:O2
- informed_by: story:http-binding-carries-transport-framing
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: Cargo.toml
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
- confidence: inferred
  path: crates/specify/ess-jsonrpc/tests/refusals.rs
- confidence: cited
  path: docs/design/rpc-bindings.md
revision: 11
---
## Need

A protocol that serves its operations as JSON-RPC 2.0 methods (one endpoint, the method named in
the envelope, an error code per failure) cannot bind them: `ess-http/1` binds a route per operation
and has no envelope. The method names and error-code tables then live in adapter code nothing checks
against the model. This body covers JSON-RPC only: #650 asks for no gRPC binding, and neither gRPC
nor an OpenRPC projection has a story in this epic. Suggested title: "A command or view binds to a
JSON-RPC 2.0 method, as ess-http/1 binds it to HTTP".

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

The binding document `rpc.yaml` below pins the source digest of the model and holds
`methods: [{command: example.orders.PlaceOrder, method: orders/place, params: by-name,
errors: {example.orders.OutOfStock: -32001}}, {view: example.orders.OrderStatus, method:
orders/status, params: by-name}]`.

- `docs/design/rpc-bindings.md` line 3 reads `Status: accepted` for the JSON-RPC sections, and
  `## Open questions` items 1, 3 and 4 each carry their answer: 1, batch requests are refused by
  name in `/1`; 3, a streamed answer is an exchange (ADR
  `messages-are-exchanged-independent-of-the-wire`); 4, the error table is per binding.
- `ess specify jsonrpc validate --path model --binding rpc.yaml` exits 0 with no diagnostic.
- `ess specify jsonrpc compile --path model --binding rpc.yaml` exits 0 and writes an
  `ess-jsonrpc-ir/1` document; a second run writes identical bytes. Both checked by
  `cargo test -p ess-jsonrpc --locked --test compile`.
- `cargo test -p ess-jsonrpc --locked --test refusals` passes, one case per refusal, each asserting
  its code on a one-line change of `rpc.yaml`: `ESS-JSONRPC-001` an unknown command, view or error;
  `ESS-JSONRPC-002` a method name given twice; `ESS-JSONRPC-003` `params: by-position`;
  `ESS-JSONRPC-004` an error code given twice; `ESS-JSONRPC-005` a code in -32768 to -32000 outside
  -32099 to -32000; `ESS-JSONRPC-006` an error the bound operation can answer left without a code;
  `ESS-JSONRPC-007` a `batch` key; `ESS-JSONRPC-008` a digest that does not match.
- The byte-pin targets AGENTS.md § "A gate of named test targets includes the repository-wide byte
  pins" names pass with no row re-pinned: without a JSON-RPC binding every artifact keeps its bytes.

## Scope

- cited: docs/design/rpc-bindings.md
- inferred: crates/specify/ess-jsonrpc/src/lib.rs (new crate, shaped like crates/specify/ess-http/src/lib.rs)
- inferred: crates/specify/ess-jsonrpc/tests/compile.rs (new)
- inferred: crates/specify/ess-jsonrpc/tests/refusals.rs (new)
- inferred: crates/specify/ess-jsonrpc/Cargo.toml (new)
- inferred: Cargo.toml (workspace member)
- inferred: crates/edge/ess-cli/src/jsonrpc.rs (new, shaped like crates/edge/ess-cli/src/http.rs)
- cited: crates/edge/ess-cli/src/main.rs (`SpecifyAreaCommand`, line 114, beside `Http`)
- inferred: README.md (crate tree list)
confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no `ess/N` row and no byte-pin row.

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
refused; the envelope is the sibling binding `ess-jsonrpc/1`. Narrowed to JSON-RPC (one crate). The
`depends_on` edge on the protobuf projection story is dropped: #650 asks for no gRPC binding and
this epic has no gRPC story. The refusal codes are numbered in the order the design page's "What
compiling checks" lists them, so each test case names one fixed code.

## Source

github:beyond10x/ess#650
