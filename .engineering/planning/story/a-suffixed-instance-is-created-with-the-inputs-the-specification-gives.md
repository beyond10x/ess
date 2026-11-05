---
format: aep.planning-md/3
id: story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives
kind: story
status: active
title: A suffixed instance is created with the inputs the specification gives
summary: Synthesis sends generated placeholders instead of input examples for a second instance, and a forced external create of it references an undeclared row (beyond10x/ess#736; 0.78.0 to 0.86.0)
tags:
- from-consumer-report
- next-wave-first
refs:
- provider: github
  reference: beyond10x/ess#736
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:13:28Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T00:13:28Z", actor: "human:timo", revision: 3}
---
## Outcome

A synthesized scenario that creates a second, suffixed instance (`widget-2`, `room-2`) builds that step's input the
way the specification says, so an implementation that honours the specification passes it.

## Context

Reported in beyond10x/ess#736. A downstream consumer has 30 of its 48 red conformance scenarios behind these two
defects. Both concern a step that creates a suffixed instance: its inputs are generated values, not what the
specification provides for the plain instance. They may share a cause, so they are one story.

Reproduced by the lead on 2026-11-05 with ess 0.86.0 (no `requires` pin in the repro, so no delegation), using the
issue's two repros verbatim:

- (a) `CreateWidget`'s first create sends the input `example:` (`config: {type: object, properties: ...}`,
  `name: "Front door"`). The second create, for `widget-2`, sends `config: {"config-1": {"config.0": "config.0-1"}}`
  and `name: "name-1"`. The interpreter passes all 4 scenarios, because the model needs no `properties`. The gap
  shows only against an implementation that does.
- (b) `demo v1 against interpreted 0.86.0 — failed`: `AddNote/outcome/added` and `OpenRoom/outcome/opened` fail
  (4 passed, 2 failed). The second `OpenRoom`, forced to the `external:` branch `opened`, sends the literal
  `model_id-1`, which no step declared. The model answers `unknown-model`. Without the `external:` line, the
  second open references a captured, declared model.

## Problem

- (a) The example is applied only where synthesis creates the plain instance. The suffixed instance's inputs are
  all generated placeholders.
- (b) When the branch that creates the second instance is forced through `configure_external_outcome`, the related
  reference (`when_related: {via: input.model_id, exists: false}` on a sibling refusal) is not arranged. The
  non-external path does arrange it.

## Acceptance

Named tests in `ess-conformance`; each fails on 0.86.0 and passes after the change.

- `a_suffixed_instance_sends_the_input_examples` - in the widget repro, every create of a `Widget` sends `name` and
  `config` from their `example:`. Only the identity, and any field a filter or uniqueness check needs to tell the
  instances apart, differ between instances.
- `a_forced_external_create_of_a_second_instance_references_a_declared_row` - in the room repro, the second
  `OpenRoom` references a captured `Model` instance, or one a preceding `DeclareModel` arranges. The interpreted run
  passes all 6 scenarios.
- `a_second_instance_without_examples_is_unchanged` - a command without `example:` inputs synthesizes the same
  steps as today.
- Every existing synthesis byte-pin or count that moves is re-pinned in the same change, with the reason in its
  commit.

## Out of scope

- How a downstream consumer pins its `ess` release (see Notes).

## Notes

The reporter observed that installed ess 0.86.0 "hands off" to 0.78.0 for a specification with
`requires: ess 0.78.0`. That is the documented toolchain delegation: an exact `requires: ess X.Y.Z` pin makes any
newer `ess` exec the pinned release from its cache. It is documented in `website/docs/start/install.md` ("any `ess`
run in `tasks` or below it runs 0.86.0 from the cache, whatever release is on your `PATH`") and in
`crates/edge/ess-cli/src/toolchain.rs` and `docs/design/specification-requires-release.md`, and
`ess specify toolchain which` names the release that would run.

Two consequences:

- The downstream counts quoted as identical on 0.78.0, 0.85.0 and 0.86.0 were likely all produced by 0.78.0
  through that delegation, which is inferred, not checked. The issue's two repros carry no pin and do reproduce
  on 0.86.0, as above.
- `website/docs/guides/specify/layout-and-validation.md` ("Name the `ess` release ...") still says a newer `ess`
  prints a warning and continues, and does not mention delegation for an exact pin. That page should say what
  install.md says. Fix it when this story lands, or as its own docs fix.

## Schedule

Accepted by the orchestrator on 2026-11-05. It goes first in the next ESS wave after 0.87.0; it joins 0.87.0 only
if that wave has a free unit slot and no collision (it lands in `synthesize.rs`, which U1 holds).

## Decisions

The 0.88.0 adversary pass on this unit found two introduced defects; the coordinator decided on
2026-11-05 that one correction in this wave fixes both:

- A further instance takes the input's example even where a `when_related` row-set uniqueness check
  on the create needs the instances told apart, and synthesis then refuses the created outcome.
- A further instance takes the example where a later command's related-row or subject predicate reads
  that field, and that command's branch loses its scenario: the code inferred that no search chose the
  value from placeholder equality and checked only the creating command's guards.

The correction treats every stored field that any command's row selection, related-row predicate or
subject predicate reads as told apart (or carries whether a search steered the value), and adds both
models to `suffixed_instance_inputs`.
