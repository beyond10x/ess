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
- depends_on: story:an-event-channel-may-be-bound-to-server-sent-events
revision: 2
---
## Need

The generated React app connects a channel to `<base>/channels/<channel name>` and reads each frame
as `{event, payload}` or as a bare payload (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:115-131`,
`:215-223`). After the revised `story:an-event-channel-may-be-bound-to-server-sent-events`, a binding
declares the real stream route. Its compiled `ess-http-ir/1` resolves, per event, where the frame
name sits (the `event:` line or a data member, with its fixed value) and where the payload sits.
The protocol document is the source of that resolution. The UI binding carries routes for views
and commands only (`crates/ui/ess-ui/src/binding.rs:25-40`), so the app keeps the conventional
address.

## Outcome

- When the binding has a stream route carrying every event a `server_to_client` channel carries,
  the UI binding names that route for the channel.
- The generated app connects there and reads each frame by the resolved frame name and payload
  place. It maps each frame to the event the channel's `live.on` names.
- A channel no stream route carries keeps today's address.
- The UI reads the compiled IR, never the protocol file, so `ess ui` needs `--protocol` only where
  it compiles the binding itself (`pr0-http-binding-names-a-protocol`).
- The terminal stays on fixture scripts. It has no network channel today
  (`crates/ui/ess-ui-tui/src/live.rs:1-6`), and this story adds none.

## Acceptance

- **Red first.** The case is in `crates/ui/ess-ui-react/tests/live_binding.rs`. The binding holds
  the revised SSE story's route (`GET /v1/changes`, `shop.delivery.ShipmentChanged` as protocol
  message `shipment` with the fixed member `kind: shipment` and `application: {member: shipment}`),
  and a channel `carries: {events: [shop.delivery.ShipmentChanged]}` connects to `/v1/changes`. The
  frame `{"kind": "shipment", "shipment": {"id": 7}}` patches row 7 of a section with
  `live: {channel: shipments, effect: patch_row}`. Before the change it connects to
  `/channels/shipments`.
- A frame named by its `event:` line (`event: depot`, data the bare payload of
  `shop.delivery.DepotChanged`, as in the SSE story's example) is read for a channel carrying that
  event.
- A frame whose name the channel does not carry is dropped, and the channel stays live.
- `ess ui check --model --http <binding>` reports a channel whose events no single stream route
  carries as a warning at the channel, naming the events. The app keeps the conventional address
  for it.
- A binding without a stream route keeps its JSON bytes.
- Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/ui/ess-ui/src/binding.rs` (`Binding` 25-40)
- cited: `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl` (115-131, 215-223)
- inferred: `crates/ui/ess-ui-check/src/model.rs` (`binding_over` copies the stream route),
  `crates/ui/ess-ui-react/src/emit.rs`, `crates/ui/ess-ui-react/tests/live_binding.rs`
- Size: M, about 320 changed lines.

## Gate

- `task fmt-check`
- `cargo clippy -p ess-ui -p ess-ui-check -p ess-ui-react --all-targets --locked -- -D warnings`
- `cargo test -p ess-ui-react --locked --test live_binding`
- `cargo test -p ess-ui-check --locked`
- `task ci-lint` if the UI binding's public shape changes

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `story:an-event-channel-may-be-bound-to-server-sent-events` (revised), unit a.
- It shares `binding.rs` with `sl01-member-client` and `sl05-union-variant`, so the three merge one
  after another.

## Fit review

Second report, F18. The fit review is in the revised SSE story. `ess-ui/1` gains no key: the
channel still says what it carries, and the binding supplies the transport, as the channel's
reference promises ("the renderer picks the transport").

## Decisions

None beyond PR-D4, which supersedes SL-D12.

## Source

A downstream application, 2026-11-24, second report, friction entry F18.
