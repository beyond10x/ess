---
format: aep.planning-md/3
id: story:an-event-channel-may-be-bound-to-server-sent-events
kind: story
status: draft
title: An HTTP route streams an exchange as server-sent events
summary: SSE is refused as a broker protocol (a2a wave 1, G1); blocked on the transport and IDL compile-targets ADR.
tags:
- adopter-report
- ess-0.70.0
- feature-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#757
relations:
- serves: vision:O2
- depends_on: story:a-response-stream-follows-one-row-until-a-terminal-state
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
- informed_by: architecture-decision-record:messages-are-exchanged-independent-of-the-wire
- decomposes: epic:messages-are-exchanged-independent-of-the-wire
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-exchange-is-declared-in-the-specification
- depends_on: story:asyncapi-and-openapi-project-an-exchange
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
revision: 7
---
**Blocked on the exchange layer: `story:an-exchange-is-declared-in-the-specification` and
`story:asyncapi-and-openapi-project-an-exchange` land first.** The relation to
`story:a-response-stream-follows-one-row-until-a-terminal-state` is replaced by those two: the
accepted `architecture-decision-record:messages-are-exchanged-independent-of-the-wire` § 4 answers
that a streamed answer is an exchange.

## Need

A service answers one HTTP `GET` with `200`, `Content-Type: text/event-stream`, keeps the response
open and writes typed frames: named frames (`event: <name>`), unnamed `data:` frames, comment lines
as keepalive. Nothing in ESS binds such a response. Reported twice: an adopter streaming task
updates over SSE (ess 0.67.0, `brokers[0].protocol: unknown variant sse, expected nats`) and
beyond10x/ess#757 (ess 0.66.0 to 0.88.0). Minimal reproduction (view `demo.queues.QueueStats` over
entity `demo.queues.Queue`, served by a `reached_by: network` component; ess 0.92.0):

```console
$ ess specify http validate --path spec --binding stream.http.yaml          # body: stream
error[ESS-HTTP-024]: routes[0].body: view `demo.queues.QueueStats` answers `rows`, `array` or `single`, and `stream` is not a view's body
$ ess specify http validate --path spec --binding stream-route.http.yaml    # stream: … media: text/event-stream
error: reading stream-route.http.yaml as HTTP binding YAML: routes[0]: unknown field `stream`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`, `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals` at line 4 column 6
$ ess specify transport validate --path sse.transport.yaml --spec spec       # brokers: [{protocol: sse}]
error: reading sse.transport.yaml as transport YAML: brokers[0].protocol: unknown variant `sse`, expected `nats` at line 4 column 25
$ ess specify validate --path spec-exchange                                   # exchanges: […]
  - domains/queues.yaml: unknown field `exchanges`, expected one of `format`, …
```

Design, under the accepted ADR (SSE binds an exchange "with no `initiator_sends` beyond `opens`,
over `text/event-stream`", § 2): an `ess-http` route key `stream:` on the route of the exchange's
`opens` command, the SSE counterpart of the `upgrade:` key
(`story:an-http-route-upgrades-to-an-exchange`):

```yaml
routes:
  - command: example.jobs.WatchJob
    method: get
    path: /jobs/{job_id}/events
    parameters: {job_id: input.job_id}
    stream:
      exchange: example.jobs.JobProgress
      frames:
        example.jobs.JobAdvanced: advanced        # event: advanced
        example.jobs.JobFinished: finished
      keepalive: 30s
