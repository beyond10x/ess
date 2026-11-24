---
format: aep.planning-md/3
id: story:an-event-channel-may-be-bound-to-server-sent-events
kind: story
status: active
title: An HTTP route binds the events a component publishes as server-sent events, its frames stated by a protocol document
summary: SSE is refused as a broker protocol (a2a wave 1, G1); blocked on the transport and IDL compile-targets ADR.
tags:
- adopter-report
- ess-0.70.0
- feature-request
- gap-p1
- gap-reports-2026-11-24
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#757
relations:
- serves: vision:O2
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
- depends_on: story:an-ess-http-1-binding-names-the-protocol-document-its-routes-read-by-its-compiled-digest
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/stream.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:39:12Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:39:13Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
**Revised on 2026-11-24 after the owner's note.** A server-sent-event stream is an event stream.
Frame and header detail belongs to the protocol document the binding names. The route binds the
events one component already publishes. It does not bind an exchange, so the story no longer waits
for the exchange layer. The stream that an exchange's `opens` command starts follows once that
layer exists (Not in this story).

## Need

A service answers one HTTP `GET` with `200` and `Content-Type: text/event-stream`, keeps the
response open, and writes one frame per event as it happens. A frame is named by its `event:` line
or by a member of its data, and comment lines keep the connection alive. Nothing in ESS binds such
a route.

It was reported by an adopter streaming task updates (ess 0.67.0), in beyond10x/ess#757 (ess 0.66.0
to 0.88.0), and twice by a downstream application on 2026-11-24: a change feed a UI should follow,
and a stream beside a bus contract.

Today's refusals (from the earlier body, ess 0.92.0; unchanged in `bcfc51555`):

```console
$ ess specify http validate --path spec --binding stream-route.http.yaml
error: reading stream-route.http.yaml as HTTP binding YAML: routes[0]: unknown field `stream`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`, `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals` at line 4 column 6
$ ess specify transport validate --path sse.transport.yaml --spec spec
error: reading sse.transport.yaml as transport YAML: brokers[0].protocol: unknown variant `sse`, expected `nats` at line 4 column 25
```

What exists, and is not restated here:

- **The semantics.** A component declares the events it publishes
  (`crates/specify/ess-domain/src/component.rs:137`, `publishes`), and its commands' outcomes emit
  them. The accepted ADR `messages-are-exchanged-independent-of-the-wire` counts every event as a
  one-message exchange with no initiator message, which today's bindings bind (§ 1, lines 87-89).
  `ess-transport` binds such an event to a broker subject. This story binds a set of them to an
  HTTP stream.
- **The frame structure.** Frame names, a name carried in `event:` or in a data member, data bare or
  under a member, and the keepalive are stated in an `ess-protospec/1` document
  (`pr0-protocol-message-structure`). The binding names that document
  (`pr0-http-binding-names-a-protocol`).

## Outcome

The protocol document (abbreviated: transitions that send each frame on input, and re-arm the
`keepalive` timer at 30 000 ms, are left out):

```yaml
format: ess-protospec/1
name: delivery-changes
participants:
  - {name: server, initial: live, states: [live], timers: [keepalive], inputs: […], transitions: […]}
  - {name: client, initial: live, states: [live], transitions: […]}
messages:
  - name: shipment                                       # data: {"kind":"shipment","shipment":<payload>}
    fields: [{name: kind, type: String, value: shipment}]
    application: {member: shipment}
  - name: depot                                          # event: depot / data: <payload>
    headers: [{name: event, type: String, value: depot}]
    application: body
  - name: keepalive                                      # no header, no body: a comment line
channels:
  - {name: frames, from: server, to: client, ordering: fifo, capacity: 64, loss: false, duplication: false}
bounds: {max_steps: 200, max_states: 10000, max_time_ms: 600000}
```

The binding:

```yaml
type: ess-http/1
specification: {system: shop, version: v1, source_digest: "sha256:<hex>"}
protocol: "sha256:<compiled digest of delivery-changes>"
routes:
  - method: get
    path: /v1/changes
    stream:
      component: shop.delivery.Dispatch
      channel: frames
      events:
        shop.delivery.ShipmentChanged: shipment
        shop.delivery.DepotChanged: depot
