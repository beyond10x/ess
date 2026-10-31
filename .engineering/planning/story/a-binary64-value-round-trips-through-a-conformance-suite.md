---
format: aep.planning-md/3
id: story:a-binary64-value-round-trips-through-a-conformance-suite
kind: story
status: active
title: A Binary64 value round-trips through a conformance suite
summary: 'Split from the Binary64 admission story: a reached Binary64 value is written by synthesis and compared by every runner under a stated suite contract.'
tags:
- adopter-report
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-binary64-field-is-admitted-by-a-conformance-run
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T19:59:00Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-31T19:59:00Z", actor: "human:timo", revision: 4}
---
## Problem

Split from `story:a-binary64-field-is-admitted-by-a-conformance-run` (fit review 2026-10-26): that
story admits a `Binary64` position nothing reaches. Carrying a reached `Binary64` value through a
conformance suite (synthesis writes it, runners compare it, the exact observers admit it) needs a
suite contract that `docs/design/model-binary64.md` § "Admission, equality and conformance
publication decisions" does not state yet.

## Acceptance

- Acceptance lines 2-4 of the parent story as written before the split, with "non-finite" read as
  an overflowing token such as `1e400` (JSON has no NaN token).
- The exact observers that `story:a-view-may-publish-a-decimal` opens for `Decimal` admit
  `Binary64` under the same suite format pair, compared by value.
- `docs/design/model-binary64.md` states the suite contract.

## Second report: validate refuses a reached Binary64 once authored scenarios are listed (2026-10-25)

Reported by the Loom specification session against ESS 0.69.0: `ess specify validate` refuses a
`Binary64` field with `ESS-AUTHOR-036` once the specification lists authored scenarios; Loom's gate
tests strip `scenarios:` from their copy as a workaround, and the specification itself types the
value as `Decimal`.

Triage on main (f692aab, 0.77.0), 2026-10-31: still refused. A copy of that specification with its
four `Optional<Decimal>` confidences retyped to `Optional<Binary64>`, authored scenarios kept:

```text
refusal[ESS-AUTHOR-036]: model
  command.….RecordSelection.input.confidence.of, command.….SelectAction.input.confidence.of, entity ….Selection.fields.confidence.of, entity ….SelectionRecord.fields.confidence.of: Binary64 requires a qualified conformance codec
  help: use checked format-5 normalization; finite Binary64 conformance codecs are not implemented
```

This is the reached-`Binary64` half this story owns; wave D325 leaves it for the next wave.

## Fit review (wave D335 U3, 2026-10-31)

Reproduced on 0.77.0 with a brand-free model (an `Optional<Binary64>` command input and entity
field, a view, one authored scenario): `ess specify validate` exits 1 with ESS-AUTHOR-036 "Binary64
requires a qualified conformance codec"; the same model with `Integer` validates.
`ess verify conform synthesize` refuses both paths as `UnsupportedPrimitive`.

The suite contract is the section "Suite contract for a reached Binary64 (suite/62 and /63)" of
`docs/design/model-binary64.md`.

## Decisions

**accept as designed** (wave D335, the coordinator's call inside
`approval-record:ess-d335-next-adopter-gaps`; no ADR, no breaking change).

- Carried: entity fields, command input and response, event and error fields, view fields and row
  shape. Not carried: entity identity, view parameters, actor attributes, bindings and conversions,
  which keep `UnsupportedPrimitive` and ESS-AUTHOR-036 with a help naming the contract. The browser
  routes, mutation and whole-system `ess synthesize` keep refusing (`admission::model_without_binary64`).
- Witness: the n-th instance is `n + 0.5`, never at or above 2^53 in magnitude, written as the
  existing shortest round-trip token, so no other bytes move.
- Comparison: by nearest binary64 under IEEE equality, no epsilon (`1.5` equals `1.50`, `0` equals
  `-0`). The exact observers admit `Binary64` beside `Decimal`. An overflowing token such as `1e400`
  is refused with its path in a suite, and fails the step with its path in a Go or TypeScript answer.
- Format: `ess-conformance/62` (ordinary) and `/63` (coverage), floored by the model. Not the
  Decimal pair: released readers at `/58`-`/61` would run a `Binary64` as an untyped number without
  refusing it. Below `/62`, a suite recording a `Binary64` type is refused, "a Binary64 value
  requires suite/62 or /63".
- Public API: `admission::model` keeps its signature and admits more; `admission::model_without_binary64`
  and `ess_compiler::binary64::uncarried_uses` are added. No change to `ess-synth`, the source format
  or normalization.

## Acceptance

- `crates/verify/ess-conformance/tests/suite.rs`: `a_reached_binary64_field_round_trips_through_a_suite`,
  `a_suite_without_a_binary64_keeps_its_bytes`, `binary64_outside_the_suite_contract_refuses_by_name`,
  `non_finite_binary64_is_refused_by_name`; the existing refusal tests restated with this story's id.
- `crates/verify/ess-conformance/tests/binary64_count_adversary.rs`: legacy majors 1-61 refused at
  admission; web still refused (restated).
- `crates/verify/ess-conformance/tests/retained_replay.rs`: `binary64_exact_observers_replay_by_value`,
  `binary64_values_compare_by_value_in_every_runner`.
- `crates/verify/ess-conformance/src/ts/response.test.ts`: the same comparison vectors.
- `crates/edge/ess-cli/tests/binary64_publication.rs` (restated: conformance routes publish,
  whole-system synthesize and `conform web` refuse) and `validate_completeness.rs`:
  `a_reached_binary64_with_authored_scenarios_validates`.
