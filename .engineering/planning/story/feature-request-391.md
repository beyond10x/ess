---
format: aep.planning-md/3
id: story:feature-request-391
kind: story
status: active
title: Bind parameterized channel addresses to event payload fields
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#391
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/transport.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/client.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/transport.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/asyncapi_transport.rs
- confidence: cited
  path: crates/generate/ess-publisher
- confidence: cited
  path: crates/specify/ess-transport
- confidence: cited
  path: docs/design/event-publishers.md
- confidence: cited
  path: docs/design/event-transport-binding.md
- confidence: cited
  path: docs/design/parameterized-event-channel-addresses.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T16:38:51Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-03T16:38:51Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Bind a NATS channel's whole-token address expressions to required String paths in the event payload,
prove every legal concrete address is captured by exactly one declared stream, project the relation
to AsyncAPI, and generate Rust and Go publishers that render, validate and batch by concrete subject.

## Fit review

1. **Need, apart from proposed syntax.** A producer event carries required routing values, but ESS
   cannot state that two concrete subject tokens equal `event.source.service` and
   `event.source.environment`, or require each value to be one NATS subject token. The requester's
   proposal is an address template with AsyncAPI-like channel parameters bound to payload or context
   fields (beyond10x/ess#391). The brand-free retained reproduction declares only a required Source
   struct and UsageRecorded event. Current 0.52.0 accepts
   `subject: usage.{service}.{environment}` as one literal `/1` subject, compiles the braces
   unchanged, emits no AsyncAPI parameters, and generates them as Rust and Go constants
   (`transport-design/probe-evidence.md`; `probe/v1-validate.log`; generated
   `probe/client-rust/lib.rs:246` and `probe/client-go/publisher.go:160`). Thus the need is the checked
   equality and rendering, not merely braces in an address.

2. **Class.** Gap. `ChannelSpec.subject` and compiled `Channel.subject` are plain strings
   (`crates/specify/ess-transport/src/lib.rs:127-141,201-218`); compilation validates one concrete
   publish subject and matches streams against that string
   (`crates/specify/ess-transport/src/lib.rs:519-525,575-616`). Nothing represents a payload source
   or dynamic address. This does not contradict `/1`'s documented literal semantics, so it is not a
   defect in `/1`.

3. **Can it already be expressed?** No. A separate fixed transport document per producer can choose
   one literal subject, but cannot assert that the event payload agrees with it. `naming.wire` is
   also literal and `/1` requires equality with the transport subject
   (`crates/specify/ess-transport/src/lib.rs:500-517`; `docs/design/event-transport-binding.md:58-76`).
   The minimal CLI probe validates the apparent template only because braces are ordinary `/1`
   characters, and the generated clients publish them literally (`probe-evidence.md`). The
   proposed `/2` example is refused by the released reader because `parameters` is unknown
   (`probe/v2-old-reader.log`), establishing both the missing surface and old-reader fence.

4. **Fit with existing design.** The chosen authored map reuses a binding's established
   target-to-source shape and `event.` prefix. Binding mappings already distinguish flat event fields
   from bounded two/three-member accessors (`crates/specify/ess-domain/src/binding.rs:120-139,
   742-820`). The channel remains in the digest-pinned sibling transport document, preserving the
   event/transport boundary (`docs/design/event-transport-binding.md:5-31`). AsyncAPI 3 already
   defines `{name}` channel address expressions, requires the matching parameters map and defines
   `Parameter.location` as a message runtime expression
   (<https://www.asyncapi.com/docs/reference/specification/v3.0.0#parameterObject>). Literal
   `naming.wire` is not reinterpreted; a parameterized channel with one refuses. Only required
   primitive String event paths are admitted. `context.*` is excluded because its existing meaning
   is inbound delivery context (`crates/specify/ess-domain/src/binding.rs:290-291,746-751`), not
   producer configuration. Rust and Go are the only existing generated publisher targets and both
   implement rendering/batching; the CLI already enumerates only those targets. AsyncAPI handles the
   documentation target. Web, TypeScript, the interpreter, synthesis, entity runtime and
   `ess verify diff` accept no transport input, so their interfaces do not silently ignore the new
   construct; other `--kind` values continue to refuse `--transport`. Repeated expressions and
   differently named parameters that resolve to the same semantic event path form one equality
   class, which matches the fact that they read one payload value rather than independent wildcards.
   Generated client accounting selects `ess-client-report/2` for a dynamic operation.

5. **Second adopter.** An audit producer emits `AuditRecorded` with required
   `routing.tenant` and `routing.region` Strings and publishes it to
   `audit.{tenant}.{region}`, captured by `audit.>`. It needs the same checked payload equality,
   per-token validation, stream-language proof and subject-separated batching. This is independent
   of usage metering and contains no adopter policy.

6. **Cost.** `/1` is released in 0.52.0
   (`crates/edge/ess-xtask/src/docs.rs:263-265`), so the change adds authored
   `ess-transport/2`, compiled `ess-transport-ir/2`, four diagnostics
   `ESS-TRANSPORT-017`–`020`, and `ess-client-report/2` when a generated package has a dynamic
   operation. It adds exact NATS pattern-language coverage/intersection checks with correlated
   variable unification by resolved source path, AsyncAPI parameter and source metadata, payload
   access rendering, runtime token validation and per-concrete-subject batching with mandatory empty
   bucket/deadline retirement in Rust and Go. Opting into a dynamic client changes its generated
   subject constant to a template constant and makes `PublishError.Subject` owned; `/1` source, IR,
   projections, clients and report bytes stay unchanged. No `ess/N`, compiler IR, conformance suite,
   report, entity runtime or diff format changes.

7. **Alternatives.** (a) Change nothing and keep one fixed transport document/client per concrete
   producer: this cannot check payload/address equality and duplicates deployment policy. (b) Put a
   template in `naming.wire`: this changes a model-level literal name into broker behavior and cannot
   bind payload values. (c) Add typed producer context now: ESS declares no such outgoing authority,
   and the actual request is satisfied by payload paths. (d) Make `subject` a new nested tagged
   object: this duplicates AsyncAPI's established scalar-address-plus-parameters shape and makes
   literal channels noisier. (e) Reinterpret braces inside `/1`: rejected because released clients
   already publish them literally, as the retained probe demonstrates. The chosen `/2` scalar
   `subject` plus `parameters` map is the smallest shape that states the missing fact, reuses
   `event.` paths and projects directly to AsyncAPI.

## Decisions

**Accept, redesigned.** Accept the payload-bound address need and the requester's AsyncAPI parameter
direction, with these constraints:

- authored `ess-transport/2` uses `subject: '...{name}...'` plus
  `parameters: {name: event.<required String path>}`;
- expressions occupy whole NATS tokens, and paths have one to three members through required
  structs to a required primitive String;
- producer/context sources, defaults, transforms, Optional and named/string-like types remain out of
  scope;
- a parameterized subject and literal event `naming.wire` cannot coexist;
- compilation proves the whole substitution language has exactly one capturing stream and no
  overlapping stream, unifying exact-token constraints by resolved semantic source path so repeated
  and source-aliased expressions remain correlated;
- IR2 carries a tagged normalized `event_path`; AsyncAPI carries standard `location` plus closed
  `x-ess-source` equality/token metadata;
- generated array publishers maintain one nonempty buffer per concrete subject under one operation
  worker, retire empty buckets and deadline state on every drain path, and preserve FIFO if a fresh
  bucket appears while an older owned batch is in flight; `_now` refuses mixed subjects;
- released `/1` remains literal and byte-stable. Dynamic generated accounting uses
  `ess-client-report/2`.

The full revised normative body is
`docs/design/parameterized-event-channel-addresses.md`.

## Acceptance

- **Literal compatibility.** Retained `/1` transport, IR1, AsyncAPI, Rust/Go clients and
  client-report bytes are unchanged. The released 0.52 reader refuses `/2`; the new reader accepts
  valid `/1` and `/2`, and refuses `parameters` under `/1` before compilation.
- **Closed template admission.** Whole-token expressions accept; unmatched/embedded braces and bad
  names refuse with `ESS-TRANSPORT-017`. Missing, extra, empty and unused mappings refuse with
  `018`.
- **Typed event paths.** Flat and two/three-member required-struct paths resolve. Bad prefix,
  `context.*`, excess depth and unknown members refuse with `019`; Optional, collection, union,
  enum, newtype and non-String terminal/intermediate shapes refuse with `020`.
- **Wire-name coexistence.** Literal subjects retain exact `naming.wire` equality. A parameterized
  channel with `naming.wire` refuses with `ESS-TRANSPORT-011`.
- **All-expansion stream proof.** Exact prefix, `*` and terminal `>` cover cases pass; short/long/no
  cover fail `015`; two covers and full-plus-partial overlap fail `016`. Exact-token constraints are
  unified by resolved source path. `usage.{x}.{x}` and two differently named expressions mapped to
  one path are disjoint from `usage.a.b` for `a != b`, intersect `usage.a.a` and `usage.*.a`, and are
  fully covered only where every occurrence is wildcard-covered. A two-independent-path control
  intersects `usage.a.b`. No sampled substitution is used as proof.
- **IR2 determinism.** Parameters normalize into ordered tagged `event_path` entries. Recompilation
  is byte-identical and source path mutations change the exact canonical bytes.
- **AsyncAPI.** A validator accepts the generated 3.0 document. Every expression has a parameter;
  `location` uses the wire JSON Pointer, `x-ess-source.path` uses semantic names, and array channels
  state `scope: every_item`. Differently named parameters mapped to one resolved source emit the
  same normalized semantic path and wire location, exposing their exact-token equality. Field wire
  renames and RFC 6901 escaping are covered. Literal `/1` output is unchanged.
- **Rust publisher.** Actual generated code renders semantic paths using code field names, substitutes
  repeated and source-aliased expressions consistently, rejects invalid tokens before
  encoding/enqueue/I/O, separates two subjects' buffers and deadlines, caps each message at
  `max_items`, drains deterministically, preserves per-subject FIFO, refuses mixed `_now`, and
  retains closed/encoding/transport/on-error behavior. Empty buckets and deadline state retire on
  max-items, timer, flush, close and failed-send drains. A long unique-subject-then-flush sequence
  repeatedly returns internal bucket/timer counts to zero; a same-subject enqueue during an in-flight
  drain is ordered after the old batch and also retires.
- **Go publisher.** The same cases execute through the generated Go library and its existing context
  seam, including races around timer/max-items, flush and close, with the same zero-retained-bucket
  invariant after the long unique-subject sequence.
- **Actual broker.** A Rust harness starts/uses NATS JetStream with an externally created
  `usage.>` stream and runs both generated adapters. For each, 150 items on each of two concrete
  subjects yield subject-homogeneous arrays of at most 100; invalid tokens and mixed `_now` publish
  nothing; generated code never changes the stream.
- **Accounting and gates.** Dynamic packages write strict `ess-client-report/2`; old report readers
  refuse it, and static `/1` packages keep report/1. Focused tests, strict lint, generated-output
  drift, format/reference checks, `task check` and independent review pass on the exact held bundle
  candidate.

## Scope

Expected production and focused-test surfaces:

- `crates/specify/ess-transport/src/lib.rs`
- `crates/specify/ess-transport/tests/compile.rs`
- `crates/edge/ess-cli/src/transport.rs`
- `crates/edge/ess-cli/src/main.rs`
- `crates/edge/ess-cli/tests/transport.rs`
- `crates/edge/ess-cli/tests/client.rs`
- `crates/generate/ess-gen/src/asyncapi.rs`
- `crates/generate/ess-gen/tests/asyncapi_transport.rs`
- `crates/generate/ess-publisher/src/lib.rs`
- `crates/generate/ess-publisher/src/rust_support.rs.txt`
- `crates/generate/ess-publisher/src/go_support.go.txt`
- `crates/generate/ess-publisher/tests/generated.rs`
- a Rust actual-NATS acceptance harness and its CI service wiring
- `docs/design/event-transport-binding.md`
- `docs/design/event-publishers.md`
- `website/docs/reference/formats.md`
- `website/docs/reference/spec-versions.md`
- generated CLI/reference pages, release/change records and
  `crates/edge/ess-xtask/src/docs.rs` format registry

Compiler/domain source remains a read-only dependency unless implementation proves a narrowly
checked path/name helper is unavailable. No source ESS, conformance, diff, entity runtime,
TypeScript/Web publisher, consumer transport, producer-context, retry or aggregate-completion scope
is implied. Any new committed executable or test harness is Rust.
