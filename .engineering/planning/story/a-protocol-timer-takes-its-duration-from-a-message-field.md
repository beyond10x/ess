---
format: aep.planning-md/3
id: story:a-protocol-timer-takes-its-duration-from-a-message-field
kind: story
status: draft
title: An ess-protospec/1 timer takes its duration from a field of the received message, with a declared ceiling
summary: Timer durations are fixed or backoff only, so a server-advised reconnect interval cannot be modelled (adopter item 9, 2026-10-22); same on 0.67.0.
tags:
- adopter-report
- ess-0.70.0
- ess-0.70.0-first
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by an adopter on 2026-10-22 (item 9 of the second batch, relayed by the orchestrator; recorded on 0.56.0, re-probed
here). In a pub/sub protocol the server's advice carries an `interval` in milliseconds that the client must wait before it
reconnects. `ess-protospec/1` timers take a fixed duration or a backoff of an earlier interval, so the adopter pins 1000 ms.

Needed: a timer duration read from a field of the received message (or a local field set from it), with a declared bound, so
the bounded model stays finite.

## Reproduction

Brand-free `ess-protospec/1` document: participants `client` (states `connected, waiting, reconnecting`, timer `R`) and
`server`; `server` sends message `advice {interval: Integer}` on channel `advice`; `client` on receiving it arms `R`, and on
`R` firing emits `reconnected`.

With `after: {kind: fixed, millis: 1000}`, `ess specify protocol validate --path fixed.yaml` on 0.68.0 and 0.67.0:
`reconnect-advice — valid ess-protospec/1 (52962b69...)`.

With `after: {kind: input, field: interval, ceiling_ms: 5000}`, exit 1 on 0.68.0 and 0.67.0, identical:

```text
error: $ [MalformedProtocol]: unknown variant `input`, expected `fixed` or `backoff`
```

## Where it lives

- `crates/specify/ess-domain/src/protocol.rs:228` (`Duration`): `Fixed { millis }` and `Backoff { timer, factor, ceiling_ms }`
  only.
- `ValueSource::Input { field }` (:146) already reads a field of the triggering message for `set` and `send` effects; the
  timer has no such source. The checker's time bound (`bounds.max_time_ms`) is where a declared ceiling would have to fit.

## Regression status

Not a 0.68.0 regression: identical refusal on 0.67.0; no commit in `0.67.0..52bd110dd` touches `protocol.rs`. Gap. Does not
block 0.68.0.