```

- **What the binding states.** A `stream` route names no command and no view. It states the
  publishing component, the protocol channel the frames travel on, and which protocol message
  carries each event. Its `refusals` and `authentication`
  (`story:an-http-binding-declares-how-a-caller-authenticates`) work as on any route.
- **What it means.** Every occurrence of a listed event that the component publishes after the
  caller connected is one frame, in publish order: the protocol message mapped to it, with the
  event's payload in the message's application slot. The answer is `200` with
  `Content-Type: text/event-stream`, held open. A refusal before it keeps its ordinary answer.
  Nothing is replayed. Which events a caller may see is decided by the model, as it is for a
  broker subject. The binding filters nothing.
- **The SSE writing rule** (the transport's, per the event-stream format of the HTML Living
  Standard):
  - a frame's headers can be only `event`, `id` and `retry`;
  - `data:` is the application value as one-line JSON, or with `{member}` the JSON object of the
    fixed members plus the value under the member;
  - a message with neither header nor body is a comment line;
  - an optional header that is absent writes no line.
- **The IR.** `ess-http-ir/1` carries `stream`. Each event is resolved to its frame: where the name
  is carried (the `event` header, or a member, with its fixed value) and where the payload sits.
  Consumers (OpenAPI, the UI's live channel) read the IR and not the protocol file. It is absent
  when undeclared.
- **No new semantics.** No command, outcome, event or channel construct is added, and no
  conformance format changes. The synthesized suite already holds the scenarios that make the
  component publish these events. Running them over the stream is a `--target process` adapter
  that speaks the route (PR-D7 A). The frame structure is checked against the protocol document
  through the native protocol adapter. An ESS SSE target is a story of its own.

## Acceptance

Unit a (binding, IR, design):

- **Red first.** With the protocol document above passed as `--protocol`, the binding validates
  (exit 0), and `ess specify http compile` writes `stream` with resolved frames into
  `ess-http-ir/1`. Today it is refused with "unknown field `stream`". The case is in
  `crates/specify/ess-http/tests/compile.rs`, plus a CLI case in `crates/edge/ess-cli/tests/http_presentation.rs`.
- **Refusals.** Each is refused with one new code (the next free `ESS-HTTP-0NN` when the unit
  lands), naming the key:
  - `stream` beside `command`, `view`, `parameters`, `request`, `body`, `outcomes`, `errors`,
    `alike` or `empty`, or on a method other than `get`;
  - a component the specification does not declare, one not `reached_by: network`, or one that does
    not publish a listed event;
  - an event listed twice;
  - a `channel` the protocol does not declare;
  - a mapped message that has no application slot, that no transition sends on that channel, or
    that is mapped twice;
  - a message with an application slot that is sent on that channel and left unmapped;
  - a frame header other than `event`, `id` or `retry`, or a `credential` header;
  - a fixed `event` value holding CR or LF;
  - `stream` in a binding without `protocol:`.
- **Old bindings.** A binding without `stream` compiles to the same IR bytes.
- **Docs.** `docs/design/http-presentation-binding.md` gains § "Server-sent events" (the route, the
  writing rule, what the IR resolves, and where conformance runs). "Still refused" no longer lists
  SSE among non-JSON bodies.

Unit b (projections):

- `ess generate --kind openapi --http … --protocol …` writes the route's `200` as
  `text/event-stream`, with `x-ess-protocol` (the digest) and `x-ess-frames` (frame name to event
  schema, name place and payload member). A binding without `stream` keeps its bytes. The case is in
  `crates/generate/ess-gen/tests/openapi_http_presentation.rs`.
- `ess generate synthesize --http` for Rust and Go refuses a binding with a `stream` route by name
  ("a server-sent-event route needs a stream runtime, which this target does not generate"), instead
  of serving it as an ordinary route. The case is in `crates/generate/ess-synth/tests/http.rs`.

Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RouteSpec` 1561-1610, route IR)
- cited: `crates/specify/ess-http/tests/compile.rs`, `crates/edge/ess-cli/tests/http_presentation.rs`
- cited: `crates/generate/ess-gen/src/openapi.rs`, `crates/generate/ess-gen/tests/openapi_http_presentation.rs`
- inferred: `crates/generate/ess-synth/src/rust/http.rs`, `crates/generate/ess-synth/src/go/http.rs`
  (the by-name refusal), `crates/generate/ess-synth/tests/http.rs`
