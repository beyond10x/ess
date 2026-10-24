---
format: aep.planning-md/3
id: story:generated-dispatch-and-handle-return-the-emitted-events
kind: story
status: draft
title: Generated dispatch and handle return the events their command emitted
summary: 'Adopter report row 26: generated dispatch and handle discard the system log.'
tags:
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The generated Rust `dispatch` and `handle` run the command on the system, pump the system's event log, and then discard it (adopter report, row 26, ess 0.67.0: `router.rs:602-606` and `:733` in the generated server). An adapter calling them cannot see the events the command emitted, so it rebuilds its own frames (cancel frames in the report) from the response instead of from the events. The generator is `crates/generate/ess-synth/src/rust/http.rs:836` (`dispatch`) and `:1022` (`handle`), with the bound variant in `rust/http/bound.rs:608`.

## Acceptance

- `generated_dispatch_returns_the_events_its_command_emitted`: the generated `dispatch` hands the caller the events the command appended to the system log, in log order, beside the HTTP answer.
- `generated_handle_returns_the_events_its_command_emitted`: the same for `handle`.
- `a_refused_command_returns_no_events`: a refusal answers with an empty event list.
- `the_existing_answer_bytes_are_unchanged`: the response status and body a caller already reads keep their bytes.
