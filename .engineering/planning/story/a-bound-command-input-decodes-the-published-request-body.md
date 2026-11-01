---
format: aep.planning-md/3
id: story:a-bound-command-input-decodes-the-published-request-body
kind: story
status: implemented
title: A bound command input type decodes the request body its OpenAPI document publishes
summary: Under an ess-http/1 route with path parameters, the generated input type requires the path fields and refuses the body OpenAPI publishes.
tags:
- ess-0.70.0
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/http.rs
- confidence: cited
  path: crates/edge/ess-cli/src/model_types.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T06:49:18Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-01T06:49:18Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-11-01T10:37:58Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `Message::of_command` (`crates/generate/ess-gen/src/model_types.rs:313`) keeps every input field. With a route carrying `note_id` in its path, `demo.notes.Create.input` refuses `{"isPinned":true,"title":"Buy milk"}` with `"note_id" is a required property`.
- story:generate-types-emits-command-input-response-and-view-row-types accepts on "decodes what the OpenAPI document publishes"; the 0.69.0 CHANGELOG states the exception ("Under an ess-http/1 binding the input type still holds the fields a route carries in its path").
- Changing the input type later changes generated output adopters may pin: decide here whether the bound body gets its own root (for example `<command>.body`) or the input root drops path fields, and classify it for `ess verify diff`.
- Red test: `a_bound_command_input_decodes_the_request_body_the_bound_projection_publishes` (ess-gen, adversary_gu15_command_view_roots.rs).

## Fit review

Written 2026-11-01 by the D343 coordinator from the story-scoper's cited reading of 45a1beba7.

