---
format: aep.planning-md/3
id: epic:ui-live-apps
kind: epic
status: active
title: One model and one ess-ui document produce running apps with nothing hand-written
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:49Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:49Z", actor: "human:timo", revision: 6}
---
## Outcome

One ESS model and one `ess-ui/1` document produce running applications with nothing hand-written:
a server synthesized in Go (or Rust) with its store and entry point, and two generated user
interfaces bound to that server by the model, a plain React web app (react and react-dom only) and
a Rust terminal app. A refusal the model declares reaches the user where they acted.

## Origin

beyond10x/uilab builds `examples/todo-app` (a todo list whose unique feature is "blocked by": a
task names the task blocking it, and completing it is refused until the blocker is done) as the
showcase of what ESS UI can do (operator, 2026-10-01: "uilab just is a UI/lab for showing off what
the ess ui spec can do"). Requests: beyond10x/ess #304, #311, #314, #315, #318, #319. Operator
decisions, 2026-10-01: ESS gains these features itself (nothing hand-written in the example); a
served component gets a generated in-memory store and entry point, reversing the 2026-09-29
decision "storage is a port the implementor provides" for that case; commands guarded by
`when_related` are generated in the code targets.

## Stories and order

Order follows the `depends_on` edges; a row's order is one more than its latest dependency's.

| order | story | issue | depends on |
|---|---|---|---|
| 1 | story:ui-react-plain | #315 | — |
| 1 | story:ui-binding-contract | #311 | — |
| 2 | story:ui-react-live-binding | #311 | ui-react-plain, ui-binding-contract |
| 3 | story:ui-tui-live-binding | #311 | ui-binding-contract, ui-react-live-binding (`ess-cli/src/ui.rs`) |
| 4 | story:ui-tui-app-generator | #311 | ui-tui-live-binding |
| 1 | story:go-generated-behaviour | #314 | #310 (0.51.0) |
| 2 | story:served-view-params | #311 | #310, go-generated-behaviour |
| 3 | story:served-store-and-entry | #318 | go-generated-behaviour, served-view-params (`{go,rust}/http.rs`) |
| 1 | story:related-via-optional-input | #304 | #287 (ships in the ess/21 bundle; not an edge) |
| 2 | story:related-via-stored-reference | #304 | #282, related-via-optional-input (#287 through it) |
| 4 | story:related-guard-behaviour | #319 | #310, go-generated-behaviour, served-store-and-entry, related-via-optional-input, related-via-stored-reference |

The UI chain, the Go chain and the ess/21 chain run as parallel units and meet at story:related-guard-behaviour. One file is shared across chains without an edge: `ess-gen/src/openapi.rs` (served-view-params `:560-564`, related-via-optional-input `:1155`); whichever lands second rebases. Outside the epic, `aep plan artifact waves` also orders three draft stories against it: story:native-realization-ci (`ci.yml`, with ui-react-plain), story:a-refusal-records-the-document-it-was-read-from (`ess-compiler/src/resolve.rs`) and story:the-interpreter-executes-stored-field-guards (`interpret/execute.rs`, both with related-via-optional-input); whichever lands second rebases.

## Excluded

- Generating queries for views with parameters: story:served-view-params passes the parameters to
  the view port; the query stays an obligation in every target.
- Server paging, server streams (SSE/WebSocket), server-held UI state and CORS headers: refused by
  name or answered by an idiom (story:ui-binding-contract); the generated web app is served from the
  server's origin (story:served-store-and-entry `--static`).
- A durable generated store.
- The live-channel asks on #311 (comment 5939511183: reset as refetch, a bearer on the stream,
  `per` as a query parameter, dedupe by event id): they follow server streams out of this epic and
  need their own fit review. Its asks (a) base URL and (c) caller on reads and commands are in
  story:ui-binding-contract and its renderer stories.
- Renderer defects #320–#326: not in this epic.

## Merge-time edits

`CHANGELOG.md` `[Unreleased]` is edited once per story at merge into the integration branch, by
the coordinator, never inside a unit.

## Coordination

The ess session (gaps waves) owns #282, #287, #310 and the ess/21 bundle. It handed over the
stories `feature-request-304` and `feature-request-311`; when this epic's stories are accepted they
supersede those two. Fit reviews were run per `.agents/skills/assessing-external-requests` and are
in each story. The Go and served stories base on 0.51.0, which carries #310.
