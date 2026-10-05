---
format: aep.planning-md/3
id: story:served-replays-response
kind: story
status: draft
title: Served servers answer a replays outcome with its declared response
tags:
- follow-up
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A `replays` (retained-result) outcome that OpenAPI declares with a required `response` is answered with that response by the generated Rust and Go servers, and the Rust served answer for such a variant compiles.

## Origin

Found while implementing beyond10x/ess#423/#424 (2026-10-04): the OpenAPI projection declares a required `response` for a `replays` outcome, but neither served server writes it, and the Rust served answer pattern for that variant may not compile. Pre-existing; not part of #423/#424.
