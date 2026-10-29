---
format: aep.planning-md/3
id: story:two-outcomes-answered-alike-may-share-a-status-and-body
kind: story
status: active
title: Two outcomes answered alike may share a status and body
summary: 'Adopter report S3: Two outcomes answered alike cannot share a status and body (ess#652).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#652
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/openapi_http.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/http_binding/answer_forms.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/http_binding/go.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/served_publication.rs
- confidence: cited
  path: crates/specify/ess-http/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-http/tests/alike_and_text.rs
- confidence: cited
  path: docs/design/http-presentation-binding.md
- confidence: inferred
  path: website/docs/guides/http-routes.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T05:17:19Z", actor: "human:timo", revision: 6, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T05:17:19Z", actor: "human:timo", revision: 7, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

ESS-HTTP-025 refuses two outcomes on one status with bodies a reader cannot tell apart. Where the service answers both alike (one status, the bare response), the only valid binding is the `envelope` body, which projects `{outcome, response, published}` the service does not send. Adopter report S3, ess 0.68.0.

## Fit review

Scoped on 2026-10-28 against 0.74.0; every line below re-checked on 2026-10-29 against 0.76.0, after batch 1's U4 (#653, #651) landed in `ess-http` and `ess-gen`. Line numbers are 0.76.0's.

