# Protobuf IDL projection (`ess generate --kind proto`, `ess-proto/1`)

Status: draft, pending ADR `architecture-decision-record:transport-and-idl-compile-targets`. Nothing
on this page is built or scheduled. It is the outline for
`story:ess-projects-a-protobuf-idl-with-http-annotations`, which stays in draft and is not
implemented until that record is accepted and this page leaves draft. The request came from an
adopter specifying A2A (beyond10x/a2a, item 7, 2026-10-22).

## Problem

ESS projects docs, a site, `ess-docs/1`, JSON Schema, OpenAPI and AsyncAPI
(`crates/edge/ess-cli/src/main.rs:550-562`). It projects no interface definition language. A
protocol whose normative message source is a protobuf IDL therefore keeps that IDL by hand beside
the specification, and nothing checks the two against each other.

Protobuf was deferred on 2026-09-15 ("Track generic protobuf, UI and Flutter as separate deferred
follow-up", `approval-record:ess-evolution-20260915`; `task:deferred-protocol-ui-bindings`;
`docs/design/ess-evolution/feature-preservation.md:41`). The ADR reverses that deferral for the
projection direction only. The operator's words, relayed: "technically ESS needs compile targets
for all of these specs".

## Decision (proposed)

### 1. Direction

ESS to `.proto` only. Importing an existing `.proto` into ESS stays deferred under
`task:deferred-protocol-ui-bindings`, whose preserved requirements (names, field numbers, reserved
fields, presence, oneofs, enum and JSON behaviour, streaming, HTTP annotations) this projection
must not contradict.

### 2. Placement

`ess generate --kind proto`, in `crates/generate/ess-gen/` beside `openapi.rs` and `asyncapi.rs`.
`ess/N` does not change.

### 3. Field numbers are wire identity

A protobuf field number, unlike a JSON member name, is the identity a deployed client holds. It is
not derivable from the model: declaration order changes under an innocent edit. Numbers are
authored in a sibling binding, `ess-proto/1`, pinned to the specification's source digest as
`ess-http/1` is (`docs/design/http-presentation-binding.md`, § Envelope):

```yaml
type: ess-proto/1
specification:
  system: catalog
  version: v1
  source_digest: sha256:<hex>
package: catalog.v1
messages:
  catalog.orders.Order:
    id: 1
    lines: 2
    note: 3
```

Without a binding the projection refuses, or with an explicit flag writes numbers marked unstable
in a header comment. Which of the two is the default is an open question.

### 4. Type map

| ESS | protobuf |
|---|---|
| `Optional<T>` | `optional T` |
| `List<T>` | `repeated T` |
| `Map<K, V>` | `map<K, V>`, refused for a key type protobuf does not admit |
| `Json` | `google.protobuf.Value` or `google.protobuf.Struct` (open question) |
| `Timestamp` | `google.protobuf.Timestamp` |
| `Bytes` | `bytes` |
| integer width | `int32` / `int64` / `uint*` by declared range; refused where none fits |

### 5. Unions

A union maps to a `oneof`. ProtoJSON encodes a `oneof` as one member named after the chosen
variant, which ESS cannot state yet; the JSON projections and this one agree only after
`story:a-union-may-be-encoded-as-an-object-keyed-by-its-variant`.

### 6. Enums

protobuf requires a zero value, by convention `<ENUM>_UNSPECIFIED`, and style prefixes values with
the enum's name. Both must be reconciled with ESS enum wire names (`docs/design/unique-wire-field-identity.md`):
the projection either adds the zero value and prefixes, with ProtoJSON names mapped back, or the
binding names them. Open question.

### 7. Services

Commands, and views, become rpcs of one service per domain. A streamed answer has no ESS construct
yet: the projection refuses a streaming rpc by name, or takes the marker decided in
`docs/design/rpc-bindings.md` § Streaming.

### 8. HTTP annotations

`google.api.http` options come from `ess-http/1` when a binding is given, otherwise from ESS's
default route (`crates/generate/ess-gen/src/http.rs:448-473`). This decides body mapping (`body: "*"`
or a named field), which must agree with the OpenAPI projection of the same binding.

### 9. Evolution

`ess verify diff` emits `reserved` for removed numbers and names, and compatibility is checked on
the compiled descriptor, not on the `.proto` text.

### 10. Refusals and coverage

Every construct `.proto` cannot express is refused by name, and coverage is reported as the other
projections report it.

## Still refused

- Importing `.proto` (deferred).
- Proto2 syntax, extensions, and custom options other than `google.api.http`.

## Open questions

1. Without an `ess-proto/1` binding: refuse, or emit numbers marked unstable?
2. `Json`: `Value` or `Struct`?
3. Enum zero value and prefixes: added by the projection or named in the binding?
4. Are `ess-proto/1` and `ess-grpc/1` one document (`docs/design/rpc-bindings.md` § gRPC)?
