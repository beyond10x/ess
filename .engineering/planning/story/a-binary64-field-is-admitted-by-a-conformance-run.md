---
format: aep.planning-md/3
id: story:a-binary64-field-is-admitted-by-a-conformance-run
kind: story
status: draft
title: A binary64 field is admitted by a conformance run
summary: 'Adopter report R8: A Binary64 field blocks a whole conformance run (ess#646).'
refs:
- provider: github
  reference: beyond10x/ess#646
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ess verify conform run` refuses a whole model that declares any `Binary64` field (`UnsupportedPrimitive`, `crates/verify/ess-conformance/src/admission.rs:1146-1157`), even where nothing a scenario sends or reads reaches it, and the suite format cannot carry a `Binary64` value at all. `Decimal` runs but generates a string, wrong for a JSON-number wire field. Adopter report R8, ess 0.68.0.

## Acceptance

- `an_unreached_binary64_type_does_not_block_a_run`: a model whose only `Binary64` field sits in a type no command, event or view reaches runs, and the run reports every scenario.
- `a_reached_binary64_field_round_trips_through_a_suite`: a command input and a view field of type `Binary64` are synthesized, written to the suite as JSON numbers at a new suite format, and pass on the interpreted target; an older suite format refuses them by name.
- `binary64_values_compare_by_value_in_every_runner`: the Rust, Go and TypeScript runners compare `1.5` and `1.50` equal and `1.5` and `1.25` unequal.
- `non_finite_binary64_is_refused_by_name`: a non-finite value in setup or an answer is refused with its path.