```

- The opening outcome answers `200` with `text/event-stream` and holds the response open; every
  refusal before it keeps its ordinary answer (`outcomes`, `errors`, `refusals`).
- Each `responder_sends` event of the exchange is one frame: `event: <frame name>` (or no `event:`
  line for `default`), `data:` its payload as JSON on one line. Every event the exchange lists is
  named exactly once; `keepalive` writes a comment line at that interval.
- The response ends when the exchange closes. A message sent before the caller connected is not
  replayed.

## Acceptance

- With an exchange declared, `ess specify http validate` accepts the route above (exit 0) and
  `ess specify http compile` writes `stream` into `ess-http-ir/1`; a binding without `stream`
  compiles to the same bytes.
- Refused with one new code (the next free `ESS-HTTP-0NN`), naming the key: `stream` on a view or on
  a route whose command opens no exchange; an exchange that is not the one the command opens; an
  exchange with `initiator_sends` (that is a socket, `upgrade:`); a listed event the exchange does
  not send, one it sends left out, or a frame name that is not an SSE event name (no CR, LF or
  `:`), or two events on one name; a `keepalive` outside `1s..=3600s`; `stream` beside `upgrade` or
  beside a `body`/`status` for the opening outcome.
- `ess generate --kind openapi --http` writes the opening outcome's `200` as `text/event-stream`
  with `x-ess-exchange` naming the exchange and `x-ess-frames` mapping frame names to event
  schemas.
- `ess generate synthesize --http` for Rust and Go refuses a binding with a `stream` route by name
  ("a server-sent-event route needs the exchange runtime, which this target does not generate"),
  rather than serving it as an ordinary route.
- `docs/design/http-presentation-binding.md` gains § "Server-sent events" and "Still refused" no
  longer lists SSE among non-JSON bodies.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/stream.rs
- cited: crates/generate/ess-gen/src/openapi.rs
- inferred: crates/generate/ess-synth/src/rust/http.rs
- inferred: crates/generate/ess-synth/src/go/http.rs
- cited: docs/design/http-presentation-binding.md
confidence: medium

## Fit review

1. **Need.** One HTTP response carries a sequence of typed messages the server sends as they occur,
   until the conversation ends; the specification cannot state the route, the media type, the
   frame names or which type each frame carries. Reproduction above. Requesters' proposals
   (theirs): `brokers[].protocol: sse` in `ess-transport`; in beyond10x/ess#757, a route
   `stream: <type>` with `media: text/event-stream`, `frames: {<event name>: <type>}`,
   `keepalive: 30s` and `replay: last | none | from_id`.
2. **Class: gap.** `ess-http/1` refuses non-JSON bodies but one (`docs/design/http-presentation-binding.md`
   § "Still refused"); `ess-transport` closes its protocols at `nats`
   (`crates/specify/ess-transport/src/lib.rs:46-58`).
3. **Expressible today? No.** No route form, no protocol, no `exchanges:` in any released format
   (reproduction).
4. **Fit.** The language grids do not apply. The accepted ADR decides the shape: a stream is an
   exchange, SSE one of its bindings (`messages-are-exchanged-independent-of-the-wire.md:105`), and
   the HTTP route carries the HTTP facts, as F3 (a) gives the handshake to `ess-http`
   (`…:166`). So: no `protocol: sse` broker (an SSE response is not a broker), no `stream:` of a
   free type (frames carry the exchange's events, so the same conversation over a WebSocket and
   over SSE cannot disagree). `frames` keeps the requester's word; `keepalive` keeps theirs.
   `replay` is not a key: the ADR defines no resume, so nothing is replayed, and `from_id`
   (`Last-Event-ID`) waits for an exchange that states one. Targets: OpenAPI here; AsyncAPI
   projects the exchange in `story:asyncapi-and-openapi-project-an-exchange`; the generated servers
   refuse by name, since the exchange epic plans no server runtime; conformance synthesizes the
   exchange's scenarios without a wire (`story:conformance-synthesizes-exchange-scenarios`).
5. **Second adopter.** A build service streams `GET /builds/{build_id}/log` as SSE: `event: line`
   frames while the build runs and one `event: done` when it reaches `Finished`, opened by a
   `WatchBuild` command on the build row.
6. **Cost.** One optional route key in `ess-http/1` / `ess-http-ir/1`, additive as `operation_id`
   was; one new code; the `exchanges:` source format is the exchange story's, not this one's.
7. **Alternatives.** (a) Change nothing: streams stay hand-written adapters nothing checks.
   (b) `protocol: sse` in `ess-transport`: a broker vocabulary for a point-to-point response, the
   option the ADR rejects for sockets. (c) A sibling `ess-sse/1`: a whole format for four keys whose
   every other fact (method, path, parameters, refusals) is already an `ess-http` route's. Chosen:
   the `stream:` route key.

## Decisions

**Accept, redesigned.** The need is real; the ADR fixes that a stream is an exchange and SSE one of
its bindings. Changed from the requests: no broker protocol, no route over a free type; the route
names the exchange and maps its events to frame names; `replay` is dropped. Not decided here, and
carried by `story:a-view-answer-may-be-followed-as-it-changes`: a stream opened by a read that
re-sends a view's answer after each change, which is the case beyond10x/ess#757 reports.

## Source

github:beyond10x/ess#757; also an adopter report relayed on 2026-10-22 (ess 0.67.0)
