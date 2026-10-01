---
format: aep.planning-md/3
id: story:ui-binding-contract
kind: story
status: active
title: An ess-ui document binds to the served surface through a route table from the model
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- supersedes: story:feature-request-311
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/generate/ess-gen/src/http.rs
- confidence: cited
  path: crates/ui/ess-ui-check/Cargo.toml
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: cited
  path: crates/ui/ess-ui-check/tests/binding.rs
- confidence: cited
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: cited
  path: crates/ui/ess-ui/src/binding.rs
- confidence: cited
  path: crates/ui/ess-ui/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui/tests/answers.rs
- confidence: cited
  path: crates/ui/ess-ui/tests/vectors/answers.json
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:50Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":7}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:50Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":7}}}
---
## Outcome

A binding contract between an `ess-ui/1` document and the HTTP surface ESS synthesizes: a route
table computed from the model through `ess_gen::http::routes`, one typed classification of command
answers with recorded vectors, and `ess ui check --model` errors for view parameters a read binds
wrongly (S1 of five from #311).

## Fit review

Read at `origin/main` = `8700d0808` (0.50.0 release merge; only `Cargo.toml` differs from `f86180f30` among the cited files). ESS already holds `.engineering/planning/story/feature-request-311.md` ("Fit review: Pending"). Read from code; no runtime observation.

1. **Need.** One `ess-ui/1` document should run in both renderers against the HTTP surface ESS synthesizes for a served component: live reads, sent commands, a typed refusal shown where the user acted. Today: the React `httpAdapter` calls `GET {base}/views/<dotted>?params=<json>` and `POST {base}/commands/<dotted>` (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:157,161,169`); servers answer only the paths `ess_gen::http::routes` derives, `/{domain wire}/views|commands/{wire}` (`crates/generate/ess-gen/src/http.rs:338-424`), anything else is a 404 (`ess-synth/src/go/http.rs:1106-1107`); `answer()` throws away every non-2xx body (`data.ts.tmpl:146-151`) and `runCommand` shows a toast with the status text (`actions.tsx.tmpl:57-58`); the TUI has only `FixtureAdapter`, which accepts every command (`ess-ui-tui/src/data.rs:241-253`, `app.rs:236-240`). Reproduction: `examples/gatepass` and a one-section `ui.yaml` reading `gatepass.visit.ExpectedVisits` with an action `does: gatepass.visit.RegisterVisit`: the adapter asks for `/views/gatepass.visit.ExpectedVisits?params=%7B%7D`, the server serves `/visits/views/expected` (`examples/gatepass/domains/visit.yaml:6,291`); both 404. The requester proposed `ess ui run --tui --endpoint <url>`, a base URL for the React build, routes from the same derivation, and `ess generate ui --target tui`.
2. **Class.** A gap with two defects: A (refusals) `website/docs/reference/ess-ui.md:946` promises a refusal shown on the form as the command's typed error; React only toasts (`composites/form.tsx.tmpl:69`, `actions.tsx.tmpl:57-58`), the TUI only notifies (`app.rs:2321-2325`). B (view parameters) OpenAPI publishes view `params:` as query parameters (`ess-gen/src/openapi.rs:420-425,1648-1687`) while both servers drop the query string (`ess-synth/src/rust/http.rs:1289-1294`, `:1184`; `go/http.rs:1094-1098`); planning keeps a view with parameters as an obligation whose port can never see them (`ess-synth/src/view_query.rs:31-35`); `openapi.rs:560-564` is stale.
3. **Already expressible?** Partly: `setDataAdapter(myAdapter)` (`data.ts.tmpl:197`), `App::with_adapter(…)` (`app.rs:243-250`); both make every adopter copy the route derivation, which `http.rs:3-12` exists to stop. `ess ui run` cannot take an adapter (`ess-cli/src/ui.rs:85-91`). Channels have an idiom: the renderer profile capability `no_live: {fallbacks: [poll, refuse]}` (`ess-ui.md:1585`).
4. **Fit.** `endpoint` already means "traceability to the HTTP call replaced" (`ess-ui/src/model.rs:2070-2071`, `ess-ui.md:375,965`): the flag is `--base-url`. A domain has one owning component which alone accepts its commands (`ess-domain/src/component.rs:1357,1388`): base URLs are per component. View routes exist only for `reached_by: network` (`http.rs:364`). `ess-ui-check` already compiles the model and qualifies dotted names (`ess-ui-check/src/model.rs:76-110,169`) but does not check `reads.params` against the view's params. The React generator has no model input (`ess-ui-react/src/lib.rs:130-139`). Grants (0.49): command routes call `admit` (`go/http.rs:1079-1086`), 403 body `{"refused":"not granted","actor":…}` (`http.rs:101-109`), the realization's `authenticate` decides the caller (`go/http.rs:958-971`, `rust/http.rs:613-636`, `http.rs:95-97`), views are not grant-checked (`synthesize.md:279`); the examples' demonstration header is `Authorization: Actor <name>` (`examples/gatepass-go-realization/cmd/gatepass-server/main.go:27-35`). Answer shapes: declared outcome `outcome`, `published`, `error`/`payload` (`go/http.rs:1322-1349`); surface refusal `refused`, plus `actor` on 403 and `committed` on 501 (`go/http.rs:1675-1689`); classify by body members, not status alone (`http.rs:95-98`). No SSE/WebSocket server (no hits in `ess-synth/src`). No CORS headers (no hits); the Go `Serve` builds its own handler and `dispatch…` is unexported (`go/http.rs:950-956,1049`). Paging refused by every code target (`ess-synth/src/paging.rs:1-8,18-40`). `/state/...` served by no synthesized server (`data.ts.tmpl:176-188`). The Playwright spec is hard-wired to `/views/<dotted>` and `/commands/**` (`ess-ui-test/src/playwright.rs:8,115-118,167-172`; `tests/playwright.rs:99-135`).
5. **Second adopter.** Two already: the issue (an operator console, one `ui.yaml` for terminal and web) and uilab. Brand-free third: an on-call console for a job scheduler, terminal for operators and web for support.
6. **Cost.** No `ess-ui/1` format bump, no spec keyword, no persisted artifact. New flags `--model`, `--base-url` on `ess generate ui` and `ess ui run`, and `--target tui`. New `ess ui check --model` errors for read parameters. Breaking generated APIs: React `httpAdapter(base)` signature; TUI `DataAdapter::run` return type (`data.rs:54`; `publish = false`, `Cargo.toml:57`); Playwright routes; the view port signature for views that declare params (others byte-identical). New dependency edge `ess-ui-check` → `ess-gen`.
7. **Designs.** Change nothing: rejected. Serve the adapter's `/views/<dotted>` shape: rejected (a second path per construct). Runtime discovery from `/openapi.json`: rejected as primary (still needs each component URL, an OpenAPI reader in TS and Rust, drift only at run time); later milestone as a startup self-check. **Chosen: a route table built from the model via `ess_gen::http::routes`, plus a typed answer classification.**


## Decisions

Accept, redesigned. The request is split into five stories (recorded in epic:ui-live-apps); this
story is S1 only.

- `ess_ui::binding` (new, no dependencies, BTreeMap/Vec, serialisable to JSON for the React
  generator): `Binding { system, components: BTreeMap<String, ServedComponent> }`,
  `ServedComponent { views: BTreeMap<qualified, ViewRoute>, commands: BTreeMap<qualified, CommandRoute> }`,
  `ViewRoute { path, params: Vec<QueryParam{name, wire, required, scalar}> }`,
  `CommandRoute { path, body_required, errors: BTreeMap<wire_code, ErrorRoute{status, display}> }`,
  `names: BTreeMap<as_written, qualified>`.
- Built by `ess_ui_check::binding(document, sources)` from `http::routes`, `http::status` and
  `http::body_required`, covering only what the document names. Refused by node path: a name no
  `reached_by: network` component serves; a non-scalar view parameter; a read with server paging (`paging:` other than client-side), since every code target refuses paging (`ess-synth/src/paging.rs:1-8,18-40`); state placed in `server` or
  `server_session`.
- `ess ui check --model` (exists, `ess-cli/src/ui.rs:17,72`) adds two errors: a read binding an undeclared view parameter; a required
  view parameter left unbound.
- One classifier, owned here: `ess_ui::binding::classify(status, body) -> Answer`, variants
  Accepted, Refused{error, payload}, NotGranted{actor}, Malformed{refused}, Unfinished{committed},
  Transport, classified by body members as the fit review states. Its cases are data:
  `crates/ui/ess-ui/tests/vectors/answers.json` (status, body, expected answer, source server and
  command), recorded from the gatepass Rust and Go servers; an answer neither server produces (501 with and without `committed`) is a case marked `source: constructed` and copied from the shape at `go/http.rs:1675-1689`. story:ui-react-live-binding runs the
  same file against its TypeScript port; story:ui-tui-live-binding calls the Rust function.
- Caller: an opaque `Authorization` value from configuration, never derived. Channels: no server
  stream (the document's `poll` fallback, or refuse). Cross-origin: none; story:served-store-and-entry
  serves the generated web app from the same origin.

## Acceptance

- `crates/ui/ess-ui-check/tests/binding.rs`: `a_view_and_a_command_bind_to_the_paths_the_served_surface_answers`
  (gatepass: `/visits/views/expected`, `/visits/commands/register-visit`, equal to `http::routes`),
  `a_name_no_network_component_serves_is_refused_by_node_path`,
  `contested_paths_bind_to_the_qualified_fallback`, `a_non_scalar_view_param_is_refused`,
  `server_held_state_is_refused_for_a_live_binding`, `a_server_paged_read_is_refused_by_name`, `the_binding_serialises_to_stable_json`.
- `crates/ui/ess-ui-check/tests/checks.rs`: `a_read_binding_an_undeclared_view_param_is_an_error`,
  `an_unbound_required_view_param_is_an_error`.
- `crates/ui/ess-ui/tests/answers.rs`: `every_answer_vector_classifies_as_recorded` (202, 422, 409,
  404, 403 not granted, 403 declared, 400 declared, 400 refused, 501 committed and not, 405),
  `every_vector_names_its_source_server_and_command`.

## Scope

`crates/ui/ess-ui/src/{binding.rs (new), lib.rs}`, `crates/ui/ess-ui/tests/{answers.rs, vectors/answers.json}`
(new), `crates/ui/ess-ui-check/src/{model.rs, lib.rs}`, `crates/ui/ess-ui-check/Cargo.toml`
(+`ess-gen`), `Cargo.lock`, `crates/generate/ess-gen/src/http.rs` (read only; at most `pub`
helpers). `ess-cli` is not touched.

## Sequencing

Order 1, parallel to story:ui-react-plain: no shared file (that story touches neither `ess-ui`,
`ess-ui-check` nor `ess ui check`). #284, #300, #303 and #305 also edit `ess-ui-check`
`model.rs`/`rules.rs`; none is in an open unit, so whoever lands second rebases. `CHANGELOG.md`
is a merge-time edit (epic).
