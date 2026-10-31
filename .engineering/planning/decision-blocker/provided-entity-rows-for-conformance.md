---
format: aep.planning-md/3
id: decision-blocker:provided-entity-rows-for-conformance
kind: decision-blocker
status: cleared
title: How a conformance suite gets a row of an entity the specification does not create
summary: 'G2 of wave D325: options A-E, recommendation B (entity marker, ess/33, suite 60/61)'
relations:
- blocks: story:a-fixture-may-be-a-row-the-deployment-already-holds
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-31T11:36:09Z", actor: "human:timo", revision: 3}
---
## Question

How does a conformance suite get a row of an entity that no command of the specification creates,
because the deployment already holds such rows (a downstream deployment's external record, G2 of
wave D325)? The shape is a new construct with a real choice between designs, so it is ADR-level and
goes to the operator.

Today: the interpreted target has no pre-execution fixture provider (`ESS-CF-TARGET`,
`crates/verify/ess-conformance/src/target.rs:108-117`, not overridden in `interpret.rs:322-328`);
synthesis refuses the accepted branch with `ESS-SYNTH-004` because no outcome creates the related
row (`synthesize.rs:6564-6568`, `subject_fact.rs:5408-5412`); seeds apply only to the command's
subject (`subject_fact.rs:8904-8908`, `docs/design/synthesis-seeds.md:127-130`).

## Options

| | Design | Format cost | Reaches |
|---|---|---|---|
| A | A named fixture declared as `{entity, state}`, in the specification and in authored `fixtures:` (the requester's proposal) | `ess/33` key, `ess-scenario/6`, suite pair 60/61 | interpreted (creates the row), runners (provider returns an identity), synthesis where declared, authored |
| B | An entity-level marker that the deployment provides its rows (spelling open). Where arrangement ends in "nothing creates", synthesis asks for a fixture `{entity, state}`; a fresh identity where the row must not exist | `ess/33` (one entity key), suite pair 60/61 | the same targets, at every position that needs such a row (subject, owner, related row, view population) |
| C | Synthesis seeds apply to related rows `when_related:` reads | no source format; a seed-reader change | only targets that accept writes; a read-only deployment answers unsupported |
| D | Change nothing: the authored `arrange:` idiom, plus a clearer `ESS-SYNTH-004` help | none | the accepted branch stays unchecked on a read-only deployment |
| E | The interpreted target answers a type-only fixture whose type is the identity of the one entity nothing creates | none | guesses entity and state from a type, against "Imports never guess"; refused |

## Recommendation

**B.** It is the smallest surface that reaches synthesis (one entity key and one suite pair). That a
deployment provides an entity's rows is a fact about the entity, not about one command, and every
position that needs such a row is served. Ordinary arrangement keeps precedence, so a model without
the key keeps its IR and suite bytes. Open points for the ADR: the key's spelling; decoy rows that
synthesis arranges beside the related row; a guard over a provided row's fields (cannot be steered,
refused by name); identity collisions for non-`Uuid` identity types.

Wave D325 continues without it: the defect half of the story (a `fixture_inputs` binding on a field
that `when_related: {via: …}` reads validates, and synthesis then sends the fixture to the
`exists: false` branch) needs no format and ships in the wave. The construct goes to the next wave
with the answer.

Decider: operator

## Answer

Option B, 2026-10-31, from the operator's proxy relayed by the orchestrator: "decided B. One entity
marker 'deployment provides its rows'; synthesis emits the fixture wherever nothing creates the row; the
interpreted target creates it; ess/33 and suite 60/61, no scenario-format bump." Recorded as
`architecture-decision-record:a-deployment-may-provide-an-entitys-rows`. The construct takes the
fourth D325 slot (D327: consumer-blocked work first); the nested-member claim moves to the next wave.
