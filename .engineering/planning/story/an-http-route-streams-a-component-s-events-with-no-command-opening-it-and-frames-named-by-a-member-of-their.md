---
format: aep.planning-md/3
id: story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their
kind: story
status: archived
title: An HTTP route streams a component's events, with no command opening it and frames named by a member of their data
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-event-channel-may-be-bound-to-server-sent-events
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-11-24T01:56:03Z", actor: "human:timo", revision: 2}
---
## Need

A service publishes every change of its records on one long-lived `GET`, a server-sent event
stream. Each frame's data is an object naming what changed in a member, such as
`{"kind": "shipment", "shipment": {...}}`. A UI channel should read that stream. Nothing in ESS can declare
it, so the UI polls a view every two seconds instead.

- `ess-http/1` binds a command or a view per route. A route has no `stream` key, and a view's body
  is `rows`, `single` or `array` (`crates/specify/ess-http/src/lib.rs:619-646`, `RouteSpec`).
- `story:an-event-channel-may-be-bound-to-server-sent-events` (active) designs an `ess-http` route
  key `stream:` under the accepted
  `architecture-decision-record:messages-are-exchanged-independent-of-the-wire` (§ 2, the SSE row).
  It puts the key on the route of an exchange's `opens` command. It names frames by the SSE
  `event:` line, from `frames: {<Event>: <frame name>}`. Its generated servers refuse such a route
  by name.
- The ADR's exchange needs `opens`, a command (§ 1). It counts "every event a one-message exchange
  with no initiator message" as an implicit exchange that today's bindings bind (§ 1, last
  bullet).
- `ess-ui/1` channels name what they carry and leave the transport to the renderer
  (`website/docs/reference/ess-ui.md:2248-2281`). The generated React app connects to
  `<base>/channels/<channel name>` and reads frames as `{event, payload}` or as a bare payload
  (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:115-131`, `:215-223`). No ESS document
  names that address, and no generated server serves it.

Two gaps remain beyond the active SSE story. A stream of a component's events that no command
opens, which is a broadcast of implicit event exchanges. And frames that carry their name in a
data member rather than in the `event:` line.

Minimal reproduction (fresh, not run here): a binding route

```yaml
- method: get
  path: /v1/changes
  stream:
    events: {shop.delivery.ShipmentChanged: shipment, shop.delivery.DepotChanged: depot}
    frame_name: {member: kind}
    payload: {member_named_by_frame: true}
```

`ess specify http validate` refuses it: "routes[0]: unknown field `stream`, expected one of
`command`, `view`, …" (the SSE story's own reproduction shows the same refusal).

## Outcome

`ess-http/1` admits a stream route that names no command: `stream: {events: {<Event>: <frame
name>}, …}` on a `get` route of a `reached_by: network` component. Each listed event, which the
component's commands emit, is one frame. Whether the frame name is the SSE `event:` line (the SSE
story's form) or a member of the data object is declared. Whether the payload is the data, or sits
under a member, is declared. The compiled `ess-http-ir/1` carries it, and OpenAPI writes the
route's `200` as `text/event-stream` with `x-ess-frames`. The generated servers refuse it by name,
as the SSE story's do. The exact keys are fixed in the design section this story writes
(SL-D12).

## Acceptance

- Red first, in `crates/specify/ess-http/tests/`: the reproduction route compiles, once its keys
  are fixed by the design section. It fails before the change with the unknown-field refusal.
- Refused by name (the next free `ESS-HTTP-0NN`, shared with the SSE story):
  - an event no command of the component emits;
  - one event listed twice;
  - a frame name holding CR, LF or `:`;
  - a data member name that is empty;
  - a `stream` route with a method other than `get`;
  - `stream` beside `command` with an exchange, which is the SSE story's form, unless that form is
    meant.
- A binding without `stream` compiles to the same bytes.
- `ess generate --kind openapi --http` writes the route's `200` as `text/event-stream`, with
  `x-ess-frames` mapping each frame name to the event's schema.
- `ess generate synthesize --http` for Rust and Go refuses such a binding by name, with the SSE
  story's sentence.
- `docs/design/http-presentation-binding.md` § "Server-sent events" (written by the SSE story)
  gains the commandless stream and the member-named frame.

## Fit review

Tree `bcfc51555`, read-only. Second report, friction entry F18 (not in its G-list).

1. **Need.** A UI follows a service's changes as they happen, over the service's own event stream.
   Reproduction above.
2. **Class.** *Gap*.
3. **Already expressible?** No (above). Polling a view with `refresh:` is the application's
   workaround.
4. **Fit.** It is placed under the accepted ADR. An HTTP fact goes in `ess-http`, and an event
   stream with no initiator message is the ADR's implicit event exchange, so no `exchanges:`
   declaration and no specification format is needed. The frame vocabulary is the SSE story's.
   The member-named frame reuses the body vocabulary's member form, as `{<member>: error}` does
   (`http-presentation-binding.md:377`).
5. **Second adopter.** A build service's `GET /events` streams `{"kind": "build", "build": {...}}`
   to every dashboard.
6. **Cost.** Additive within `ess-http/1`, under the format policy the ADR names for `upgrade:`
   (§ 2, row C), and the SSE story's own format decision. One new code. No specification format and
   no suite major.
7. **Considered.** (a) Wait for the exchange layer and an `opens` command: that invents a "watch"
   command, the same invention the second report's G13 objects to. (b) A channel address in the UI
   document: a transport fact in `ess-ui`, against the channel's "the renderer picks the transport".
   (c) Chosen: an `ess-http` stream route, with the UI reading it through the binding
   (`sl18-stream-channel`).

## Decisions

**Accept, redesigned.**

- **SL-D12. How does `ess-http/1` take a stream no command opens?** (A) Widen the active SSE story
  in this story: a commandless `stream:` of implicit event exchanges, and frames named by a data
  member. The UI half is `sl18-stream-channel`. (B) Wait for the exchange layer (ADR stage 1,
  `exchanges:` at the next `ess/N`) and bind the stream to an `opens` command. (C) Decline: the
  application keeps polling. **Recommended: A.**

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RouteSpec`, `Route` 2119-2186)
- inferred: `crates/generate/ess-gen/src/openapi.rs`, `crates/generate/ess-synth/src/rust/http/bound.rs`
  and `go/http/bound.rs` (the by-name refusal), `crates/specify/ess-http/tests/`,
  `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`
- Size: M, about 480 changed lines (unit 1 of 2).

## Depends on

`story:an-event-channel-may-be-bound-to-server-sent-events` (active), for the `stream:` key and its
frame vocabulary. It shares `crates/specify/ess-http/src/lib.rs` with `sl01-member-body`, so the two
are merged one after the other.

## Source

A downstream application, 2026-11-24, second report, friction entry F18.
