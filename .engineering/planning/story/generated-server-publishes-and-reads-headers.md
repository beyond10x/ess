---
format: aep.planning-md/3
id: story:generated-server-publishes-and-reads-headers
kind: story
status: implemented
title: The generated rust server answers with what a command published, names every event, and keeps request headers
relations:
- serves: vision:O2
- informed_by: epic:generated-determined-behaviour
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T02:26:30Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T02:26:30Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-30T10:30:12Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

A client of the generated rust HTTP surface learns everything a command published, a runner can
name and encode any system event without a hand-kept table, and a shell can read request headers
to authenticate the caller before dispatch.

## Acceptance

- A command's HTTP answer (and `entry::handle`'s) carries the events its outcome published:
  `{"outcome": "<name>", "published": [{"event": "<qualified name>", "payload": {…}}]}` in
  publication order, with the created identity reachable from the payload; a scenario that
  `capture_instance`s a created identity and `expect_event`s over HTTP passes against a generated
  server with generated behaviours.
- The system crate has `SystemEvent::name() -> &'static str` (qualified name) and, behind the
  `server` feature, `wire::encode_system_event(&SystemEvent) -> String` covering every variant; a
  test iterates every variant of a fixture model.
- `server::http::Request` carries the request headers in arrival order with lower-cased names;
  existing routing is unchanged.
- Workspace and `--layout crate` both; committed generated trees regenerated and named; models
  without commands that publish keep their answer shape apart from the added `published` list
  (state in the changelog whether that is a wire change for existing clients).

## Origin

A downstream specification's conformance suite, run over the generated HTTP surface, cannot
capture created identities or expect events, keeps an 85-line hand-written event table, and cannot
see the Authorization header.
