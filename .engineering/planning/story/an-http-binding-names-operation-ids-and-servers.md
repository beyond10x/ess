---
format: aep.planning-md/3
id: story:an-http-binding-names-operation-ids-and-servers
kind: story
status: implemented
title: An http binding names operation ids and servers
summary: 'Adopter report S5: HTTP binding has no operation_id and no servers (ess#653).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#653
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/http_presentation.rs
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_http_presentation.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/compile.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/presentation.rs
- confidence: inferred
  path: docs/design/http-presentation-binding.md
- confidence: inferred
  path: website/docs/guides/http-routes.md
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:30Z", actor: "human:timo", revision: 12}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:31Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-10-29T00:14:15Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Problem

`ess-http/1` has no `operation_id` per route and no `servers`. The projected operationId is the qualified command name and the projected documents have no `servers`, so clients generated from them call the wrong URL and name operations differently from the served document. Adopter report S5, ess 0.68.0.

## Acceptance

Named tests are new files unless marked existing. Run the `ess-http` ones with `cargo test -p ess-http --locked --test presentation <fn>`.

- An id compiles into the IR and is absent when unset: `crates/specify/ess-http/tests/presentation.rs::a_route_operation_id_is_compiled_into_the_ir`.
- A duplicate id is refused naming both routes: `crates/specify/ess-http/tests/presentation.rs::a_duplicate_operation_id_is_refused_naming_both`.
- An id equal to another operation's qualified name is refused: `crates/specify/ess-http/tests/presentation.rs::an_operation_id_equal_to_another_operations_name_is_refused`.
- An id outside the grammar is refused: `crates/specify/ess-http/tests/presentation.rs::an_operation_id_outside_the_grammar_is_refused`.
- A refinement keeps a fixed id and may add one: `crates/specify/ess-http/tests/presentation.rs::a_refinement_keeps_a_fixed_operation_id`.
- Servers compile per network component: `crates/specify/ess-http/tests/presentation.rs::servers_compile_per_network_component`.
- Servers on a non-network component are refused: `crates/specify/ess-http/tests/presentation.rs::servers_on_a_component_not_reached_over_the_network_are_refused`.
- A bad server URL is refused: `crates/specify/ess-http/tests/presentation.rs::a_server_url_outside_the_rule_is_refused`.
- The IR reader holds the new invariants: `crates/specify/ess-http/tests/presentation.rs::the_ir_reader_refuses_duplicate_ids_and_bad_servers`.
- The id is projected as the operation's `operationId`: `crates/generate/ess-gen/tests/openapi_http_presentation.rs::a_route_operation_id_is_projected`, run with `cargo test -p ess-gen --locked --test openapi_http_presentation <fn>`.
- Servers appear on that component's document only: `crates/generate/ess-gen/tests/openapi_http_presentation.rs::servers_are_projected_on_their_components_document`.
- A binding without the keys is unchanged, proved by existing tests:
  - `cargo test -p ess-gen --locked --test openapi_http a_stage_1_binding_keeps_its_openapi_bytes`
  - `cargo test -p ess-http --locked --test request_encoding a_route_without_request_keeps_json_and_the_stage_1_ir_bytes`
- The CLI path generates both: `crates/edge/ess-cli/tests/http_presentation.rs::generate_openapi_projects_operation_ids_and_servers`, run with `cargo test -p ess-cli --locked --test http_presentation`.
- The code list stays exhaustive: `cargo test -p ess-http --locked --test compile every_code_this_crate_can_emit_is_one_of_the_codes_built_so_far` (existing, `crates/specify/ess-http/tests/compile.rs:1181`).
## Fit review

