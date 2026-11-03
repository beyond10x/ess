---
format: aep.planning-md/3
id: story:go-synthesis-refuses-an-optional-into-required-assignment
kind: story
status: active
title: Go synthesis refuses an Optional-to-required assignment it cannot represent instead of emitting code that does not compile
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/binding_optional_wrap_go.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:24Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:24Z", actor: "human:timo", revision: 8}
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

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-synth/src/go/system.rs` (Go binding emission) — cited
- **Files:** `crates/generate/ess-synth/src/go/system.rs:526-530` (`mapping_expression`, the `Copy` arm writes `event.<field>` with no wrap) and `:342-358` (the input struct literal, `checks == false` path) — cited
- **Symbols:** `mapping_expression`, `input_expression` (`:400-417`), `proved_statements` (`:420-487`), `determined_prepared_input` (`crates/generate/ess-synth/src/plan.rs:1562-1566`), `proved_levels` (`crates/generate/ess-synth/src/condition.rs:143-171`) — cited
- **Also likely:** a new ess-synth test target for the Go wrap (name a suggestion: `tests/binding_optional_wrap_go.rs`) — inferred
- **Documents:** none
- **Confidence:** medium — the defect site is read; no Go compile was attempted
- **Would collide with:** any unit editing Go binding emission in `go/system.rs` (the #194 proved-copy path shares `input_expression` and `proved_statements`) and Go binding golden bytes; `story:rust-binding-wraps-a-plain-copy-into-an-optional-member` shares the rule, not the files — cited
- **Safety fact:** the wrap is safe because `is_assignable(T, Optional<T>)` holds (`crates/specify/ess-domain/src/types.rs:2855-2863`) — inferred
- **Already in the tree (partly):** Optional into required, unproven, binding: refused by the compiler (`ess-compiler/src/resolve.rs:5550-5562`, `:5745-5762`, `MAPPING_TYPE_MISMATCH`); event payload: refused at `resolve.rs:2986-2999` (`COMMAND_TYPE_MISMATCH`); presence-narrowed reads become an obligation (`ess-synth/src/determined.rs:458-468`); proved by a binding condition: built (`go/system.rs:420-487`). Required into Optional (acceptance bullet 2): not implemented — `go/system.rs:526-530` and `:342-358` emit `event.<field>` into a `*T` field (`go/layout.rs:1162-1177`); Rust refuses it at `rust/feasibility.rs:915-938` — cited
- **Not established:** whether the story's `behaviour.go:322` case reaches generated Go on main (the reporter's 0.56.0 build is not in this tree); the sibling story says Go "already does" wrap a plain copy, and no such arm was found.
- **Size:** medium, about 160 hand-written lines; no format version — inferred
- **Targeted run:** ess-synth new target and `--test go`; ess-conformance `--test binding_condition`, `--test adversary_runtime_parity_go` — inferred
