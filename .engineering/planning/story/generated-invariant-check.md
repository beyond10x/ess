---
format: aep.planning-md/3
id: story:generated-invariant-check
kind: story
status: active
title: Each generated entity data type checks its declared invariants
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 3}
---
## Outcome

Each generated entity data type carries a check of the entity's declared `invariants:`, so a
behaviour (generated or hand-written) asks the type whether a value may be stored.

## Acceptance

- `impl <Entity>Data { pub fn broken_invariant(&self) -> Option<&'static str> }` returns the first
  declared invariant the value breaks (its declared name or text), else `None`.
- Every invariant form the compiler accepts is evaluated; a form the generator cannot evaluate is
  refused at synthesis naming the invariant (never silently skipped).
- A model without invariants generates identical bytes.
