---
format: aep.planning-md/3
id: story:http-binding-carries-transport-framing
kind: story
status: draft
title: 'An HTTP binding states transport framing: long-polling, CORS, JSONP, WebSocket'
tags:
- ess-0.71.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

An HTTP surface whose protocol is a publish/subscribe exchange can state its transport framing in
the binding: long-polling, cross-origin access (CORS), JSONP and WebSocket framing. Generated
servers and the OpenAPI projection follow it; a framing ESS does not support is refused by name.

## Evidence

A consumer serving a publish/subscribe protocol over HTTP needs long-polling, CORS, JSONP and
WebSocket framing beside the routes. The `ess-http/1` design
(`docs/design/http-presentation-binding.md`) covers routes, path parameters and refinement in stage
1, and bodies, statuses and headers in stage 2; neither stage names framing, and the plan has no
story for it. decision-blocker:http-surface-scope decided that ESS describes an HTTP surface whose
wire shape it does not choose, as a staged binding; this story is a candidate for a later stage of
that binding.

## Acceptance

- A design section decides, per framing (long-polling, CORS, JSONP, WebSocket), whether it is a
  binding key, a deployment concern outside ESS, or refused, with the reason; the format
  consequence for `ess-http/1` is decided and recorded.
- Each framing the design admits validates and compiles into `ess-http-ir/1`, is projected into
  OpenAPI where OpenAPI can express it, and is honoured by the Rust and Go servers; each one it does
  not admit is refused by name.
- Bindings without framing keep their bytes.