- cited: `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`, `CHANGELOG.md`
- Size: unit a about 450 changed lines (M), unit b about 200 (S).

## Gate

- `task fmt-check`
- `cargo clippy -p ess-http -p ess-cli -p ess-gen -p ess-synth --all-targets --locked -- -D warnings`
- `cargo test -p ess-http --locked --test compile`
- `cargo test -p ess-cli --locked --test http_presentation`
- `cargo test -p ess-gen --locked --test openapi_http_presentation` (unit b)
- `cargo test -p ess-synth --locked --test http` (unit b)
- `cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6` (unit b, byte pin)
- `task ci-lint`
- `task test-xtask` (new code, docs)

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure` and `pr0-http-binding-names-a-protocol`.
- PR-D1, PR-D2, PR-D3 and PR-D4 (architecture).
- Not on `story:an-exchange-is-declared-in-the-specification` or
  `story:asyncapi-and-openapi-project-an-exchange` any more: those two relations are dropped.
- It shares `crates/specify/ess-http/src/lib.rs` with the authentication story, `sl01-member-body`
  and the header-input story's unit a, so these merge one after another.

## Not in this story

- The stream an exchange's `opens` command starts. When `story:an-exchange-is-declared-in-the-specification`
  lands, it binds with the same keys and `exchange:` in place of `component:`. That story is written
  then, and it needs the receiver-side close trigger that `ess-protospec/1` lacks (see `ASSESSMENT.md`).
- Resume from `Last-Event-ID`; a path parameter that narrows the events; a view followed as it
  changes (`story:a-view-answer-may-be-followed-as-it-changes`).

## Fit review

1. **Need.** One response carries the events a component publishes as they occur. The
   specification cannot state the route, the media type, or how each event is framed.
   Reproduction above. The requesters proposed `brokers[].protocol: sse`, a route
   `stream: <type>` with `frames`, `keepalive` and `replay` (#757), and a stream route with
   member-named frames (second report, F18).
2. **Class: gap.** `ess-http/1` refuses non-JSON bodies except one ("Still refused"). `ess-transport`
   closes its protocols at `nats` (`crates/specify/ess-transport/src/lib.rs:46-58`).
3. **Already expressible?** No.
4. **Fit.**
   - The note fixes the split. The events are the model's. The route, its component and its
     answers before the stream are `ess-http/1`'s. Frames, their names, the keepalive and the data
     placement are the protocol document's.
   - Binding published events is binding the ADR's implicit event exchanges (§ 1), so no `opens`
     command is invented: this was the second report's objection (G13) to a "watch" command.
   - It is not a broker protocol, because an SSE response is point to point.
   - Bindings are outside the language grids (`docs/design/language-grids.md`).
5. **Second adopter.** A build service streams `GET /builds/{build_id}/log` lines and a final
   `done`. Its row-keyed form waits for the exchange-opened stream. A dashboard's `GET /events` of
   every build change binds here.
6. **Cost.** One optional route key in `ess-http/1` and `ess-http-ir/1`, absent when undeclared, so
   no format version. One code. The protocol keys belong to the `pr0` units.
7. **Considered.**
   - (a) Change nothing: streams stay hand-written adapters.
   - (b) `protocol: sse` in `ess-transport`: a broker vocabulary for a point-to-point response.
   - (c) Frame names and keepalive in the route (the earlier body, ADR § 2 as written). Refused by
     the note.
   - (d) Wait for the exchange layer (PR-D4 B).
   - Chosen: a `stream` route over published events, with frames in the protocol document.

## Decisions

**Accept, redesigned again.**

- PR-D4 (architecture, default A) binds published events with no opening command. It supersedes
  AC-D1 and SL-D12.
- PR-D1, PR-D2 and PR-D3 (architecture). PR-D7 (default A).
- `replay` stays dropped. The exchange-opened form is a follow-up.
- AC-D3, AC-D4 and AC-D5 are untouched and still open.

## Source

github:beyond10x/ess#757; an adopter report relayed on 2026-10-22 (ess 0.67.0); a downstream
application, 2026-11-24 (second report, F18; third report, finding 2, stream half).
