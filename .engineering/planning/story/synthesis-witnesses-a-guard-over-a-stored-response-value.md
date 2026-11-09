---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-guard-over-a-stored-response-value
kind: story
status: draft
title: Synthesis witnesses a guard over a stored response value
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#498
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:stored-field-equals-returned-response-value
- depends_on: story:the-interpreter-answers-the-response-value-an-outcome-stores
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/replay.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/responded_values.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/stored_response_values.rs
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 4
---
## Need

With `{response: pin}` in `sets:` (`story:stored-field-equals-returned-response-value`) and the
interpreter answering it (`story:the-interpreter-answers-the-response-value-an-outcome-stores`), a
synthesized suite must (a) hold a target to storing exactly the value it returned, and (b) witness
a later command guarded on the stored value, by sending back the value the earlier step returned
and a value that differs from it. Today no scenario value reads an earlier step's actual response
(`ScenarioValue` has `Observed {event, field}` for event payloads, `crates/verify/ess-conformance/src/scenario.rs:1883`,
and `Awaited`, `:1927`), so synthesis refuses every such guard on 0.92.0:

```console
$ ess verify conform synthesize --path generated --out generated-suite.json
refused: refusal[ESS-SYNTH-001]: outcome example.order.Confirm/mismatch has no scenario …
  no witness: `example.order.Order.pin` … no arranging branch sets this stored field from an input or a literal without a conversion …
(same for Confirm/confirmed and the confirm transition; ESS-SYNTH-004 for Confirmed)
1 scenario(s) (0 authored), 4 refusal(s)
```

## Acceptance

- `ScenarioValue::Responded {field}` reads the named field of the actual response the scenario's
  last `capture_command_result` retained (`scenario.rs:2278`); a suite carrying it is the next
  free ordinary/coverage suite pair (shared with any other suite construct of the same release,
  as the round pairs are), and an older major refuses it at admission; the Rust runner resolves it.
- `ess verify conform synthesize` on the reproduction at the new source format writes scenarios
  for `Confirm/confirmed` (input `pin` = `Responded {field: pin}` of the `Place` step),
  `Confirm/mismatch` (a literal different from it), the `confirm` transition and the `Confirmed`
  wrong-state cell, with no `ESS-SYNTH-001`, `-003` or `-004` for them; the `Place` scenario reads
  the row back and requires `pin` equal to `Responded {field: pin}`.
- `ess verify conform run --target interpreted --report-format 2` passes every scenario; a faulty
  target storing a value other than the one it returned fails the `Place` scenario, and one that
  ignores the stored pin in `Confirm` fails `Confirm/mismatch` (a mutant pair in an
  ess-conformance test).
- A suite without the value keeps its format and bytes.

## Scope

- cited: crates/verify/ess-conformance/src/scenario.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/src/responded_values.rs
- inferred: crates/verify/ess-conformance/src/admission.rs
- inferred: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/verify/ess-conformance/src/replay.rs
- inferred: crates/verify/ess-conformance/tests/stored_response_values.rs
- inferred: website/docs/reference/formats.md
confidence: medium

`synthesize.rs` treats `ResponseField` as determining nothing at `:9995`, `:11350`, `:12188`,
`:12753`; the arranging check is in `synthesize/subject_fact.rs`.

## Fit review

1. **Need.** A stored-equals-returned fact the suite does not check is a fact that stops being
   true (#498). Requester's proposal (theirs): the construct; the suite half is implied by
   "a later command compares its input with the stored value".
2. **Class.** Gap in the suite vocabulary (`run.txt` § 2, § 3).
3. **Already expressible?** No: authored scenarios cannot read an earlier response into an input
   either; `capture:` takes an event field.
4. **Fit.** `Responded` is the sibling of `Observed` (event field) and `Awaited`, resolved from the
   same retained capture `ExpectReplayResult` already uses (`scenario.rs:2278-2286`); synthesis
   arranges a guard over a stored response value the way it arranges one over a stored input.
5. **Second adopter.** The pairing flow of the parent story; any one-time code that is not marked
   `one_time_response:`.
6. **Cost.** One suite format pair (or a share of the release's pair); Go and TypeScript in
   `story:the-go-and-typescript-runtimes-resolve-a-responded-value`, refusing it by major until
   then.
7. **Alternatives.** An authored-only scenario per model: leaves the synthesized suite blind.
   Carrying the value in the suite as a literal: a suite whose meaning depends on the target.
   Chosen: a reference to the actual response.

## Decisions

Accept, redesigned (suite and synthesis part of `story:stored-field-equals-returned-response-value`).

## Source

github:beyond10x/ess#498
