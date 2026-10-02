---
format: aep.planning-md/3
id: story:ui-tui-live-binding
kind: story
status: implemented
title: The TUI reads and commands a synthesized server (ess ui run --tui --model --base-url)
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:ui-binding-contract
- depends_on: story:ui-react-live-binding
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/ui.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/ui_run_live.rs
- confidence: cited
  path: crates/ui/ess-ui-tui
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-10-02T02:07:01Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":16}}}
---
## Outcome

The terminal renderer reads and commands a synthesized server: `ess-ui-tui` gains an HTTP
`DataAdapter`, `ess ui run --tui --model <spec> --base-url <url>` runs a document live, and a
refusal shows on the form, confirm or action that sent it (S3 of #311).

## Fit review

One request (#311), one review: see story:ui-binding-contract, `## Fit review`. This story is S3
of its five.

## Decisions

Accept, redesigned.

- `HttpAdapter` in `ess-ui-tui` over `std::net` HTTP/1.1, `http://` only (`https://` refused by
  name); `DataAdapter::run` returns `ess_ui::binding::Answer` from story:ui-binding-contract's
  `classify`.
- `ess ui run --tui --path --model --base-url …`: `<url>` when the document binds one component,
  else `<component>=<url>` repeatable; the credential from `ESS_UI_AUTHORIZATION`, never argv.
- A Refused answer shows where the user acted: on the open form overlay, on the confirm overlay,
  or beside the action row; the draft is kept.

## Acceptance

`crates/ui/ess-ui-tui/tests/http_adapter.rs` (against the Rust gatepass server from
`examples/gatepass-realization`, started on `127.0.0.1:0`):
- `reads_and_commands_reach_the_gatepass_rust_server` (register as Receptionist accepted;
  zero length refused 422; as SecurityAuditor not granted; a wrong-state sign-out refused 409)
- `a_refusal_shows_on_the_open_form`, `a_refusal_shows_on_the_confirm`,
  `a_refusal_shows_beside_the_action` (headless screen)

`crates/edge/ess-cli/tests/ui_run_live.rs`: `base_url_without_model_is_refused`,
`an_unnamed_base_url_with_two_served_components_is_refused`, `https_is_refused_by_name`.

## Scope

`crates/ui/ess-ui-tui/src/{data.rs, app.rs, lib.rs, http.rs (new)}`,
`crates/ui/ess-ui-tui/tests/http_adapter.rs` (new), `crates/edge/ess-cli/src/ui.rs`,
`crates/edge/ess-cli/tests/ui_run_live.rs` (new).

## Sequencing

After story:ui-binding-contract and story:ui-react-live-binding (the edge orders the shared
`ess-cli/src/ui.rs`). The Go server runs against the TUI in uilab's example, after
story:served-store-and-entry. `CHANGELOG.md` is a merge-time edit (epic).

## Accepted live-binding delivery reconciliation, 2026-10-02

## Accepted live-binding delivery reconciliation, 2026-10-02

Independent source/acceptance audit by scope_boolean found every enumerated acceptance item complete for ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params. The accepted redesign uses model-derived routes, explicit authorization and base URLs, polling or named no_live refusal, and HTTP-only TUI support. No SSE/WebSocket transport or private adopter replay is claimed.

Retained actual execution: React live_binding11, TUI http_adapter5, generated TUI3, CLI live-run3, served-view-params8 passed with no failed/ignored cases. Their adversarial suites also passed. Full audit identifies each assertion and source/log location in ess-backlog-served-entry-20261002/target/backlog-input/311-acceptance-reconciliation.md; its own new execution count is0. The current PR387 full Gate atad45061626 passed all workspace tests and merged as1ff305685 with the identical tree. These stories are delivered on main; a release after0.51.0 is still pending.
