---
format: aep.planning-md/3
id: story:authored-instances-inside-structured-inputs
kind: story
status: implemented
title: '{$instance: …} is refused inside a list input in authored scenarios'
refs:
- provider: github
  reference: beyond10x/ess#242
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T08:12:54Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T08:12:55Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T12:13:04Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

An authored step may put `{$instance: name}` wherever the declared type at that position is an identity type — list elements, map values and struct members as well as scalar inputs — and it resolves to the instance (beyond10x/ess#242).

## Acceptance

- `ring_sequence: [{$instance: a}, {$instance: b}]` for a `List<ReleaseRingId>` input authors and runs with both instances;
- the same inside a map value and a struct member;
- a `{$instance: …}` at a position whose type is not an identity is still refused, naming the position.
