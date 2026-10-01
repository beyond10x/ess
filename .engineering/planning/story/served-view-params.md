---
format: aep.planning-md/3
id: story:served-view-params
kind: story
status: active
title: Synthesized servers pass declared view parameters from the query string to the view port
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:feature-request-310
- depends_on: story:go-generated-behaviour
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/port.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/port.rs
- confidence: cited
  path: crates/generate/ess-synth/src/view_query.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:52Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Synthesized Go and Rust servers accept the view parameters a view declares, from the query string, and pass them to the view port (defect B of #311).

## Fit review

One request (#311), one review: see story:ui-binding-contract, `## Fit review`. This story is S5 of its five.

## Decisions

Accept, redesigned. Decode declared params from the query string by wire name; a missing required or undecodable value is a 400 `{"refused":…}`; undeclared keys ignored; typed params passed to the view port; queries for views with params stay obligations in every target (excluded by the epic); paging stays refused; fix the stale comments `openapi.rs:560-564`, `rust/http.rs:1289-1294`.

## Acceptance

`crates/generate/ess-synth/tests/view_params_served.rs`: `a_view_param_reaches_the_port_from_the_query_string_rust`, `a_view_param_reaches_the_port_from_the_query_string_go`, `a_missing_required_param_is_a_400_refusal`, `a_view_without_params_keeps_its_bytes`.

## Scope

`ess-synth/src/{go,rust}/http.rs`, `{go,rust}/port.rs`, `view_query.rs`, `ess-gen/src/openapi.rs` (comment).

## Sequencing

After #310 (0.51.0). Shares Go files with story:go-generated-behaviour: one at a time. Queries for views with parameters stay obligations in every target; the epic excludes generating them. `CHANGELOG.md` is a merge-time edit (epic).
