---
format: aep.planning-md/3
id: story:interpreted-eventual-views
kind: story
status: active
title: An eventual view is really eventual under the interpreter
summary: Views derive rows from source, filter, params, order_by and shape, and eventual consistency lags
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
- depends_on: story:interpreted-command-execution
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T16:54:45Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T16:54:45Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
---
# Story: an eventual view is really eventual under the interpreter

## Outcome

A view's rows are derived from what the model declares — `source`, `filter`, `params`, `order_by`
and `shape` — and a view declared `consistency: eventual` lags behind the writes that feed it rather
than answering immediately.

A single in-memory map is tempted into immediacy by its own architecture, which makes this the first
thing an interpreter gets wrong. `reference.rs` lags by `Billing::DEFAULT_LAG` further reads and
says why: a suite that never waits never tests the word `eventual`, and the first real projection
would find that out in production.

## Acceptance

Every view scenario in `examples/billing`'s committed suite reports the same result under `--target
interpreted` as under `--target billing`, including those that read an eventual view before and
after the lag.

## Scope

The interpreter's view path and its lag model. No change to `reference.rs` or to the suite.

## Complete runtime batch authorization and ownership

The operator's explicit mandate, “EVERY conformance test must support all the features,” authorizes completing this existing interpreter work as part of the grouped consumer-runtime batch. The native audit identified actual implementation gaps, not intrinsic model or consumer-adapter limits: bindings/redelivery/invocation observation, ordinary typed responses, view ordering/paging/aggregates and scenario-supplied facts. The original named acceptance remains required; a passing native Runner using another Target does not deliver this story.

Implementation owner: server_corrections in managed tree ess-backlog-one-time-response-20261002, after its explicit partial checkpoint. Its production ownership is interpret.rs and interpret/**, dedicated interpreter tests, and any reviewed observer/recording corrections. Coordinator owns synthesize/disclosure.rs, one_time_response/produce.rs and producer tests in carrier ess-backlog-next-20261002 after checkpoint handoff. Shared authored::compile_one policy attachment is frozen in that checkpoint; later changes require coordination. Go and TypeScript workers retain their own runtime paths. The interpreter trust comparison and fault matrix must run before claiming complete support. All work remains in the grouped PR; no additional remote gate is scheduled for intermediate checkpoints.
