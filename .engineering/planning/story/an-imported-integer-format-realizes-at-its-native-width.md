---
format: aep.planning-md/3
id: story:an-imported-integer-format-realizes-at-its-native-width
kind: story
status: draft
title: An imported integer format realizes at its native width
summary: 'Adopter report R10: An imported integer format does not realize at native width (ess#648).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#648
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

`story:feature-request-394` realizes a model integer at a native width when an invariant states both bounds, and keeps bundle input (imported JSON Schema or OpenAPI) unchanged. So an imported `{type: integer, format: int64}` generates `serde_json::Number` with a runtime `integer` obligation where the protocol's own SDK has `i64`. Adopter report R10, ess 0.68.0 (15 fields in one catalog).

## Acceptance

- `an_imported_int64_realizes_as_i64`: an imported integer with `format: int64` generates Rust `i64` and Go `int64`; `format: int32` generates `i32` / `int32`.
- `an_imported_integer_without_format_keeps_the_exact_number_type`: no format keeps `serde_json::Number` / `EssNumber` and its obligation.
- `the_format_range_is_published`: the projected JSON Schema carries the range implied by the format, and a value outside it is refused by a plain validator and by the generated decoder.
- `model_input_widths_are_unchanged`: model integers keep the #394 inference byte for byte.
