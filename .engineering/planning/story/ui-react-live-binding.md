---
format: aep.planning-md/3
id: story:ui-react-live-binding
kind: story
status: implemented
title: The generated React app reads and commands a synthesized server; refusals show where the user acted
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:ui-react-plain
- depends_on: story:ui-binding-contract
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/edge/ess-cli/src/ui.rs
- confidence: cited
  path: crates/ui/ess-ui-react
- confidence: cited
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: cited
  path: crates/ui/ess-ui-test/tests/playwright.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-02T13:47:18Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

The generated React app reads and commands a synthesized server through the binding contract:
`ess generate ui --target react --model <spec>` emits `src/binding.ts` and a binding-driven
`httpAdapter`, and a refusal shows on the form, confirm or action that sent it (S2 of #311).

## Fit review

One request (#311), one review: see story:ui-binding-contract, `## Fit review`. This story is S2
of its five.

## Decisions

Accept, redesigned.

- `ess generate ui --target react --path ui.yaml --model <spec>` emits `src/binding.ts` (the
  story:ui-binding-contract binding as data) and a binding-driven `httpAdapter`; `main.tsx`
  switches to it when a binding exists; without `--model` the output is byte-identical to
  story:ui-react-plain's.
- Base URL per component from `<meta name="ess-base-url:<component>" content="…">` in
  `www/index.html`; same origin when absent. `setAuthorization(value)` exported from
  `runtime/data.ts` (opaque, never derived).
- Command answers through a TypeScript port of `ess_ui::binding::classify`, held to
  `crates/ui/ess-ui/tests/vectors/answers.json`. Forms, confirms and actions render Refused with the
  error's display text and payload where the user acted; the draft is kept; an optimistic change is
  reverted; reads are invalidated on Accepted and on a 409 refusal.
- The Playwright emitter takes the binding's paths. Channels: no server stream; live sections fall
  back to `poll` or refuse.

## Acceptance

`crates/ui/ess-ui-react/tests/live_binding.rs`:
- `without_a_model_the_project_is_byte_identical`
- `binding_ts_holds_the_served_paths_and_query_wire_names`
- `the_bound_project_type_checks_offline`
- `the_typescript_classifier_agrees_with_every_answer_vector` (node over `answers.json`)
- `a_refused_command_renders_on_the_form_that_sent_it`, `…_on_the_confirm_that_sent_it`,
  `…_on_the_action_that_sent_it` (node with DOM stubs, as `adversary_pass2.rs`)
- `the_built_app_reads_and_commands_a_synthesized_gatepass_server` (builds the generated project,
  starts the Rust gatepass server from `examples/gatepass-realization`, drives the adapter over HTTP:
  `ExpectedVisits` read ready, `RegisterVisit` accepted, a zero-length visit refused with
  `InvalidVisitLength` on the form).

`crates/ui/ess-ui-test/tests/playwright.rs`: `commands_and_fixture_overrides_go_through_the_bound_routes`.

## Scope

`crates/ui/ess-ui-react/templates/runtime/{data.ts,actions.tsx}.tmpl`,
`templates/runtime/composites/{form,confirm}.tsx.tmpl`, `templates/project/{main.tsx,index.html,README.md}.tmpl`,
`crates/ui/ess-ui-react/src/{lib.rs,emit.rs}`, `crates/ui/ess-ui-react/Cargo.toml`,
`crates/ui/ess-ui-react/tests/live_binding.rs` (new), `crates/ui/ess-ui-test/src/playwright.rs`,
`crates/edge/ess-cli/src/ui.rs` (`--model` on `ess generate ui`), `Cargo.lock`.

## Sequencing

After story:ui-react-plain (shares `main.tsx`, `index.html`, README) and story:ui-binding-contract.
Before story:ui-tui-live-binding (both edit `ess-cli/src/ui.rs`). The server under test is the Rust
gatepass realization, which story:go-generated-behaviour does not touch. `CHANGELOG.md` is a
merge-time edit (epic).

## Accepted live-binding delivery reconciliation, 2026-10-02

Independent source/acceptance audit by scope_boolean found every enumerated acceptance item complete for ui-react-live-binding, ui-tui-live-binding, ui-tui-app-generator and served-view-params. The accepted redesign uses model-derived routes, explicit authorization and base URLs, polling or named no_live refusal, and HTTP-only TUI support. No SSE/WebSocket transport or private adopter replay is claimed.

Retained actual execution: React live_binding11, TUI http_adapter5, generated TUI3, CLI live-run3, served-view-params8 passed with no failed/ignored cases. Their adversarial suites also passed. Full audit identifies each assertion and source/log location in ess-backlog-served-entry-20261002/target/backlog-input/311-acceptance-reconciliation.md; its own new execution count is0. The current PR387 full Gate atad45061626 passed all workspace tests and merged as1ff305685 with the identical tree. These stories are delivered on main; a release after0.51.0 is still pending.
