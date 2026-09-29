---
format: aep.planning-md/3
id: story:transport-free-entry-point
kind: story
status: active
title: The generated server has a transport-free entry point
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 3}
---
## Outcome

The generated server crate exposes a transport-free entry point, so a conformance runner or an
in-process caller drives the system without sockets and without a hand-kept dispatch table.

## Acceptance

- `pub fn handle(system: &…, name: &str, input: json::Value) -> Result<json::Value, …>` (or the
  existing `dispatch` made public with that shape) routes every command and view by qualified name,
  with the same decoding, refusals and encoding the HTTP surface uses.
- An unknown name is a typed error naming it.
- A test drives a fixture model's commands and views through it with no socket.
