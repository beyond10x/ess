---
format: aep.planning-md/3
id: story:synthesis-reaches-every-state-of-a-provided-entity-through-its-provider-and-never-drives-a-move-the-deployment
kind: story
status: draft
title: Synthesis reaches every state of a provided entity through its provider and never drives a move the deployment makes
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-entity-whose-rows-the-deployment-provides-declares-lifecycle-moves-the-deployment-makes-with-no-command
revision: 2
---
## Need

After `sl13-provided-moves`, a provided entity may hold states that no command reaches. Synthesis
arranges a row in a state by driving the commands that move it there. For a state reached only by
the deployment's own move, it finds no route and stops with `ESS-SYNTH-004` (nothing creates or
moves it; see `decision-blocker:provided-entity-rows-for-conformance`, which cites
`crates/verify/ess-conformance/src/synthesize.rs:6564-6568`). A command guarded on that state, for
example a `Recall` refused unless the shipment is `Packed`, goes unwitnessed.

## Outcome

Where a scenario needs a row of a provided entity in a state, synthesis asks the provider for it
with the existing entity fixture `{name, type, entity, state}` (`ess-conformance/60` and `/61`).
It never writes a step that claims a deployment move happened. A view reading the entity is
checked against provided rows in each state. A suite without such an entity keeps its bytes and
its major.

## Acceptance

- Red first, in `crates/verify/ess-conformance/tests/` (beside the provided-row tests of the
  `ess/33` story): a model with the `sl13-provided-moves` entity and a command
  `shop.delivery.Recall` refused as `wrong_state` outside `Packed` gets a scenario arranging an
  `InTransit` row through an entity fixture and expecting the refusal. Before the change it is
  reported as `ESS-SYNTH-004`.
- The interpreted target passes the scenario, and the Rust runner passes it against a provider
  answering the fixture.
- No scenario contains an `execute_command` or `await_outcome` for a transition that only the
  deployment makes.
- A suite for a model without such an entity is byte-identical before and after.

## Fit review

Second report, G13, unit 2 of 2. The fit review is in `sl13-provided-moves`. The provider fixture
is the construct `decision-blocker:provided-entity-rows-for-conformance` chose (option B) for rows
the specification does not create. This story extends it to states the specification does not
reach.

## Decisions

None beyond SL-D11 (open, with the owner as AQ3). Under its option B, this story is not needed.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (the arrangement search's dead end at
  6564-6568), `crates/verify/ess-conformance/src/subject_fact.rs`, tests under
  `crates/verify/ess-conformance/tests/`
- Run `ess-conformance` one `--test` at a time (`AGENTS.md`, package-gate table).
- Size: M, about 400 changed lines.

## Depends on

`sl13-provided-moves`.

## Source

A downstream application, 2026-11-24, second report, G13.
