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
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/json.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/duration_member_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-admission.js
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
revision: 9
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
only that the value is a JSON string. `--target go` does the same (`textAt` in
`crates/generate/ess-synth/src/go/http.rs` `decode_primitive`, then `primitives.NewDuration`,
which stores the text).

The reference admission table agrees with the decoders, not with the served schema:
`crates/specify/ess-domain/src/types.rs`, `Primitive::admits` (around line 2879), admits any text
as a `Duration`, and `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` only
vectors `PT1S` (admitted) against `false` (refused). The Go conformance runtime
(`crates/verify/ess-conformance/src/go/runtime.go`, `primitive`) and the browser adapter
(`crates/verify/ess-conformance/assets/coverage-admission.js`, `primitiveAdmits`) also admit any
string. ESS's own expression grammar is narrower (`ess-primitives` `time.rs`
`parse_elapsed_seconds`, `expr.rs` `parse_elapsed`: `PT…` spellings only).

## Decisions

- The admitted grammar is the RFC 3339 Appendix A `duration` production, upper-case designators,
  whole-number amounts: it is what the served `format: duration` names (JSON Schema 2020-12
  § 7.3.1), and the generated Go primitive documents `P30D` as its example, which the narrower
  `PT…` expression grammar refuses.
- The grammar is one function, `is_rfc3339_duration`, in `crates/specify/ess-primitives/src/facts.rs`
  beside `is_canonical_uuid`, for the reason that function gives there; `time.rs` and `expr.rs` keep
  the expression grammar unchanged.
- The served schema keeps `{"type": "string", "format": "duration"}` with no `pattern`: the format
  already names the grammar, and a pattern would move every generated schema's bytes for no
  change in meaning.

## Acceptance

- `cargo test -p ess-primitives --locked --test primitive_corpus` passes with
  `primitive-semantics.json` holding these `duration` vectors: `"abc"`, `"P"`, `"PT"`, `"P1DT"`,
  `"PT1.5S"`, `"-PT1S"` and `"pt1s"` refused; `"PT1S"`, `"P30D"`, `"P1W"` and `"P1Y2M3DT4H5M6S"`
  admitted; and `Primitive::admits(Duration, "abc")` returns `None`.
- `cargo test -p ess-conformance --locked --test primitive_corpus` passes on the same vectors: the
  Go runtime's `primitive("duration", …)` and the browser adapter's `primitiveAdmits('duration', …)`
  answer every one as the vector states.
- `cargo test -p ess-synth --locked --test duration_member_admission` generates the Rust server
  from the model above and asserts that `POST` of `{"timer_id": "t1", "span": "abc"}` answers 400
  with the path `span` in the refusal, and `POST` of `{"timer_id": "t1", "span": "PT1S"}` answers
  the `set` outcome.
- The same test generates the Go server from the model above and asserts the same two answers.
- The same test reads the `span` member schema from the served `timers.openapi.json` and asserts
  that a JSON Schema validator with format assertion on refuses `"abc"` and admits `"PT1S"`.

## Scope

- cited: crates/generate/ess-synth/src/rust/wire.rs
- cited: crates/generate/ess-synth/src/rust/json.rs
- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/specify/ess-domain/src/types.rs
- cited: crates/specify/ess-primitives/src/facts.rs
- cited: crates/specify/ess-primitives/tests/vectors/primitive-semantics.json
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/assets/coverage-admission.js
- inferred: crates/generate/ess-synth/tests/duration_member_admission.rs (new)

confidence: medium

Shared surfaces moved: re-pins the generated-code rows of the byte-pin targets (`ess-synth`
`served_codec_names` and `go`) that the new `duration_at`/`durationAt` decoder moves.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
Related: story:an-admitted-timestamp-is-an-instant-the-server-can-order (the same decoder/grammar
question for `Timestamp`).
