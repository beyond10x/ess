---
format: aep.planning-md/3
id: story:the-interpreter-answers-the-response-value-an-outcome-stores
kind: story
status: active
title: The interpreter answers the response value an outcome stores
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#498
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:stored-field-equals-returned-response-value
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/determined.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/response.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/interpreted_response_values.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:45Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:46Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The interpreted target cannot produce a typed response field the implementation mints: an outcome
whose payload reads `{response: pin}` is reported unsupported on 0.92.0,

```console
$ ess verify conform run --path payload --target interpreted --report-format 2
  unsupported example.order.Place/outcome/placed
      ESS-CF-TARGET: cannot expose invoking `example.order.Place`: an actual response field or its executable conversion is not interpreted yet
```

because the evaluator needs an actual response it does not have
(`crates/verify/ess-conformance/src/interpret/execute/determined.rs:414-427`). Once
`story:stored-field-equals-returned-response-value` admits `{response: f}` in `sets:`, the
interpreted target (the reference every synthesized suite is checked against) must mint such a
field once and use that one value for the reply, every payload field and every stored field that
read it.

## Acceptance

- On the reproduction of `story:stored-field-equals-returned-response-value` at the new format,
  `ess verify conform run --target interpreted --report-format 2` passes `example.order.Place/outcome/placed`,
  and the row read back holds the `pin` the reply carried (an ess-conformance test reading both).
- The payload-routed variant (`payload/` in the triage reproduction) passes the same scenario
  instead of reporting it unsupported.
- A response field with a declared conversion stays `NotInterpreted`, named as today.

## Scope

- cited: crates/verify/ess-conformance/src/interpret/execute/determined.rs
- inferred: crates/verify/ess-conformance/src/interpret/execute.rs
- inferred: crates/verify/ess-conformance/src/interpret/response.rs
- inferred: crates/verify/ess-conformance/tests/interpreted_response_values.rs
confidence: medium

## Fit review

1. **Need.** The reference target must answer what the model states: one minted value, returned
   and stored (#498). Requester's proposal (theirs): the construct only; the interpreter half is
   implied.
2. **Class.** Gap in the interpreter (a named `NotInterpreted`, not a silent wrong answer).
3. **Already expressible?** n/a: no model construct is added here.
4. **Fit.** The interpreter already mints `{generated: true}` and reads a minted payload field
   into `sets:` (`docs/design/minted-value-sources.md`); this extends the same mint to a typed
   response field, used once for every reader.
5. **Second adopter.** Any `returns: true` command whose response field an event payload reads
   (`run.txt` § 3), with or without the new `sets:` source.
6. **Cost.** No format; interpreter only.
7. **Alternatives.** Leave it unsupported: every scenario of such a command is unsupported, so
   the construct is unwitnessed. Chosen: mint once.

## Decisions

Accept, redesigned (interpreter part of `story:stored-field-equals-returned-response-value`).

## Source

github:beyond10x/ess#498
