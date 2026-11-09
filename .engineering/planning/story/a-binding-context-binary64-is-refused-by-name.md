---
format: aep.planning-md/3
id: story:a-binding-context-binary64-is-refused-by-name
kind: story
status: draft
title: A binding-context Binary64 is refused by name
summary: A binding's delivery-context field typed Binary64 directly is not walked by owner_uses, so it may be admitted where ESS-AUTHOR-036 should refuse it
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:ess-d335-wave-adversary-u3
- decomposes: epic:zero-open-issues-2026-11
revision: 3
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

- `ess verify conform synthesize` on the direct model above refuses as the declared-type model
  does: exit non-zero, a refusal naming `binding.received` (or the equivalent position path) and
  the field `scale`, with `ESS-AUTHOR-036`'s reason; no suite is written.
- `binary64::locations`/`uses` list the direct binding position (a unit test in
  `crates/specify/ess-compiler` with a binding whose context field is `Binary64`, `List<Binary64>`
  and `Optional<Binary64>`).
- A binding mapping value is not a field declaration and stays out of scope; a model with no
  `Binary64` in a binding keeps its suite bytes (an existing suite-bytes test stays green).

## Scope

- cited: `crates/specify/ess-compiler/src/binary64.rs` (`owner_uses`, `uncarried_uses`)
- cited: `crates/verify/ess-conformance/src/synthesize.rs` (refusal at `:3385`, unchanged)
- inferred: `crates/verify/ess-conformance/tests/adversary_d335_u3_pass1.rs` or a new case beside it
- inferred: `crates/specify/ess-compiler/tests/` (unit test of the position inventory)

confidence: high

## Origin

Found by the wave D335 U3 adversary pass as an out-of-scope row
(`review-result:ess-d335-wave-adversary-u3`), read from the code; reproduced on 0.92.0.
