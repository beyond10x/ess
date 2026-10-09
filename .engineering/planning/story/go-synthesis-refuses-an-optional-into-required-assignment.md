---
format: aep.planning-md/3
id: story:go-synthesis-refuses-an-optional-into-required-assignment
kind: story
status: draft
title: Go synthesis refuses an Optional-to-required assignment it cannot represent instead of emitting code that does not compile
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The Go synthesis target refuses, by name, an assignment between an Optional value and a required
field that it cannot represent, instead of emitting code that does not compile; and
`ess generate synthesize` reports as generated only what compiles or type-checks.

## Evidence

An adopter on ess 0.56.0: `ess generate synthesize --target go` reports "56 capabilities: 56
generated, 0 refused", but the generated workspace does not compile.
`types/behaviour/behaviour.go:322` puts two Optional command inputs (`channel`, `payload`) into
required fields of the emitted event `MessagePublished`; `system/system.go:160` assigns the
reverse direction.

The Rust target refuses Optional/required mismatches in bindings
(`crates/generate/ess-synth/src/rust/feasibility.rs:852-880`, `BindingAssignment`); the Go target
has no equivalent feasibility check. Related:
`story:rust-binding-wraps-a-plain-copy-into-an-optional-member` (Rust refuses a plain-to-Optional
copy it could emit). The two stories share one rule: for each direction, a target either emits
the conversion (wrap; unwrap where the guard proves presence) or refuses it by name.

## Acceptance

- A Go synthesis test: an outcome that emits an event whose required fields are set from Optional
  inputs that no guard proves present is refused with a named obligation; with a guard proving
  presence it synthesizes and the generated module builds.
- The reverse direction (a required value into an Optional field) is emitted as a wrap and builds.
- A synthesis test over the Go target asserts that every capability reported as generated is in a
  module that `go build` (or `go vet`) accepts, where a Go toolchain is present.
