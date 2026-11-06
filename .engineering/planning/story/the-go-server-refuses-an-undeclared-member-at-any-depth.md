---
format: aep.planning-md/3
id: story:the-go-server-refuses-an-undeclared-member-at-any-depth
kind: story
status: draft
title: The Go server refuses an undeclared member at any depth of a request body
summary: Go typed decoder accepts undeclared nested members that Rust refuses (0.89.0 U2 adversary; from-finding)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
revision: 1
---
## Need

Found by the U2 adversary pass of wave-2026-11-06-ess (ESS 0.89.0). Pre-existing, outside that unit's
acceptance. The generated Go server's typed decoder (`crates/generate/ess-synth/src/go/json.rs`)
accepts an undeclared member inside a nested struct, both in a wrapped JSON body and in a bare body
below its first level. The generated Rust server answers such a request `malformed`. The two
servers therefore disagree on one request. The adversary's control case was a wrapped body whose
list element carries an extra member: Go 202, Rust 400.

## Acceptance

- One case in `crates/generate/ess-synth/tests/http_binding_rust.rs` (Go module) sends a wrapped
  body and a bare body, each with an undeclared member two levels down. Both servers answer
  `malformed`.
- The existing Go decoding cases stay green.

## Scope

- inferred: `crates/generate/ess-synth/src/go/json.rs` (typed decoder), `crates/generate/ess-synth/src/go/http/bound.rs`.
