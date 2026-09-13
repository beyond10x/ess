---
format: aep.planning-md/1
id: story:interpreted-eventual-views
kind: story
status: draft
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
revision: 3
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
