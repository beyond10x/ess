---
format: aep.planning-md/3
id: story:bindings-read-the-delivery-context
kind: story
status: active
title: An event-triggered binding cannot read the delivery context (the channel or subscription the event arrived on)
refs:
- provider: github
  reference: beyond10x/ess#195
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:46Z", actor: "human:timo", revision: 3}
---
# Story: An event-triggered binding cannot read the delivery context (the channel or subscription the event arrived on)

## Why

beyond10x/ess#195. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #195 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The issue is open. Nothing in this tree does it: event bindings have no typed delivery context, and the design says so and defers it. Tree: `ess-scope-next` at `e932781965` (chore: release 0.40.0).

**(1) Reproduces, and is not fixed**
- cited: `crates/specify/ess-domain/src/binding.rs:1040-1050` (`check_mapping_cause`). `host_context.*` or `host_read.*` without a periodic cause is refused as `UnobservableFact` with "host mappings require a periodic cause and one declared field", the issue's first error.
- cited: `binding.rs:1463` raises "`event.<f>` is not a field of `<event>`", the issue's second error for `event.channel`.
- cited: `binding.rs:162-167`. `BindingCause::Event(QualifiedName)` carries no fields. `RawTrigger` (`binding.rs:241-247`) has only `event` and `periodic`, and `deny_unknown_fields` makes `context_fields` under `when:` a parse error.
- cited: `crates/specify/ess-compiler/src/resolve.rs:3883-3890`. The compiler refuses a second time: "event bindings cannot use periodic host inputs".
- cited: `docs/design/binding-mapping-bounded-accessor.md:581-588`. The follow-up contract is specified there but not built: a declared, typed context record, `context.<field>`, host-bound authority, missing context refused, no `event.<channel>`.
- cited: `.engineering/planning/task/runtime-adopter-gaps-9-13.md:163` records the same gap as open.

**(2) Where the fix lands**
- cited: `ess-domain/src/binding.rs`. Add context fields to `RawTrigger`, `BindingCause::Event` and `MappingSource::parse` (`:741`), a new variant next to `HostContext` (`:700`), and a new check in `check_mapping_cause` (`:1026`).
- cited: `ess-domain/src/binding/periodic.rs:166,202,247`. The `context_fields` validation (primitive admission, duplicates) there is the model to reuse or factor out.
- cited: `ess-compiler/src/resolve.rs:3556` (`event_binding`), `:3701/:3732` (periodic field resolution to copy), `:4184` (display), and `ess-compiler/src/ir.rs:1614` (`ResolvedMappingValue`).
- cited: every exhaustive match on `ResolvedMappingValue` must get the new variant: `ess-synth/src/plan.rs:1135,1258`, `ess-synth/src/go/system.rs:443`, `ess-gen/src/asyncapi.rs:402,904`, `ess-gen/src/docs.rs:1886,2931`, `ess-diff/src/diff.rs:1104`, `ess-service-contract/src/lib.rs:528`, `ess-conformance/src/synthesize.rs:8551`, `ess-conformance/src/periodic.rs:723`.
- inferred: conformance needs a way to deliver one event under two contexts. Today's event path only runs the upstream command, which publishes the event, plus `redeliver_event` (`ess-conformance/src/target.rs:266,784`). Neither carries a context. That means a new or extended target method and scenario steps in `scenario`/`synthesize.rs`.
- inferred: the JSON schema and published format docs change (`website/docs/guides/write-a-specification.md` around `:1272`), and the format may need an ess/3 gate like the periodic one (`periodic.rs:191`).

**(3) Collisions**
- inferred: #209, #198, #199 are arrangement and witness search in `ess-conformance/src/synthesize.rs`. They share that file with #195's binding-scenario section (`:8551`, `:8618`), but in different regions. Merge risk is low to medium.
- inferred: #196 (Map witness) is in `ess-conformance/src/witness.rs`/`synthesize.rs`. It overlaps only if #195's context values are witnessed through the same generator.
- inferred: #201 is in `ess-domain/src/command/*` (outcomes, wrong_state). No overlap.
- inferred: #210 is in the mutate `--collect` scoring. No overlap.

**(4) Design decisions for the implementor**
- Spelling: `context.<f>` (the design's choice, `:582`) or reusing `host_context.<f>`, which already parses (`binding.rs:742`) but means the periodic host.
- Where the declaration lives: `when.context_fields` (issue), or a host contract with `owner`/`authority` like `HostInputContract` (`periodic.rs:156-170`). The design requires naming the source of authority (`:585`), and the issue's shape has none.
- Events published inside the system: where their context comes from, or whether a context-bearing binding is admitted only for events from an external channel.
- The target-protocol change and its default: whether an event can be delivered with a context and whether it may answer `unsupported`, plus what that does to generated handler signatures (Go synth).
- Delivery semantics: whether a context-bearing binding is limited to `at_most_once`/`drop` like periodic ones (`binding.rs:1015`), or whether redelivery carries the original context.

Confidence: high that it reproduces and is unfixed (read on this tree, not run); medium on the change surface and collisions (grep-derived).

Paths:
- crates/specify/ess-domain/src/binding.rs
- crates/specify/ess-domain/src/binding/periodic.rs
- crates/specify/ess-compiler/src/resolve.rs
- crates/specify/ess-compiler/src/ir.rs
- crates/verify/ess-conformance/src/target.rs (inferred)
- crates/verify/ess-conformance/src/synthesize.rs (inferred)
- crates/generate/ess-synth/src/plan.rs, go/system.rs (inferred)
- crates/generate/ess-gen/src/asyncapi.rs, docs.rs (inferred)
- crates/verify/ess-diff/src/diff.rs (inferred)
- crates/specify/ess-service-contract/src/lib.rs (inferred)
- docs/design/binding-mapping-bounded-accessor.md

Verdict: needs-design. The defect is open, and the context's authority source and the target-protocol shape are unresolved.
