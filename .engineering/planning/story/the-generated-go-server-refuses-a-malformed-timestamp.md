---
format: aep.planning-md/3
id: story:the-generated-go-server-refuses-a-malformed-timestamp
kind: story
status: draft
title: The generated Go server refuses a malformed Timestamp
summary: Go reads Timestamp as any text while Rust refuses it (W1 GEN1 review, pre-existing, backlog)
tags:
- backlog
- review-finding
relations:
- informed_by: review-result:security-w1-gen1-pass-1
revision: 1
---
## Need

The generated Go server reads a `Timestamp` with `textAt` and admits any text
(`crates/generate/ess-synth/src/go/http.rs:1168`, pre-existing), while the generated Rust server now refuses
one that is not an RFC 3339 date-time. Found by review-result:security-w1-gen1-pass-1.

## Acceptance

- The generated Go server answers 400 at decode, naming the path, for a `Timestamp` member that is not an
  RFC 3339 date-time, as the Rust server and the served OpenAPI do.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
