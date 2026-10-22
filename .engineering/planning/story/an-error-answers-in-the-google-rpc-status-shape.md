---
format: aep.planning-md/3
id: story:an-error-answers-in-the-google-rpc-status-shape
kind: story
status: draft
title: An HTTP error may answer in the google.rpc.Status shape
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). The adopter's protocol answers errors on its HTTP surface in the shape of `google.rpc.Status`: `{"error": {"code": <int>, "status": "<CANONICAL_CODE>", "message": "…", "details": [{"@type": "…", "reason": "…", "domain": "…", "metadata": {…}}]}}`. Every error of every operation takes it. ESS has no body form that writes it, so the adopter hand-writes the error answer at every error site its server has.

## Reproduction

Reproduced on 0.68.0 by reading what the format admits rather than by a refusal: an `ess-http/1` outcome body is `envelope`, `response`, `none` or `{<member>: error}`, and the last writes the error's wire name under one member beside the error's own fields. None nests the error under `error` with a numeric `code`, a canonical `status` text and a `details` list. A minimal binding with `body: {error: error}` generates `http::error_object("error", "Unsupported", payload)`, a flat object.

## Where it lives

- `crates/specify/ess-http/src/lib.rs:488` (`pub enum Body`: `Envelope`, `Response`, `None`, `Error { member }`), traced.
- The generated writer `error_object`: `crates/generate/ess-synth/src/rust/http/bound.rs:566`, emitted at `:988` (traced to the generated call); the Go server has its own (not located).

## Acceptance (provisional)

- A route or the binding's top-level `refusals:` may declare an error body in the `google.rpc.Status` shape, with `code` and `status` per outcome and `details` carrying the error's wire name and fields.
- The generated Rust and Go servers write it byte for byte as declared; the OpenAPI projection describes it.
- A conformance scenario of a refused command reads the declared shape, not the envelope.
- Decide first whether this is a named body form or a general error template; the design belongs with `architecture-decision-record:transport-and-idl-compile-targets`, which covers gRPC and JSON-RPC but not this error shape.
