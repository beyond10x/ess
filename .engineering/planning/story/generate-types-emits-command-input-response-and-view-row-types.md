---
format: aep.planning-md/3
id: story:generate-types-emits-command-input-response-and-view-row-types
kind: story
status: draft
title: Generate types emits command input response and view row types
summary: 'Adopter report N1: No generated type for a command input, response or view row (ess#656).'
refs:
- provider: github
  reference: beyond10x/ess#656
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ess generate types --root <command|view>` answers `unknown_root`: only event and model types are roots (`crates/generate/ess-gen/src/model_types.rs`). HTTP request and response bodies cannot be generated, so adopters keep them by hand. Adopter report N1, ess 0.68.0; the reporter's first priority.

## Acceptance

- `a_command_input_is_a_root`: `--root <Command>.input` generates the input type in Rust, Go and TypeScript.
- `a_command_response_is_a_root`: `--root <Command>.response` generates the response type.
- `a_view_row_is_a_root`: `--root <View>.row` generates the row type.
- `all_switches_include_them`: `--all-commands` and `--all-views` generate every such type.
- `the_types_match_the_http_projection`: each generated type decodes the example the OpenAPI projection publishes for the same body, and refuses a body that projection refuses.
