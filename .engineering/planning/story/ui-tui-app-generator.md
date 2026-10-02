---
format: aep.planning-md/3
id: story:ui-tui-app-generator
kind: story
status: active
title: ess generate ui --target tui emits a Rust terminal app crate
refs:
- provider: github
  reference: beyond10x/ess#311
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:ui-tui-live-binding
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: crates/edge/ess-cli/src/ui.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/generate_ui_tui.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/generate.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/lib.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:51Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A generated Rust terminal application for a document: `ess generate ui --target tui --path ui.yaml
--model <spec> --out <dir>` emits a crate with a clap-derived command line that runs the document
against a base URL (S4 of #311).

## Fit review

One request (#311), one review: see story:ui-binding-contract, `## Fit review`. This story is S4
of its five.

## Decisions

Accept, redesigned. The generator lives in `ess-ui-tui` (`src/generate.rs`, new), so the
workspace gains no crate. The generated crate: `src/main.rs` (clap derive, `--base-url`,
credential from `ESS_UI_AUTHORIZATION`), the document via `include_str!`, `src/binding.rs`;
it depends on `ess-ui-tui` by git tag equal to the generating ESS version (the workspace is
`publish = false`); no renderer code in it.

## Acceptance

`crates/edge/ess-cli/tests/generate_ui_tui.rs`:
- `the_generated_crate_builds_and_its_help_names_base_url` (`[patch]` to the workspace)
- `generation_is_deterministic`
- `the_built_crate_reads_a_view_from_a_synthesized_gatepass_server` (runs the built binary
  headless against the Rust gatepass server on `127.0.0.1:0` and finds an `ExpectedVisits` row on
  screen)

## Scope

`crates/ui/ess-ui-tui/src/generate.rs` (new), `crates/ui/ess-ui-tui/src/lib.rs`,
`crates/edge/ess-cli/src/ui.rs` (`--target tui`), `crates/edge/ess-cli/tests/generate_ui_tui.rs`
(new).

## Sequencing

After story:ui-tui-live-binding (same crate, same CLI file). `CHANGELOG.md` is a merge-time edit
(epic).
