---
format: aep.planning-md/3
id: story:defined-and-not-defined-branches-are-exhaustive
kind: story
status: draft
title: Branches over defined(x) and not defined(x) are proved exhaustive
summary: 'Adopter report R4: two update branches over one stored Optional field draw ESS-COMMAND-005.'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Report

An adopter retrofitting an existing service (2026-10-23, ess 0.67.0, ess/29): two `when_subject` update branches guarded `defined(token)` and `not defined(token)` over one `Optional` field are refused as ESS-COMMAND-005 (coverage not proved); dropping the second guard validates.

## Acceptance (draft, fit review pending)

- A pair of branches over `defined(x)` and `not defined(x)` on one stored Optional field is proved exhaustive, so both branches can state their condition.

## Fit review

Request (an adopter, verbatim): two update branches `when_subject: {predicate: defined(token)}`
and `{predicate: not defined(token)}` over one Optional field draw "error[ESS-COMMAND-005]:
subject-fact/input coverage is not proved". Need: the pair recognised as exhaustive.

## Fit review

1. **Need.** Branch an update on whether an Optional stored field is set, naming both sides.

2. **Class.** Convenience. One branch can be the default today, and that is witnessed.

3. **Already expressible?** Yes, as a default. Reproduction:

   ```yaml
   entities:
     - name: demo.link.Device
       identity: {name: device_id, type: String}
       fields:
         - {name: token, type: Optional<String>}
   commands:
     - name: demo.link.Pair
       input: [{name: device_id, type: String}, {name: token, type: String}]
       outcomes:
         - {name: repaired, when_subject: {predicate: defined(token)}, updates: demo.link.Device, instance: device_id, sets: {token: input.token}}
         - {name: paired, when_subject: {predicate: not defined(token)}, updates: demo.link.Device, instance: device_id, sets: {token: input.token}}
         - {name: no-device, unknown_instance: true, error: demo.link.NoDevice}
   ```

   ess 0.68.0 (ess/30): ESS-COMMAND-005 with the Decline::Unsupported text "a guard reads an
   optional or collection path, or a construct other than equality and membership"
   (fit/repro/r4-adopter-shape.yaml). Dropping the second guard makes `paired` the default: valid,
   4 scenarios, all pass on the interpreted target (fit/repro/r4-idiom.yaml). The same pair on an
   Optional input (`when: defined(token)` / `when: {not: defined(token)}`) is refused too, as
   ESS-COMMAND-005 "every outcome of `demo.link.Pair` is conditional"
   (fit/repro/r4-input-pair.yaml).

4. **Fit with grids and targets.** The cause is the finite prover,
   crates/specify/ess-domain/src/command/finite.rs. `input_paths` handles Always/Never/All/Any/Not/
   Truthy, Compare Eq/Ne against Text or Bool, and AnyOf/NoneOf; `Predicate::Defined`
   (crates/specify/ess-primitives/src/predicate.rs:1498) falls to `_ => Err(Decline::Unsupported)`.
   `analyze_inputs` declines any `resolved.optional` path, and the doc comment (around line 163)
   says optional paths are "deliberately unsupported". That was a scope cut, not a semantic rule:
   presence is a closed two-value domain {absent, present}, exactly what the prover handles for
   Boolean. Opening it fills the `defined` cell of grid R for both the stored side (subject_fact.rs:
   590-630) and the input side (command.rs:4072) at once. No target changes: a stored Optional
   already round-trips absent/present in the interpreter and generated code.

5. **Second adopter.** The input-side refusal (fit/repro/r4-input-pair.yaml) is the same gap on a
   different surface; any model with an Optional field hits it.

6. **Cost.** Prover: one domain kind and the `Defined` arm. A path also compared by value
   (`defined(token)` beside `token == "x"`) keeps declining, so no mixed domain is needed.
   Synthesis: a creator that leaves the Optional unwritten arranges "absent", an update that sets
   it arranges "present"; inferred to work with today's arrangement, confirmed by the first red
   test.

7. **Alternatives.** (a) Decline with the default idiom: works, but leaves a two-value domain
   unproved for no semantic reason. (b) New syntax (`when_subject: {absent: token}`): rejected,
   `defined` already says it. (c) Extend the prover with a presence domain: chosen.

## Decisions

**Accept, redesigned (no new syntax).**

- **Rule.** A path read only through `defined(…)` (and `not defined(…)`) in a command's guards is a
  finite path with domain {absent, present}, on the stored and the input side. Optional is allowed
  for such a path. A path read through `defined` and by value keeps today's decline.
- **Format.** ess/31, an additive row in FORMAT_HISTORY (crates/specify/ess-domain/src/system.rs):
  "a `defined`/`not defined` pair over one Optional path is proved exhaustive; below ess/31 refused
  as before; a model without it keeps its IR bytes". Below ess/31 the 005 refusal stays and its
  hint names ess/31 and the default idiom. No IR change: the predicate already lowers.
- **ValidationCode.** None new.
- **Files.** finite.rs (`input_paths`, `analyze_inputs`, `field_coverage`/`field_inputs`, the
  Decline text), subject_fact.rs:590-630, command.rs around 4072, system.rs FORMAT_HISTORY,
  docs/design/language-grids.md grid R, CHANGELOG.md.
- **Pins.** Format version and hint move: run `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics`; gate ess-domain and ess-cli.
- **Reply to the adopter.** "Accepted for ess/31: a `defined`/`not defined` pair over one Optional
  field is proved exhaustive, on stored fields and on input. Until then make one side the default."
- **Size.** S.

## Acceptance

In crates/specify/ess-domain/tests/subject_guard_input.rs:

- `defined_and_not_defined_over_one_optional_field_are_exhaustive_at_ess_31` (first red test): the
  adopter shape at ess/31 validates with no ESS-COMMAND-005.
- `the_same_pair_below_ess_31_names_the_format`: at ess/30 the 005 refusal stays and the hint names
  ess/31.
- `a_defined_pair_over_an_optional_input_is_exhaustive_at_ess_31`: the input-side pair validates.
- `defined_beside_a_value_comparison_still_declines`: `defined(token)` plus `token == "a"` keeps 005.
- `defined_alone_is_not_exhaustive`: one guarded branch with no sibling and no default keeps 005.

In crates/verify/ess-conformance/tests/when_subject_witness.rs:

- `a_defined_pair_witnesses_both_presence_states`: one scenario per branch, each passing on the
  interpreted target, and a mutant swapping the two branches is killed.
