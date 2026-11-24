---
format: aep.planning-md/3
id: story:a-generated-ui-s-live-channel-connects-to-the-stream-route-its-binding-declares-and-reads-its-frames
kind: story
status: draft
title: A generated UI's live channel connects to the stream route its binding declares and reads its frames
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-http-route-streams-a-component-s-events-with-no-command-opening-it-and-frames-named-by-a-member-of-their
revision: 1
---
## Need

The generated React app connects a channel to `<base>/channels/<channel name>` and reads each
frame as `{event, payload}` or as a bare payload
(`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:115-131`, `:215-223`). After
`sl18-stream-route` a binding can declare the real stream route and how its frames are named. The
UI binding carries routes for views and commands only (`crates/ui/ess-ui/src/binding.rs:25-40`), so
the app keeps the conventional address.

## Outcome

When the binding has a stream route carrying every event a `server_to_client` channel carries, the
binding names that route for the channel. The generated app connects there and reads each frame by
the declared frame name and payload member, mapping it to the event the channel's `live.on` names.
A channel no stream route carries keeps today's address. The terminal stays on fixture scripts. It
has no network channel today (`crates/ui/ess-ui-tui/src/live.rs:1-6`), and this story adds none.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/live_binding.rs`: with a binding holding the
  `sl18-stream-route` reproduction, a channel `carries: {events: [shop.delivery.ShipmentChanged]}`
  connects to `/v1/changes`. The frame `{"kind": "shipment", "shipment": {"id": 7}}` patches row 7 of a
  section with `live: {channel: shipments, effect: patch_row}`. It fails before the change, connecting
  to `/channels/shipments`.
- A frame whose name the channel does not carry is dropped, and the channel stays live.
- A channel whose events no single stream route carries is reported by `ess ui check --model
  --http <binding>` as a warning at the channel, naming the events. The app keeps the conventional
  address for it.
- A binding without a stream route keeps its JSON bytes.

## Fit review

Second report, F18, unit 2 of 2. The fit review is in `sl18-stream-route`. `ess-ui/1` gains no key:
the channel still says what it carries, and the binding supplies the transport, as the channel's
reference promises ("the renderer picks the transport").

## Decisions

None beyond SL-D12.

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`Binding` 25-40)
- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (115-131, 215-223)
- inferred: `crates/ui/ess-ui-check/src/model.rs` (`binding_over` copies the stream route),
  `crates/ui/ess-ui-react/src/emit.rs`, `crates/ui/ess-ui-react/tests/live_binding.rs`
- Size: M, about 320 changed lines.

## Depends on

`sl18-stream-route`. It shares `binding.rs` with `sl01-member-client` and `sl05-union-variant`, so
the three are merged one after another.

## Source

A downstream application, 2026-11-24, second report, friction entry F18.
