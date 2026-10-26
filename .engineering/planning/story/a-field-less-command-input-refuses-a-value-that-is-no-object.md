---
format: aep.planning-md/3
id: story:a-field-less-command-input-refuses-a-value-that-is-no-object
kind: story
status: draft
title: A field-less command input refuses a value that is no object
summary: The generated Rust decoder of a command input with no fields reads a string, null or array as the input; the field-less event decoder refuses one.
tags:
- ess-0.71.0
relations:
- derived_from: review-result:adversary-ess-0700-unit-c-pass-2
revision: 1
---
## Finding

Found 2026-10-26 by adversary pass 2 on the 0.70.0 wave's unit C
(`review-result:adversary-ess-0700-unit-c-pass-2`). The generated Rust decoder of a command whose
input declares no field reads any JSON value as that input: `"text"`, `null` and `[1,2]` each
decode, and `handle(name, input)` runs the command on one. The field-less event decoder beside it
refuses a value that is no object, at the payload's path, since 0.70.0. Whether the served `POST`
route reaches the same path was not run.

## Acceptance

- `decode_command_<name>` of a field-less input refuses `"text"`, `null` and an array at `body`,
  and reads `{}`; `handle` answers a refusal and publishes nothing for each of the three.
- A `POST` of a non-object body to the command's route answers `400` and publishes nothing.

## Repro

The red case `adversary_c2_a_fieldless_command_input_refuses_a_value_that_is_no_object` and its
fixture are in the adversary pass 2 test file as it was handed over (kept in the wave's scratch;
re-derive from `crates/generate/ess-synth/tests/adversary_c_event_wire_2.rs`, whose harness still
answers the `ping-text`, `ping-null` and `ping-handled` lines).