1. **Need.** An adopter decodes an HTTP request body with the type `ess generate types` emits for
   the command. Under an `ess-http/1` route that carries input fields in its path, the bound OpenAPI
   document publishes `{command}.Input` without those fields (`crates/generate/ess-gen/src/openapi.rs:2216-2229`,
   `carried` at `:718-728`), but `<command>.input` keeps every input field
   (`crates/generate/ess-gen/src/model_types.rs:299-352`, `Message::of_command` at `:309`), so the
   generated type refuses the published body (`"note_id" is a required property`). Minimal repro:
   `demo.notes.Create` with `routes: [{command: demo.notes.Create, method: put, path: /notes/{note_id},
   parameters: {note_id: input.note_id}}]`. The adopter asked (beyond10x/ess#656) for body types
   "with the same wire shape the HTTP projection publishes for that body".
2. **Class.** Defect: `BodyRoot::definition` documents itself as "the schema the HTTP projection
   publishes for the body" (`model_types.rs` `BodyRoot`), and the 0.69.0 CHANGELOG states the bound
   case as an exception (`CHANGELOG.md:214`).
3. **Already expressible?** No. `ess generate types` takes no binding (`crates/edge/ess-cli/src/model_types.rs:9-110`);
   only `ess generate --kind openapi --http` and `ess conform synthesize --http` load one
   (`crates/edge/ess-cli/src/http.rs:106-150`).
4. **Fit.** The binding reaches `generate types` the way it reaches the OpenAPI projection: an
   `--http <binding>` flag, the same spelling and the same compile (`ess specify http compile`). Under
   it, each body root is the body the bound document publishes; without it, every root keeps today's
   bytes. No new root name, so `<command>.input` keeps one meaning: the command's request body on the
   wire the caller chose. Targets: Rust, Go and TypeScript realization read the same root definition,
   so all three follow; synthesis, the interpreter and the entity runtime do not read type roots.
5. **Second adopter.** Any service that serves a command at `PUT /things/{id}` and decodes the body
   with generated types needs it.
6. **Cost.** One CLI flag; no format version, no new keyword, no diagnostic code unless a selected
   body does not exist under the binding (a route with `request: {encoding: query}`, or a path that
   carries every field: the bound document publishes no body, so `--root <command>.input` is refused
   by name and `--all-commands` omits it). No committed generated tree moves (none is produced by
   `generate types`); `ess verify diff` diffs the compiled IR, not projections
   (`crates/verify/ess-diff/src/lib.rs:19-30`), so there is no diff classification to add.
7. **Considered.** (A) drop path fields from `.input` unconditionally: impossible without a binding
   and breaking for unbound callers. (B) a new `<command>.body` root beside `.input`: a second
   spelling for one wire body, and `--all-commands` output grows for every bound adopter. (C) change
   nothing and keep the documented exception: leaves the adopter writing bodies by hand. Chosen: the
   `--http` flag (A restricted to an explicit binding).

## Decisions

**Accept, redesigned** (D343, 2026-11-01, coordinator under the orchestrator's grant). The request's
"same wire shape the HTTP projection publishes" is taken; the shape is decided here:
`ess generate types --http <binding>` makes every selected body root the body the bound OpenAPI
document publishes (`<command>.input` without the fields the route carries in its path); a command
whose bound route publishes no body has no `.input` root under the binding (selected by name, it is
refused naming the command and the reason; `--all-commands` omits it). Without `--http` every
generated output keeps its bytes. The red test the finding names was never committed; the unit writes
it (`a_bound_command_input_decodes_the_request_body_the_bound_projection_publishes`).

## Acceptance

- With `--http` and a route carrying `note_id` in its path, `demo.notes.Create.input` decodes the
  request body the bound OpenAPI document publishes (`{"isPinned":true,"title":"Buy milk"}`) and the
  generated schema equals the bound `{command}.Input` schema, in Rust, Go and TypeScript.
- A command whose bound route publishes no body: `--root <command>.input` is refused by name;
  `--all-commands` omits that root.
- Without `--http`, `the_types_match_the_http_projection` and every existing `generate types` test
  keep their bytes.

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D343 on 45a1beba7. Every line is cited or inferred.

- **Primary surface:** `crates/generate/ess-gen/src/model_types.rs` — cited. `body_root` (:299-352; the `INPUT_ROOT` arm calls `Message::of_command` at :309, the finding's :313 has drifted), root constants (:17-21), `select` (:70-95), `command_roots` (:162-179), `select_root` (:215-235).
- **CLI:** `crates/edge/ess-cli/src/model_types.rs:9-110` (`ess generate types` args and `run`; no binding input today) — cited.
- **Binding seam:** `ess_gen::http::carried` (`ess-gen/src/http.rs:795`), `Binding::route` (`http.rs:459`); the bound body is `openapi.rs:2216-2229` (`command_schemas`, filtered by `carried` at :718-728) — cited. `crates/edge/ess-cli/src/http.rs:60-150` (`load`, `Binding::new`) is reusable for `--http` — inferred.
- **Must not change:** `Message::of_command` (`types.rs:1731`), read by `schema.rs:255` and `openapi.rs:2227`; editing it would move `generated/schema/commands/*.schema.json` and every OpenAPI document — cited.
- **Also likely:** a separate `ModelTypes` entry point taking `Option<&Binding>` keeps the ~20 two-argument `select` call sites unchanged (`ess-cli/src/client.rs:65`, `normalize.rs:112`, tests) — inferred; `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` if a new pub fn needs an entry — inferred; `website/docs/reference/cli.md` — inferred; `CHANGELOG.md` — cited.
- **Fixture:** `ess-gen/tests/fixtures/command-view-roots.yaml` is pinned by digest tables elsewhere; the test builds the binding in code or adds a new fixture outside the walkers — cited.
- **Tests:** `ess-gen` `command_view_roots`, `model_types`, `openapi_http`, `agreement`; `ess-cli` `model_type_events`, `model_types`, `http`; `schema-contract` `rust_realization`, `typescript_typecheck` — inferred.
- **Confidence:** medium; the design decision (how a binding reaches `generate types`) is taken in `## Decisions`.
- **Would collide with:** a unit touching `ModelTypes`/`body_root` or the `generate types` command.

## Delivered

D343, 2026-11-01: unit 7978d1b5d (merged f9f63ab57), adversary cases 9dbe58275. Scope check: ess-cli
`http.rs` `load` reused unedited; `entry-classifications.json` needed no entry (wrong in Scope);
`cli.md` regenerated; two-argument `select` call sites unchanged. Adversary pass 1: no defect; F2,
F3 coverage closed by its cases; F1 a note; F4 became `story:bound-response-root-follows-the-published-result`.
