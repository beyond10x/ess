---
format: aep.planning-md/3
id: story:a-lifecycle-state-carries-a-wire-name
kind: story
status: draft
title: A lifecycle state carries a wire name
summary: 'Adopter report G4: A lifecycle state cannot carry a wire spelling (ess#639).'
refs:
- provider: github
  reference: beyond10x/ess#639
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

A lifecycle state is a bare name, so a service whose wire spelling differs (`indexing` for `Indexing`) needs a separate enum kept in step by hand. `states: [{name:, wire:}]` is refused as "invalid type: map, expected a string". Reported by two adopters (G4 and an earlier report), ess 0.67.0 and 0.68.0. The enum variant's wire name (`RawEnumVariant`, `crates/specify/ess-domain/src/types.rs`) is the pattern. Already planned under unit U19 of the 0.69.0 wave; not yet in the store.

## Acceptance

- `a_lifecycle_state_declares_its_wire_name`: `{name: Indexing, wire: indexing}` validates at the new format and is refused below it by name; a bare state keeps its bytes.
- `two_states_with_one_wire_name_are_refused`: duplicate wire names in one lifecycle are refused.
- `the_wire_name_is_projected_wherever_the_state_is`: JSON Schema, OpenAPI, AsyncAPI and the generated Rust, Go and TypeScript types spell the state with its wire name; the model and `ess-ir` keep the declared name.
- `the_interpreter_and_suites_compare_states_by_wire_name`: a view row or response carrying `indexing` passes on the interpreted target; a target answering `Indexing` fails.