1. Need: a binding cannot state (a) the operationId a protocol or a generated client uses for a route, or (b) the base URL its paths resolve against. Today the projected operationId is the qualified name (`crates/generate/ess-gen/src/openapi.rs:43,763,942`) and the document has no `servers` (`crates/generate/ess-gen/src/http.rs:25-26`). Reproduction: a repro outside the repository, `653/opid.yaml` and `653/servers.yaml` against `651/system.yaml`. `ess specify http validate` at 0.74.0 gives "routes[0]: unknown field `operation_id`" and "unknown field `servers`". Requester's proposal, as theirs: `routes[].operation_id`, and a top-level `servers: [{url, description}]`.
2. Class: gap. `RouteSpec` and `HttpSpec` are closed (`deny_unknown_fields`, `crates/specify/ess-http/src/lib.rs:1342-1343, 1614-1615`), and nothing else carries either fact.
3. Not expressible. `ess/N` refuses keys that carry a route (`docs/design/http-presentation-binding.md:641-642`). The realization's `Invocation::Url` (`crates/.../ess-realization/src/lib.rs:353-363`) is an entrypoint to open, may hold `${NAME}`, and `--kind openapi` does not read it.
4. Fit: the keys are spelled like existing ones (snake_case, e.g. `source_digest`; `servers` as in OpenAPI and `asyncapi.rs:302`). They compose with views and commands alike, and with refinement as a fixed fact like `method` (ESS-HTTP-012, `lib.rs:2884-2894`). A binding spans every component (`DerivedRoute.component`, `lib.rs:1860-1870`), so one top-level list would give every component's document the same URLs, the documentation-only ones included. Hence `servers` is keyed by component and refused on a component not `reached_by: network`. The URL rule is copied from the realization's: no query, no fragment, no credentials (`lib.rs:361`). Targets: OpenAPI projects both. The Rust and Go servers publish `bound_json`, so they carry both with no dispatch change. No `ess_http` reader exists in ess-conformance, ess-diff or ess-ui (git grep). `ess ui` refuses `--http` (`docs/design/http-presentation-binding.md:644`). Grids: no `ess/N` construct, so no grid cell applies.
5. Second adopter: a team implementing a published API whose description fixes `listOrders`-style operationIds and a `https://api.example.test/v2` base, generating its SDK from the ESS document.
6. Cost: two codes (ESS-HTTP-032 operation_id, ESS-HTTP-033 servers); no migration; no generated bytes move without the keys; an older reader refuses an IR that uses them by name.
7. Considered: (a) change nothing, the adopter post-processes the document, which loses the check; (b) the requester's top-level `servers`, refused for the reason in 4; (c) chosen: per-route `operation_id` plus `servers` per network component; (d) servers from the realization, refused (see 3); (e) operationId derived from `naming.wire` in `ess/N`, refused: it needs a source format and would move every document's bytes.
8. Not established: whether the adopter's "three planes under three path prefixes" means one component or three (their words: "The served document mounts three planes under three path prefixes"). Unclear, ask them. Per-path `servers` is not offered.
## Decisions

Decision kind: accept, redesigned (D137, approved 2026-10-28). Format: none. The keys are additive optional keys in `ess-http/1` and `ess-http-ir/1`, the same way stage 2 added keys inside /1 (`docs/design/http-presentation-binding.md:455-460`). An IR without the keys keeps its bytes. An older reader refuses an IR that uses them, by name ("unknown field"), and does not misread it. No `ess/N` change. Codes are ESS-HTTP-032 and ESS-HTTP-033; ESS-HTTP-034 and up belong to the http-answers unit.

- `routes[i].operation_id`, optional, on a command or view route.
  - Grammar `[A-Za-z_][A-Za-z0-9_.-]*`. Widening it later is additive; narrowing it would not be.
  - Unique across the binding, and never equal to another operation's qualified name, which is that operation's derived operationId. Duplicates are refused with ESS-HTTP-032, naming both routes.
  - Projected as `operationId`; an unbound route keeps its qualified name.
  - Refinement: an id the refined binding fixed cannot change (ESS-HTTP-012 at `routes[i].operation_id`). One may be added where the refined route has none. The check is a new pass, so the http-answers unit's `changes()` is untouched.
- `servers: {<component>: [{url, description?}]}`, optional.
  - Entry order is kept: the first entry is the OpenAPI default server.
  - Refused with ESS-HTTP-033: a component that does not exist or is not `reached_by: network`; an empty list; a URL that is neither absolute `http(s)://` nor `/`-rooted; a URL holding whitespace, `{`, `}`, `?`, `#` or userinfo; a URL given twice for one component.
  - Projected as that component's document `servers`, placed after `info`.
  - Not compared under refinement: each server binding chooses its own deployment.
- IR: `Route.operation_id` and `HttpIr.servers` (in component order), omitted when absent. `from_json` refuses duplicate ids, a bad grammar, unordered components and bad URLs.
- Replaces the story's `## Acceptance`; keeps `## Problem`.
## Scope

Derived 2026-10-28 by story-scoper and checked by the prep agent. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/specify/ess-http/src/lib.rs`, for the DTOs, IR, compile passes and two new codes — cited
- **Files:** `crates/generate/ess-gen/src/openapi.rs:43,203,482,763,942,2785` (operationId, servers) — cited
- **Files:** `crates/generate/ess-gen/src/http.rs:24-26` (doc only) — cited
- **Tests:** new `crates/specify/ess-http/tests/presentation.rs`, `crates/generate/ess-gen/tests/openapi_http_presentation.rs`, `crates/edge/ess-cli/tests/http_presentation.rs`; `crates/specify/ess-http/tests/compile.rs:1186-1211` (code list) — inferred
- **Documents:** `docs/design/http-presentation-binding.md`, `website/docs/guides/http-routes.md`, `website/docs/reference/formats.md:191-192`, `CHANGELOG.md`, a new `changes/*.yaml` — inferred
- **Confidence:** high — the issue names both keys and the tree has one DTO, one IR and one projection site for each
- **Would collide with:** the http-answers unit (U5) on `RouteSpec`, `Route`, the `routes()` literal, `CODES`, the `from_json` doc, the `compile` doc table, `openapi.rs`, and the shared documents; split by symbol, with the new passes in their own functions
- **Safety fact:** an absent key is omitted from the IR and the document (`skip_serializing_if`, as at `lib.rs:1786-1787`), so bindings without the keys keep their bytes — unproven until the two existing byte tests pass

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
