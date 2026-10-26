---
format: aep.planning-md/3
id: story:generated-dispatch-and-handle-return-the-emitted-events
kind: story
status: implemented
title: Generated dispatch and handle return the events their command emitted
summary: 'Adopter report row 26: generated dispatch and handle discard the system log.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:51Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:52Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T11:30:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Problem

The generated Rust `dispatch` and `handle` run the command on the system, pump the system's event log, and then discard it (adopter report, row 26, ess 0.67.0: `router.rs:602-606` and `:733` in the generated server). An adapter calling them cannot see the events the command emitted, so it rebuilds its own frames (cancel frames in the report) from the response instead of from the events. The generator is `crates/generate/ess-synth/src/rust/http.rs:836` (`dispatch`) and `:1022` (`handle`), with the bound variant in `rust/http/bound.rs:608`.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/generate/ess-synth/tests/served_publication.rs`:
  `dispatch_with_events_returns_what_the_command_and_its_bindings_published`: in both layouts, `ops.core.Open` returns `[Opened, …the binding's events]` in log order, and the system log is empty afterwards.
- same binary, `handle_with_events_returns_the_same_events_as_dispatch`.
- same binary, `a_refused_command_returns_no_events`: a grant or input refusal returns an empty list.
- same binary, `dispatch_and_handle_keep_their_answer_bytes`: status and body of every route equal the pre-change answer, and `serve` is unchanged.
- `crates/generate/ess-synth/tests/http_binding_rust.rs`:
  `a_bound_dispatch_with_events_returns_the_taken_events`.

## Fit review

1. **Need.** A caller of the generated Rust `dispatch`/`handle` must see the typed events a command caused: its own and those its bindings' commands published in turn. Today each `run_*` pumps and then drops the log: `let _ = system.take_published();` (`crates/generate/ess-synth/src/rust/http.rs:1300`, in `settle`). Reproduction, ess 0.69.2: `ess generate synthesize --path examples/gatepass --target rust` writes `let _ = system.take_published();` three times (`gatepass-server/src/pass_service.rs:346, :442, :529`). The answer body already lists the command's own events as JSON `published` (since 0.47.0, 3b5ccfe87; `http.rs:1279-1282`). Cascaded events and typed values reach the caller nowhere. The requester's wording is "pump then discard the system log; adapter cannot see events, cancel frames rebuilt". Whether their frames come from cascaded events or only from the command's own is unclear; ask them.
2. **Class.** Gap in generated code. No authored document can reach it.
3. **Already expressible?** Partly. The JSON `published` list in the answer covers the command's own events (`served_publication.rs`, `a_served_command_answers_what_it_published_and_the_server_keeps_nothing`). Cascaded events and typed `SystemEvent`s have no idiom: `take_published` runs inside `run_*`, before control returns.
4. **Fit.** No authored surface. It reuses the existing `System::take_published` (`rust/system.rs:1048`) and `SystemEvent`. Siblings: the bound dispatch (`rust/http/bound.rs:608`) goes through the same `run_*`, so it gets the same change. Go has no public dispatch or handle (only `Serve<X>`, `go/http.rs:1261`), so there is nothing to return through there, and Go is out by that reason. Web and TypeScript serve nothing.
5. **Second adopter.** A component embedded in a host process forwards every event a command caused, cascades included, onto the host's own bus without re-reading the HTTP body.
6. **Cost.** No format, keyword or diagnostic. The generated API gains additive functions; the existing `dispatch`/`handle` signatures and answer bytes are unchanged. An in-place signature change would break every caller of the generated server, so it is rejected.
7. **Alternatives.** (a) Change nothing: the caller parses `published` JSON and has no cascade. (b) Change the `dispatch`/`handle` return types: breaking. (c) Leave the log for the caller to take: `dispatch` in a loop would then accumulate, breaking the "server keeps nothing" property (`served_publication.rs` module doc). (d) `run_*` returns the taken events; new `dispatch_with_events`/`handle_with_events` hand them out; the old functions call these and drop them (chosen).

## Decisions

- Accept, redesigned. The request asked for `dispatch`/`handle` themselves to return events. Additive siblings `dispatch_with_events(...) -> (http::Response, Vec<SystemEvent>)` and `handle_with_events(...) -> (Result<json::Value, entry::Refused>, Vec<SystemEvent>)` are chosen instead. The existing pair keeps its signature and bytes, and `serve` keeps calling `dispatch`.
- The events are everything `take_published` took for this call, in log order: the command's own first, then what delivering them published. A refusal before the port runs yields an empty list. `Refused::Undelivered` (effect committed, `http.rs:1304-1310`) still returns the taken events.
- Go: out of scope. It has no transport-free entry to return through.
- No `ess/32`, suite or `ess-http` format move.

## Scope

| unit | files | confidence |
|---|---|---|
| settle and runner | `crates/generate/ess-synth/src/rust/http.rs`: `settle` (`:1291-1311`) keeps the taken events; `command_handler`'s `run_*` (`:1236-1268`) returns them | cited |
| entry points | `rust/http.rs` `dispatch` (`:836-922`), `handle` (`:1022-1056`), `handle_arms` (`:1057`), `view_call` (views take nothing) | cited |
| bound surface | `crates/generate/ess-synth/src/rust/http/bound.rs` `dispatch` (`:608`), `handler` (`:758`) | cited |
| single-crate layout | `crates/generate/ess-synth/src/rust/single.rs` (feature-gated server) | inferred |
| docs | `website/docs/guides/synthesize.md` (served entry) | inferred |

Crate: ess-synth (sizing: M). Format gate: none. Tests: `crates/generate/ess-synth/tests/served_publication.rs` (binding-cascaded fixture, both layouts) and `crates/generate/ess-synth/tests/http_binding_rust.rs` (bound dispatch). Shares `rust/http.rs` and `served_publication.rs` with `story:a-published-event-payload-has-a-generated-decoder`, so serialize the two.

confidence: high
