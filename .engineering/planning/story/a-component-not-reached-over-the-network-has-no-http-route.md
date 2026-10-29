---
format: aep.planning-md/3
id: story:a-component-not-reached-over-the-network-has-no-http-route
kind: story
status: implemented
title: A component not reached over the network has no http route
summary: 'Adopter report S2: A component not reached over the network still gets HTTP routes (ess#651).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#651
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/corpus/projection-digests.txt
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_reach.rs
- confidence: cited
  path: generated/openapi/email-service.yaml
- confidence: cited
  path: generated/openapi/invoice-service.yaml
- confidence: cited
  path: website/data/generate.terminal.json
- confidence: inferred
  path: website/docs/concepts/ess.md
- confidence: inferred
  path: website/docs/guides/generate-artifacts.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:32Z", actor: "human:timo", revision: 14}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:33Z", actor: "human:timo", revision: 15}
- {from: "active", to: "implemented", at: "2026-10-29T00:14:17Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Problem

`reached_by: network` gates grant projection (`crates/generate/ess-gen/src/http.rs:112`, `crates/generate/ess-gen/src/openapi.rs:558`), but every component's commands and views are still projected as HTTP routes. An adopter's OpenAPI diff carries 24 operations the service does not serve. Adopter report S2, ess 0.68.0.

## Acceptance

- A non-network document says no request reaches its paths, and its paths and the absent `x-ess-reached-by` are unchanged: `crates/generate/ess-gen/tests/openapi_reach.rs::a_component_not_reached_over_the_network_says_no_request_reaches_its_paths` (new file).
- A command-line component's document names its command line: `crates/generate/ess-gen/tests/openapi_reach.rs::a_command_line_component_names_its_command_line` (new).
  - Run both with `cargo test -p ess-gen --locked --test openapi_reach <fn>`.
- Network documents keep their bytes: `cargo test -p ess-gen --locked --test openapi_http a_stage_1_binding_keeps_its_openapi_bytes` (existing).
- The tutorial excerpt still matches: `cargo test -p ess-cli --locked --test example_page every_block_on_the_example_page_is_the_file_it_names` (existing).
- A binding may still route an in-process command: `cargo test -p ess-http --locked --test compile ess_http_013_an_operation_without_an_http_surface_is_refused` (existing, unchanged).
- Committed projections and site data are regenerated: `cargo xtask generate --check`, `cargo xtask site-data --check`.
## Fit review

1. Need: a reader or tool cannot tell, from a document's prose, that a component's OpenAPI document describes commands no request reaches. The document opens "The HTTP surface of `notes-api`" (`crates/generate/ess-gen/src/openapi.rs:518-523`; a repro outside the repository, `651/out/openapi/notes-api.yaml`) for a component with no `reached_by`. Requester's proposal, as theirs: no operations for such a component, "or the projection reports why it does".
2. Class: a small defect in wording. The documented semantics say these documents are documentation only: `Reach::Network` is "the declaration that makes the published OpenAPI document load-bearing rather than documentary" (`crates/generate/ess-domain/src/component.rs:247-249`). The document's opening sentence contradicts that. Removing the routes would reverse a documented decision, not fix a defect:
   - `website/docs/concepts/ess.md:58`: "POST command routes and, for network-reached components, GET view routes"
   - `crates/generate/ess-gen/src/openapi.rs:37-38`: every component gets a document
   - `crates/generate/ess-gen/src/http.rs:16-17`
   - `docs/design/http-presentation-binding.md:153`: a binding may name a command of any accepting component
3. Partly expressible today. Only a network component's document carries `info.x-ess-reached-by: network` (`openapi.rs:2812-2820`; `generated/go/gatepass/server/pass-service.openapi.json` has it, `generated/openapi/*.yaml` do not). An adopter's diff can select on that key, and nothing moves out of the specification.
4. Fit: views are already gated on reach (`http.rs:424`; ESS-HTTP-013 for views, `crates/specify/ess-http/src/lib.rs:3510-3522`); commands are not, by the decision cited in 2. The redesign keeps that row as it is and states it in the document. Every target is unchanged: servers are generated only for network components (`crates/generate/ess-gen/src/rust/http.rs:115`, `go/http.rs:61`, `go/entry.rs:19`).
5. Second adopter: a modular monolith whose library components are in-process and whose gateway is network, generating one SDK from all documents.
6. Cost: no code, format or diagnostic. Bytes that move:
   - `generated/openapi/email-service.yaml` and `generated/openapi/invoice-service.yaml` (one line each)
   - `website/data/generate.terminal.json` byte counts
   - every adopter's non-network documents (one sentence)
   Line counts stay the same, because each paragraph is emitted as one YAML line (repro output), so the tutorial excerpt `lines=66-93` holds.
7. Considered: (a) change nothing, which leaves the wording contradiction; (b) the requester's proposal, gating commands in `http::routes`, held for the operator because it breaks a documented contract and consumers (costs below); (c) chosen: one sentence, plus documenting the `x-ess-reached-by` selection.
8. Not established: the issue's "4 for in-process steps, 1 view route" sit outside the per-component rule (a view route can only belong to a network component). What they are is unclear, ask the adopter. Whether `website/data/billing-invoice-money.code-pair.json` embeds the description: `cargo xtask site-data` settles it.
## Decisions

Decision kind: accept, redesigned (D137, approved 2026-10-28). Format: none, no source format and no `ess-http` change. Only the first `info.description` paragraph of a non-network component's OpenAPI document moves, by one sentence on the same line.

- `description()` appends to its first paragraph, for `in_process`: "It is not declared `reached_by: network`: its callers are deployed with it, so these paths describe its commands and no request reaches them." For `command_line` the same, ending "callers reach it through its command line".
- `routes()`, `derived()`, ESS-HTTP-013 and every network document are unchanged. Routes stay: removing them breaks the `specification-to-contracts` tutorial and the ESS-HTTP-013 pins, and would reverse a documented decision.
- The guide documents selecting served documents by `info.x-ess-reached-by: network`.
- The story's third acceptance (refuse a binding on an in-process command) is dropped: it contradicts `docs/design/http-presentation-binding.md:153` and the pinned test `crates/specify/ess-http/tests/compile.rs:471-488`.
- Replaces `## Acceptance`; keeps `## Problem`.
- Held for the operator, not decided here: removing the non-network command routes (the requester's proposal). If the operator chooses it, it needs its own decision-blocker: "Does a component not reached over the network have HTTP command routes?". Its effects, by caller of `ess_gen::http::{routes, bound_routes, derived}`:
  - ess-cli `derived_routes` (`src/http.rs:96-98`): the derived table loses non-network command routes, and `ess-http-ir` `derived_routes` bytes move for those specs. A binding route on such a command would be dropped silently by `bound_routes` (`http.rs:463-488`) unless ESS-HTTP-013 (`lib.rs:3474-3486`) is extended, which flips `compile.rs:480-487`.
  - ess-conformance `view_grant.rs:59-70`: unchanged (filters `grants_checked_on`, keeps views only). No conformance suite bytes move.
  - ess-ui-check `model.rs:1600-1606`: unchanged (skips non-network components first).
  - ess-domain `wire.rs:67-69`: comment only.
  - ess-synth (`served.rs:24`, `view_query.rs:218`, rust and go http): network only, so unchanged.
  - openapi.rs: non-network documents become `paths: {}` while `schemas()` (`openapi.rs:2042-2045`) still emits command schemas. That fails `every_schema_the_document_declares_is_pointed_at_by_something` (`crates/generate/ess-gen/tests/openapi.rs:389`) and the billing tutorial excerpt (`example_page.rs`).
## Scope

Derived 2026-10-28 by story-scoper and checked by the prep agent. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-gen/src/openapi.rs:38,518-523` (`description()` first paragraph, module table row) — cited
- **Files:** `generated/openapi/email-service.yaml`, `generated/openapi/invoice-service.yaml`, `website/data/generate.terminal.json` (regenerated) — cited
- **Tests:** new `crates/generate/ess-gen/tests/openapi_reach.rs` — inferred
- **Documents:** `website/docs/concepts/ess.md:58`, `website/docs/guides/generate-artifacts.md:27`, `CHANGELOG.md`, a new `changes/*.yaml` — inferred
- **Not touched:** `crates/generate/ess-gen/src/http.rs`, `crates/specify/ess-http` — cited
- **Confidence:** high — one function writes the sentence, and the callers of `routes()` were walked
- **Would collide with:** the http-answers unit (U5) only if it edits `description()`'s first `format!` (518-523); its sentence for stage 2 is at 587-604
- **Safety fact:** the sentence joins a paragraph already emitted as one YAML line, so line counts and the tutorial excerpt hold — unproven until the repro output and the tutorial test confirm it

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
- Shared with U1 (`story:an-entity-invariant-is-projected-into-its-schema`): both units regenerate `generated/openapi/invoice-service.yaml` (U1 the billing `x-ess-entities`, this unit its `info.description`), the byte count in `website/data/generate.terminal.json` and the openapi and served lines of `crates/generate/ess-gen/tests/corpus/projection-digests.txt` for billing, oracle-fixture and revision-pair. U1 merges first. This unit merges second, and on a conflict in any of these files the integration branch takes neither side: it regenerates them from the merged sources and commits the result, then `task projection-check` must pass.
