---
format: aep.planning-md/3
id: story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views
kind: story
status: active
title: ess-http/1 admits text bodies, route refusals, empty bodies and array views
summary: 'Adopter reports N21, N26, N27, N29: four answer forms refused by ess-http/1 (ess#673).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#673
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http/bound.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/http/bound.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/answer_forms.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/alike_and_text.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/route_refusals_and_view_answers.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: cited
  path: website/docs/guides/http-routes.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T05:17:19Z", actor: "human:timo", revision: 6, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T05:17:20Z", actor: "human:timo", revision: 7, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

`ess-http/1` refuses four answer forms an adopter binding served operations needs: a text body (N21, `ESS-HTTP-024`), a per-route `refusals:` (N26), a body on `empty` (N27) and a view answered as a bare array (N29, `ESS-HTTP-024`). Each was a narrow stage-2 choice in `docs/design/http-presentation-binding.md` ("Still refused", answers 17 and 20, `rows`/`single` only), recorded there as relaxable without a format change. ess#673, ess 0.68.0.

Related: `story:a-view-answer-takes-its-protocols-list-shape` (rows member renamed, cursor paging) stays separate and is not in this unit.

## Units

This story is delivered in two units, one after the other, in one tree:

- **U5a** (with `story:two-outcomes-answered-alike-may-share-a-status-and-body`, #652): N21, the text body.
- **U5b**: N26 route refusals, N27 a body on `empty`, N29 the `array` view answer. U5b changes the signatures of `overridden` and `distinguishable`, whose bodies U5a rewrites, so it starts when U5a is green.

The story is implemented, and #673 closes, when U5b's acceptance passes.

## Fit review

Scoped on 2026-10-28 against 0.74.0; every line below re-checked on 2026-10-29 against 0.76.0, after batch 1's U4 (#653, #651) landed in `ess-http` and `ess-gen`. Line numbers are 0.76.0's.

1. Need: four answer forms an adopter's served operations use cannot be stated. Each reproduced with `ess specify http validate` on 0.76.0, a repro outside the repository against `673/system.yaml`:
   - N21, an outcome answering `text/plain`: `n21-text.yaml` gives `error[ESS-HTTP-024]: routes[0].outcomes.fetched.body: text is not a body`. Today every bare body is JSON (`crates/generate/ess-synth/src/rust/http/bound.rs`, `outcome_arm` 959-1044, `http::JSON`).
   - N26, one route refusing in its own words: `n26-route-refusals.yaml` gives `routes[0]: unknown field refusals`.
   - N27, a `404` that carries a body: `n27-empty-body.yaml` gives `routes[0].empty: unknown field body, expected status`.
   - N29, a list answered as a bare JSON array: `n29-array.yaml` gives `error[ESS-HTTP-024]: routes[0].body: view repro.g.AllNotes answers rows or single, and array is not a view's body`. The control `control-rows.yaml` (`body: rows`) is valid.

   Requester's syntax, as theirs: `body: text` (N21; it names no value, and which text the adopter sends is not stated: unclear, ask them); route-level `refusals:` (N26); `empty: {status: 404, body: {error: error}}` (N27); `body: array` (N29).
2. Class: gap for all four. Each was a narrow stage-2 choice the design page says a later release can relax ("Still refused", `docs/design/http-presentation-binding.md:688-695`; answers 17 and 20, 748-760). For N21, `response.<field>` on a `String` field answers a quoted JSON string as `application/json`, a different wire.
3. Expressible today?
   - N21: no. `Content-Type` cannot be declared (ESS-HTTP-026, design page 379-382).
   - N26: no. Top-level `refusals:` applies to every named route (design page 562; `website/docs/guides/http-routes.md:166-167`), and a server serves one binding.
   - N27: no. `EmptySpec` has only `status` (`crates/specify/ess-http/src/lib.rs:652-655`).
   - N29: no. `ViewBody` is `rows` or `single` (`lib.rs:579-585`).
4. Fit: no `ess/N` construct, so no cell of `docs/design/language-grids.md` applies.
   - N21 keeps the `response.<field>` source. A new outcome key `encoding: json | text` mirrors `request.encoding` (design page 298-313) and "`Content-Type` belongs to the encoding" (379-381). `text` is admitted only with `body: response.<field>` whose field is `String` or a newtype of `String` (`string_type`, `lib.rs:4904-4913`); never under `errors` (its body is never a response field), on a refusal (it holds no model value) or on a view row (an object). ESS-HTTP-025 counts a text body as a bare body, told apart from nothing except through `alike` (#652). OpenAPI: `content_as("text/plain", <field schema>)` (`crates/generate/ess-gen/src/openapi.rs:2776-2785`). Servers: `text/plain; charset=utf-8` and the string's raw bytes, beside `JSON` and `MARKDOWN` (`crates/generate/ess-synth/src/rust/http.rs:1813-1819`; Go's `mediaMarkdown`, `crates/generate/ess-synth/src/go/http.rs:2438`).
   - N26 reuses `RefusalSpec` (`lib.rs:720-729`) and ESS-HTTP-029 unchanged, at `routes[i].refusals.<key>`. All three keys apply, on command and view routes. Answer 17 still holds: never a derived route, a `404` or a `405`. ESS-HTTP-025's refusal-override check (`overridden`, `lib.rs:5484-5526`) reads the route's effective map.
   - N27 reuses the refusal body shape (`none` or `{<member>: {literal: <text>}}`, `RefusalBody`, `lib.rs:733-774`). The requester's `{error: error}` is refused: a view declares no error. It stays open as an addition once `story:view-declares-its-answer-for-an-unknown-identity` gives a view an error.
   - N29 is a third `ViewBody`. It is refused on a paged view, for the reason `single` is (answer 20), and `empty` beside it is refused (ESS-HTTP-030).
   - Targets: OpenAPI per operation (`bound_responses` 1290-1350, `bound_view_responses` 1356-1409); Rust and Go `refusals()` and `view()` (`rust/http/bound.rs` `view` 1139-1205, `refusals` 1209-1284, `answers_standard_not_granted` 1290-1305; `go/http/bound.rs` `view` 1523-1599, `refusals` 1604-1657) and the emitted `single` helper (`rust/http/bound.rs:589-599`).
   - Conformance: `crates/verify/` holds no `ess_http` reader and no HTTP conformance target exists (design page 660-686), so the unit table's `ess-conformance` entry does not hold; the server tests carry the proof.
5. Second adopter: an endpoint answering a generated identifier or a signed token as plain text (N21); an admin route refusing `not_granted` with a message other routes do not use (N26); a resource read answering `404 {"error":"not_found"}` (N27); a public REST list endpoint answering `[...]` (N29).
6. Cost: no new diagnostic code; widens ESS-HTTP-012, -024, -025, -029 and -030. Adds `OutcomeSpec.encoding`/`Outcome.encoding`, `RouteSpec.refusals`/`Route.refusals`, `Empty.body` and `ViewBody::Array`. Generated servers emit one refusal function per overriding route; no generated API changes for a binding without the new keys. `task test-xtask` (`AGENTS.md:151`).
7. Considered:
   (a) Change nothing.
   (b) N21 as the requester's `body: text`: refused, it names no value. `body: text.<field>`: refused, a second spelling for reading a response field. `body: {text: response.f}`: refused, it collides with the `{<member>: …}` object form. `encoding: text`: chosen.
   (c) N27 as the requester's `{error: error}`: refused, there is no error to read.
   (d) N26 as a per-route `not_granted` key only: refused; `malformed` and `unfinished` differ per route for the same reason.
   (e) The design in 4: chosen.

## Decisions

Decision kind: accept, redesigned. N26 and N29 as proposed; N21 as `encoding: text` on an outcome with `body: response.<field>`, not `body: text`; N27 with `none` or a literal body, not `{error: error}`. Format: none. All four are optional `ess-http/1` keys, written to `ess-http-ir/1` only when present (`encoding` only when it is not `json`), so existing documents keep their bytes (`docs/design/http-presentation-binding.md:455-460`). No new diagnostic code (D137 reserves ESS-HTTP-034 and up for this unit should one be needed).

Unchanged: the `single` row answer, and what top-level `refusals:` does on routes with no override.

Amended on the design page: "Still refused" (a non-JSON body is admitted only in the N21 form); answer 17 (a route may override for itself); answer 20 (`empty` may carry a literal body); the response body table (315-330) gains `array` and the outcome `encoding`.

Conformance lines of the old acceptance (`conformance reads the body as text`, `the generated route and conformance`) are replaced by server tests: no HTTP conformance target exists.

Replaces the story's `## Acceptance`; keeps `## Problem` and the "Related" line.

## Acceptance

Run the `ess-http` tests with `cargo test -p ess-http --locked --test <file> <fn>`. New files: `crates/specify/ess-http/tests/alike_and_text.rs` (U5a), `crates/specify/ess-http/tests/route_refusals_and_view_answers.rs` (U5b) and `crates/generate/ess-synth/tests/http_binding/answer_forms.rs` (created in U5a, extended in U5b; a `#[path]` module of `crates/generate/ess-synth/tests/http_binding/go.rs`).

### U5a: N21, a text body

- A text outcome validates: `{status: 201, body: response.note_id, encoding: text}` compiles and the IR writes `encoding: text`.
  `cargo test -p ess-http --locked --test alike_and_text a_command_outcome_answers_a_text_body`
- Text out of its rules is refused: `text` beside `response`, `none`, `envelope`, an error object, a non-`String` field, or under `errors`, and an encoding outside `json`/`text`, are ESS-HTTP-024 at `routes[i].outcomes.<o>.encoding` (or `routes[i].errors.encoding`).
  `cargo test -p ess-http --locked --test alike_and_text text_out_of_its_rules_is_refused`
- The IR and refinement hold it: `from_json` refuses `encoding: json` written out and `text` beside any body but `response.<field>`; a refining document that changes an outcome's `encoding` is ESS-HTTP-012.
  `cargo test -p ess-http --locked --test alike_and_text encoding_is_held_by_the_ir_reader_and_refinement`
- OpenAPI answers `text/plain` with the field's string schema.
  `cargo test -p ess-gen --locked --test openapi_http a_text_outcome_projects_text_plain`
- Both servers write `Content-Type: text/plain; charset=utf-8` and the raw string bytes, the same in Rust and Go.
  `cargo test -p ess-synth --locked --test http_binding_rust go::answer_forms::both_servers_answer_a_text_body`
- Docs: the outcome body table, "Still refused", the ESS-HTTP-024 row (design page 429) and formats.md 193-194 state `encoding`. `task test-xtask`

### U5b: N26 route refusals, N27 a body on empty, N29 an array view

- A route refusal wins on its route: `routes[0].refusals.not_granted` answers that route with its own status and body, other routes keep the top-level one, and the override is checked under ESS-HTTP-029 at `routes[0].refusals.not_granted`.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers a_route_refusal_overrides_the_binding_refusal`
- ESS-HTTP-025 holds per route: a route refusal on the status of one of that route's outcomes, with no body that tells them apart, is refused.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers a_route_refusal_is_held_apart_from_its_outcomes`
- Refinement covers route refusals: a refining document that changes `routes[i].refusals` is ESS-HTTP-012.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers a_refinement_restates_route_refusals`
- An empty answer may carry a literal body: `empty: {status: 404, body: {error: {literal: not_found}}}` compiles and the IR writes the body; `{error: error}` is ESS-HTTP-030.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers an_empty_answer_may_carry_a_literal_body`
- `body: array` compiles; on a paged view it is ESS-HTTP-024, with `empty` ESS-HTTP-030.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers a_view_route_answers_a_bare_array`
- IR rules hold: `from_json` refuses `empty.body: none` written out, `array` beside `empty`, and route `refusals` out of key order or empty.
  `cargo test -p ess-http --locked --test route_refusals_and_view_answers compiled_ir_with_view_answers_out_of_rules_is_refused`
- OpenAPI states the per-route forms: the route's refusal response on that operation only, a literal schema at the empty status, an `array` of the row schema at `200`.
  `cargo test -p ess-gen --locked --test openapi_http route_refusals_empty_bodies_and_arrays_project`
- Both servers write the per-route refusal, the empty answer's body and a top-level array with the same bytes, and an unbound (derived) route keeps its answer.
  `cargo test -p ess-synth --locked --test http_binding_rust go::answer_forms::both_servers_answer_route_refusals_empty_bodies_and_arrays`
- Docs: "Still refused", answers 17 and 20, the response body table (315-330), the keys table (469-511), the code table (421-441), "Precedence" steps 4, 6, 7 and 8 (551-560) and the paragraph after them (562), `website/docs/guides/http-routes.md:166-167`, and formats.md 193-194 are amended. `task test-xtask`

### Both units: existing bindings keep their bytes

Run at the end of U5a and again at the end of U5b (existing tests):

- `cargo test -p ess-http --locked --test compile`
- `cargo test -p ess-gen --locked --test openapi_http a_stage_1_binding_keeps_its_openapi_bytes`
- `cargo test -p ess-synth --locked --test http_binding_rust without_a_binding_the_rust_server_keeps_its_bytes`

## Scope

Derived 2026-10-28 by story-scoper; re-checked 2026-10-29 against 0.76.0 by the prep agent. Every line is **cited** (read in the tree) or **inferred** (worked out, could be wrong).

- **Primary surface:** `crates/specify/ess-http/src/lib.rs` — cited:
  - U5a: `OutcomeSpec`/`Outcome` 610-647, `outcome_body` 4641-4750, `string_type` 4904-4913;
  - U5b: `ViewBody` 579-604 and `EmptySpec`/`Empty` 652-664, `refusal_changes` 3336-3401, `standard` 2857-2933, `answers` 4351-4407, `overridden` 5484-5526, `view_answers` 5562-5662;
  - both: `RouteSpec` 1352-1388, `Route` 1887-1935, the `Route { … }` literal 3777-3790, `answer_invariants` 2333-2401, `answer_changes` 3408-3498, `distinguishable` 5348-5435, the compile doc table 2761-2802 and the `HttpIr::from_json` doc 2063-2105.
- **Files:**
  - `crates/generate/ess-gen/src/openapi.rs`: U5a `rendered_outcome` 1575-1626, `command_schemas` 2101-2204, `content_as` 2776-2785; U5b `refusals_of`/`answered_by_binding` 626-647, `paths` 676-701 (the refusals at 689-691), `query` 917-986, `bound_view_responses` 1356-1409, `rendered` 1495-1571, `refusal_sentence` 1711-1727, the module doc row at 222 — cited
  - `crates/generate/ess-gen/src/http.rs`: `Binding::refusal` 424-426 becomes a per-route lookup (U5b) — cited
  - `crates/generate/ess-synth/src/rust/http/bound.rs`: `outcome_arm` 959-1044 (U5a); the emitted `single` 589-599, `view` 1139-1205, `refusals` 1209-1284, `answers_standard_not_granted` 1290-1305 (U5b) — cited
  - `crates/generate/ess-synth/src/go/http/bound.rs`: `outcome_arm` 1366-1463 (U5a); `view` 1523-1599, `refusals` 1604-1657 (U5b) — cited
  - `crates/generate/ess-synth/src/rust/http.rs`: `JSON`/`MARKDOWN` 1813-1819 (U5a), the `bound::refusals` call 482 (U5b) — cited
  - `crates/generate/ess-synth/src/go/http.rs`: `mediaMarkdown` 2438 (U5a), the `bound::refusals` call 218 (U5b) — cited
  - `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md:193-194`, `website/docs/guides/http-routes.md:112-170` — cited
  - `CHANGELOG.md` — inferred
- **Tests:**
  - `crates/specify/ess-http/tests/alike_and_text.rs` and `crates/specify/ess-http/tests/route_refusals_and_view_answers.rs` (new) — inferred
  - `crates/generate/ess-gen/tests/openapi_http.rs` — cited
  - `crates/generate/ess-synth/tests/http_binding/answer_forms.rs` (new, `mod` in `go.rs` after 1741) — inferred
- **Not touched:** `crates/verify/ess-conformance` (no `ess_http` use) — cited
- **Confidence:** medium. Compile and OpenAPI were read; how each bound route's handler calls its `refusal_*` function (emitted from `bound.rs`) was not traced, so U5b's server cost is estimated.
- **Would collide with:** `story:two-outcomes-answered-alike-may-share-a-status-and-body` (U5a) on `distinguishable`, `answer_changes` and `bound_response`, ordered U5a then U5b in one tree. U4 (landed) added `operation_id` to the same structs and touched `ess-gen/src/http.rs` (commit `6da9fa0ef`); U5 leaves those fields and the `operation_ids`/`operation_id_changes` passes alone.
- **Safety fact:** today one `refusal_*` function per surface serves every bound route (`rust/http/bound.rs:1209-1284`). A per-route override must not change the answer of an unbound (derived) route. Level 2, unproven until the U5b server test passes.
