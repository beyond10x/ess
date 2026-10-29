---
format: aep.planning-md/3
id: story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed
kind: story
status: draft
title: A cascade onto rows whose creator needs the moving subject active and a second related row is witnessed
summary: 'affects: rows created only by a command guarded by the subject''s state and another related row get no arrangement (ESS-SYNTH-001); adopter report 2026-10-29, 0.76.0 and 0.77.0'
tags:
- ess-issues-3
relations:
- informed_by: story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed
revision: 1
---
## Problem

A downstream adopter's suite keeps one ESS-SYNTH-001 refusal on 0.76.0 and on the 0.77.0 head
(84671d6b8): a command that moves its subject and cascades onto rows of another entity through
`affects:` gets no scenario for that outcome, because synthesis arranges no affected row.

Shape, as neutral names:

- `CancelRun` moves `Run.cancel` on `instance: run_id` and lists
  `affects: [{entity: Generation, where: run_id == input.run_id, moves: Generation.cancel}]`.
- `Generation` rows are created only by `StartGeneration/started`. That command refuses first when
  the run is missing (`when_related: {via: input.run_id, exists: false}`), when the run is not
  Active (`when_related: {via: input.run_id, predicate: state != Active}`), and when a second
  related row, the node, is missing (`when_related: {via: input.node_id, exists: false}`).

The refusal on the 0.77.0 head:

```text
refusal[ESS-SYNTH-001]: outcome inference.graph.CancelRun/cancelled has no scenario `inference.graph.CancelRun/outcome/cancelled`
  no witness: `cancelled.affects[0]` is `inference.stream.Generation` and created only by `inference.stream.StartGeneration/started`, which no arrangement takes for a row the filter selects with the rows the scenario holds when it is sent, so no row is arranged
```

`story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` (implemented) arranges the
creator past a guarded sibling branch. It does not take a creator guarded by a related row's state
(the run must be Active, which is also the subject `CancelRun` moves) together with a second
related row (the node) that must be created first.

Reported by the adopter's agent on 2026-10-29 (orchestrator D194). Its specification is public;
the reproduction is kept out of this repository.

## Acceptance

- A neutral model of the shape above: `ess verify conform synthesize` writes
  `<CancelRun>/outcome/cancelled`, which arranges the node and an Active run, takes the creator's
  accepting branch so a `Generation` row the filter selects exists, then sends the cancel; the
  interpreted target passes it. Before: ESS-SYNTH-001 as quoted.
- The existing cascade story's cases keep their scenarios and bytes.
- Where no arrangement exists (the creator's guard contradicts the state the cascade needs), the
  refusal stays ESS-SYNTH-001 and names the guard that answers first.
