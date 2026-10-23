---
format: aep.planning-md/3
id: story:a-struct-that-ignores-undeclared-fields-may-declare-none
kind: story
status: draft
title: A struct that ignores undeclared fields may declare none
summary: 'Adopter report R11: A struct ignoring undeclared fields cannot declare none (ess#649).'
refs:
- provider: github
  reference: beyond10x/ess#649
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

ESS-TYPE-007 refuses a struct with `fields: []` because "no value can be one" (`crates/specify/ess-domain/src/types.rs:1254`). That holds for a closed struct only: with `undeclared_fields: ignored` every JSON object is a value. Adopters model empty wire objects as a `newtype` of `Map<String, Json>` instead. Adopter report R11, ess 0.68.0.

## Acceptance

- `an_open_struct_with_no_fields_validates`: `fields: []` with `undeclared_fields: ignored` validates at the new source format and projects as `{type: object}`.
- `a_closed_struct_with_no_fields_is_still_refused`: `fields: []` on a closed struct keeps ESS-TYPE-007.
- `an_older_format_keeps_the_refusal`: the same open struct at the previous format is refused, naming the format that admits it.
- `the_generated_types_accept_any_object`: Rust, Go and TypeScript types for the struct decode `{}` and `{"x": 1}`.
