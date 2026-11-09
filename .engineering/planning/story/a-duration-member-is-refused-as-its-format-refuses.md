---
format: aep.planning-md/3
id: story:a-duration-member-is-refused-as-its-format-refuses
kind: story
status: draft
title: A Duration member is refused as its served format refuses it
summary: 'Rust reads Duration as any text though format: duration refuses it (W1 GEN1 review, pre-existing, backlog)'
tags:
- backlog
- defect
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:adversary-w1-gen1-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

The generated Rust decoder reads `Duration` with `text_at` (`crates/generate/ess-synth/src/rust/wire.rs`,
`decode_primitive`, around line 1016, pre-existing) and admits any string, though the served
OpenAPI document declares the member `type: string, format: duration`. Found by
review-result:adversary-w1-gen1-pass-1 from the code; confirmed on ess 0.92.0 by generating code.

Minimal reproduction (ess 0.92.0, no build): a command input `{name: span, type: Duration}` on a
network component.

```yaml
format: ess/22
system: demo
version: v1
domain: demo.timer
types:
  - {name: demo.timer.TimerId, kind: newtype, of: String}
entities:
  - name: demo.timer.Timer
    identity: {name: timer_id, type: demo.timer.TimerId}
    fields:
      - {name: span, type: Duration}
    lifecycle: {initial: Set, states: [Set], terminal: [Set]}
events:
  - name: demo.timer.TimerSet
    fields: [{name: timer_id, type: demo.timer.TimerId}]
actors:
  - name: demo.timer.Operator
    may: [demo.timer.SetTimer]
commands:
  - name: demo.timer.SetTimer
    input:
      - {name: timer_id, type: demo.timer.TimerId}
      - {name: span, type: Duration}
    outcomes:
      - name: set
        creates: demo.timer.Timer
        instance: timer_id
        sets: {span: input.span}
        emits: [demo.timer.TimerSet]
        payload: {demo.timer.TimerSet: {timer_id: input.timer_id}}
views:
  - name: demo.timer.Timers
    source: demo.timer.Timer
    consistency: read_your_writes
    fields:
      - {name: timer_id, type: demo.timer.TimerId}
      - {name: span, type: Duration}
components:
  - component: timers
    owns: {domains: [demo.timer]}
    accepts: {commands: [demo.timer.SetTimer]}
    publishes: {events: [demo.timer.TimerSet]}
    reached_by: network
```

`ess generate synthesize --target rust` writes `"span": {"type": "string", "format": "duration"}`
into `crates/demo-server/src/timers.openapi.json` and decodes it in `crates/demo-server/src/wire.rs`
as `Duration(json::text_at(member1, &at1, "an ISO 8601 duration")?.to_owned())`; `text_at` checks
only that the value is a JSON string. `--target go` does the same (`textAt` then
`primitives.NewDuration`, which stores the text).

The reference admission table agrees with the decoders, not with the served schema:
`crates/specify/ess-domain/src/types.rs`, `Primitive::admits` (around line 2879), admits any text
as a `Duration`, and `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` only
vectors `PT1S` (admitted) against `false` (refused). ESS's own duration grammar is narrower
(`ess-primitives` `time.rs` `parse_elapsed_seconds`, `expr.rs` `parse_elapsed`: `PT…` spellings).

## Acceptance

- One `Duration` grammar is chosen and written down; the admission table (`Primitive::admits`),
  the shared vectors, the Rust and Go decoders and the served schema agree on it.
- `primitive-semantics.json` gains `duration` vectors for a non-duration text (`"abc"`, refused)
  and the boundary spellings the grammar decides; the Rust, Go and browser corpora answer them.
- The generated Rust server, from the model above, answers `POST` of `{"timer_id": "t1", "span": "abc"}`
  with 400 naming the path `span`, and admits `{"timer_id": "t1", "span": "PT1S"}`; a test checks
  the same values against the served `openapi.json`.

## Scope

- cited: crates/generate/ess-synth/src/rust/wire.rs
- cited: crates/specify/ess-domain/src/types.rs
- cited: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- inferred: crates/generate/ess-synth/src/go/ (the Go decoder of `Duration`)
- inferred: crates/specify/ess-primitives/src/time.rs (the shared grammar)

confidence: medium

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
Related: story:an-admitted-timestamp-is-an-instant-the-server-can-order (the same decoder/grammar
question for `Timestamp`).
