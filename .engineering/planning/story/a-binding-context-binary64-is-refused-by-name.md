---
format: aep.planning-md/3
id: story:a-binding-context-binary64-is-refused-by-name
kind: story
status: active
title: A binding-context Binary64 is refused by name
summary: A binding's delivery-context field typed Binary64 directly is not walked by owner_uses, so it may be admitted where ESS-AUTHOR-036 should refuse it
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:ess-d335-wave-adversary-u3
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/binary64.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/delivery_context_ir.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_d335_u3_pass1.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:37:42Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:37:43Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Problem

A binding's delivery-context field typed `Binary64` directly, not through a declared type, is
admitted by conformance synthesis, while the same field typed through a declared type is refused.
The suite contract does not carry a `Binary64` in a binding (`ESS-AUTHOR-036`,
`website/docs/reference/diagnostics.md:168`; `CHANGELOG.md` 0.79.0: "A `Binary64` reached
through ... a binding ... is still refused").

Cause: `owner_uses` (`crates/specify/ess-compiler/src/binary64.rs:239-317`) walks entities,
commands, events, errors, views and actors, not bindings, so a direct `Binary64` in
`binding.context.fields` is never a position of `uses`/`reached_uses`. `uncarried_uses`
(`binary64.rs:61-83`) catches a binding position only through a declared type
(`types_reached_from`, `binary64.rs:136-138`), so the direct field is never listed and synthesis
(`crates/verify/ess-conformance/src/synthesize.rs:3385`) and admission
(`crates/verify/ess-conformance/src/admission.rs:818`) do not refuse it.

Minimal reproduction on 0.92.0 (`format: ess/18`, domain `demo.inbox`):

```yaml
bindings:
  - id: received
    when:
      event: demo.inbox.ReadingReceived
      context_authority: sensor-readings
      context_fields:
        - {name: scale, type: Binary64}          # direct
    invoke: {command: demo.inbox.RecordReading}   # input scale: Binary64
    mapping: {reading_id: event.reading_id, scale: context.scale}
    delivery: at_least_once
    on_failure: retry
```

```text
$ ess verify conform synthesize --path direct.yaml --out suite.json
6 scenario(s) (0 authored), 0 refusal(s), written to suite.json          # exit 0
                                   # deliver_event steps carry "context": {"scale": 1.5}
$ ess verify conform synthesize --path declared.yaml --out suite.json    # same model, scale: demo.inbox.Scale (newtype of Binary64)
error: types.demo.inbox.Scale.of: UnsupportedPrimitive: a Binary64 this route does not carry: ...   # exit 1
```

## Acceptance

- `ess verify conform synthesize --path direct.yaml --out suite.json` on the direct model above
  exits 1, writes no `suite.json`, and prints the declared-type model's line at the binding's
  position: `error: binding.received.context.fields.scale: UnsupportedPrimitive: a Binary64 this
  route does not carry: …` (`ESS-AUTHOR-036`'s reason, from `admission::model`).
- On the same model, `ess_conformance::admission::model` returns one issue at
  `binding.received.context.fields.scale`, and `ess_conformance::synthesize::synthesize` returns an
  empty suite and one refusal at that path. Test: a case in
  `crates/verify/ess-conformance/tests/adversary_d335_u3_pass1.rs`, run with
  `cargo test -p ess-conformance --locked --test adversary_d335_u3_pass1`.
- `binary64::uses` on a binding `received` whose context fields are `scale: Binary64`,
  `scales: List<Binary64>` and `maybe: Optional<Binary64>` holds exactly
  `binding.received.context.fields.scale`, `binding.received.context.fields.scales.of` and
  `binding.received.context.fields.maybe.of`, each owned by the binding `received`, and
  `binary64::uncarried_uses` lists the same three. Test: a case in
  `crates/specify/ess-compiler/tests/delivery_context_ir.rs`, run with
  `cargo test -p ess-compiler --locked --test delivery_context_ir`.
- A model with no `Binary64` in a binding keeps its suite bytes:
  `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no row of
  `synthesis-suite-bytes-table.tsv` changed.

## Decisions

- The position is spelled `binding.<name>.context.fields.<field>`, the `<kind>.<name>.<member>.<field>`
  form `owner_uses` already gives view parameters and actor attributes and `resolve.rs` gives
  `binding.<name>.mapping.<input>`, so the refusal names one fixed path.
- A binding mapping value is not a field declaration and stays out of scope.

## Scope

- cited: crates/specify/ess-compiler/src/binary64.rs (`owner_uses` gains the binding context fields; `uncarried_uses` gains the `binding.<name>.context.` prefix)
- cited: crates/verify/ess-conformance/src/synthesize.rs (refusal at `:3385`, unchanged)
- inferred: crates/verify/ess-conformance/tests/adversary_d335_u3_pass1.rs
- inferred: crates/specify/ess-compiler/tests/delivery_context_ir.rs

confidence: high

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no byte-pin row moves.

## Origin

Found by the wave D335 U3 adversary pass as an out-of-scope row
(`review-result:ess-d335-wave-adversary-u3`), read from the code; reproduced on 0.92.0.
