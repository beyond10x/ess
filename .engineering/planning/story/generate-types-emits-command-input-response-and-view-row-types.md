---
format: aep.planning-md/3
id: story:generate-types-emits-command-input-response-and-view-row-types
kind: story
status: active
title: Generate types emits command input response and view row types
summary: 'Adopter report N1: No generated type for a command input, response or view row (ess#656).'
refs:
- provider: github
  reference: beyond10x/ess#656
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T09:23:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-23T09:24:00Z", actor: "human:timo", revision: 3}
---
## Problem

`ess generate types --root <command|view>` answers `unknown_root`: only event and model types are roots (`crates/generate/ess-gen/src/model_types.rs`). HTTP request and response bodies cannot be generated, so adopters keep them by hand. Adopter report N1, ess 0.68.0; the reporter's first priority.

## Acceptance

- `a_command_input_is_a_root`: `--root <Command>.input` generates the input type in Rust, Go and TypeScript.
- `a_command_response_is_a_root`: `--root <Command>.response` generates the response type.
- `a_view_row_is_a_root`: `--root <View>.row` generates the row type.
- `all_switches_include_them`: `--all-commands` and `--all-views` generate every such type.
- `the_types_match_the_http_projection`: each generated type decodes the example the OpenAPI projection publishes for the same body, and refuses a body that projection refuses. Exception: under an `ess-http/1` route with path parameters, the command input type keeps the fields the route carries in its path, as the 0.69.0 CHANGELOG states, so it does not decode that route's published request body. Decoding it is story:a-bound-command-input-decodes-the-published-request-body (0.70.0).

## Decisions

- 2026-10-23: acceptance narrowed to what 0.69.0 ships, after the adversary pass on G-U15 found the bound-input gap. Approved by session b10x-chef-orchestrator under its grant: "D1 kept: G-U15 ships in 0.69.0 with the bound-input gap, the fix is in 0.70.0".
