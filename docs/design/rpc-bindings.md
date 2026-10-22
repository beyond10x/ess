# RPC bindings (`ess-jsonrpc/1`, `ess-grpc/1`)

Status: draft, pending ADR `architecture-decision-record:transport-and-idl-compile-targets`. Nothing
on this page is built or scheduled. It is the outline for
`story:a-command-binds-to-json-rpc-and-grpc-beside-http`, which stays in draft and is not
implemented until that record is accepted and this page leaves draft. The request came from an
adopter specifying A2A (beyond10x/a2a, item 8, 2026-10-22).

## Problem

A command or view binds to HTTP through `ess-http/1` (`docs/design/http-presentation-binding.md`),
and an event to a broker through `ess-transport/1` (`docs/design/event-transport-binding.md`). A
protocol that serves the same operations over JSON-RPC 2.0 and gRPC as well keeps its method names,
error codes and status tables in adapter code that nothing checks against the model. The adopter's
protocol, for instance, maps its errors to JSON-RPC codes -32001 to -32009.

## Decision (proposed)

### 1. Two sibling documents

`ess-jsonrpc/1` and `ess-grpc/1`, shaped like `ess-http/1` and `ess-transport/1`: authored
separately, pinned to the specification's source digest, compiled against the resolved IR into a
deterministic `ess-jsonrpc-ir/1` and `ess-grpc-ir/1`, and never part of `ess/N`. Without a binding
every artifact keeps its bytes.

### 2. JSON-RPC: methods, params, results, errors

```yaml
type: ess-jsonrpc/1
specification:
  system: catalog
  version: v1
  source_digest: sha256:<hex>
methods:
  - command: catalog.orders.PlaceOrder
    method: orders/place
    params: by-name
    errors:
      catalog.orders.OutOfStock: -32001
```

One method name per command or view, by-name params (by-position refused by name), the result per
outcome, and one error code per ESS error.

### 3. JSON-RPC envelope

The request `id` and notifications (a request with no `id`, admitted only for a command with no
answer the caller reads). Batch requests: admitted or refused by name (open question).

### 4. gRPC: services, methods, statuses, metadata

One service and method per command or view, a status code per outcome and error, and metadata read
from and written to the call.

### 5. gRPC numbering

gRPC messages and field numbers come from the protobuf projection
(`docs/design/protobuf-projection.md`). Whether `ess-proto/1` and `ess-grpc/1` are one document is
open.

### 6. Streaming

A unary call versus a server stream needs a model-level notion of a streamed answer. Decide whether
it is binding-only or a construct in `ess/N`, shared with the HTTP SSE framing draft
(`story:http-binding-carries-transport-framing`). If `ess/N`, it takes the next format under its own
decision.

### 7. One error, three codes

One ESS error maps to an HTTP status, a JSON-RPC code and a gRPC status. Decide whether that table
lives per binding or in one shared table the three read.

### 8. Generated code

Projections first, then Rust servers and clients, then Go, following `ess-http/1`'s staging
(`http-presentation-binding.md`, stage table).

### 9. Conformance

A target adapter that reaches an implementation through each binding, on the pattern of
`ess-http/1` stage 3 (`http-presentation-binding.md` § Stage 3).

### 10. Out of scope

Authentication schemes, TLS and service discovery. Each stage gets its own format version, and
unsupported keys are refused by name.

## What compiling checks

- Every named command, view and error exists in the pinned specification.
- Method names are unique per binding; error codes are unique per binding and, for JSON-RPC, outside
  the reserved range -32768 to -32000 except the implementation-defined -32099 to -32000.
- Every ESS error a bound operation can answer has a code, or the binding refuses by name.

## Open questions

1. JSON-RPC batch: admitted or refused?
2. `ess-proto/1` and `ess-grpc/1`: one document or two?
3. Streaming: binding-only, or `ess/N`?
4. The error table: per binding or shared?