1. Need: two outcomes of one command that the service answers the same way (one status, one bare body) cannot be bound. Only `envelope` passes, and it projects `{outcome, response, published}`, which the service does not send. Reproduction, a repro outside the repository (`652/alike-response.yaml` against `673/system.yaml`): `ess specify http validate` on 0.76.0 still gives `error[ESS-HTTP-025]: routes[0].outcomes.fetched: outcome fetched of command repro.g.CreateNote answers 201 with response, and so does inline (response), with a body a reader cannot tell apart`; `alike-none.yaml` gives the same with `none`; `alike-envelope.yaml` is valid. (The scoper's 652 files pinned a digest of no system in the repro tree; re-pinned to `673/system.yaml`.) Requester's proposal, as theirs: "a binding may declare that two outcomes answer alike", explicitly rather than inferred; no syntax given.
2. Class: gap. ESS-HTTP-025 is the documented rule (`docs/design/http-presentation-binding.md:343-354`, "Status per outcome"), so this is not a defect.
3. Expressible today? No. `envelope` validates but projects a body the service does not send. `errors:` covers only error outcomes and answers them all with one answer (`docs/design/http-presentation-binding.md:356-364`). Merging the two outcomes in the model would delete a domain fact (two guards, two branches).
4. Fit: no `ess/N` construct, so no cell of `docs/design/language-grids.md` applies. Any two declared answers on one status (listed, covered by `errors`, redirects, error objects) may be declared alike, never only `response`. It reuses the outcome names the route already keys `outcomes:` by (`outcomes`, `crates/specify/ess-http/src/lib.rs:4415-4486`) and keeps answer 16 (`docs/design/http-presentation-binding.md:745-747`): each alike outcome still states its own status. Per consumer:
   - OpenAPI already groups by status and writes `x-ess-outcomes` (`crates/generate/ess-gen/src/openapi.rs`, `bound_responses` 1290-1350, `bound_response` 1445-1492). It must write one schema where `one_of` (1629-1669) would hold two equal ones.
   - Rust and Go servers: no change. Each arm writes its own outcome's compiled answer (`outcome_arm`, `crates/generate/ess-synth/src/rust/http/bound.rs:959-1044`, `crates/generate/ess-synth/src/go/http/bound.rs:1366-1463`).
   - Conformance: `crates/verify/` holds no `ess_http` reader (`git grep -ln ess_http -- crates/verify` is empty) and the stage-3 HTTP target (`docs/design/http-presentation-binding.md:660-686`) was never built, so the unit table's `ess-conformance` entry does not hold. The design page records what stage 3 does with an alike answer; the server tests carry the proof.
   - Binding diff: none exists (stage 1 answer 5, design page 777-778). Interpreter and web: untouched (design page 452-453).
   - Refusal overrides stay apart from outcomes (answer 24, design page 770-776); a refusal is never alike.
5. Second adopter: an upsert route whose `created` and `replaced` outcomes both answer `200` with the stored resource; a token refresh whose `rotated` and `kept` both answer `204`.
6. Cost: one optional route key; no new code, no migration, no format bump. ESS-HTTP-022 and ESS-HTTP-025 are widened; `CODES` (`lib.rs:35-68`, now ending at ESS-HTTP-033 after U4) is unchanged. A doc or diagnostic change needs `task test-xtask` (`AGENTS.md:151`).
7. Considered:
   (a) Change nothing: the adopter keeps four `envelope` rows in its diff.
   (b) Admit equal answers with no key: refused. ESS-HTTP-025 exists to catch accidental collisions, and the requester asked for an explicit form.
   (c) Per-outcome `same_as: <outcome>` with no status of its own: refused, it breaks answer 16.
   (d) A route-level group `alike: [[fetched, inline]]`: chosen.

## Decisions

Decision kind: accept, redesigned. The requester's need is taken as is; the shape is chosen here. Format: none. `ess-http/1` gains one optional route key and `ess-http-ir/1` writes it only when present, so existing documents keep their bytes (`docs/design/http-presentation-binding.md:455-460`). No new diagnostic code (D137 reserves ESS-HTTP-034 and up for this unit should one be needed).

- `routes[].alike` is a list of groups, each holding two or more of the command's outcome names.
- Every member has a declared answer that compiled (listed under `outcomes`, or covered by `errors`), and the members' compiled answers are equal: status, body, `encoding` (the text body of `story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views`, also in U5a), headers and redirect.
- The answers must also be ones the wire cannot tell apart. A group of `envelope` bodies, or of `{<member>: error}` objects with different error wire names, is refused: the wire already names the outcome.
- ESS-HTTP-022 refuses an unknown name, a name in two groups, a group of one, a name with no declared answer, and `alike` on a view.
- ESS-HTTP-025 refuses a group whose answers differ (naming the first differing key) or that the wire tells apart, and skips pairs inside one group.
- IR: `alike` is written only when present, members in name order (the IR reader has no specification to read declaration order from), groups ordered by first member; `from_json` refuses other orders and an empty list. Refinement compares it (ESS-HTTP-012, answer 21).
- OpenAPI: one response per status, `x-ess-outcomes` naming every member, one schema per group.
- The old acceptance line `conformance_accepts_either_outcome_for_the_shared_answer` is replaced by a design-page note: no HTTP conformance target exists to test.
- Replaces the story's `## Acceptance`; keeps `## Problem`.

## Acceptance

Unit U5a. Every test below belongs to U5a and runs before U5b starts. `crates/specify/ess-http/tests/alike_and_text.rs` and `crates/generate/ess-synth/tests/http_binding/answer_forms.rs` are new (the latter a `#[path]` module of `crates/generate/ess-synth/tests/http_binding/go.rs`, as `bare_field_and_errors` is at 1737-1738).

- An alike group validates: `fetched` and `inline` both `{status: 201, body: response}` with `alike: [[fetched, inline]]` compiles, and the IR records the group.
  `cargo test -p ess-http --locked --test alike_and_text outcomes_declared_alike_share_a_status_and_body`
- Answers that differ are refused: `response` against `none`, or two headers that differ, is ESS-HTTP-025 at `routes[0].alike`.
  `cargo test -p ess-http --locked --test alike_and_text alike_outcomes_whose_answers_differ_are_refused`
- Answers the wire tells apart are refused: two `envelope` bodies, or two error objects with different wire names, in one group is ESS-HTTP-025.
  `cargo test -p ess-http --locked --test alike_and_text alike_outcomes_the_wire_tells_apart_are_refused`
- Bad names are refused: an unknown name, a name in two groups, a group of one, a name with no declared answer, or `alike` on a view, is ESS-HTTP-022.
  `cargo test -p ess-http --locked --test alike_and_text alike_names_out_of_their_rules_are_refused`
- IR order holds: `from_json` refuses `alike` out of canonical order or written empty.
  `cargo test -p ess-http --locked --test alike_and_text compiled_ir_with_alike_out_of_order_is_refused`
- Refinement covers `alike`: a refining document that changes a group is ESS-HTTP-012.
  `cargo test -p ess-http --locked --test alike_and_text a_refinement_restates_alike`
- OpenAPI writes one response: one `201` response, `x-ess-outcomes: [fetched, inline]`, a single schema (no `oneOf`).
  `cargo test -p ess-gen --locked --test openapi_http outcomes_declared_alike_project_one_response`
- Both servers answer both outcomes with the same status and body bytes, in Rust and in Go.
  `cargo test -p ess-synth --locked --test http_binding_rust go::answer_forms::both_servers_answer_outcomes_declared_alike_with_one_answer`
- Existing bindings keep their bytes (existing tests):
  `cargo test -p ess-http --locked --test compile`
  `cargo test -p ess-gen --locked --test openapi_http a_stage_1_binding_keeps_its_openapi_bytes`
- Docs record the rule: the ESS-HTTP-022 and -025 rows (`docs/design/http-presentation-binding.md:427,430`) and "Status per outcome" state `alike`; the keys table (469-511) gains `routes[].alike`; "Stage 3" (660-686) states that an alike answer maps to the set and a scenario expecting either outcome passes, reported as not distinguishable on the wire; the `ess-http/1` and `ess-http-ir/1` rows of `website/docs/reference/formats.md` (193-194) and the compile doc table and `HttpIr::from_json` doc in `lib.rs` (2761-2802, 2063-2105) are amended.
  `task test-xtask`

## Scope

Derived 2026-10-28 by story-scoper; re-checked 2026-10-29 against 0.76.0 by the prep agent. Every line is **cited** (read in the tree) or **inferred** (worked out, could be wrong).

- **Primary surface:** `crates/specify/ess-http/src/lib.rs` — cited:
  - `RouteSpec` 1352-1388 and `Route` 1887-1935 (U4's `operation_id` now sits at 1361 and 1894);
  - the `Route { … }` literal in `routes()` 3777-3790;
  - `answer_invariants` 2333-2401, `changes()` 3254-3330 and `answer_changes` 3408-3498;
  - `outcomes` 4415-4486, `Counted` 5246, `Declared` 5301-5327, `distinguishable` 5348-5435, `counted` 5441-5478;
  - the compile doc table 2761-2802 and the `HttpIr::from_json` doc 2063-2105.
- **Files:**
  - `crates/generate/ess-gen/src/openapi.rs`: `bound_responses` 1290-1350, `bound_response` 1445-1492, `one_of` 1629-1669 — cited
  - `docs/design/http-presentation-binding.md`: 343-364, 421-441, 469-511, 660-686 — cited
  - `website/docs/reference/formats.md`: 193-194 — cited
  - `website/docs/guides/http-routes.md`: 112-170 — inferred
  - `CHANGELOG.md` — inferred
- **Tests:**
  - `crates/specify/ess-http/tests/alike_and_text.rs` (new) — inferred
  - `crates/generate/ess-gen/tests/openapi_http.rs` — cited
  - `crates/generate/ess-synth/tests/http_binding/answer_forms.rs` (new, `mod` in `go.rs` after 1741) — inferred
  - `crates/generate/ess-synth/tests/served_publication.rs`: a `oneOf` of two equal schemas fails contract validation, so the one-schema rule must hold there — inferred
- **Symbols:** `distinguishable`, `Counted`, `Declared`, `one_of`, `Route::answer` (`lib.rs:1949`) — cited
- **Not touched:** `crates/verify/ess-conformance` (no `ess_http` use) — cited
- **Confidence:** high; the ESS-HTTP-025 path was read end to end.
- **Would collide with:** U5b on `distinguishable`/`overridden` and `bound_response`; U5b runs after U5a in the same tree. U4 (landed) owns `operation_id` in the same structs and the `operation_id_changes` pass (3214-3248), which U5a leaves alone.
- **Safety fact:** the servers write each outcome's own compiled answer (`outcome_arm`), so equal answers need no server change. Level 2, unproven until the server test passes.
