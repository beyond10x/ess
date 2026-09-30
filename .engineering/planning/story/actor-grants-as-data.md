---
format: aep.planning-md/3
id: story:actor-grants-as-data
kind: story
status: implemented
title: Actor grants are generated as data
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:02Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T20:57:20Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Actor grants stay unenforced by generated code, but their declared `may:` lists are generated as
data, so a caller enforces a generated table instead of a hand-copied one.

## Acceptance

- The types crate has an `Actor` enum with one variant per declared actor and
  `pub fn may(actor: Actor) -> &'static [&'static str]` listing the qualified command names the
  actor may invoke, in name order (the compiled IR holds `may` as an ordered set).
- The plan's "actor grants" rows say the grant is available as data and enforcement stays with the
  caller.
- A model without actors generates identical bytes.
