---
format: aep.planning-md/3
id: story:view-paging-and-caller-filters
kind: story
status: implemented
title: A view can declare paging and a caller-supplied filter
refs:
- provider: github
  reference: beyond10x/ess#174
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/reference.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T22:01:15Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T22:02:28Z", actor: "human:timo", revision: 6, imported: true}
- {from: "active", to: "implemented", at: "2026-09-28T04:35:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Scope

`page`/`size` parameters that slice the declared order with a total, and a caller filter parameter, without the unobservable-parameter refusal.

## Acceptance

The #174 repro validates; a scenario with more rows than `size` requires the page length, slice and total.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

**Decision (coordinator, 2026-09-27):** this story delivers paging only. The caller-supplied free-form filter is declined for this cut: the acceptance names only page length, slice and total; #174 itself defers its semantics ("would need a declared grammar"); `validate_params` (`view.rs:1547-1575`) and `SemanticViewRequest.params` (`target.rs:736-741`) state that a parameter nothing selects on is refused; and `bound()` (`synthesize.rs:4645`) cannot settle a free-form value.

- **Primary surface:** `crates/specify/ess-domain` (view declaration) + `crates/verify/ess-conformance` (synthesis, runner, Go/TS) — cited
- **Domain:** `view.rs` `validate_params` (1552, the `unobservable_fact` refusal), `ViewSpec`/`RawViewSpec` `params`/`order_by` (860, 877, 1608, 1617), `validate_order` (1492) — cited
- **IR:** `ess-compiler/src/resolve.rs:2757-2808`, `ir.rs` `ResolvedView` (1253, 1271) — cited
- **Synthesis:** `synthesize.rs` `view_expectations` (3842), `RANKING_ROWS` (3793), `bound` (4645), `rows_shown` (4542) — cited
- **Suite:** `scenario.rs` `ViewExpectation` (2240, `Ranked` 2331); `target.rs` `query_view`/`SemanticViewRequest`/`SemanticViewResult` (187, 733, 758, rows only, a total needs a field); `runner.rs` (799, 2595, 2681-2738) — cited
- **Runtimes:** Go `runtime.go` (1090, 1190, 1206, 3220), TS `runtime.ts` (1800, 1901, 1918, 2992) — cited
- **Also likely:** `reference.rs`, `interpret.rs`, `faulty.rs`, `admission.rs`, `go/explore.go`, `ts/fixtures.ts`, `ess-diff`, `ess-gen/src/{docs,openapi}.rs` — inferred
- **Registries:** source + suite format bump — inferred
- **Confidence:** medium
- **Would collide with:** `view.rs`, `synthesize.rs`/`scenario.rs`/`runner.rs`/`target.rs`, Go/TS runtimes, format registries
