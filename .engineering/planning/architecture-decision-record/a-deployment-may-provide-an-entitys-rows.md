---
format: aep.planning-md/3
id: architecture-decision-record:a-deployment-may-provide-an-entitys-rows
kind: architecture-decision-record
status: accepted
title: A deployment may provide an entity's rows, and conformance asks its fixture provider for them
summary: Option B of decision-blocker provided-entity-rows-for-conformance, decided 2026-10-31
relations:
- decides: story:a-fixture-may-be-a-row-the-deployment-already-holds
revision: 2
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-31T11:36:04Z", actor: "human:timo", revision: 2}
---
## Context

A command may bind a new row to a row of an entity that no command of the specification creates,
because the deployment already holds such rows (a downstream deployment's external record; wave
D325, G2; `story:a-fixture-may-be-a-row-the-deployment-already-holds`). On 0.77.0 synthesis refuses
the branch that needs such a row (`ESS-SYNTH-004`, nothing creates it), the interpreted target has
no pre-execution fixture provider (`ESS-CF-TARGET`), seeds apply only to a command's subject, and
the authored `arrange:` idiom has the runner write the row, which a read-only deployment cannot
accept. Options A-E are in `decision-blocker:provided-entity-rows-for-conformance`.

## Decision

Option B. From `format: ess/33` an entity may declare that the deployment provides its rows (one
entity key; its spelling is chosen in the implementing unit and recorded in the story). Where
synthesis needs a row of such an entity in a given state and ordinary arrangement ends in "nothing
creates it", the scenario asks the target's pre-execution fixture provider for a row of that entity
in that state, and uses its identity; where the row must not exist, synthesis keeps a fresh
identity. The interpreted target provides such a fixture by creating the row. An implementation's
runner gets it from its own fixture provider. The fixture contract gains the entity and state under
the suite pair `ess-conformance/60` and `/61`; older readers refuse it by version. No authored
scenario format changes. Ordinary arrangement keeps precedence, so a model without the key keeps
its IR and suite bytes.

Decided 2026-10-31 by the operator's proxy, relayed by the orchestrator: "decided B. One entity
marker 'deployment provides its rows'; synthesis emits the fixture wherever nothing creates the row;
the interpreted target creates it; ess/33 and suite 60/61, no scenario-format bump. Record it as the
ADR with this quote and time." The proxy acts on the operator's word of 2026-10-31 11:23Z: "nothing
must be waiting on me - its your job to act on my behalf".

## Consequences

- A needs per-scenario authoring; C does not work for a read-only deployment; D leaves the adopter
  blocked; E guesses an entity from a type, against "Imports never guess".
- Every position that needs an existing row (a command's subject, an owner, a related row, a view's
  population) takes a provided row the same way, or is refused by name.
- Open points the unit settles and records: the key's spelling; decoy rows arranged beside a related
  row; a guard over a provided row's fields (cannot be steered; refused by name); identity collisions
  for non-`Uuid` identity types.

Decider: operator
